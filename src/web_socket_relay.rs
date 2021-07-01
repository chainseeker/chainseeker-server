use std::sync::Arc;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

pub struct WebSocketRelay {
}

impl WebSocketRelay {
    pub fn new() -> Self {
        Self {
        }
    }
    pub async fn run(&self, zmq_endpoint: &str, ws_endpoint: &str) {
        let stop = Arc::new(RwLock::new(false));
        // Install Ctrl-C watch.
        {
            let stop = stop.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
                *stop.write().await = true;
            });
        }
        // Connect to ZMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        socket.connect(zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        socket.set_subscribe(b"hashtx").expect("Failed to subscribe to a ZeroMQ topic.");
        //println!("WebSocketRelay: waiting for a ZeroMQ message...");
        // Create a WebSocket server.
        let ws_endpoint = ws_endpoint.to_string();
        let (tx, rx) = tokio::sync::watch::channel("".to_string());
        tokio::spawn(async move {
            let listener = TcpListener::bind(&ws_endpoint).await.unwrap();
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let mut rx = rx.clone();
                    tokio::spawn(async move {
                        let addr = stream.peer_addr().unwrap();
                        let ws_stream = tokio_tungstenite::accept_async(stream).await;
                        if ws_stream.is_err() {
                            // Invalid request from client.
                            return;
                        }
                        println!("WebSocketRelay: new connection from {}.", addr);
                        let (mut write, _read) = ws_stream.unwrap().split();
                        while rx.changed().await.is_ok() {
                            let message = (*rx.borrow()).to_string();
                            match write.send(Message::Text(message)).await {
                                Ok(_) => {},
                                // Connection lost.
                                Err(_) => break,
                            }
                        }
                    });
                }
            }
        });
        loop {
            if *stop.read().await {
                break;
            }
            match socket.recv_multipart(1) {
                Ok(multipart) => {
                    assert_eq!(multipart.len(), 3);
                    let topic = std::str::from_utf8(&multipart[0]).expect("Failed to decode ZeroMQ topic.").to_string();
                    let hash = &multipart[1];
                    //println!("WebSocketRelay: {} {}", topic, hex::encode(hash));
                    let json = serde_json::to_string(&vec![topic, hex::encode(hash)]).unwrap();
                    tx.send(json).unwrap();
                },
                Err(_) => {
                    //println!("WebSockerRelay: failed to receive a message from ZeroMq.");
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                },
            }
        }
        println!("WebSocketRelay stopped.");
    }
}
