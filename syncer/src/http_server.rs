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

struct State {
    addr_index_db: Arc<RwLock<AddressIndexDB>>,
    utxo_server: Arc<RwLock<UtxoServer>>,
}

pub struct HttpServer {
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
    async fn addr_index_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let state = req.data::<State>().unwrap();
        let addr_index_db = &state.addr_index_db;
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
    /// `/utxo/SCRIPT` endpoint.
    async fn utxo_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let state = req.data::<State>().unwrap();
        let utxo_server = &state.utxo_server;
        let script_hex = req.param("script").unwrap();
        let script = Script::from_hex(script_hex);
        match script {
            Ok(script) => {
                let utxo_server = utxo_server.read().await;
                let values = utxo_server.get(&script);
                let json = serde_json::to_string(&values);
                match json {
                    Ok(json) => return Ok(Self::ok(json)),
                    Err(_) => return Ok(Self::internal_error("Failed to encode to JSON.")),
                };
            },
            Err(_) => return Ok(Self::not_found("Failed to decode input script.")),
        }
    }
    pub async fn run(&self, coin: &str, config: &Config) {
        let ip = &config.http_ip;
        let port = config.coins[coin].http_port;
        let addr = SocketAddr::from((
            ip.parse::<std::net::IpAddr>().expect("Failed to parse HTTP IP address."),
            port));
        let router = Router::builder()
            .data(State {
                addr_index_db: self.addr_index_db.clone(),
                utxo_server: self.utxo_server.clone(),
            })
            .middleware(Middleware::pre(|req| async move {
                req.set_context(Instant::now());
                Ok(req)
            }))
            .get("/addr_index/:script", Self::addr_index_handler)
            .get("/utxo/:script", Self::utxo_handler)
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
