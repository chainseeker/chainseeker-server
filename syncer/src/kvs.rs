/// An abstraction struct for key-value store.

use std::marker::PhantomData;
use rocksdb::{DBWithThreadMode, MultiThreaded, DBIteratorWithThreadMode};

use crate::*;

pub trait ConstantSize {
    fn len() -> usize;
}

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

impl<S> Serialize for Vec<S>
    where S: Serialize,
{
    fn serialize(&self) -> Vec<u8> {
        self.iter().map(|item| item.serialize()).collect::<Vec<Vec<u8>>>().concat()
    }
}

pub trait Deserialize {
    fn deserialize(buf: &[u8]) -> Self;
}

impl<D> Deserialize for Vec<D>
    where D: Deserialize + ConstantSize,
{
    fn deserialize(buf: &[u8]) -> Self {
        let len = D::len();
        let mut offset = 0usize;
        let mut ret = Vec::new();
        while offset < buf.len() {
            ret.push(D::deserialize(&buf[offset..offset+len]));
            offset += len;
        }
        ret
    }
}

pub trait KVS<K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    fn new(path: &str) -> Self;
    fn get(&self, key: &K) -> Option<V>;
    fn put(&self, key: &K, value: &V);
    fn delete(&self, key: &K);
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_>;
    fn prefix_iter(&self, prefix: Vec<u8>) -> Box<dyn Iterator<Item = (K, V)> + '_>;
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
    pub fn new(rocks_db: &'a RocksDB<K, V>) -> Self {
        Self {
            base: rocks_db.db.iterator(rocksdb::IteratorMode::Start),
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

pub struct RocksDBPrefixIterator<'a, K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    base: DBIteratorWithThreadMode<'a, Rocks>,
    prefix: Vec<u8>,
    _k: PhantomData<fn() -> K>,
    _v: PhantomData<fn() -> V>,
}

impl<'a, K, V> RocksDBPrefixIterator<'a, K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    pub fn new(rocks_db: &'a RocksDB<K, V>, prefix: Vec<u8>) -> Self {
        Self {
            base: rocks_db.db.prefix_iterator(prefix.clone()),
            prefix: prefix,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for RocksDBPrefixIterator<'a, K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        match self.base.next() {
            Some((key, value)) => {
                if self.prefix != key[0..self.prefix.len()] {
                    None
                } else {
                    Some((K::deserialize(&key), V::deserialize(&value)))
                }
            },
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct RocksDB<K, V>
    where K: Serialize + Deserialize, V: Serialize + Deserialize,
{
    db: Rocks,
    _k: PhantomData<fn() -> K>,
    _v: PhantomData<fn() -> V>,
}

impl<K, V> KVS<K, V> for RocksDB<K, V>
    where K: Serialize + Deserialize + 'static, V: Serialize + Deserialize + 'static,
{
    fn new(path: &str) -> Self {
        Self {
            db: rocks_db(path),
            _k: PhantomData,
            _v: PhantomData,
        }
    }
    fn get(&self, key: &K) -> Option<V> {
        match self.db.get(key.serialize()).unwrap() {
            Some(value) => Some(V::deserialize(&value)),
            None => None,
        }
    }
    fn put(&self, key: &K, value: &V) {
        self.db.put(key.serialize(), value.serialize()).unwrap();
    }
    fn delete(&self, key: &K) {
        self.db.delete(key.serialize()).unwrap();
    }
    fn iter(&self) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        Box::new(RocksDBIterator::new(&self))
    }
    fn prefix_iter(&self, prefix: Vec<u8>) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        Box::new(RocksDBPrefixIterator::new(&self, prefix))
    }
}
