use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::*;

const MAX_ENTRIES: usize = 1000_000;

#[derive(Debug)]
pub struct RocksDBLazy {
    buffer: Arc<RwLock<HashMap<Script, UtxoServerElement>>>,
    db: Arc<RwLock<RocksDBBase>>,
}

impl RocksDBLazy {
    pub fn new(path: &str) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(RwLock::new(rocks_db(path))),
        }
    }
    pub async fn get(&self, key: &Script) -> UtxoServerElement {
        match self.buffer.read().await.get(key) {
            Some(value) => (*value).clone(),
            None => {
                let ser = self.db.read().await.get(key).unwrap();
                match ser {
                    Some(ser) => UtxoServerElement::from(&ser[..]),
                    None => UtxoServerElement::new(),
                }
            },
        }
    }
    pub async fn insert(&mut self, key: Script, value: UtxoServerElement) {
        self.buffer.write().await.insert(key, value);
    }
    pub async fn push(&mut self, key: &Script, value: UtxoServerValue) {
        loop {
            if self.buffer.read().await.len() > MAX_ENTRIES {
                std::thread::sleep(std::time::Duration::from_millis(100));
            } else {
                break;
            }
        }
        let mut buffer = self.buffer.write().await;
        match buffer.get_mut(key) {
            Some(element) => {
                element.values.push(value);
            },
            None => {
                let mut element = UtxoServerElement::new();
                element.values.push(value);
                buffer.insert((*key).clone(), element);
            }
        }
    }
    pub async fn remove(&mut self, script_pubkey: &Script, txid: &Txid, vout: u32) {
        let element = self.get(script_pubkey).await;
        let values = element.values.iter().filter(|&utxo_value| {
            !(utxo_value.txid == *txid && utxo_value.vout == vout)
        }).cloned().collect();
        self.insert((*script_pubkey).clone(), UtxoServerElement { values }).await;
    }
    pub fn run(&self) {
        let stop = Arc::new(RwLock::new(false));
        {
            let stop = stop.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
                println!("Ctrl-C was pressed. Exiting RocksDBLazy...");
                *stop.write().await = true;
            });
        }
        let buffer = self.buffer.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            loop {
                if *stop.read().await {
                    break;
                }
                if buffer.read().await.len() == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                let buffer: Vec<(Script, UtxoServerElement)> = {
                    let mut buffer = buffer.write().await;
                    let buf = buffer.iter().map(|(key, value)| ((*key).clone(), (*value).clone())).collect();
                    *buffer = HashMap::new();
                    buf
                };
                for (key, val) in buffer.iter() {
                    db.write().await.put(serialize_script(key), Vec::<u8>::from(val)).expect("Failed to put to DB.");
                }
            }
        });
    }
}
