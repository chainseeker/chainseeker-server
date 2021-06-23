use std::io::Read;
use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;

use bitcoin_hashes::hex::FromHex;
use bitcoin::consensus::Encodable;
use bitcoin::{Block, BlockHash, Script};

use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use chainseeker_syncer::*;

struct Syncer {
    coin: String,
    config: Config,
    block_db: BlockDB,
    addr_index_db: Arc<RwLock<AddressIndexDB>>,
    utxo_db: UtxoDB,
    utxo_server: Arc<RwLock<UtxoServer>>,
    rest: bitcoin_rest::Context,
    stop: Arc<RwLock<bool>>,
}

impl Syncer {
    fn new(coin: &str, config: &Config) -> Self {
        let utxo_db = UtxoDB::new(coin);
        let utxo_server = Arc::new(RwLock::new((&utxo_db).into()));
        Self {
            coin: coin.to_string(),
            config: (*config).clone(),
            block_db: BlockDB::new(coin),
            addr_index_db: Arc::new(RwLock::new(AddressIndexDB::new(coin))),
            utxo_db,
            utxo_server,
            rest: get_rest(&config.coins[coin]),
            stop: Arc::new(RwLock::new(false)),
        }
    }
    fn coin_config(&self) -> &CoinConfig {
        &self.config.coins[&self.coin]
    }
    pub async fn synced_height(&self) -> Option<u32> {
        self.block_db.get_synced_height()
    }
    async fn fetch_block(&self, height: u32) -> (BlockHash, Block) {
        let block_hash = self.rest.blockhashbyheight(height).await
            .expect(&format!("Failed to fetch block at height = {}.", height));
        let block = self.rest.block(&block_hash).await.expect(&format!("Failed to fetch a block with blockid = {}", block_hash));
        (block_hash, block)
    }
    async fn process_block(&mut self, height: u32) {
        let begin = Instant::now();
        // Fetch block from REST.
        let begin_rest = Instant::now();
        let (block_hash, block) = self.fetch_block(height).await;
        let rest_elapsed = begin_rest.elapsed();
        // Process for UTXOs.
        let begin_utxo = Instant::now();
        let previous_pubkeys = self.utxo_db.process_block(&block);
        let utxo_elapsed = begin_utxo.elapsed();
        // Process for address index.
        let begin_addr_index = Instant::now();
        self.addr_index_db.write().await.process_block(&block, previous_pubkeys);
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
            "Height={:6}, #tx={:4}, #vin={:5}, #vout={:5} (rest:{:4}ms, utxo:{:3}ms, addr:{:3}ms, total:{:4}ms)",
            height, block.txdata.len(), vins, vouts,
            rest_elapsed.as_millis(), utxo_elapsed.as_millis(),
            addr_index_elapsed.as_millis(), begin.elapsed().as_millis());
    }
    async fn process_reorgs(&mut self) {
        let mut height = match self.synced_height().await {
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
        let start_height = match self.synced_height().await {
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
    async fn construct_utxo_server(&mut self) {
        self.utxo_server.write().await.load_from_db(&self.utxo_db);
    }
    async fn run(&mut self) {
        // Register Ctrl-C watch.
        let stop = self.stop.clone();
        tokio::task::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
            println!("Ctrl-C was pressed. Exiting syncer...");
            *stop.write().await = true;
        });
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
        println!("Initial sync: synced {} blocks in {}ms.", synced_blocks, begin.elapsed().as_millis());
        if synced_blocks > 0 {
            self.construct_utxo_server().await;
        }
        // Subscribe to ZeroMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        let coin_config = self.coin_config();
        socket.connect(&coin_config.zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        socket.set_subscribe(b"rawtx").expect("Failed to subscribe to a ZeroMQ topic.");
        loop {
            if *self.stop.read().await {
                println!("Exiting syncer...");
                break;
            }
            println!("Waiting for a ZeroMQ message...");
            let multipart = socket.recv_multipart(0).expect("Failed to receive a ZeroMQ message.");
            assert_eq!(multipart.len(), 3);
            let topic = std::str::from_utf8(&multipart[0]).expect("Failed to decode ZeroMQ topic.");
            if topic != "hashblock" {
                continue;
            }
            let blockhash = &multipart[1];
            println!("Received a new block from ZeroMQ: {}", hex::encode(blockhash));
            self.sync().await;
            self.construct_utxo_server().await;
        }
        println!("Syncer stopped.");
    }
}

struct HttpServer {
    addr_index_db: Arc<RwLock<AddressIndexDB>>,
    utxo_server: Arc<RwLock<UtxoServer>>,
}

impl HttpServer {
    pub fn new(addr_index_db: Arc<RwLock<AddressIndexDB>>, utxo_server: Arc<RwLock<UtxoServer>>) -> Self {
        Self{
            addr_index_db,
            utxo_server,
        }
    }
    fn response(status: &StatusCode, body: String) -> Response<Body> {
        Response::builder()
            .header("Content-Type", "application/json")
            .status(status)
            .body(body.into())
            .unwrap()
    }
    fn error(status: &StatusCode, msg: &str) -> Response<Body> {
        Self::response(status, format!("{{\"error\":\"{}\"}}", msg))
    }
    fn not_found(msg: &str) -> Response<Body> {
        Self::error(&StatusCode::NOT_FOUND, msg)
    }
    fn internal_error(msg: &str) -> Response<Body> {
        Self::error(&StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
    fn ok(json: String) -> Response<Body> {
        Self::response(&StatusCode::OK, json)
    }
    /// `/addr_index/SCRIPT` endpoint.
    async fn addr_index(addr_index_db: &Arc<RwLock<AddressIndexDB>>, hex: &str) -> Response<Body> {
        let script = Script::from_hex(hex);
        match script {
            Ok(script) => {
                let txids = addr_index_db.read().await.get(&script);
                let mut success = true;
                let txids: Vec<String> = txids.iter().map(|txid| {
                    let mut buf: [u8; 32] = [0; 32];
                    match txid.consensus_encode(&mut buf[..]) {
                        Ok(_) => {},
                        Err(_) => { success = false },
                    };
                    hex::encode(buf)
                }).collect();
                if !success {
                    return Self::internal_error("Failed to encode txids.");
                }
                let json = serde_json::to_string(&txids);
                match json {
                    Ok(json) => return Self::ok(json),
                    Err(_) => return Self::internal_error("Failed to encode to JSON."),
                };
            },
            Err(_) => return Self::not_found("Failed to decode input script."),
        }
    }
    /// `/utxo/SCRIPT` endpoint.
    async fn utxo(utxo_server: &Arc<RwLock<UtxoServer>>, hex: &str) -> Response<Body> {
        let script = Script::from_hex(hex);
        match script {
            Ok(script) => {
                let utxo_server = utxo_server.read().await;
                let values = utxo_server.get(&script);
                let json = serde_json::to_string(&values);
                match json {
                    Ok(json) => return Self::ok(json),
                    Err(_) => return Self::internal_error("Failed to encode to JSON."),
                };
            },
            Err(_) => return Self::not_found("Failed to decode input script."),
        }
    }
    async fn route(
        addr_index_db: &Arc<RwLock<AddressIndexDB>>,
        utxo_server: &Arc<RwLock<UtxoServer>>,
        req: &Request<Body>) -> Response<Body> {
        if req.method() != Method::GET {
            return Self::not_found("Invalid HTTP method.");
        }
        let path: Vec<&str> = req.uri().path().split('/').collect();
        if path.len() < 3 {
            return Self::not_found("Invalid number of params.");
        }
        if path[1] == "addr_index" {
            return Self::addr_index(addr_index_db, path[2]).await;
        }
        if path[1] == "utxo" {
            return Self::utxo(utxo_server, path[2]).await;
        }
        Self::not_found("Invalid API.")
    }
    async fn handle_request(
        addr_index_db: Arc<RwLock<AddressIndexDB>>,
        utxo_server: Arc<RwLock<UtxoServer>>,
        req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let begin = Instant::now();
        let res = Self::route(&addr_index_db, &utxo_server, &req).await;
        println!("HTTP: {} {} {}us.", req.method(), req.uri().path(), begin.elapsed().as_micros());
        Ok(res)
    }
    async fn run(&self, coin: &str, config: &Config) {
        let ip = &config.http_ip;
        let port = config.coins[coin].http_port;
        let addr = SocketAddr::from((
            ip.parse::<std::net::IpAddr>().expect("Failed to parse HTTP IP address."),
            port));
        let make_svc = make_service_fn(move |_conn| {
            let addr_index_db = self.addr_index_db.clone();
            let utxo_server = self.utxo_server.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    Self::handle_request(addr_index_db.clone(), utxo_server.clone(), req)
                }))
            }
        });
        let server = Server::bind(&addr).serve(make_svc);
        println!("HTTP server is listening on http://{}:{}/", ip, port);
        let graceful = server.with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C signal handler.");
        });
        if let Err(e) = graceful.await {
            panic!("HttpServer failed: {}", e);
        }
        println!("HTTP server stopped.");
    }
}

fn load_config() -> Config {
    let config_path = format!("{}/config.toml", get_data_dir_path().expect("Failed to get data directory path."));
    let mut config_file = std::fs::File::open(&config_path)
        .expect("Failed to open config file.\nPlease copy \"config.example.toml\" to \"~/.chainseeker/config.toml\".");
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str).expect("Failed to read config file.");
    toml::from_str(&config_str).expect("Failed to parse config file.")
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} COIN", args[0]);
        return;
    }
    let config = load_config();
    let coin = args[1].to_string();
    let mut syncer = Syncer::new(&coin, &config);
    let addr_index_db = syncer.addr_index_db.clone();
    let utxo_server = syncer.utxo_server.clone();
    let mut handles = Vec::new();
    handles.push(tokio::spawn(async move {
        syncer.run().await;
    }));
    handles.push(tokio::spawn(async move {
        let server = HttpServer::new(addr_index_db, utxo_server);
        server.run(&coin, &config).await;
    }));
    // Join for the threads.
    for handle in handles.iter_mut() {
        handle.await.expect("Failed to await a tokio JoinHandle.");
    }
}
