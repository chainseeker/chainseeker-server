use std::sync::Arc;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub struct WebSocketRelay {
    stop: Arc<RwLock<bool>>,
    ready: Arc<RwLock<bool>>,
}

impl WebSocketRelay {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(RwLock::new(false)),
            ready: Arc::new(RwLock::new(false)),
        }
    }
    pub async fn run(&self, zmq_endpoint: &str, ws_endpoint: &str, install_ctrl_c: bool) {
        // Install Ctrl-C watch.
        if install_ctrl_c {
            let stop = self.stop.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
                *stop.write().await = true;
            });
        }
        //println!("WebSocketRelay: waiting for a ZeroMQ message...");
        // Create a WebSocket server.
        let ws_endpoint_string = ws_endpoint.to_string();
        let (tx, rx) = tokio::sync::watch::channel("".to_string());
        let ready = self.ready.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(&ws_endpoint_string).await.unwrap();
            println!("WebSocketRelay: listening on {}", ws_endpoint_string);
            *ready.write().await = true;
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
        // Connect to ZMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        socket.connect(zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        socket.set_subscribe(b"hashtx").expect("Failed to subscribe to a ZeroMQ topic.");
        loop {
            if *self.stop.read().await {
                break;
            }
            let multipart = socket.recv_multipart(zmq::DONTWAIT);
            match multipart {
                Ok(multipart) => {
                    assert_eq!(multipart.len(), 3);
                    let topic = std::str::from_utf8(&multipart[0]).expect("Failed to decode ZeroMQ topic.").to_string();
                    let hash = &multipart[1];
                    //println!("WebSocketRelay: {} {} {}", topic, hex::encode(hash), hex::encode(&multipart[2]));
                    let json = serde_json::to_string(&vec![topic, hex::encode(hash)]).unwrap();
                    tx.send(json).unwrap();
                },
                Err(_) => {
                    //println!("WebSockerRelay: failed to receive a message from ZeroMq.");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                },
            }
        }
        println!("WebSocketRelay stopped.");
    }
    pub async fn ready(&self) -> bool {
        *self.ready.read().await
    }
    pub async fn wait_for_ready(&self) {
        while !self.ready().await {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    pub async fn stop(&self) {
        *self.stop.write().await = true;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test(flavor = "multi_thread")]
    async fn web_socket_relay() {
        //const ZMQ_ENDPOINT: &str = "inproc://web-socket-relay-zmq";
        const ZMQ_PORT: u16 = 5555;
        const WS_PORT: u16 = 6666;
        const BLOCK_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        const TXID: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
        // Create ZeroMQ server.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::PUB).unwrap();
        //socket.bind(ZMQ_ENDPOINT).unwrap();
        socket.bind("tcp://lo:5555").unwrap();
        println!("ZeroMQ server created.");
        // Run relay.
        let relay = WebSocketRelay::new();
        {
            let relay = relay.clone();
            tokio::spawn(async move {
                relay.run(&format!("tcp://localhost:{}", ZMQ_PORT), &format!("localhost:{}", WS_PORT), false).await;
            });
        }
        // Wait before WebSocketRelay is ready.
        relay.wait_for_ready().await;
        // Create WebSocket client.
        println!("Creating WebSocket client...");
        let (ws_stream, _) = tokio_tungstenite::connect_async(&format!("ws://localhost:{}", WS_PORT)).await.unwrap();
        let (_write, mut read) = ws_stream.split();
        // Send "hashblock" message.
        println!("Sending \"hashblock\"...");
        socket.send_multipart(vec![
            "hashblock".to_string().into_bytes(),
            hex::decode(BLOCK_HASH).unwrap(),
            0u32.to_le_bytes().to_vec(),
        ], zmq::DONTWAIT).unwrap();
        println!("Reading a message from WebSocket...");
        let msg = read.next().await.unwrap().unwrap().into_data();
        assert_eq!(String::from_utf8(msg).unwrap(), format!("[\"hashblock\",\"{}\"]", BLOCK_HASH));
        // Send "hashtx" message.
        println!("Sending \"hashtx\"...");
        socket.send_multipart(vec![
            "hashtx".to_string().into_bytes(),
            hex::decode(TXID).unwrap(),
            1u32.to_le_bytes().to_vec(),
        ], zmq::DONTWAIT).unwrap();
        println!("Reading a message from WebSocket...");
        let msg = read.next().await.unwrap().unwrap().into_data();
        assert_eq!(String::from_utf8(msg).unwrap(), format!("[\"hashtx\",\"{}\"]", TXID));
        relay.stop().await;
    }
}
