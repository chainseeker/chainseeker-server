use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use routerify::prelude::*;
use routerify::{Router, RouterService};
use jsonrpc_http_server::{ServerBuilder};
use jsonrpc_http_server::jsonrpc_core::{Value, IoHandler, Params};
use bitcoin::Address;

use crate::*;

const COIN: &str = "integration";

#[derive(Debug, Clone)]
struct MockBitcoinCoreRest {
    blocks: Vec<Block>,
}

impl Default for MockBitcoinCoreRest {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBitcoinCoreRest {
    fn new() -> Self {
        Self {
            blocks: fixtures::regtest_blocks().to_vec(),
        }
    }
    /// `/rest/chaininfo.json` endpoint.
    async fn chaininfo_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mock = req.data::<MockBitcoinCoreRest>().unwrap();
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
        let mock = req.data::<MockBitcoinCoreRest>().unwrap();
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
        let mock = req.data::<MockBitcoinCoreRest>().unwrap();
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
    async fn run(&self) {
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
            panic!("MockBitcoinCoreRest failed: {}", e);
        }
    }
}

struct MockBitcoinCoreRpc {
}

impl MockBitcoinCoreRpc {
    fn new() -> Self {
        Self {
        }
    }
    fn run(&self) {
        let mut io = IoHandler::default();
        io.add_sync_method("sendrawtransaction", |params: Params| {
            println!("MockBitcoinCoreRpc: {:?}", params);
            let values: Vec<String> = params.parse().unwrap();
            let tx: bitcoin::Transaction = consensus_decode(&hex::decode(&values[0]).unwrap());
            Ok(Value::String(tx.txid().to_string()))
        });
        ServerBuilder::new(io).start_http(&"127.0.0.1:18444".parse().unwrap()).unwrap().wait();
    }
}

