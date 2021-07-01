use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::*;

#[derive(Debug)]
pub struct RocksDBLazy<K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static + ConstantSize,
{
    buffer: Arc<RwLock<HashMap<K, Vec<V>>>>,
    db: Arc<RwLock<RocksDBMulti<K, V>>>,
}

impl<K, V> RocksDBLazy<K, V>
    where K: Serialize + Deserialize + Sync + Send + Clone + 'static + Eq + std::hash::Hash,
          V: Serialize + Deserialize + Sync + Send + Clone + 'static + ConstantSize + PartialEq,
{
    pub fn new(path: &str, temporary: bool) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(RwLock::new(RocksDBMulti::new(path, temporary))),
        }
    }
    async fn get_from_buffer(&self, key: &K) -> Vec<V> {
        match self.buffer.read().await.get(key) {
            Some(value) => (*value).clone(),
            None => Vec::new(),
        }
    }
    async fn get_from_db(&self, key: &K) -> Vec<V> {
        self.db.read().await.get(key)
    }
    pub async fn get(&self, key: &K) -> Vec<V> {
        vec![self.get_from_buffer(key).await, self.get_from_db(key).await].concat()
    }
    pub async fn insert(&mut self, key: K, value: Vec<V>) {
        self.buffer.write().await.insert(key, value);
    }
    pub async fn push(&mut self, key: &K, value: &V) {
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
    async fn remove_from_buffer(&self, key: &K, value: &V) {
        match self.buffer.write().await.get_mut(&key) {
            Some(values) => *values = values.iter().filter(|v| *v != value).cloned().collect(),
            None => (),
        }
    }
    async fn remove_from_db(&self, key: &K, value: &V) {
        let db = self.db.write().await;
        let values = db.get(key).iter().filter(|v| *v != value).cloned().collect();
        db.put(key, &values);
    }
    pub async fn remove(&mut self, key: &K, value: &V) {
        self.remove_from_buffer(key, value).await;
        self.remove_from_db(key, value).await;
    }
    pub async fn flush(&self) {
        for key in self.buffer.read().await.keys() {
            let mut buffer = self.buffer.write().await;
            let db = self.db.write().await;
            let (key, value) = buffer.remove_entry(key).unwrap();
            db.put(&key, &value);
        }
    }
}
