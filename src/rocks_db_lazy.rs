use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::*;

#[derive(Debug)]
pub struct RocksDBLazy<K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static + ConstantSize,
{
    // (buffer, db) tuple.
    lock: Arc<RwLock<(HashMap<K, Vec<V>>, RocksDBMulti<K, V>)>>
}

impl<K, V> RocksDBLazy<K, V>
    where K: Serialize + Deserialize + Sync + Send + Clone + 'static + Eq + std::hash::Hash,
          V: Serialize + Deserialize + Sync + Send + Clone + 'static + ConstantSize + PartialEq,
{
    pub fn new(path: &str, temporary: bool) -> Self {
        Self {
            lock: Arc::new(RwLock::new((HashMap::new(), RocksDBMulti::new(path, temporary)))),
        }
    }
    pub async fn get(&self, key: &K) -> Vec<V> {
        let lock = self.lock.read().await;
        let values_buffer = match lock.0.get(key) {
            Some(values) => (*values).clone(),
            None => Vec::new(),
        };
        let values_db = lock.1.get(key);
        vec![values_buffer, values_db].concat()
    }
    pub async fn insert(&mut self, key: K, value: Vec<V>) {
        self.lock.write().await.0.insert(key, value);
    }
    pub async fn push(&mut self, key: &K, value: &V) {
        let buffer = &mut self.lock.write().await.0;
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
        let mut lock = self.lock.write().await;
        match lock.0.get_mut(&key) {
            Some(values) => *values = values.iter().filter(|v| *v != value).cloned().collect(),
            None => (),
        }
        lock.1.put(key, &lock.1.get(key).iter().filter(|v| *v != value).cloned().collect());
    }
    pub async fn flush(&mut self) {
        loop {
            let mut lock = self.lock.write().await;
            let key = {
                let data = lock.0.iter().next();
                if data.is_none() {
                    break;
                }
                let (key, value) = data.unwrap();
                lock.1.put(key, value);
                (*key).clone()
            };
            lock.0.remove(&key);
        }
        self.lock.write().await.0 = HashMap::new();
    }
}
