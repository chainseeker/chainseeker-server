use std::cmp::min;
use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;

use bitcoin_hashes::hex::FromHex;
use bitcoin::Script;

use hyper::{Body, Request, Response, Server, StatusCode};

use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};

use super::*;

#[derive(Debug, Clone)]
pub struct HttpServer {
    coin: String,
    pub addr_index_db: Arc<RwLock<AddressIndexDB>>,
    pub utxo_server: Arc<RwLock<UtxoServer>>,
    pub rich_list: Arc<RwLock<RichList>>,
}

impl HttpServer {
    pub fn new(coin: &str) -> Self {
        Self{
            coin: coin.to_string(),
            addr_index_db: Arc::new(RwLock::new(AddressIndexDB::new(coin))),
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
    /// `/addr_index/:script` endpoint.
    async fn addr_index_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let addr_index_db = &server.addr_index_db;
        let script_hex = req.param("script").unwrap();
        let script = Script::from_hex(script_hex);
        match script {
            Ok(script) => {
                let txids = addr_index_db.read().await.get(&script);
                let txids: Vec<String> = txids.iter().map(|txid| {
                    let mut txid = serialize_txid(&txid);
                    txid.reverse();
                    hex::encode(txid)
                }).collect();
                let json = serde_json::to_string(&txids);
                match json {
                    Ok(json) => return Ok(Self::ok(json)),
                    Err(_) => return Ok(Self::internal_error("Failed to encode to JSON.")),
                };
            },
            Err(_) => return Ok(Self::not_found("Failed to decode input script.")),
        }
    }
    /// `/utxo/:script` endpoint.
    async fn utxo_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let utxo_server = &server.utxo_server;
        let script_hex = req.param("script").unwrap();
        let script = Script::from_hex(script_hex);
        match script {
            Ok(script) => {
                let utxo_server = utxo_server.read().await;
                let element = utxo_server.get(&script).await;
                let json = serde_json::to_string(&element.values);
                match json {
                    Ok(json) => return Ok(Self::ok(json)),
                    Err(_) => return Ok(Self::internal_error("Failed to encode to JSON.")),
                };
            },
            Err(_) => return Ok(Self::not_found("Failed to decode input script.")),
        }
    }
    /// `/rich_list/count` endpoint.
    async fn rich_list_count_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let server = req.data::<HttpServer>().unwrap();
        let rich_list = server.rich_list.read().await;
        let json = format!("{{\"count\":{}}}", rich_list.len());
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
        let json = serde_json::to_string(&addresses);
        match json {
            Ok(json) => return Ok(Self::ok(json)),
            Err(_) => return Ok(Self::internal_error("Failed to encode to JSON.")),
        };
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
