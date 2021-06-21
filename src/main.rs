use std::io::Read;
use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;

use bitcoin::blockdata::block::Block;

use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use chainseeker_syncer::*;
use chainseeker_syncer::address_index::*;
use chainseeker_syncer::utxo::*;

struct Syncer {
    coin: String,
    config: Config,
    addr_index_db: AddressIndexDB,
    utxo_db: UtxoDB,
    rest: bitcoin_rest::Context,
}

impl Syncer {
    fn new(coin: &str, config: &Config) -> Self {
        let addr_index_db = AddressIndexDB::new(coin);
        let synced_height = addr_index_db.get_synced_height();
        Self {
            coin: coin.to_string(),
            config: (*config).clone(),
            addr_index_db,
            utxo_db: match synced_height {
                Some(h) => UtxoDB::load(coin, h),
                None => UtxoDB::new(),
            },
            rest: get_rest(&config.coins[coin]),
        }
    }
    fn coin_config(&self) -> &CoinConfig {
        &self.config.coins[&self.coin]
    }
    pub fn synced_height(&self) -> Option<u32> {
        self.addr_index_db.get_synced_height()
    }
    async fn fetch_block(&self, height: u32) -> Block {
        let blockid = self.rest.blockhashbyheight(height).await
            .expect(&format!("Failed to fetch block at height = {}.", height));
        self.rest.block(blockid).await.expect(&format!("Failed to fetch a block with blockid = {}", blockid))
    }
    async fn process_block(&mut self, height: u32, save: bool) {
        let begin = Instant::now();
        print!("Height={:6}", height);
        let begin_rest = Instant::now();
        let block = self.fetch_block(height).await;
        let rest_elapsed = begin_rest.elapsed();
        print!(", #tx={:4}", block.txdata.len());
        let begin_utxo = Instant::now();
        let previous_pubkeys = self.utxo_db.process_block(&block);
        let utxo_elapsed = begin_utxo.elapsed();
        let begin_addr_index = Instant::now();
        self.addr_index_db.process_block(&block, previous_pubkeys);
        let addr_index_elapsed = begin_addr_index.elapsed();
        let mut vins: usize = 0;
        let mut vouts: usize = 0;
        for tx in block.txdata.iter() {
            vins += tx.input.len();
            vouts += tx.output.len();
        }
        println!(
            ", #vin={:5}, #vout={:5}, #utxo={:9} (rest:{:4}ms, utxo:{:3}ms, addr:{:3}ms, total:{:4}ms)",
            vins, vouts, self.utxo_db.len(),
            rest_elapsed.as_millis(), utxo_elapsed.as_millis(), addr_index_elapsed.as_millis(), begin.elapsed().as_millis());
        if save {
            self.utxo_db.save(&self.coin, height);
            if height > self.config.utxo_delete_threshold {
                let deleted_cnt = UtxoDB::delete_older_than(&self.coin, height - self.config.utxo_delete_threshold);
                println!("Deleted {} old UTXO database(s).", deleted_cnt);
            }
            self.addr_index_db.put_synced_height(height);
        }
    }
    async fn process_reorgs(&mut self) {
        let mut height = match self.synced_height() {
            Some(h) => h,
            None => return (),
        };
        loop {
            let block_hash_rest = self.rest.blockhashbyheight(height).await
                .expect(&format!("Failed to fetch block at height = {}.", height));
            let block_hash_me = self.utxo_db.block_hash.unwrap();
            if block_hash_rest == block_hash_me {
                break;
            }
            println!("Reorg detected at block height = {}.", height);
            height = self.utxo_db.reorg(&self.coin, height);
            self.addr_index_db.put_synced_height(height);
        }
    }
    async fn sync(&mut self, utxo_save_interval: u32) -> u32 {
        self.process_reorgs().await;
        let start_height = match self.synced_height() {
            Some(h) => h + 1,
            None => 0,
        };
        let chaininfo = self.rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let target_height = chaininfo.blocks;
        for height in start_height..(target_height + 1) {
            let save = ((height - start_height) % utxo_save_interval == utxo_save_interval - 1) || height == target_height;
            self.process_block(height, save).await;
        }
        target_height - start_height + 1
    }
    async fn run(&mut self) {
        // Do initial sync.
        let begin = Instant::now();
        let mut synced_blocks = 0;
        loop {
            let tmp = self.sync(self.config.default_utxo_save_interval).await;
            synced_blocks += tmp;
            if tmp == 0 {
                break;
            }
        }
        println!("Initial sync: synced {} blocks in {}ms.", synced_blocks, begin.elapsed().as_millis());
        // Subscribe to ZeroMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        let coin_config = self.coin_config();
        let zmq_endpoint = format!("tcp://{}:{}", coin_config.zmq_host, coin_config.zmq_port);
        socket.connect(&zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        loop {
            println!("Waiting for a ZeroMQ message...");
            let topic = socket.recv_string(0).expect("Failed to receive a ZeroMQ topic.").unwrap();
            let blockhash = socket.recv_bytes(0).expect("Failed to receive a blockhash from ZeroMQ.");
            println!("Received ZeroMQ message: {}: {:02x?}", topic, blockhash);
            self.sync(1).await;
        }
    }
}

struct HttpServer {
}

impl HttpServer {
    pub fn new() -> Self {
        Self{
        }
    }
    async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let not_found: Result<Response<Body>, Infallible> = {
            let mut res: Response<Body> = Response::new("404 Not Found".into());
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        };
        if req.method() != Method::GET {
            return not_found;
        }
        let path: Vec<&str> = req.uri().path().split('/').collect();
        if path.len() < 2 {
            return not_found;
        }
        /*
        if path[1] == "addr_index" {
            let script = Script::from_str(path[2]);
            let txids = addr_index
        }
        */
        Ok(Response::new("Hello, world!\n".into()))
    }
    async fn run(&self) {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8090));
        let make_svc = make_service_fn(|_conn| async {
            Ok::<_, Infallible>(service_fn(Self::handle_request))
        });
        let server = Server::bind(&addr).serve(make_svc);
        if let Err(e) = server.await {
            panic!("HttpServer failed: {}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} COIN=(btc|tbtc)", args[0]);
        return;
    }
    let config_path = format!("{}/config.toml", get_data_dir_path().expect("Failed to get data directory path."));
    let mut config_file = std::fs::File::open(&config_path)
        .expect("Failed to open config file.\nPlease copy \"config.example.toml\" to \"~/.chainseeker/config.toml\".");
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str).expect("Failed to read config file.");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config file.");
    tokio::join!(
        // Run syncer.
        async {
            let coin = args[1].to_string();
            tokio::task::spawn(async move {
                let mut syncer = Syncer::new(&coin, &config);
                syncer.run().await;
            }).await.unwrap();
        },
        // Run HTTP server.
        async {
            tokio::task::spawn(async {
                let server = HttpServer::new();
                server.run().await;
            }).await.unwrap();
        }
    );
}
