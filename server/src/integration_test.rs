use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use routerify::prelude::*;
use routerify::{Router, RouterService};

use crate::*;

const COIN: &str = "integration";

#[derive(Debug, Clone)]
struct MockBitcoinCore {
    blocks: Vec<Block>,
}

impl Default for MockBitcoinCore {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBitcoinCore {
    pub fn new() -> Self {
        Self {
            blocks: fixtures::regtest_blocks().to_vec(),
        }
    }
    /// `/rest/chaininfo.json` endpoint.
    async fn chaininfo_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mock = req.data::<MockBitcoinCore>().unwrap();
        let last_block = &mock.blocks.last().unwrap();
        Ok(HttpServer::json(bitcoin_rest::ChainInfo {
            chain: "regtest".to_string(),
            blocks: mock.blocks.len() as u32 - 1,
            headers: mock.blocks.len() as u32 - 1,
            bestblockhash: last_block.block_hash().to_string(),
            difficulty: last_block.header.difficulty(Network::Regtest) as f64,
            mediantime: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32,
            verificationprogress: 1.0,
            chainwork: last_block.header.work().to_string(),
            pruned: false,
            pruneheight: 0,
            warnings: "".to_string(),
        }, false))
    }
    /// `/rest/headers/:count/:block_hash.bin` endpoint.
    async fn headers_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mock = req.data::<MockBitcoinCore>().unwrap();
        let count: usize = req.param("count").unwrap().parse().unwrap();
        let mut block_hash = req.param("block_hash.bin").unwrap().clone();
        block_hash.truncate(64);
        let block_hash = BlockHash::from_hex(&block_hash).unwrap();
        for (height, block) in mock.blocks.iter().enumerate() {
            if block.block_hash() == block_hash {
                let blocks = &mock.blocks[height..height+count];
                let blocks = blocks.iter().map(|block| consensus_encode(&block.header)).collect::<Vec<Vec<u8>>>().concat();
                return Ok(Response::builder().body(blocks.into()).unwrap());
            }
        }
        panic!("Block not found.");
    }
    /// `/rest/block/:block_hash.bin` endpoint.
    async fn block_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mock = req.data::<MockBitcoinCore>().unwrap();
        let mut block_hash = req.param("block_hash.bin").unwrap().clone();
        block_hash.truncate(64);
        let block_hash = BlockHash::from_hex(&block_hash).unwrap();
        for block in mock.blocks.iter() {
            if block.block_hash() == block_hash {
                return Ok(Response::builder().body(consensus_encode(block).into()).unwrap());
            }
        }
        panic!("Block not found.");
    }
    pub async fn run(&self) {
        let addr = SocketAddr::from((
            "127.0.0.1".parse::<std::net::IpAddr>().expect("Failed to parse HTTP IP address."),
            18443));
        let router = Router::builder()
            .data((*self).clone())
            .get("/rest/chaininfo.json", Self::chaininfo_handler)
            .get("/rest/headers/:count/:block_hash.bin", Self::headers_handler)
            .get("/rest/block/:block_hash.bin", Self::block_handler)
            .any(|req| async move {
                println!("{} {}", req.method(), req.uri());
                Ok(HttpServer::not_found("invalid URL."))
            })
            .err_handler_with_info(|err, _| async move {
                eprintln!("{}", err);
                HttpServer::internal_error(&format!("Something went wrong: {}", err))
            })
            .build()
            .unwrap();
        let service = RouterService::new(router).unwrap();
        let server = Server::bind(&addr).serve(service);
        if let Err(e) = server.await {
            panic!("MockBitcoinCore failed: {}", e);
        }
    }
}

/// Remove `~/.chainseeker/integration` directory
fn cleanup() {
    let path = format!("{}/{}", data_dir(), COIN);
    if std::path::Path::new(&path).exists() {
        std::fs::remove_dir_all(&path).unwrap();
    }
}

#[tokio::test]
async fn integration_test() {
    cleanup();
    // Load config.
    let mut config = config_example("rbtc");
    // Launch MockBitcoinCore.
    let mock: MockBitcoinCore = Default::default();
    config.genesis_block_hash = mock.blocks[0].block_hash();
    let mock_block_height = (mock.blocks.len() - 1) as i32;
    {
        tokio::spawn(async move {
            mock.run().await;
        });
    }
    // Launch main.
    {
        let config = config.clone();
        tokio::spawn(async move {
            main(COIN, &config).await
        });
    }
    let client = chainseeker::new(&format!("http://{}:{}/api", config.http_ip, config.http_port));
    // Wait until sync finishes.
    let mut retry_count = 0;
    loop {
        if retry_count > 100 {
            panic!("Maximum retry count reached.");
        }
        if let Ok(status) = client.status().await {
            println!("Synced blocks: {}", status.blocks);
            if status.blocks >= mock_block_height {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        retry_count += 1;
    }
    cleanup();
}
