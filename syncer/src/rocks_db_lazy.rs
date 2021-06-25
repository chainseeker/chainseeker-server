use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;
use rocksdb::Error;

use crate::*;

#[derive(Debug)]
pub struct RocksDBLazy {
    buffer: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    db: Arc<RwLock<RocksDB>>,
}

impl RocksDBLazy {
    pub fn new(path: &str) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(RwLock::new(rocks_db(path))),
        }
    }
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        match self.buffer.read().await.get(key) {
            Some(value) => Ok(Some((*value).clone())),
            None => self.db.read().await.get(key),
        }
    }
    pub async fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.buffer.write().await.insert(key, value);
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
                    db.write().await.put(key, val).expect("Failed to put to DB.");
                    buffer.write().await.remove(key);
                }
            }
        });
    }
}
