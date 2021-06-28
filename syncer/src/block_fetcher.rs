use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;

use bitcoin::{Block, BlockHash};

use super::*;

pub struct BlockFetcher {
    rest: bitcoin_rest::Context,
    /// (height, (block_hash, block))
    blocks: Arc<RwLock<HashMap<u32, (BlockHash, Block)>>>,
    /// The block height that should be returned at next `get()` call.
    current_height: u32,
    /// The next block height the workers should fetch for.
    next_height: Arc<RwLock<u32>>,
    stop_height: u32,
    stop: Arc<RwLock<bool>>,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl BlockFetcher {
    pub async fn fetch_block(rest: &bitcoin_rest::Context, height: u32) -> (BlockHash, Block) {
        let block_hash = rest.blockhashbyheight(height).await
            .expect(&format!("Failed to fetch block at height = {}.", height));
        let block = rest.block(&block_hash).await.expect(&format!("Failed to fetch a block with blockid = {}", block_hash));
        (block_hash, block)
    }
    pub fn new(coin: &str, config: &Config, start_height: u32, stop_height: u32) -> Self {
        Self {
            rest: get_rest(&config.coins[coin]),
            blocks: Arc::new(RwLock::new(HashMap::new())),
            current_height: start_height,
            next_height: Arc::new(RwLock::new(start_height)),
            stop_height,
            stop: Arc::new(RwLock::new(false)),
            handles: Vec::new(),
        }
    }
    pub async fn len(&self) -> usize {
        self.blocks.read().await.len()
    }
    pub async fn get(&mut self, height: u32) -> (BlockHash, Block) {
        let mut blocks = self.blocks.write().await;
        let block = blocks.get(&height);
        match block {
            Some(block) => {
                let (block_hash, block) = block.clone();
                // Remove unneeded blocks.
                for h in self.current_height..(height + 1) {
                    blocks.remove(&h);
                }
                self.current_height = height + 1;
                return (block_hash, block);
            },
            None => {
                // Fallback.
                let (block_hash, block) = Self::fetch_block(&self.rest, height).await;
                return (block_hash, block);
            },
        }
    }
    pub fn run(&mut self, n_threads: Option<usize>) {
        // Register Ctrl-C handler.
        {
            let stop = self.stop.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
                println!("Ctrl-C was pressed. Exiting BlockFetcher...");
                *stop.write().await = true;
            });
        }
        let n_threads = n_threads.unwrap_or(num_cpus::get());
        for _thread_id in 0..n_threads {
            let rest = self.rest.clone();
            let blocks = self.blocks.clone();
            let next_height = self.next_height.clone();
            let stop = self.stop.clone();
            let stop_height = self.stop_height;
            // Launch a worker.
            let handle = tokio::spawn(async move{
                loop {
                    if *stop.read().await {
                        break;
                    }
                    if blocks.read().await.len() > 1000 {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    let height = {
                        let mut next_height = next_height.write().await;
                        let height = *next_height;
                        *next_height += 1;
                        height
                    };
                    if height > stop_height {
                        break;
                    }
                    let (block_hash, block) = Self::fetch_block(&rest, height).await;
                    blocks.write().await.insert(height, (block_hash, block));
                }
            });
            self.handles.push(handle);
        }
    }
    pub async fn stop(&mut self) {
        *self.stop.write().await = true;
        for handle in self.handles.iter_mut() {
            handle.await.unwrap();
        }
    }
}
