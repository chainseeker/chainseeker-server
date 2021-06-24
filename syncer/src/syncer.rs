use std::time::Instant;
use std::sync::Arc;

use tokio::sync::RwLock;

use bitcoin::Block;

use super::*;

pub struct Syncer {
    coin: String,
    config: Config,
    block_db: BlockDB,
    addr_index_db: Arc<RwLock<AddressIndexDB>>,
    utxo_db: UtxoDB,
    utxo_server: Arc<RwLock<UtxoServer>>,
    rich_list: Arc<RwLock<RichList>>,
    rest: bitcoin_rest::Context,
    stop: Arc<RwLock<bool>>,
    block_fetcher: BlockFetcher,
}

impl Syncer {
    pub async fn new(coin: &str, config: &Config) -> Self {
        let utxo_db = UtxoDB::new(coin);
        let utxo_server = Arc::new(RwLock::new(UtxoServer::new()));
        let rich_list = Arc::new(RwLock::new(RichList::new()));
        let block_db = BlockDB::new(coin);
        let start_height = match block_db.get_synced_height() {
            Some(h) => h + 1,
            None => 0,
        };
        let rest = get_rest(&config.coins[coin]);
        let chaininfo = rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let stop_height = chaininfo.blocks;
        let mut syncer = Self {
            coin: coin.to_string(),
            config: (*config).clone(),
            block_db,
            addr_index_db: Arc::new(RwLock::new(AddressIndexDB::new(coin))),
            utxo_db,
            utxo_server,
            rich_list,
            rest,
            stop: Arc::new(RwLock::new(false)),
            block_fetcher: BlockFetcher::new(coin, config, start_height, stop_height),
        };
        syncer.reconstruct_utxo().await;
        syncer
    }
    pub fn addr_index_db(&self) -> Arc<RwLock<AddressIndexDB>> {
        self.addr_index_db.clone()
    }
    pub fn utxo_server(&self) -> Arc<RwLock<UtxoServer>> {
        self.utxo_server.clone()
    }
    pub fn rich_list(&self) -> Arc<RwLock<RichList>> {
        self.rich_list.clone()
    }
    fn coin_config(&self) -> &CoinConfig {
        &self.config.coins[&self.coin]
    }
    pub fn synced_height(&self) -> Option<u32> {
        self.block_db.get_synced_height()
    }
    async fn fetch_block(&mut self, height: u32) -> (BlockHash, Block) {
        self.block_fetcher.get(height).await
    }
    async fn process_block(&mut self, height: u32) {
        let begin = Instant::now();
        // Fetch block from REST.
        let begin_rest = Instant::now();
        let (block_hash, block) = self.fetch_block(height).await;
        let rest_elapsed = begin_rest.elapsed();
        // Process for UTXOs.
        let begin_utxo = Instant::now();
        let previous_pubkeys = self.utxo_db.process_block(&block, false);
        let utxo_elapsed = begin_utxo.elapsed();
        // Process for address index.
        let begin_addr_index = Instant::now();
        self.addr_index_db.write().await.process_block(&block, &previous_pubkeys);
        let addr_index_elapsed = begin_addr_index.elapsed();
        // Count vins/vouts.
        let mut vins: usize = 0;
        let mut vouts: usize = 0;
        for tx in block.txdata.iter() {
            vins += tx.input.len();
            vouts += tx.output.len();
        }
        // Put best block information.
        self.block_db.put_block_hash(height, &block_hash);
        self.block_db.put_synced_height(height);
        println!(
            "Height={:6}, #tx={:4}, #vin={:5}, #vout={:5} (rest:{:4}ms, utxo:{:3}ms, addr:{:3}ms, total:{:4}ms) (blocks:{:3})",
            height, block.txdata.len(), vins, vouts,
            rest_elapsed.as_millis(), utxo_elapsed.as_millis(),
            addr_index_elapsed.as_millis(), begin.elapsed().as_millis(), self.block_fetcher.len().await);
    }
    async fn process_reorgs(&mut self) {
        let mut height = match self.synced_height() {
            Some(h) => h,
            None => return (),
        };
        loop {
            let block_hash_rest = self.rest.blockhashbyheight(height).await
                .expect(&format!("Failed to fetch block at height = {}.", height));
            let block_hash_me = self.block_db.get_block_hash(height).unwrap();
            if block_hash_rest == block_hash_me {
                break;
            }
            println!("Reorg detected at block height = {}.", height);
            let block = self.rest.block(&block_hash_me).await.expect("Failed to fetch the reorged block from REST.");
            self.utxo_db.reorg_block(&self.rest, &block).await;
            height -= 1;
            self.block_db.put_synced_height(height);
        }
    }
    async fn sync(&mut self) -> u32 {
        self.process_reorgs().await;
        let start_height = match self.synced_height() {
            Some(h) => h + 1,
            None => 0,
        };
        let chaininfo = self.rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let target_height = chaininfo.blocks;
        let mut synced_blocks = 0;
        for height in start_height..(target_height + 1) {
            if *self.stop.read().await {
                break;
            }
            self.process_block(height).await;
            synced_blocks += 1;
        }
        synced_blocks
    }
    async fn reconstruct_utxo(&mut self) {
        let begin = Instant::now();
        let utxo: Utxo = (&self.utxo_db).into();
        let utxo = Arc::new(RwLock::new(utxo));
        let utxo1 = utxo.clone();
        let utxo_server_join = tokio::task::spawn(async move {
            let utxo = &*utxo1.read().await;
            UtxoServer::from(utxo)
        });
        let utxo2 = utxo.clone();
        let rich_list_join = tokio::task::spawn(async move {
            let utxo = &*utxo2.read().await;
            RichList::from(utxo)
        });
        *self.utxo_server.write().await = utxo_server_join.await.unwrap();
        *self.rich_list.write().await = rich_list_join.await.unwrap();
        println!("Syncer.reconstruct_utxo(): executed in {}ms.", begin.elapsed().as_millis());
    }
    pub async fn run(&mut self) {
        // Register Ctrl-C watch.
        let stop = self.stop.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
            println!("Ctrl-C was pressed. Exiting Syncer...");
            *stop.write().await = true;
        });
        // Run BlockFetcher.
        self.block_fetcher.run(None);
        // Do initial sync.
        let begin = Instant::now();
        let mut synced_blocks = 0;
        loop {
            let tmp = self.sync().await;
            synced_blocks += tmp;
            if tmp == 0 {
                break;
            }
        }
        let begin_elapsed = begin.elapsed().as_millis();
        println!("Initial sync: synced {} blocks in {}ms.", synced_blocks, begin_elapsed);
        if *self.stop.read().await {
            println!("Syncer stopped.");
            return;
        }
        if synced_blocks > 0 {
            self.reconstruct_utxo().await;
        }
        // Subscribe to ZeroMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        let coin_config = self.coin_config();
        socket.connect(&coin_config.zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        println!("Waiting for a ZeroMQ message...");
        loop {
            if *self.stop.read().await {
                println!("Exiting Syncer...");
                break;
            }
            match socket.recv_multipart(1) {
                Ok(multipart) => {
                    assert_eq!(multipart.len(), 3);
                    //let topic = std::str::from_utf8(&multipart[0]).expect("Failed to decode ZeroMQ topic.");
                    let blockhash = &multipart[1];
                    println!("Received a new block from ZeroMQ: {}", hex::encode(blockhash));
                },
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                },
            }
            self.sync().await;
            self.reconstruct_utxo().await;
        }
        println!("Syncer stopped.");
    }
}
