use std::cmp::min;
use std::str::FromStr;
use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use serde::Serialize;
use tokio::sync::RwLock;
use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use bitcoin_hashes::hex::{FromHex, ToHex};
use bitcoin::{Script, TxIn, TxOut, Address, Network, AddressType};
use bitcoin::blockdata::constants::WITNESS_SCALE_FACTOR;
use bitcoincore_rpc::{Auth, Client, RpcApi};

use super::*;

#[derive(Debug, Clone, Serialize)]
pub struct RestScriptSig {
    asm: String,
    hex: String,
}

impl RestScriptSig {
    pub fn new(script: &Script) -> Self {
        Self {
            asm: script.asm(),
            hex: script.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestVin {
    txid: String,
    vout: u32,
    script_sig: RestScriptSig,
    txinwitness: Vec<String>,
    sequence: u32,
    value: u64,
    address: Option<String>,
}

impl RestVin {
    pub fn new(txin: &TxIn, previous_txout: &Option<TxOut>, network: Network) -> Self {
        Self {
            txid: txin.previous_output.txid.to_string(),
            vout: txin.previous_output.vout,
            script_sig: RestScriptSig::new(&txin.script_sig),
            txinwitness: txin.witness.iter().map(|witness| hex::encode(consensus_encode(witness))).collect(),
            sequence: txin.sequence,
            value: match previous_txout {
                Some(pt) => pt.value,
                None => 0,
            },
            address: match previous_txout {
                Some(previous_txout) => match Address::from_script(&previous_txout.script_pubkey, network) {
                    Some(address) => Some(format!("{}", address)),
                    None => None,
                },
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestScriptPubKey {
    asm: String,
    hex: String,
    r#type: String,
    address: Option<String>,
}

impl RestScriptPubKey {
    pub fn new(script_pubkey: &Script, network: Network) -> Self {
        let address = Address::from_script(&script_pubkey, network);
        Self {
            asm: script_pubkey.asm(),
            hex: script_pubkey.to_string(),
            r#type: match address.clone() {
                Some(address) => match address.address_type() {
                    Some(address_type) => match address_type {
                        AddressType::P2pkh  => "pubkeyhash",
                        AddressType::P2sh   => "scripthash",
                        AddressType::P2wpkh => "witnesspubkeyhash",
                        AddressType::P2wsh  => "witnessscripthash",
                    },
                    None => "unknown",
                }
                None => "unknown",
            }.to_string(),
            address: match address {
                Some(address) => Some(format!("{}", address)),
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestVout {
    value: u64,
    n: usize,
    script_pub_key: RestScriptPubKey,
}

impl RestVout {
    pub fn new(txout: &TxOut, n: usize, network: Network) -> Self {
        Self {
            value: txout.value,
            n,
            script_pub_key: RestScriptPubKey::new(&txout.script_pubkey, network),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestTx {
    confirmed_height: Option<u32>,
    hex: String,
    txid: String,
    hash: String,
    size: usize,
    vsize: usize,
    weight: usize,
    version: i32,
    locktime: u32,
    vin: Vec<RestVin>,
    vout: Vec<RestVout>,
    fee: i64,
    //counterparty: ,
}

impl RestTx {
    pub fn from_tx_db_value(value: &TxDBValue, network: Network) -> Self {
        let tx = &value.tx;
        let mut input_value = 0;
        let mut vin = Vec::new();
        let mut previous_txout_index = 0;
        for input in tx.input.iter() {
            if input.previous_output.is_null() {
                vin.push(RestVin::new(input, &None, network));
            } else {
                input_value += value.previous_txouts[previous_txout_index].value;
                vin.push(RestVin::new(input, &Some(value.previous_txouts[previous_txout_index].clone()), network));
                previous_txout_index += 1;
            }
        }
        let output_value: u64 = tx.output.iter().map(|output| output.value).sum();
        Self {
            confirmed_height: value.confirmed_height,
            hex: hex::encode(&consensus_encode(tx)),
            txid: tx.txid().to_string(),
            hash: tx.wtxid().to_string(),
            size: tx.get_size(),
            // TODO: waiting for upstream merge.
            //vsize: tx.get_vsize(),
            vsize: (tx.get_weight() + WITNESS_SCALE_FACTOR - 1) / WITNESS_SCALE_FACTOR,
            weight: tx.get_weight(),
            version: tx.version,
            locktime: tx.lock_time,
            vin,
            vout: tx.output.iter().enumerate().map(|(n, vout)| RestVout::new(vout, n, network)).collect(),
            // TODO: compute for coinbase transactions!
            fee: (input_value as i64) - (output_value as i64),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockHeader {
    pub height: u32,
    pub header: String,
    pub hash: String,
    pub version: i32,
    pub previousblockhash: String,
    pub merkleroot: String,
    pub time: u32,
    pub bits: String,
    pub difficulty: u64,
    pub nonce: u32,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub ntxs: usize,
}

impl RestBlockHeader {
    pub fn from_block_content(block_content: &BlockContentDBValue, network: Network) -> Self {
        let block_header = &block_content.block_header;
        let mut hash = consensus_encode(&block_header.block_hash());
        hash.reverse();
        let mut prev_blockhash = consensus_encode(&block_header.prev_blockhash);
        prev_blockhash.reverse();
        let mut merkle_root = consensus_encode(&block_header.merkle_root);
        merkle_root.reverse();
        Self {
            height           : block_content.height,
            header           : hex::encode(consensus_encode(&block_header)),
            hash             : hex::encode(&hash),
            version          : block_header.version,
            previousblockhash: hex::encode(&prev_blockhash),
            merkleroot       : hex::encode(&merkle_root),
            time             : block_header.time,
            bits             : format!("{:x}", block_header.bits),
            difficulty       : block_header.difficulty(network),
            nonce            : block_header.nonce,
            size             : block_content.size,
            strippedsize     : block_content.strippedsize,
            weight           : block_content.weight,
            ntxs             : block_content.txids.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockWithTxids {
    pub height: u32,
    pub header: String,
    pub hash: String,
    pub version: i32,
    pub previousblockhash: String,
    pub merkleroot: String,
    pub time: u32,
    pub bits: String,
    pub difficulty: u64,
    pub nonce: u32,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub txids: Vec<String>,
}

impl RestBlockWithTxids {
    pub fn from_block_content(block_content: &BlockContentDBValue, network: Network) -> Self {
        let rest_block_header = RestBlockHeader::from_block_content(block_content, network);
        Self {
            height           : rest_block_header.height,
            header           : rest_block_header.header,
            hash             : rest_block_header.hash,
            version          : rest_block_header.version,
            previousblockhash: rest_block_header.previousblockhash,
            merkleroot       : rest_block_header.merkleroot,
            time             : rest_block_header.time,
            bits             : rest_block_header.bits,
            difficulty       : rest_block_header.difficulty,
            nonce            : rest_block_header.nonce,
            size             : rest_block_header.size,
            strippedsize     : rest_block_header.strippedsize,
            weight           : rest_block_header.weight,
            txids            : block_content.txids.iter().map(|txid| txid.to_hex()).collect::<Vec<String>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockWithTxs {
    pub height: u32,
    pub header: String,
    pub hash: String,
    pub version: i32,
    pub previousblockhash: String,
    pub merkleroot: String,
    pub time: u32,
    pub bits: String,
    pub difficulty: u64,
    pub nonce: u32,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub txs: Vec<RestTx>,
}

impl RestBlockWithTxs {
    pub fn from_block_content(tx_db: &TxDB, block_content: &BlockContentDBValue, network: Network) -> Self {
        let rest_block_header = RestBlockHeader::from_block_content(block_content, network);
        let txs = block_content.txids.iter().map(|txid| {
            let tx = tx_db.get(txid).unwrap();
            RestTx::from_tx_db_value(&tx, network)
        }).collect::<Vec<RestTx>>();
        Self {
            height           : rest_block_header.height,
            header           : rest_block_header.header,
            hash             : rest_block_header.hash,
            version          : rest_block_header.version,
            previousblockhash: rest_block_header.previousblockhash,
            merkleroot       : rest_block_header.merkleroot,
            time             : rest_block_header.time,
            bits             : rest_block_header.bits,
            difficulty       : rest_block_header.difficulty,
            nonce            : rest_block_header.nonce,
            size             : rest_block_header.size,
            strippedsize     : rest_block_header.strippedsize,
            weight           : rest_block_header.weight,
            txs,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockSummary {
    hash        : String,
    time        : u32,
    nonce       : u32,
    size        : u32,
    strippedsize: u32,
    weight      : u32,
    txcount     : usize,
}

impl RestBlockSummary {
    pub fn new(block: &BlockContentDBValue) -> Self {
        Self {
            hash        : block.block_header.block_hash().to_string(),
            time        : block.block_header.time,
            nonce       : block.block_header.nonce,
            size        : block.size,
            strippedsize: block.strippedsize,
            weight      : block.weight,
            txcount     : block.txids.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpServer {
    coin: String,
    config: Config,
    // (height, RestBlockSummary)
    block_summary_cache: Arc<RwLock<HashMap<u32, RestBlockSummary>>>,
    pub block_db: Arc<RwLock<BlockDB>>,
    pub tx_db: Arc<RwLock<TxDB>>,
    pub addr_index_db: Arc<RwLock<AddressIndexDB>>,
    pub utxo_server: Arc<RwLock<UtxoServer>>,
    pub rich_list: Arc<RwLock<RichList>>,
}

impl HttpServer {
    pub fn new(coin: &str, config: &Config) -> Self {
        Self{
            coin: coin.to_string(),
            config: (*config).clone(),
            block_summary_cache: Arc::new(RwLock::new(HashMap::new())),
            block_db: Arc::new(RwLock::new(BlockDB::new(coin, false))),
            tx_db: Arc::new(RwLock::new(TxDB::new(coin, false))),
            addr_index_db: Arc::new(RwLock::new(AddressIndexDB::new(coin, false))),
            utxo_server: Arc::new(RwLock::new(UtxoServer::new(coin))),
            rich_list: Arc::new(RwLock::new(RichList::new())),
        }
    }
    fn response(status: &StatusCode, body: String) -> Response<Body> {
        Response::builder()
            .header("Content-Type", "application/json")
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
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
    fn bad_request(msg: &str) -> Response<Body> {
        Self::error(&StatusCode::BAD_REQUEST, msg)
    }
    fn internal_error(msg: &str) -> Response<Body> {
        Self::error(&StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
    fn ok(json: String) -> Response<Body> {
        Self::response(&StatusCode::OK, json)
    }
    fn json<S>(object: S) -> Response<Body>
        where S: serde::ser::Serialize,
    {
        let json = serde_json::to_string(&object);
        match json {
            Ok(json) => Self::ok(json),
            Err(_) => Self::internal_error("Failed to encode to JSON."),
        }
    }
    fn network(coin: &str) -> Network {
        match coin {
            "btc"  => Network::Bitcoin,
            "tbtc" => Network::Testnet,
            "rbtc" => Network::Regtest,
            "sbtc" => Network::Signet,
            _ => panic!("Coin not supported."),
        }
    }
    /// `/status` endpoint.
    async fn status_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        Ok(Self::ok(format!("{{\"blocks\":{}}}", match get_synced_height(&server.coin) {
            Some(synced_height) => synced_height as i32,
            None => -1,
        })))
    }
    /// `/tx/:txid` endpoint.
    async fn tx_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let txid = match req.param("txid").unwrap().parse() {
            Ok(txid) => txid,
            Err(_) => return Ok(Self::not_found("Failed to decode txid.")),
        };
        let network = Self::network(&server.coin);
        match server.tx_db.read().await.get(&txid) {
            Some(value) => Ok(Self::json(RestTx::from_tx_db_value(&value, network))),
            None => Ok(Self::not_found("Transaction not found.")),
        }
    }
    /// `/tx/broadcast` endpoint.
    async fn tx_broadcast_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let coin_config = &server.config.coins[&server.coin];
        let auth = Auth::UserPass(coin_config.rpc_user.clone(), coin_config.rpc_pass.clone());
        let rpc = Client::new(coin_config.rpc_endpoint.clone(), auth).unwrap();
        let hex = hyper::body::to_bytes(req.into_body()).await.unwrap();
        let hex = String::from_utf8(hex.to_vec());
        if hex.is_err() {
            return Ok(Self::bad_request("Failed to parse input."));
        }
        match rpc.send_raw_transaction(hex.unwrap()) {
            Ok(txid) => Ok(Self::json(format!("{{\"txid\":\"{}\"}}", txid))),
            Err(_) => Ok(Self::bad_request(&format!("Failed to broadcast transaction."))),
        }
    }
    /// `/block_summary/:offset/:limit` endpoint.
    async fn block_summary_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let offset: u32 = match req.param("offset").unwrap().parse() {
            Ok(offset) => offset,
            Err(_) => return Ok(Self::bad_request("Cannot parse \"offset\" as an integer.")),
        };
        let limit: u32 = match req.param("limit").unwrap().parse() {
            Ok(limit) => limit,
            Err(_) => return Ok(Self::bad_request("Cannot parse \"limit\" as an integer.")),
        };
        let server = req.data::<HttpServer>().unwrap();
        let mut ret = Vec::new();
        for height in offset..offset+limit {
            {
                let block_summary_cache = server.block_summary_cache.read().await;
                let summary = block_summary_cache.get(&height);
                if summary.is_some() {
                    ret.push((*summary.unwrap()).clone());
                    continue;
                }
            }
            let block = server.block_db.read().await.get(height);
            if block.is_none() {
                break;
            }
            let summary = RestBlockSummary::new(&block.unwrap());
            server.block_summary_cache.write().await.insert(height, summary.clone());
            ret.push(summary);
        }
        Ok(Self::json(&ret))
    }
    /// Helper function for `/block*` APIs.
    async fn block_content(req: &Request<Body>) -> Result<BlockContentDBValue, Response<Body>> {
        let server = req.data::<HttpServer>().unwrap();
        let hash_or_height = req.param("hash_or_height").unwrap();
        let block_content = if hash_or_height.len() == 64 {
            let block_hash = Vec::from_hex(hash_or_height);
            if block_hash.is_err() {
                return Err(Self::not_found("Failed to decode input block hash."));
            }
            let mut block_hash = block_hash.unwrap();
            if block_hash.len() != 32 {
                return Err(Self::not_found("Block hash has an invalid length."));
            }
            block_hash.reverse();
            let block_hash = consensus_decode(&block_hash[..]);
            server.block_db.read().await.get_by_hash(&block_hash)
        } else {
            let height = hash_or_height.parse();
            if height.is_err() {
                return Err(Self::not_found("Failed to decode input block height."));
            }
            server.block_db.read().await.get(height.unwrap())
        };
        match block_content {
            Some(block_content) => Ok(block_content),
            None => Err(Self::not_found("Block not found.")),
        }
    }
    /// `/block_with_txids/:hash_or_height` endpoint.
    async fn block_with_txids_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let network = Self::network(&req.data::<HttpServer>().unwrap().coin);
        match Self::block_content(&req).await {
            Ok(block_content) => Ok(Self::json(RestBlockWithTxids::from_block_content(&block_content, network))),
            Err(res) => Ok(res),
        }
    }
    /// `/block_with_txs/:hash_or_height` endpoint.
    async fn block_with_txs_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let tx_db = server.tx_db.read().await;
        let network = Self::network(&server.coin);
        match Self::block_content(&req).await {
            Ok(block_content) => Ok(Self::json(RestBlockWithTxs::from_block_content(&tx_db, &block_content, network))),
            Err(res) => Ok(res),
        }
    }
    /// `/block/:hash_or_height` endpoint.
    async fn block_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let network = Self::network(&req.data::<HttpServer>().unwrap().coin);
        match Self::block_content(&req).await {
            Ok(block_content) => Ok(Self::json(RestBlockHeader::from_block_content(&block_content, network))),
            Err(res) => Ok(res),
        }
    }
    fn decode_script_or_address(script_or_address: &str) -> Result<Script, ()> {
        let addr = Address::from_str(script_or_address);
        if addr.is_ok() {
            return Ok(addr.unwrap().script_pubkey());
        }
        let script = Script::from_hex(script_or_address);
        if script.is_ok() {
            return Ok(script.unwrap());
        }
        Err(())
    }
    /// `/txids/:script_or_address` endpoint.
    async fn txids_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script = Self::decode_script_or_address(req.param("script_or_address").unwrap());
        if script.is_err() {
            return Ok(Self::not_found("Failed to decode input script or address."));
        }
        let txids = server.addr_index_db.read().await.get(&script.unwrap());
        let txids = txids.iter().map(|txid| txid.to_hex()).collect::<Vec<String>>();
        Ok(Self::json(&txids))
    }
    /// `/utxo/:script_or_address` endpoint.
    async fn utxo_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script = Self::decode_script_or_address(req.param("script_or_address").unwrap());
        if script.is_err() {
            return Ok(Self::not_found("Failed to decode input script or address."));
        }
        let values = server.utxo_server.read().await.get(&script.unwrap()).await;
        Ok(Self::json(&values))
    }
    /// `/rich_list_count` endpoint.
    async fn rich_list_count_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let json = format!("{{\"count\":{}}}", server.rich_list.read().await.len());
        Ok(Self::ok(json))
    }
    /// `/rich_list/:offset/:limit` endpoint.
    async fn rich_list_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let offset: usize = match req.param("offset").unwrap().parse() {
            Ok(offset) => offset,
            Err(_) => return Ok(Self::bad_request("Cannot parse \"offset\" as an integer.")),
        };
        let limit: usize = match req.param("limit").unwrap().parse() {
            Ok(limit) => limit,
            Err(_) => return Ok(Self::bad_request("Cannot parse \"limit\" as an integer.")),
        };
        let server = req.data::<HttpServer>().unwrap();
        let rich_list = server.rich_list.read().await;
        let begin = min(offset, rich_list.len() - 1usize);
        let end = min(offset + limit, rich_list.len() - 1usize);
        let addresses = rich_list.get_in_range(begin..end);
        Ok(Self::json(&addresses))
    }
    pub async fn run(&self, ip: &str, port: u16) {
        let addr = SocketAddr::from((
            ip.parse::<std::net::IpAddr>().expect("Failed to parse HTTP IP address."),
            port));
        let router = Router::builder()
            .data((*self).clone())
            .middleware(Middleware::pre(|req| async move {
                req.set_context(Instant::now());
                Ok(req)
            }))
            .get("/api/v1/status", Self::status_handler)
            .get("/api/v1/tx/:txid", Self::tx_handler)
            .put("/api/v1/tx/broadcast", Self::tx_broadcast_handler)
            .get("/api/v1/block_summary/:offset/:limit", Self::block_summary_handler)
            .get("/api/v1/block_with_txids/:hash_or_height", Self::block_with_txids_handler)
            .get("/api/v1/block_with_txs/:hash_or_height", Self::block_with_txs_handler)
            .get("/api/v1/block/:hash_or_height", Self::block_handler)
            .get("/api/v1/txids/:script_or_address", Self::txids_handler)
            .get("/api/v1/utxo/:script_or_address", Self::utxo_handler)
            .get("/api/v1/rich_list_count", Self::rich_list_count_handler)
            .get("/api/v1/rich_list/:offset/:limit", Self::rich_list_handler)
            .any(|_req| async {
                Ok(Self::not_found("invalid URL."))
            })
            .middleware(Middleware::post_with_info(|res, req_info| async move {
                let begin = req_info.context::<Instant>().unwrap();
                println!("HTTP: {} {} {} ({}ms)",
                    req_info.method(), req_info.uri().path(), res.status(), begin.elapsed().as_millis());
                Ok(res)
            }))
            .err_handler_with_info(|err, _| async move {
                eprintln!("{}", err);
                Self::internal_error(&format!("Something went wrong: {}", err))
            })
            .build()
            .unwrap();
        let service = RouterService::new(router).unwrap();
        let server = Server::bind(&addr).serve(service);
        println!("HTTP server is listening on http://{}:{}/", ip, port);
        // Fill BlockSummary cache.
        /*
        let mut height = 0;
        loop {
            let block = self.block_db.read().await.get(height);
            if block.is_none() {
                break;
            }
            let summary = RestBlockSummary::new(&block.unwrap());
            self.block_summary_cache.write().await.insert(height, summary);
            height += 1;
        }
        */
        let graceful = server.with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C signal handler.");
        });
        if let Err(e) = graceful.await {
            panic!("HttpServer failed: {}", e);
        }
        println!("HTTP server stopped.");
    }
}
