use crate::*;
use std::sync::Arc;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use ZeroMQMessage::*;

#[derive(Debug, Clone)]
pub struct WebSocketRelay {
    ws_endpoint: String,
    stop: Arc<RwLock<bool>>,
    ready: Arc<RwLock<bool>>,
}

impl WebSocketRelay {
    pub fn new(ws_endpoint: &str) -> Self {
        Self {
            ws_endpoint: ws_endpoint.to_string(),
            stop: Arc::new(RwLock::new(false)),
            ready: Arc::new(RwLock::new(false)),
        }
    }
    pub async fn run(&self, receiver: tokio::sync::watch::Receiver<ZeroMQMessage>) {
        let stop = self.stop.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
            *stop.write().await = true;
        });
        //println!("WebSocketRelay: waiting for a ZeroMQ message...");
        // Create a WebSocket server.
        let ws_endpoint = self.ws_endpoint.clone();
        let (tx, rx) = tokio::sync::watch::channel("".to_string());
        let ready = self.ready.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(&ws_endpoint).await.unwrap();
            println!("WebSocketRelay: listening on {}", ws_endpoint);
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
        let mut last_message = Init;
        loop {
            if *self.stop.read().await {
                break;
            }
            let message = (*receiver.borrow()).clone();
            if message != last_message {
                last_message = message.clone();
                match message {
                    HashBlock(block_hash) => {
                        let json = serde_json::to_string(&vec!["hashblock", &hex::encode(&consensus_encode(&block_hash))]).unwrap();
                        tx.send(json).unwrap();
                    },
                    RawTx(transaction) => {
                        let txid = transaction.txid();
                        let json = serde_json::to_string(&vec!["hashtx", &txid.to_string()]).unwrap();
                        tx.send(json).unwrap();
                    },
                    Init => {},
                }
            } else {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        println!("WebSocketRelay stopped.");
    }
    pub async fn ready(&self) -> bool {
        *self.ready.read().await
    }
    pub async fn wait_for_ready(&self) {
        while !self.ready().await {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
    pub async fn stop(&self) {
        *self.stop.write().await = true;
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::watch::channel;
    use super::*;
    const WS_PORT: u16 = 6666;
    const BLOCK_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    // txid = caaacc4826fdf63ad0a4093400de5f1fd0c830be0724078ac039f9b29878b76f.
    const RAW_TX: &str = "0200000000010122f1294bc73da293dfe1a9088c6d26d71564bf538940c7ce9c4e6212f099c3b90000000000ffffffff011e272d0100000000160014af73f777fcd64ec6d9b22ac9e1a57e127ea169ee0247304402205fea552c7d5ed3330aa4a8b5c90a980c1d3bdc72abd13c2d7bccba91fbb978f5022027fac985cfb83339fc9227e1c653b8a824c63a49cda4f9f97d48d5c07e047608012102acc07439373cc2902d0ad6602ed6f5a1b7abdf7608d265c089160ac826a4600600000000";
    #[tokio::test(flavor = "multi_thread")]
    async fn web_socket_relay() {
        let (tx, rx) = channel(Init);
        // Run relay.
        let relay = WebSocketRelay::new(&format!("localhost:{}", WS_PORT));
        let handle = {
            let relay = relay.clone();
            tokio::spawn(async move {
                relay.run(rx).await;
            })
        };
        // Wait before WebSocketRelay is ready.
        relay.wait_for_ready().await;
        // Create WebSocket client.
        println!("Creating WebSocket client...");
        let (ws_stream, _) = tokio_tungstenite::connect_async(&format!("ws://localhost:{}", WS_PORT)).await.unwrap();
        let (_write, mut read) = ws_stream.split();
        // Send "hashblock" message.
        println!("Sending \"hashblock\"...");
        tx.send(HashBlock(consensus_decode(&hex::decode(BLOCK_HASH).unwrap()))).unwrap();
        println!("Reading a message from WebSocket...");
        let msg = read.next().await.unwrap().unwrap().into_data();
        assert_eq!(String::from_utf8(msg).unwrap(), format!("[\"hashblock\",\"{}\"]", BLOCK_HASH));
        // Send "hashtx" message.
        println!("Sending \"hashtx\"...");
        let transaction: bitcoin::Transaction = consensus_decode(&hex::decode(RAW_TX).unwrap());
        let txid = transaction.txid();
        tx.send(RawTx(transaction)).unwrap();
        println!("Reading a message from WebSocket...");
        let msg = read.next().await.unwrap().unwrap().into_data();
        assert_eq!(String::from_utf8(msg).unwrap(), format!("[\"hashtx\",\"{}\"]", txid));
        relay.stop().await;
        handle.await.unwrap();
    }
}
