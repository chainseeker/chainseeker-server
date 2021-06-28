use std::time::Instant;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;

use super::*;

pub struct Syncer {
    coin: String,
    config: Config,
    block_db: BlockDB,
    utxo_db: UtxoDB,
    rich_list_builder: RichListBuilder,
    rest: bitcoin_rest::Context,
    stop: Arc<RwLock<bool>>,
    pub http_server: HttpServer,
}

impl Syncer {
    pub async fn new(coin: &str, config: &Config) -> Self {
        let syncer = Self {
            coin: coin.to_string(),
            config: (*config).clone(),
            block_db: BlockDB::new(coin),
            utxo_db: UtxoDB::new(coin, false),
            rich_list_builder: RichListBuilder::new(),
            rest: get_rest(&config.coins[coin]),
            stop: Arc::new(RwLock::new(false)),
            http_server: HttpServer::new(coin),
        };
        // Install Ctrl-C watch.
        let stop = syncer.stop.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
            println!("Ctrl-C was pressed. Exiting Syncer...");
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
    fn synced_height(&self) -> Option<u32> {
        self.block_db.get_synced_height()
    }
    async fn process_block(&mut self, block_fetcher: &mut Option<&mut BlockFetcher>, height: u32) {
        let begin = Instant::now();
        // Fetch block from REST.
        let begin_rest = Instant::now();
        let (block_hash, block) = match block_fetcher {
            Some(block_fetcher) => block_fetcher.get(height).await,
            None => BlockFetcher::fetch_block(&self.rest, height).await,
        };
        let rest_elapsed = begin_rest.elapsed();
        // Process for UTXOs.
        let begin_utxo = Instant::now();
        let previous_utxos = self.utxo_db.process_block(&block, false);
        let utxo_elapsed = begin_utxo.elapsed();
        // Process for address index.
        let begin_addr_index = Instant::now();
        self.http_server.addr_index_db.write().await.process_block(&block, &previous_utxos);
        let addr_index_elapsed = begin_addr_index.elapsed();
        // Process if non initial-sync.
        if block_fetcher.is_none() {
            self.http_server.utxo_server.write().await.process_block(&block, &previous_utxos).await;
            self.rich_list_builder.process_block(&block, &previous_utxos);
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
        self.block_db.put_block_hash(height, &block_hash);
        self.block_db.put_synced_height(height);
        println!(
            "Height={:6}, #tx={:4}, #vin={:5}, #vout={:5} (rest:{:4}ms, utxo:{:3}ms, addr:{:3}ms, total:{:4}ms){}",
            height, block.txdata.len(), vins, vouts,
            rest_elapsed.as_millis(), utxo_elapsed.as_millis(),
            addr_index_elapsed.as_millis(), begin.elapsed().as_millis(),
            match block_fetcher { Some(bf) => format!(" (blocks:{:3})", bf.len().await), None => "".to_string(), });
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
            self.block_db.put_synced_height(height);
        }
    }
    async fn sync(&mut self, block_fetcher: &mut Option<&mut BlockFetcher>) -> u32 {
        self.process_reorgs().await;
        let start_height = match self.synced_height() {
            Some(h) => h + 1,
            None => 0,
        };
        let chaininfo = self.rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let target_height = chaininfo.blocks;
        let mut synced_blocks = 0;
        for height in start_height..(target_height + 1) {
            if self.is_stopped().await {
                break;
            }
            self.process_block(block_fetcher, height).await;
            synced_blocks += 1;
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
    async fn block_fetcher(&self) -> BlockFetcher {
        let start_height = match self.synced_height() {
            Some(h) => h + 1,
            None => 0,
        };
        let chaininfo = self.rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let stop_height = chaininfo.blocks;
        BlockFetcher::new(&self.coin, &self.config, start_height, stop_height)
    }
    pub async fn initial_sync(&mut self) -> u32 {
        // Do initial sync.
        let begin = Instant::now();
        let mut synced_blocks = 0;
        loop {
            // Run BlockFetcher.
            let mut block_fetcher = self.block_fetcher().await;
            block_fetcher.run(None);
            let synced_blocks_now = self.sync(&mut Some(&mut block_fetcher)).await;
            synced_blocks += synced_blocks_now;
            if synced_blocks_now == 0 {
                break;
            }
            block_fetcher.stop().await;
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
            self.sync(&mut None).await;
        }
        println!("Syncer stopped.");
    }
}
