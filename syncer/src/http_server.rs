use std::cmp::min;
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
use bitcoin::Script;

use super::*;

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
        let block_header = &block.block_header;
        let mut hash = consensus_encode(&block_header.block_hash());
        hash.reverse();
        let hash = hex::encode(hash);
        Self {
            hash,
            time        : block_header.time,
            nonce       : block_header.nonce,
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
    // (height, RestBlockSummary)
    block_summary_cache: Arc<RwLock<HashMap<u32, RestBlockSummary>>>,
    pub block_db: Arc<RwLock<BlockDB>>,
    pub addr_index_db: Arc<RwLock<AddressIndexDB>>,
    pub utxo_server: Arc<RwLock<UtxoServer>>,
    pub rich_list: Arc<RwLock<RichList>>,
}

impl HttpServer {
    pub fn new(coin: &str) -> Self {
        Self{
            coin: coin.to_string(),
            block_summary_cache: Arc::new(RwLock::new(HashMap::new())),
            block_db: Arc::new(RwLock::new(BlockDB::new(coin, false))),
            addr_index_db: Arc::new(RwLock::new(AddressIndexDB::new(coin, false))),
            utxo_server: Arc::new(RwLock::new(UtxoServer::new(coin))),
            rich_list: Arc::new(RwLock::new(RichList::new())),
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
    /// `/block/:hash_or_height` endpoint.
    async fn block_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let hash_or_height = req.param("hash_or_height").unwrap();
        let block_content = if hash_or_height.len() == 64 {
            let block_hash = Vec::from_hex(hash_or_height);
            if block_hash.is_err() {
                return Ok(Self::not_found("Failed to decode input block hash."));
            }
            let mut block_hash = block_hash.unwrap();
            if block_hash.len() != 32 {
                return Ok(Self::not_found("Block hash has an invalid length."));
            }
            block_hash.reverse();
            let block_hash = consensus_decode(&block_hash[..]);
            server.block_db.read().await.get_by_hash(&block_hash)
        } else {
            let height = hash_or_height.parse();
            if height.is_err() {
                return Ok(Self::not_found("Failed to decode input block height."));
            }
            server.block_db.read().await.get(height.unwrap())
        };
        if block_content.is_none() {
            return Ok(Self::not_found("Block not found."));
        }
        Ok(Self::json(&block_content))
    }
    /// `/addr_index/:script` endpoint.
    async fn addr_index_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script_hex = req.param("script").unwrap();
        let script = Script::from_hex(script_hex);
        if script.is_err() {
            return Ok(Self::not_found("Failed to decode input script."));
        }
        let script = script.unwrap();
        let txids = server.addr_index_db.read().await.get(&script);
        let txids = txids.iter().map(|txid| txid.to_hex()).collect::<Vec<String>>();
        Ok(Self::json(&txids))
    }
    /// `/utxo/:script` endpoint.
    async fn utxo_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let script_hex = req.param("script").unwrap();
        let script = Script::from_hex(script_hex);
        if script.is_err() {
            return Ok(Self::not_found("Failed to decode input script."));
        }
        let script = script.unwrap();
        let values = server.utxo_server.read().await.get(&script).await;
        Ok(Self::json(&values))
    }
    /// `/rich_list/count` endpoint.
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
            .get("/block_summary/:offset/:limit", Self::block_summary_handler)
            .get("/block/:hash_or_height", Self::block_handler)
            .get("/addr_index/:script", Self::addr_index_handler)
            .get("/utxo/:script", Self::utxo_handler)
            .get("/rich_list/count", Self::rich_list_count_handler)
            .get("/rich_list/:offset/:limit", Self::rich_list_handler)
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
