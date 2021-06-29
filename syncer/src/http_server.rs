use std::cmp::min;
use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use bitcoin_hashes::hex::{FromHex, ToHex};
use bitcoin::Script;

use super::*;

#[derive(Debug, Clone)]
pub struct HttpServer {
    coin: String,
    pub block_db: Arc<RwLock<BlockDB>>,
    pub addr_index_db: Arc<RwLock<AddressIndexDB>>,
    pub utxo_server: Arc<RwLock<UtxoServer>>,
    pub rich_list: Arc<RwLock<RichList>>,
}

impl HttpServer {
    pub fn new(coin: &str) -> Self {
        Self{
            coin: coin.to_string(),
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
    /// `/block/:block_hash` endpoint.
    async fn block_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let block_hash = Vec::from_hex(req.param("block_hash").unwrap());
        if block_hash.is_err() {
            return Ok(Self::not_found("Failed to decode input block hash."));
        }
        let mut block_hash = block_hash.unwrap();
        if block_hash.len() != 32 {
            return Ok(Self::not_found("Block hash has an invalid length."));
        }
        block_hash.reverse();
        let block_hash = consensus_decode(&block_hash[..]);
        let block_content = server.block_db.read().await.get_by_hash(&block_hash);
        if block_content.is_none() {
            return Ok(Self::not_found("Block not found."));
        }
        Ok(Self::json(&block_content))
    }
    /// `/blockbyheight/:height` endpoint.
    async fn blockbyheight_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let height = req.param("height").unwrap().parse();
        if height.is_err() {
            return Ok(Self::not_found("Failed to decode input block height."));
        }
        let block_content = server.block_db.read().await.get(height.unwrap());
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
            .get("/blockbyheight/:height", Self::blockbyheight_handler)
            .get("/block/:block_hash", Self::block_handler)
            .get("/addr_index/:script", Self::addr_index_handler)
            .get("/utxo/:script", Self::utxo_handler)
            .get("/rich_list/count", Self::rich_list_count_handler)
            .get("/rich_list/:offset/:limit", Self::rich_list_handler)
            .any(|_req| async {
                Ok(Self::not_found("invalid URL."))
            })
            .middleware(Middleware::post_with_info(|res, req_info| async move {
                let begin = req_info.context::<Instant>().unwrap();
                println!("HTTP: {} {} {} ({}us)",
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
        let graceful = server.with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C signal handler.");
        });
        if let Err(e) = graceful.await {
            panic!("HttpServer failed: {}", e);
        }
        println!("HTTP server stopped.");
    }
}