impl Default for MockBitcoinCoreRpc {
    fn default() -> Self {
        Self::new()
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
    config.rpc_endpoint = "http://127.0.0.1:18444".to_string();
    // Launch MockBitcoinCoreRest.
    let rest = MockBitcoinCoreRest::default();
    let blocks = rest.blocks.clone();
    config.genesis_block_hash = blocks[0].block_hash();
    let mock_block_height = (blocks.len() - 1) as i32;
    {
        tokio::spawn(async move {
            rest.run().await;
        });
    }
    // Launch MockBitcoinCoreRpc.
    {
        std::thread::spawn(|| {
            MockBitcoinCoreRpc::default().run();
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
    // Call APIs.
    let txid = blocks[0].txdata[0].txid();
    let best_block_hash = blocks.last().unwrap().block_hash().to_string();
    let address = Address::from_script(
        &blocks.last().unwrap().txdata[0].output[0].script_pubkey, Network::Regtest).unwrap().to_string();
    const NOT_FOUND_ADDRESS: &str = "bcrt1qe2g3cvljrgky86djautz8u3wvjzm90023atvyf";
    const INVALID_ADDRESS: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    const NOT_FOUND_ID: &str = "012345678abcdef012345678abcdef012345678abcdef012345678abcdef0123";
    const INVALID_ID: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    //
    // Fetch transaction (success).
    assert_eq!(client.tx(&txid.to_string()).await.unwrap().txid, txid.to_string());
    // Fetch transaction (fail).
    assert!(client.tx(NOT_FOUND_ID).await.is_err());
    //
    // Fetch block with txs at height = 0.
    assert_eq!(client.block_with_txs(0u32).await.unwrap().hash, blocks[0].block_hash().to_string());
    // Fetch best block with txs by height.
    assert_eq!(client.block_with_txs(blocks.len() - 1).await.unwrap().hash, best_block_hash);
    // Fetch invalid block by height.
    assert!(client.block_with_txs(blocks.len()).await.is_err());
    //
    // Fetch block with txids by hash (success).
    assert_eq!(client.block_with_txids(&best_block_hash).await.unwrap().hash, best_block_hash);
    // Fetch block with txids by hash (not found).
    assert!(client.block_with_txids(NOT_FOUND_ID).await.is_err());
    // Fetch block with txids by hash (invalid block hash).
    assert!(client.block_with_txids(INVALID_ID).await.is_err());
    //
    // Fetch block header at height = 0.
    assert_eq!(client.block_header(0u32).await.unwrap().hash, blocks[0].block_hash().to_string());
    // Fetch invalid block header.
    assert!(client.block_header(blocks.len()).await.is_err());
    // Fetch invalid block header.
    assert!(client.block_header("invalid").await.is_err());
    //
    // Fetch transactions (success).
    assert!(!client.txs(&address).await.unwrap().is_empty());
    // Fetch transactions (fail).
    assert!(client.txs(INVALID_ADDRESS).await.is_err());
    //
    // Fetch txids (success).
    assert!(!client.txids(&address).await.unwrap().is_empty());
    // Fetch txids (fail).
    assert!(client.txids(INVALID_ADDRESS).await.is_err());
    //
    // Fetch utxos (success).
    assert!(!client.utxos(&address).await.unwrap().is_empty());
    // Fetch txids (fail).
    assert!(client.utxos(INVALID_ADDRESS).await.is_err());
    //
    // Fetch block summary (success).
    assert_eq!(client.block_summary(0, blocks.len() as u32).await.unwrap().len(), blocks.len());
    // Fetch block summary (success, again from cache).
    assert_eq!(client.block_summary(0, blocks.len() as u32 + 100).await.unwrap().len(), blocks.len());
    // Fetch block summary (invalid offset).
    assert!(client.get::<Vec<chainseeker::BlockSummary>>("block_summary/invalid/3").await.is_err());
    // Fetch block summary (invalid limit).
    assert!(client.get::<Vec<chainseeker::BlockSummary>>("block_summary/0/invalid").await.is_err());
    //
    // Fetch rich list count.
    assert!(client.rich_list_count().await.unwrap().count > 0);
    // Fetch address rank (success).
    assert!(client.rich_list_addr_rank(&address).await.unwrap().rank > 0);
    // Fetch address rank (not found).
    assert!(client.rich_list_addr_rank(NOT_FOUND_ADDRESS).await.is_err());
    // Fetch address rank (invalid address).
    assert!(client.rich_list_addr_rank(INVALID_ADDRESS).await.is_err());
    // Fetch rich list (success).
    assert!(!client.rich_list(0, 3).await.unwrap().is_empty());
    // Fetch rich list (invalid offset).
    assert!(client.get::<Vec<Option<chainseeker::RichListEntry>>>("rich_list/invalid/3").await.is_err());
    // Fetch rich list (invalid limit).
    assert!(client.get::<Vec<Option<chainseeker::RichListEntry>>>("rich_list/0/invalid").await.is_err());
    //
    // Put transaction.
    let secp256k1 = bitcoin::secp256k1::Secp256k1::new();
    let privkey = bitcoin::PrivateKey::from_wif("cUVAkHac2bPhiJRm77nxFPj4TSejT3JzE8fhjmbtUfNUeA4Sfq2v").unwrap();
    let pubkey = privkey.public_key(&secp256k1);
    let wallet = chainseeker::Wallet {
        address_type: chainseeker::AddressType::P2wpkh,
        private_keys: vec![privkey],
    };
    let change = bitcoin::Script::new_v0_wpkh(&pubkey.wpubkey_hash().unwrap());
    let tx = client.generate_tx(&wallet, &[], &change, 100, bitcoin::Network::Regtest).await.unwrap();
    println!("Txid: {}", tx.txid());
    let tx_hex = hex::encode(consensus_encode(&tx));
    println!("Tx: {}", tx_hex);
    assert_eq!(client.put_tx(tx_hex).await.unwrap().txid, tx.txid().to_string());
}
