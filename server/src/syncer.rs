use crate::*;
use std::time::Instant;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;
use crate::db::{Database, UtxoDB};

pub struct Syncer {
    stop: Arc<RwLock<bool>>,
    rest: bitcoin_rest::Context,
    db: Database,
    utxo_db: UtxoDB,
}

impl Syncer {
    pub async fn new(db: Database) -> Self {
        let stop = Arc::new(RwLock::new(false));
        let rest = get_rest(&db.config);
        // Checks if we can access to Bitcoin Core's REST endpoint.
        rest.chaininfo().await.expect("Could not connect to Bitcoin Core's REST endpoint. Please check if the node is running, and listening on the correct IP address and port.");
        let utxo_db = UtxoDB::new(&db.coin, false);
        // Install Ctrl-C watch.
        {
            let stop = stop.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
                *stop.write().await = true;
            });
        }
        Self {
            stop,
            rest,
            db,
            utxo_db,
        }
    }
    pub async fn is_stopped(&self) -> bool {
        *self.stop.read().await
    }
    async fn shrink_to_fit(&mut self) {
        self.db.utxo_server.write().await.shrink_to_fit();
        self.db.rich_list.write().await.shrink_to_fit();
    }
    async fn process_block(&mut self, initial: bool, height: u32, block: &Block) {
        let begin = Instant::now();
        // Process for UTXOs.
        let begin_utxo = Instant::now();
        let previous_utxos = self.utxo_db.process_block(block, false);
        let utxo_elapsed = begin_utxo.elapsed();
        // Process for TxDB.
        let begin_tx = Instant::now();
        self.db.tx_db.write().await.process_block(height, block, &previous_utxos);
        let tx_elapsed = begin_tx.elapsed();
        // Process for address index.
        let begin_addr_index = Instant::now();
        self.db.addr_index_db.write().await.process_block(block, &previous_utxos);
        let addr_index_elapsed = begin_addr_index.elapsed();
        // Process if non initial-sync.
        if !initial {
            self.db.utxo_server.write().await.process_block(block, &previous_utxos);
            self.db.rich_list.write().await.process_block(block, &previous_utxos);
        }
        // Count vins/vouts.
        let mut vins: usize = 0;
        let mut vouts: usize = 0;
        for tx in block.txdata.iter() {
            vins += tx.input.len();
            vouts += tx.output.len();
        }
        // Put best block information.
        self.db.block_db.write().await.put(height, &block);
        self.db.synced_height_db.write().await.put(height);
        println!(
            "Height={}, #tx={:4}, #vin={:5}, #vout={:5} (tx:{:3}ms, utxo:{:3}ms, addr:{:3}ms, total:{:4}ms)",
            to_locale_string(height), block.txdata.len(), vins, vouts,
            tx_elapsed.as_millis(), utxo_elapsed.as_millis(),
            addr_index_elapsed.as_millis(), begin.elapsed().as_millis());
    }
    async fn process_reorgs(&mut self) {
        let mut height = match self.db.synced_height_db.read().await.get() {
            Some(h) => h,
            None => return,
        };
        loop {
            //let block_hash_rest = self.rest.blockhashbyheight(height).await
            //    .expect(&format!("Failed to fetch block at height = {}.", height));
            let block_me = self.db.block_db.read().await.get(height).unwrap();
            let block_hash_me = block_me.block_header.block_hash();
            //if block_hash_rest == block_hash_me {
            //    break;
            //}
            let block_headers = self.rest.headers(1, &block_hash_me).await.unwrap();
            if !block_headers.is_empty() {
                break;
            }
            println!("Reorg detected at block height = {}.", to_locale_string(height));
            // Fetch the reorged block.
            let block = self.db.block_db.read().await.get_by_hash(&block_hash_me).unwrap();
            let tx_db = self.db.tx_db.read().await;
            let block = bitcoin::Block {
                header: block.block_header,
                txdata: block.txids.iter().map(|txid| tx_db.get(txid).unwrap().tx).collect(),
            };
            // Fetch previous transactions.
            let mut prev_txs = Vec::new();
            for tx in block.txdata.iter() {
                for vin in tx.input.iter() {
                    if vin.previous_output.is_null() {
                        continue;
                    }
                    let prev_tx = tx_db.get(&vin.previous_output.txid).unwrap().tx;
                    prev_txs.push(prev_tx);
                }
            }
            self.utxo_db.reorg_block(&block, &prev_txs);
            height -= 1;
            self.db.synced_height_db.write().await.put(height);
        }
    }
    async fn sync(&mut self, initial: bool) -> u32 {
        self.process_reorgs().await;
        let start_height = match self.db.synced_height_db.read().await.get() {
            Some(h) => h + 1,
            None => 0,
        };
        let chaininfo = self.rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let target_height = chaininfo.blocks;
        let mut synced_blocks = 0;
        const BLOCK_QUEUE_SIZE: usize = 1000;
        let block_queue = Arc::new(RwLock::new(std::collections::VecDeque::with_capacity(BLOCK_QUEUE_SIZE)));
        if initial {
            let block_queue = block_queue.clone();
            let (start_height, start_block_hash) = if start_height == 0 {
                let block = self.rest.block(&self.db.config.genesis_block_hash).await.unwrap();
                block_queue.write().await.push_back(block);
                (1, self.db.config.genesis_block_hash)
            } else {
                let block_db_value = self.db.block_db.read().await.get(start_height - 1).unwrap();
                (start_height, block_db_value.block_header.block_hash())
            };
            let stop = self.stop.clone();
            let rest = self.rest.clone();
            tokio::spawn(async move {
                let mut current_block_hash = start_block_hash;
                let mut height = start_height;
                loop {
                    //let begin = Instant::now();
                    if *stop.read().await || height > target_height {
                        break;
                    }
                    let block_queue_len = block_queue.read().await.len();
                    if block_queue_len >= BLOCK_QUEUE_SIZE {
                        //println!("Block queue is full. Waiting for 100ms...");
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        continue;
                    }
                    let count = (BLOCK_QUEUE_SIZE - block_queue_len + 1) as u32;
                    let count = std::cmp::min(count, target_height - height + 2);
                    let block_headers = rest.headers(count, &current_block_hash).await.unwrap();
                    for block_header in block_headers[1..].iter() {
                        let block_hash = block_header.block_hash();
                        //assert_eq!(block_hash, rest.blockhashbyheight(height).await.unwrap());
                        let block = rest.block(&block_hash).await.unwrap();
                        block_queue.write().await.push_back(block);
                        current_block_hash = block_hash;
                        height += 1;
                    }
                    //println!("Block fetched in {}ms.", begin.elapsed().as_millis());
                }
            });
        }
        for height in start_height..(target_height + 1) {
            if self.is_stopped().await {
                break;
            }
            if initial {
                // Fetch block from REST.
                loop {
                    let block = block_queue.write().await.pop_front();
                    match block {
                        Some(block) => {
                            self.process_block(initial, height, &block).await;
                            synced_blocks += 1;
                            break;
                        },
                        None => {
                            println!("Block queue is empty. Waiting for blocks...");
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    }
                }
            } else {
                let block_db_value = self.db.block_db.read().await.get(height - 1).unwrap();
                let block_hash = block_db_value.block_header.block_hash();
                let block_headers = self.rest.headers(2, &block_hash).await.unwrap();
                let block_hash = block_headers[1].block_hash();
                let block = self.rest.block(&block_hash).await.unwrap();
                self.process_block(initial, height, &block).await;
            }
        }
        synced_blocks
    }
    async fn load_utxo(&mut self) {
        let begin = Instant::now();
        let mut last_print = Instant::now();
        let mut print_stat = |i: u32, force: bool| {
            if last_print.elapsed().as_millis() > 10 || force {
                print!("\rLoading UTXOs ({} entries processed)...", to_locale_string(i));
                flush_stdout();
                last_print = Instant::now();
            }
        };
        let (utxo_server_tx, mut utxo_server_rx) = channel(1024 * 1024);
        let (rich_list_tx, mut rich_list_rx) = channel(1024 * 1024);
        let utxo_server = self.db.utxo_server.clone();
        let utxo_server_join = tokio::spawn(async move {
            let mut utxo_server = utxo_server.write().await;
            while let Some(utxo) = utxo_server_rx.recv().await {
                utxo_server.push(&utxo);
            }
        });
        let rich_list = self.db.rich_list.clone();
        let rich_list_join = tokio::spawn(async move {
            let mut rich_list = rich_list.write().await;
            while let Some(utxo) = rich_list_rx.recv().await {
                rich_list.push(&utxo);
            }
            rich_list.finalize();
        });
        let mut i = 0;
        for utxo in self.utxo_db.iter() {
            if self.is_stopped().await {
                return;
            }
            print_stat(i, false);
            i += 1;
            // Ignore UTXO entries with zero value.
            if utxo.value == 0 {
                continue;
            }
            /*
            let address = Address::from_script(&utxo.script_pubkey, Network::Bitcoin);
            // Ignore script pubkeys which cannot be represented as an address.
            if address.is_none() {
                continue;
            }
            // Ignore non-standard addresses.
            if !address.unwrap().is_standard() {
                continue;
            }
            */
            utxo_server_tx.send(utxo.clone()).await.unwrap();
            rich_list_tx.send(utxo).await.unwrap();
        }
        print_stat(i, true);
        println!();
        println!("Loaded all UTXOs in {}ms.", to_locale_string(begin.elapsed().as_millis()));
        // Wait async tasks to finish.
        drop(utxo_server_tx);
        drop(rich_list_tx);
        utxo_server_join.await.unwrap();
        rich_list_join.await.unwrap();
        self.shrink_to_fit().await;
        println!("Syncer.load_utxo(): executed in {}ms.", to_locale_string(begin.elapsed().as_millis()));
    }
    pub async fn initial_sync(&mut self) -> u32 {
        // Do initial sync.
        let begin = Instant::now();
        let mut synced_blocks = 0;
        loop {
            let synced_blocks_now = self.sync(true).await;
            synced_blocks += synced_blocks_now;
            if synced_blocks_now == 0 {
                break;
            }
        }
        let begin_elapsed = begin.elapsed().as_millis();
        println!("Initial sync: synced {} blocks in {}ms.",
            to_locale_string(synced_blocks), to_locale_string(begin_elapsed));
        if !self.is_stopped().await {
            self.load_utxo().await;
            // Report the capacity / actual size.
            println!("(len, cap) = UtxoServer: ({}, {}), RichList: ({}, {})",
                to_locale_string(self.db.utxo_server.read().await.len()),
                to_locale_string(self.db.utxo_server.read().await.capacity()),
                to_locale_string(self.db.rich_list.read().await.len()),
                to_locale_string(self.db.rich_list.read().await.capacity()),
            );
            // Report the memory usage.
            println!("UtxoServer: {}MiB, RichList: {}MiB",
                self.db.utxo_server.read().await.size() / 1024 / 1024,
                self.db.rich_list.read().await.size() / 1024 / 1024,
            );
        }
        synced_blocks
    }
    pub async fn run(&mut self) {
        // Subscribe to ZeroMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        socket.connect(&self.db.config.zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        socket.set_subscribe(b"rawtx").expect("Failed to subscribe to a ZeroMQ topic.");
        println!("Waiting for a ZeroMQ message...");
        let mut last_sync = Instant::now();
        loop {
            if *self.stop.read().await {
                break;
            }
            // If we do not receive any block for some time (by missing ZMQ connection?), try to sync.
            const FORCE_SYNCE_THRESHOLD_SECS: u64 = 60;
            if last_sync.elapsed().as_secs() > FORCE_SYNCE_THRESHOLD_SECS {
                println!("No block received for {} secs, try syncing...", FORCE_SYNCE_THRESHOLD_SECS);
                self.sync(false).await;
                last_sync = Instant::now();
                continue;
            }
            let multipart = socket.recv_multipart(1);
            if multipart.is_err() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }
            let multipart = multipart.unwrap();
            assert_eq!(multipart.len(), 3);
            let topic = std::str::from_utf8(&multipart[0]).expect("Failed to decode ZeroMQ topic.");
            let bin = &multipart[1];
            match topic {
                "hashblock" => {
                    println!("Syncer: received a new block: {}.", hex::encode(bin));
                    self.sync(false).await;
                },
                "rawtx" => {
                    let tx: bitcoin::Transaction = consensus_decode(bin);
                    let txid = tx.txid();
                    println!("Syncer: received a new tx: {}.", txid);
                    if let Err(previous_txid) = self.db.tx_db.write().await.put_tx(&tx, None) {
                        println!("Syncer: failed to put transaction: {} (reason: tx {} not found).", txid, previous_txid);
                    }
                },
                _ => {
                    println!("Syncer: invalid topic received.");
                },
            }
        }
        println!("Syncer stopped.");
    }
}
