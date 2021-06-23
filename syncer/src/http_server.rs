use std::time::Instant;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;

use bitcoin_hashes::hex::FromHex;
use bitcoin::Script;

use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use super::*;

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
    async fn addr_index(addr_index_db: &Arc<RwLock<AddressIndexDB>>, hex: &str) -> Response<Body> {
        let script = Script::from_hex(hex);
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
    pub async fn run(&self, coin: &str, config: &Config) {
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
