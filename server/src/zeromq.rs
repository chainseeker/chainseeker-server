use crate::consensus_decode;
use crate::bitcoin::{BlockHash, Transaction};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::watch::{Sender, Receiver, channel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZeroMQMessage {
    HashBlock(BlockHash),
    RawTx(Transaction),
    Init,
}

use ZeroMQMessage::*;

/// Listens to the Bitcoin Core's ZeroMQ server and relay messages to other threads.
#[derive(Debug, Clone)]
pub struct ZeroMQClient {
    zmq_endpoint: String,
    stop: Arc<RwLock<bool>>,
    stopped: Arc<RwLock<bool>>,
    ready: Arc<RwLock<bool>>,
}

impl ZeroMQClient {
    pub fn new(zmq_endpoint: &str) -> Self {
        Self {
            zmq_endpoint: zmq_endpoint.to_string(),
            stop: Arc::new(RwLock::new(false)),
            stopped: Arc::new(RwLock::new(false)),
            ready: Arc::new(RwLock::new(false)),
        }
    }
    pub async fn run(&self, sender: Sender<ZeroMQMessage>) {
        let stop = self.stop.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
            *stop.write().await = true;
        });
        println!("ZeroMQClient: waiting for a ZeroMQ message...");
        // Connect to ZMQ.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::SUB).expect("Failed to open a ZeroMQ socket.");
        socket.connect(&self.zmq_endpoint).expect("Failed to connect to a ZeroMQ endpoint.");
        socket.set_subscribe(b"hashblock").expect("Failed to subscribe to a ZeroMQ topic.");
        socket.set_subscribe(b"rawtx").expect("Failed to subscribe to a ZeroMQ topic.");
        *self.ready.write().await = true;
        loop {
            if *self.stop.read().await {
                break;
            }
            let multipart = socket.recv_multipart(zmq::DONTWAIT);
            match multipart {
                Ok(multipart) => {
                    assert_eq!(multipart.len(), 3);
                    let topic = std::str::from_utf8(&multipart[0]).expect("Failed to decode ZeroMQ topic.").to_string();
                    let bin = &multipart[1];
                    //println!("ZeroMQClient: {} {} {}", topic, hex::encode(bin), hex::encode(&multipart[2]));
                    match topic.as_str() {
                        "hashblock" => {
                            let block_hash: BlockHash = consensus_decode(bin);
                            sender.send(HashBlock(block_hash)).unwrap();
                        },
                        "rawtx" => {
                            let transaction: Transaction = consensus_decode(bin);
                            sender.send(RawTx(transaction)).unwrap();
                        }
                        _ => {},
                    }
                },
                Err(_) => {
                    //println!("ZeroMQClient: failed to receive a message from ZeroMq.");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                },
            }
        }
        println!("ZeroMQClient stopped.");
        *self.stopped.write().await = true;
    }
    pub async fn start(&self) -> Receiver<ZeroMQMessage> {
        let (tx, rx) = channel(Init);
        let me = self.clone();
        tokio::spawn(async move {
            me.run(tx).await;
        });
        rx
    }
    pub async fn is_ready(&self) -> bool {
        *self.ready.read().await
    }
    pub async fn wait_for_ready(&self) {
        while !self.is_ready().await {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
    pub async fn is_stopped(&self) -> bool {
        *self.stopped.read().await
    }
    pub async fn stop(&self) {
        *self.stop.write().await = true;
    }
    pub async fn wait_for_stop(&self) {
        self.stop().await;
        while !self.is_stopped().await {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const BLOCK_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    // txid = caaacc4826fdf63ad0a4093400de5f1fd0c830be0724078ac039f9b29878b76f.
    const RAW_TX: &str = "0200000000010122f1294bc73da293dfe1a9088c6d26d71564bf538940c7ce9c4e6212f099c3b90000000000ffffffff011e272d0100000000160014af73f777fcd64ec6d9b22ac9e1a57e127ea169ee0247304402205fea552c7d5ed3330aa4a8b5c90a980c1d3bdc72abd13c2d7bccba91fbb978f5022027fac985cfb83339fc9227e1c653b8a824c63a49cda4f9f97d48d5c07e047608012102acc07439373cc2902d0ad6602ed6f5a1b7abdf7608d265c089160ac826a4600600000000";
    #[tokio::test(flavor = "multi_thread")]
    async fn client() {
        const ZMQ_PORT: u16 = 8080;
        // Create a ZeroMQ server.
        let zmq_ctx = zmq::Context::new();
        let socket = zmq_ctx.socket(zmq::SocketType::PUB).unwrap();
        socket.bind(&format!("tcp://lo:{}", ZMQ_PORT)).unwrap();
        println!("ZeroMQ server created.");
        // Run ZeroMQClient.
        let client = ZeroMQClient::new(&format!("tcp://127.0.0.1:{}", ZMQ_PORT));
        let mut rx = client.start().await;
        // Wait before ZeroMQClient is ready.
        client.wait_for_ready().await;
        // Send hashblock.
        let block_hash = hex::decode(BLOCK_HASH).unwrap();
        println!("Sending \"hashblock\"...");
        socket.send_multipart(vec![
            b"hashblock".to_vec(),
            block_hash.clone(),
            0u32.to_le_bytes().to_vec(),
        ], zmq::DONTWAIT).unwrap();
        println!("Reading a message...");
        assert!(rx.changed().await.is_ok());
        assert_eq!(*rx.borrow(), HashBlock(consensus_decode(&block_hash)));
        // Send rawtx.
        let tx = hex::decode(RAW_TX).unwrap();
        println!("Sending \"rawtx\"...");
        socket.send_multipart(vec![
            b"rawtx".to_vec(),
            tx.clone(),
            1u32.to_le_bytes().to_vec(),
        ], zmq::DONTWAIT).unwrap();
        println!("Reading a message...");
        assert!(rx.changed().await.is_ok());
        assert_eq!(*rx.borrow(), RawTx(consensus_decode(&tx)));
        // Stop.
        client.wait_for_stop().await;
    }
}
