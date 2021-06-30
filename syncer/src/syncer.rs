use std::time::Instant;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;

use super::*;

pub struct Syncer {
    coin: String,
    config: Config,
    utxo_db: UtxoDB,
    rich_list_builder: RichListBuilder,
    rest: bitcoin_rest::Context,
    stop: Arc<RwLock<bool>>,
    pub http_server: HttpServer,
}

impl Syncer {
    pub async fn new(coin: &str, config: &Config) -> Self {
        let rest = get_rest(&config.coins[coin]);
        let syncer = Self {
            coin: coin.to_string(),
            config: (*config).clone(),
            utxo_db: UtxoDB::new(coin, false),
            rich_list_builder: RichListBuilder::new(),
            rest: rest,
            stop: Arc::new(RwLock::new(false)),
            http_server: HttpServer::new(coin, config),
        };
        // Install Ctrl-C watch.
        let stop = syncer.stop.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
            *stop.write().await = true;
        });
        syncer
    }
    pub async fn is_stopped(&self) -> bool {
        *self.stop.read().await
    }
    fn coin_config(&self) -> &CoinConfig {
        &self.config.coins[&self.coin]
    }
    async fn process_block(&mut self, initial: bool, height: u32, block: &Block) {
        let begin = Instant::now();
        // Process for UTXOs.
        let begin_utxo = Instant::now();
        let previous_utxos = self.utxo_db.process_block(block, false);
        let utxo_elapsed = begin_utxo.elapsed();
        // Process for TxDB.
        let begin_tx = Instant::now();
        self.http_server.tx_db.write().await.process_block(height, block, &previous_utxos);
        let tx_elapsed = begin_tx.elapsed();
        // Process for address index.
        let begin_addr_index = Instant::now();
        self.http_server.addr_index_db.write().await.process_block(block, &previous_utxos);
        let addr_index_elapsed = begin_addr_index.elapsed();
        // Process if non initial-sync.
        if !initial {
            self.http_server.utxo_server.write().await.process_block(block, &previous_utxos).await;
            self.rich_list_builder.process_block(block, &previous_utxos);
            *self.http_server.rich_list.write().await = self.rich_list_builder.finalize();
        }
        // Count vins/vouts.
        let mut vins: usize = 0;
        let mut vouts: usize = 0;
        for tx in block.txdata.iter() {
            vins += tx.input.len();
            vouts += tx.output.len();
        }
        // Put best block information.
        self.http_server.block_db.write().await.put(height, &block);
        put_synced_height(&self.coin, height);
        println!(
            "Height={:6}, #tx={:4}, #vin={:5}, #vout={:5} (tx:{:3}ms, utxo:{:3}ms, addr:{:3}ms, total:{:4}ms)",
            height, block.txdata.len(), vins, vouts,
            tx_elapsed.as_millis(), utxo_elapsed.as_millis(),
            addr_index_elapsed.as_millis(), begin.elapsed().as_millis());
    }
    async fn process_reorgs(&mut self) {
        let mut height = match get_synced_height(&self.coin) {
            Some(h) => h,
            None => return (),
        };
        loop {
            let block_hash_rest = self.rest.blockhashbyheight(height).await
                .expect(&format!("Failed to fetch block at height = {}.", height));
            let block_me = self.http_server.block_db.read().await.get(height).unwrap();
            let block_hash_me = block_me.block_header.block_hash();
            if block_hash_rest == block_hash_me {
                break;
            }
            println!("Reorg detected at block height = {}.", height);
            // Fetch the reorged block.
            let block = self.rest.block(&block_hash_me).await.expect("Failed to fetch the reorged block from REST.");
            // Fetch previous transactions.
            let mut prev_txs = Vec::new();
            for tx in block.txdata.iter() {
                for vin in tx.input.iter() {
                    if vin.previous_output.is_null() {
                        continue;
                    }
                    let txid = &vin.previous_output.txid;
                    let prev_tx = self.rest.tx(txid).await.expect("Failed to fetch the previous transaction.");
                    prev_txs.push(prev_tx);
                }
            }
            self.utxo_db.reorg_block(&block, &prev_txs);
            height -= 1;
            put_synced_height(&self.coin, height);
        }
    }
    async fn sync(&mut self, initial: bool) -> u32 {
        self.process_reorgs().await;
        let start_height = match get_synced_height(&self.coin) {
            Some(h) => h + 1,
            None => 0,
        };
        let chaininfo = self.rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let target_height = chaininfo.blocks;
        let mut synced_blocks = 0;
        const BLOCK_QUEUE_SIZE: usize = 1000;
        let block_queue = Arc::new(RwLock::new(std::collections::VecDeque::with_capacity(BLOCK_QUEUE_SIZE)));
        {
            let block_queue = block_queue.clone();
            let stop = self.stop.clone();
            let rest = self.rest.clone();
            tokio::spawn(async move {
                let mut height = start_height;
                loop {
                    //let begin = Instant::now();
                    if *stop.read().await || height > target_height {
                        break;
                    }
                    let block_queue_len = block_queue.read().await.len();
                    if block_queue_len >= BLOCK_QUEUE_SIZE {
                        //println!("Block queue is full. Waiting for 100ms...");
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                    for _ in block_queue_len..BLOCK_QUEUE_SIZE {
                        let (_block_hash, block) = fetch_block(&rest, height).await;
                        block_queue.write().await.push_back(block);
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
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }
        synced_blocks
    }
    async fn load_utxo(&mut self) {
        let begin = Instant::now();
        let print_stat = |i: u32, force: bool| {
            if i % 1000 == 0 || force {
                print!("\rLoading UTXOs ({} entries processed)...", i);
                flush_stdout();
            }
        };
        let (utxo_server_tx, mut utxo_server_rx) = channel(1024 * 1024);
        let (rich_list_tx, mut rich_list_rx) = channel(1024 * 1024);
        let utxo_server = self.http_server.utxo_server.clone();
        let utxo_server_join = tokio::spawn(async move {
            let mut utxo_server = utxo_server.write().await;
            while let Some(utxo) = utxo_server_rx.recv().await {
                utxo_server.push(&utxo).await;
            }
        });
        let rich_list_join = tokio::task::spawn(async move {
            let mut rich_list_builder = RichListBuilder::new();
            while let Some(utxo) = rich_list_rx.recv().await {
                rich_list_builder.push(&utxo);
            }
            rich_list_builder
        });
        let mut i = 0;
        for utxo in self.utxo_db.iter() {
            if self.is_stopped().await {
                return;
            }
            print_stat(i, false);
            i += 1;
            // Ignore UTXO entries with zero value.
            if utxo.value <= 0 {
                continue;
            }
            utxo_server_tx.send(utxo.clone()).await.unwrap();
            rich_list_tx.send(utxo).await.unwrap();
        }
        print_stat(i, true);
        println!("");
        println!("Loaded all UTXOs in {}ms.", begin.elapsed().as_millis());
        // Wait async tasks to finish.
        drop(utxo_server_tx);
        drop(rich_list_tx);
        utxo_server_join.await.unwrap();
        let rich_list_builder = rich_list_join.await.unwrap();
        let rich_list = rich_list_builder.finalize();
        *self.http_server.rich_list.write().await = rich_list;
        self.rich_list_builder = rich_list_builder;
        println!("Syncer.load_utxo(): executed in {}ms.", begin.elapsed().as_millis());
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
        println!("Initial sync: synced {} blocks in {}ms.", synced_blocks, begin_elapsed);
        if !self.is_stopped().await {
            self.load_utxo().await;
        }
        synced_blocks
    }
    pub async fn run(&mut self) {
        // Subscribe to ZeroMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        let coin_config = self.coin_config();
        socket.connect(&coin_config.zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        println!("Waiting for a ZeroMQ message...");
        loop {
            if *self.stop.read().await {
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
            self.sync(false).await;
        }
        println!("Syncer stopped.");
    }
}
