/// C++'s `multimap`-like DB implementation backed by RocksDB.
use crate::*;

#[derive(Debug)]
pub struct RocksDBMulti<K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static + ConstantSize,
{
    db: RocksDB<K, Vec<V>>,
}

impl<K, V> RocksDBMulti<K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static + ConstantSize + Clone + PartialEq,
{
    pub fn new(path: &str, temporary: bool) -> Self {
        Self {
            db: RocksDB::new(path, temporary),
        }
    }
    pub fn get(&self, key: &K) -> Vec<V> {
        self.db.get(key).unwrap_or_default()
    }
    pub fn put(&self, key: &K, values: &[V]) {
        self.db.put(key, &values.to_vec());
    }
    pub fn push(&self, key: &K, value: V) {
        let mut values = self.get(key);
        values.push(value);
        self.put(key, &values);
    }
    pub fn pop(&self, key: &K, value: &V) {
        let values = self.get(key);
        let values = values.iter().filter(|v| *v != value).cloned().collect::<Vec<V>>();
        self.put(key, &values);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn rocks_db_multi() {
        let db = RocksDBMulti::<String, u32>::new("/tmp/chainseeker/test_rocks_db_multi", true);
        let key1 = "bar".to_string();
        let key2 = "foo".to_string();
        db.push(&key1, 3939);
        db.push(&key1, 4649);
        db.push(&key2, 1234);
        db.push(&key2, 5678);
        assert_eq!(db.get(&key1), vec![3939, 4649]);
        assert_eq!(db.get(&key2), vec![1234, 5678]);
        db.pop(&key1, &3939);
        assert_eq!(db.get(&key1), vec![4649]);
    }
}
