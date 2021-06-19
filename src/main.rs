use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;
use bitcoin::blockdata::block::Block;

use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use chainseeker_syncer::*;
use chainseeker_syncer::address_index::*;
use chainseeker_syncer::utxo::*;

const UTXO_DELETE_THRESHOLD: u32 = 20;
const DEFAULT_UTXO_SAVE_INTERVAL: u32 = 1000;

struct Syncer {
    addr_index_db: AddressIndexDB,
    utxo_db: UtxoDB,
    rest: bitcoin_rest::Context,
}

impl Syncer {
    fn new() -> Self {
        let addr_index_db = AddressIndexDB::new();
        let synced_height = addr_index_db.get_synced_height();
        Syncer{
            addr_index_db,
            utxo_db: match synced_height {
                Some(h) => UtxoDB::load(h),
                None => UtxoDB::new(),
            },
            rest: get_rest(),
        }
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
            ", #vin={:4}, #vout={:4}, #utxo={:9} (rest:{:3}ms, utxo:{:3}ms, addr:{:3}ms, total:{:3}ms)",
            vins, vouts, self.utxo_db.len(),
            rest_elapsed.as_millis(), utxo_elapsed.as_millis(), addr_index_elapsed.as_millis(), begin.elapsed().as_millis());
        if save {
            self.utxo_db.save(height);
            if height > UTXO_DELETE_THRESHOLD {
                let deleted_cnt = UtxoDB::delete_older_than(height - UTXO_DELETE_THRESHOLD);
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
            height = self.utxo_db.reorg(height);
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
        let _synced_blocks = self.sync(DEFAULT_UTXO_SAVE_INTERVAL).await;
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
    tokio::join!(
        // Run syncer.
        async {
            tokio::task::spawn(async {
                let mut syncer = Syncer::new();
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
