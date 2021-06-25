use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::*;

#[derive(Debug)]
pub struct RocksDBLazy {
    buffer: Arc<RwLock<HashMap<Script, UtxoServerElement>>>,
    db: Arc<RwLock<RocksDB>>,
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
    pub async fn put(&mut self, key: Script, value: UtxoServerElement) {
        self.buffer.write().await.insert(key, value);
    }
    pub async fn push(&mut self, key: &Script, value: UtxoServerValue) {
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
    pub fn run(&self) {
        let stop = Arc::new(RwLock::new(false));
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
            println!("Ctrl-C was pressed. Exiting RocksDBLazy...");
            *stop.write().await = true;
        });
        let buffer = self.buffer.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            loop {
                if buffer.read().await.len() == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                for (key, val) in buffer.read().await.iter() {
                    db.write().await.put(serialize_script(key), Vec::<u8>::from(val)).expect("Failed to put to DB.");
                    buffer.write().await.remove(key);
                }
            }
        });
    }
}
