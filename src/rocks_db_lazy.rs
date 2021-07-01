use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::*;

const MAX_ENTRIES: usize = 1000_000;

#[derive(Debug)]
pub struct RocksDBLazy<K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static + ConstantSize,
{
    buffer: Arc<RwLock<HashMap<K, Vec<V>>>>,
    db: Arc<RwLock<RocksDB<K, Vec<V>>>>,
    stop: Arc<RwLock<bool>>,
}

impl<K, V> RocksDBLazy<K, V>
    where K: Serialize + Deserialize + Sync + Send + Clone + 'static + Eq + std::hash::Hash,
          V: Serialize + Deserialize + Sync + Send + Clone + 'static + ConstantSize + PartialEq,
{
    pub fn new(path: &str, temporary: bool) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(RwLock::new(RocksDB::new(path, temporary))),
            stop: Arc::new(RwLock::new(false)),
        }
    }
    pub async fn get(&self, key: &K) -> Vec<V> {
        match self.buffer.read().await.get(key) {
            Some(value) => (*value).clone(),
            None => {
                let value = self.db.read().await.get(key);
                match value {
                    Some(value) => value,
                    None => Vec::new(),
                }
            },
        }
    }
    pub async fn insert(&mut self, key: K, value: Vec<V>) {
        self.buffer.write().await.insert(key, value);
    }
    pub async fn push(&mut self, key: &K, value: &V) {
        loop {
            if self.buffer.read().await.len() > MAX_ENTRIES {
                std::thread::sleep(std::time::Duration::from_millis(100));
            } else {
                break;
            }
        }
        let mut buffer = self.buffer.write().await;
        match buffer.get_mut(key) {
            Some(values) => {
                values.push((*value).clone());
            },
            None => {
                let mut values = Vec::new();
                values.push((*value).clone());
                buffer.insert((*key).clone(), values);
            }
        }
    }
    pub async fn remove(&mut self, key: &K, value: &V) {
        let values_old = self.get(key).await;
        let mut values = Vec::new();
        for v in values_old.iter() {
            if *v != *value {
                values.push((*v).clone());
            }
        }
        self.insert((*key).clone(), values).await;
    }
    pub fn run(&self) {
        {
            let stop = self.stop.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler.");
                println!("Ctrl-C was pressed. Exiting RocksDBLazy...");
                *stop.write().await = true;
            });
        }
        let buffer = self.buffer.clone();
        let db = self.db.clone();
        let stop = self.stop.clone();
        tokio::spawn(async move {
            loop {
                if *stop.read().await {
                    break;
                }
                if buffer.read().await.len() == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                let buffer: Vec<(K, Vec<V>)> = {
                    let buf = buffer.read().await.iter().map(|(k, v)| ((*k).clone(), (*v).clone())).collect();
                    *buffer.write().await = HashMap::new();
                    buf
                };
                for (key, val) in buffer.iter() {
                    db.write().await.put(key, val);
                }
            }
        });
    }
    pub async fn stop(&self) {
        *self.stop.write().await = true;
    }
}
