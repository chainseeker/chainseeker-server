/// An abstraction struct for key-value store.

use std::marker::PhantomData;
use rocksdb::{DBWithThreadMode, MultiThreaded, Options, DBIteratorWithThreadMode};

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Deserialize {
    fn deserialize(buf: &[u8]) -> Self;
}

pub trait KVS<K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    fn new(path: &str) -> Self;
    fn get(&self, key: K) -> Option<Vec<u8>>;
    fn put(&self, key: K, value: V);
    fn delete(&self, key: K);
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_>;
}

type Rocks = DBWithThreadMode<MultiThreaded>;

pub struct RocksDBIterator<'a, K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    base: DBIteratorWithThreadMode<'a, Rocks>,
    _k: PhantomData<fn() -> K>,
    _v: PhantomData<fn() -> V>,
}

impl<'a, K, V> RocksDBIterator<'a, K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    pub fn new(base: DBIteratorWithThreadMode<'a, Rocks>) -> Self {
        Self {
            base,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for RocksDBIterator<'a, K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        match self.base.next() {
            Some((key, value)) => {
                Some((K::deserialize(&key), V::deserialize(&value)))
            },
            None => None,
        }
    }
}

pub struct RocksDB {
    db: Rocks,
}

impl<K, V> KVS<K, V> for RocksDB
    where K: Serialize + Deserialize + 'static, V: Serialize + Deserialize + 'static,
{
    fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.set_max_open_files(100);
        opts.create_if_missing(true);
        let db = Rocks::open(&opts, path).expect("Failed to open the database.");
        Self {
            db,
        }
    }
    fn get(&self, key: K) -> Option<Vec<u8>> {
        self.db.get(key.serialize()).unwrap()
    }
    fn put(&self, key: K, value: V) {
        self.db.put(key.serialize(), value.serialize()).unwrap();
    }
    fn delete(&self, key: K) {
        self.db.delete(key.serialize()).unwrap();
    }
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        Box::new(RocksDBIterator::new(self.db.iterator(rocksdb::IteratorMode::Start)))
    }
}
