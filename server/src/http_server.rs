use crate::*;
use std::str::FromStr;
use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use bitcoin::hashes::hex::{FromHex, ToHex};
use bitcoin::{Script, Address};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use chainseeker::*;
use crate::db::Database;
use crate::db::block::BlockContentDBValue;

#[derive(Debug, Clone)]
pub struct HttpServer {
    db: Database,
    // (height, BlockSummary)
    block_summary_cache: Arc<RwLock<HashMap<u32, BlockSummary>>>,
}

impl HttpServer {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            block_summary_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    fn response(status: &StatusCode, body: String, cacheable: bool) -> Response<Body> {
        let builder = Response::builder();
        let builder = if cacheable {
            builder
                .header("Cache-Control", "public, max-age=60, s-maxage=86400")
                .header("CDN-Cache-Control", "max-age=86400")
        } else {
            builder
                .header("Cache-Control", "no-store")
                .header("CDN-Cache-Control", "no-store")
        };
        builder
            .header("Content-Type", "application/json")
            .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .status(status)
            .body(body.into())
            .unwrap()
    }
    fn error(status: &StatusCode, msg: &str) -> Response<Body> {
        Self::response(status, format!("{{\"error\":\"{}\"}}", msg), false)
    }
    pub fn not_found(msg: &str) -> Response<Body> {
        Self::error(&StatusCode::NOT_FOUND, msg)
    }
    pub fn bad_request(msg: &str) -> Response<Body> {
        Self::error(&StatusCode::BAD_REQUEST, msg)
    }
    pub fn internal_error(msg: &str) -> Response<Body> {
        Self::error(&StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
    pub fn ok(json: String, cacheable: bool) -> Response<Body> {
        Self::response(&StatusCode::OK, json, cacheable)
    }
    pub fn json<S>(object: S, cacheable: bool) -> Response<Body>
        where S: serde::ser::Serialize,
    {
        let json = serde_json::to_string(&object);
        match json {
            Ok(json) => Self::ok(json, cacheable),
            Err(_) => Self::internal_error("Failed to encode to JSON."),
        }
    }
    /// `/status` endpoint.
    async fn status_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        Ok(Self::json(Status { blocks: server.db.synced_height_db.read().await.get().map_or(-1, |h| h as i32) }, false))
    }
    /// `/tx/:txid` endpoint.
    async fn tx_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let txid = match req.param("txid").unwrap().parse() {
            Ok(txid) => txid,
            Err(_) => return Ok(Self::not_found("Failed to decode txid.")),
        };
        match server.db.tx_db.read().await.get_as_rest(&txid, &server.db.config) {
            Some(tx) => {
                let cacheable = tx.confirmed_height.is_some();
                Ok(Self::json(tx, cacheable))
            },
            None => Ok(Self::not_found("Transaction not found.")),
        }
    }
    /// `/tx/broadcast` endpoint.
    async fn tx_broadcast_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let auth = Auth::UserPass(server.db.config.rpc_user.clone(), server.db.config.rpc_pass.clone());
        let rpc = Client::new(server.db.config.rpc_endpoint.clone(), auth).unwrap();
        let hex = hyper::body::to_bytes(req.into_body()).await.unwrap();
        let hex = String::from_utf8(hex.to_vec());
        if hex.is_err() {
            return Ok(Self::bad_request("Failed to parse input."));
        }
        match rpc.send_raw_transaction(hex.unwrap()) {
            Ok(txid) => Ok(Self::ok(format!("{{\"txid\":\"{}\"}}", txid), false)),
            Err(_) => Ok(Self::bad_request("Failed to broadcast transaction.")),
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
            let block = server.db.block_db.read().await.get(height);
            if block.is_none() {
                break;
            }
            let summary = create_block_summary(&block.unwrap());
            server.block_summary_cache.write().await.insert(height, summary.clone());
            ret.push(summary);
        }
        Ok(Self::json(&ret, true))
    }
    /// Helper function for `/block*` APIs.
    async fn block_content(req: &Request<Body>) -> Result<BlockContentDBValue, Response<Body>> {
        let server = req.data::<HttpServer>().unwrap();
        let hash_or_height = req.param("hash_or_height").unwrap();
        let block_content = if hash_or_height.len() == 64 {
            let block_hash = BlockHash::from_hex(hash_or_height);
            if block_hash.is_err() {
                return Err(Self::not_found("Failed to decode input block hash."));
            }
            server.db.block_db.read().await.get_by_hash(&block_hash.unwrap())
        } else {
            let height = hash_or_height.parse();
            if height.is_err() {
                return Err(Self::not_found("Failed to decode input block height."));
            }
            server.db.block_db.read().await.get(height.unwrap())
        };
        match block_content {
            Some(block_content) => Ok(block_content),
            None => Err(Self::not_found("Block not found.")),
        }
    }
    /// `/block_with_txids/:hash_or_height` endpoint.
    async fn block_with_txids_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        match Self::block_content(&req).await {
            Ok(block_content) => Ok(Self::json(create_block_with_txids(&block_content, &server.db.config), true)),
            Err(res) => Ok(res),
        }
    }
    /// `/block_with_txs/:hash_or_height` endpoint.
    async fn block_with_txs_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let tx_db = server.db.tx_db.read().await;
        match Self::block_content(&req).await {
            Ok(block_content) => Ok(Self::json(create_block_with_txs(&tx_db, &block_content, &server.db.config), true)),
            Err(res) => Ok(res),
        }
    }
    /// `/block/:hash_or_height` endpoint.
    async fn block_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        match Self::block_content(&req).await {
            Ok(block_content) => Ok(Self::json(create_block_header(&block_content, &server.db.config), true)),
            Err(res) => Ok(res),
        }
    }
    fn decode_script_or_address(script_or_address: &str) -> Option<Script> {
        match Address::from_str(script_or_address) {
            Ok(addr) => return Some(addr.script_pubkey()),
            Err(err) => {
                println!("Failed to decode address: {}.", err);
            }
        }
        Script::from_hex(script_or_address).ok()
    }
    /// `/txids/:script_or_address` endpoint.
    async fn txids_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script = Self::decode_script_or_address(req.param("script_or_address").unwrap());
        if script.is_none() {
            return Ok(Self::not_found("Failed to decode input script or address."));
        }
        let txids = server.db.addr_index_db.read().await.get(&script.unwrap());
        let txids = txids.iter().map(|txid| txid.to_hex()).collect::<Vec<String>>();
        Ok(Self::json(&txids, false))
    }
    /// `/txs/:script_or_address` endpoint.
    async fn txs_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script = Self::decode_script_or_address(req.param("script_or_address").unwrap());
        if script.is_none() {
            return Ok(Self::not_found("Failed to decode input script or address."));
        }
        let txids = server.db.addr_index_db.read().await.get(&script.unwrap());
        let tx_db = server.db.tx_db.read().await;
        let mut txids_not_found = Vec::new();
        let txs = txids.iter().map(|txid| {
            match tx_db.get_as_rest(txid, &server.db.config) {
                Some(tx) => Some(tx),
                None => {
                    txids_not_found.push(txid.to_string());
                    None
                },
            }
        }).collect::<Vec<Option<chainseeker::Transaction>>>();
        if !txids_not_found.is_empty() {
            return Ok(Self::internal_error(&format!("Failed to resolve transactions: {}.", txids_not_found.join(", "))));
        }
        let txs: Vec<chainseeker::Transaction> = txs.into_iter().map(|x| x.unwrap()).collect();
        Ok(Self::json(&txs, false))
    }
    /// `/utxos/:script_or_address` endpoint.
    async fn utxos_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script = Self::decode_script_or_address(req.param("script_or_address").unwrap());
        if script.is_none() {
            return Ok(Self::not_found("Failed to decode input script or address."));
        }
        let tx_db = server.db.tx_db.read().await;
        let values = server.db.utxo_server.read().await.get(&script.unwrap());
        let mut utxos: Vec<Utxo> = Vec::new();
        for utxo in values.iter() {
            match tx_db.get(&utxo.txid) {
                Some(tx_db_value) => utxos.push(create_utxo(&utxo, &tx_db_value.tx, &server.db.config)),
                None => return Ok(Self::internal_error(&format!("Failed to resolve previous txid: {}", utxo.txid))),
            }
        };
        Ok(Self::json(&utxos, false))
    }
    /// `/rich_list_count` endpoint.
    async fn rich_list_count_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let json = format!("{{\"count\":{}}}", server.db.rich_list.read().await.len());
        Ok(Self::ok(json, false))
    }
    /// `/rich_list_addr_rank/:script_or_address` endpoint.
    async fn rich_list_addr_rank_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script = Self::decode_script_or_address(req.param("script_or_address").unwrap());
        if script.is_none() {
            return Ok(Self::not_found("Failed to decode input script or address."));
        }
        match server.db.rich_list.read().await.get_index_of(&script.unwrap()) {
            Some(rank) => Ok(Self::ok(format!("{{\"rank\":{}}}", rank + 1), false)),
            None => Ok(Self::ok("{\"rank\":null}".to_string(), false)),
        }
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
        let rich_list = server.db.rich_list.read().await;
        Ok(Self::json(&rich_list.get_in_range_as_rest(offset..offset+limit, &server.db.config), false))
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
            .get("/api/v1/txs/:script_or_address", Self::txs_handler)
            .get("/api/v1/utxos/:script_or_address", Self::utxos_handler)
            .get("/api/v1/rich_list_count", Self::rich_list_count_handler)
            .get("/api/v1/rich_list_addr_rank/:script_or_address", Self::rich_list_addr_rank_handler)
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
            let summary = create_block_summary(&block.unwrap());
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
