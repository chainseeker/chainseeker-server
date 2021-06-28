/// An abstraction struct for key-value store.
use std::fs::remove_dir_all;
use std::marker::PhantomData;
use rocksdb::{DBWithThreadMode, MultiThreaded, DBIteratorWithThreadMode};

use crate::*;

pub trait ConstantSize {
    const LEN: usize;
}

impl ConstantSize for u32 {
    const LEN: usize = 4;
}

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Serialize for u32 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
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

impl Deserialize for String {
    fn deserialize(buf: &[u8]) -> Self {
        Self::from_utf8(buf.to_vec()).unwrap()
    }
}

impl Deserialize for u32 {
    fn deserialize(buf: &[u8]) -> Self {
        assert_eq!(buf.len(), 4);
        let buf: [u8; 4] = [buf[0], buf[1], buf[2], buf[3]];
        u32::from_le_bytes(buf)
    }
}

impl<D> Deserialize for Vec<D>
    where D: Deserialize + ConstantSize,
{
    fn deserialize(buf: &[u8]) -> Self {
        let mut offset = 0usize;
        let mut ret = Vec::new();
        while offset < buf.len() {
            ret.push(D::deserialize(&buf[offset..offset+D::LEN]));
            offset += D::LEN;
        }
        ret
    }
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
    where K: Serialize + Deserialize,
          V: Serialize + Deserialize,
{
    base: DBIteratorWithThreadMode<'a, Rocks>,
    prefix: Vec<u8>,
    _k: PhantomData<fn() -> K>,
    _v: PhantomData<fn() -> V>,
}

impl<'a, K, V> RocksDBPrefixIterator<'a, K, V>
    where K: Serialize + Deserialize,
          V: Serialize + Deserialize,
{
    pub fn new(rocks_db: &'a RocksDB<K, V>, prefix: Vec<u8>) -> Self
    {
        Self {
            base: rocks_db.db.prefix_iterator(prefix.clone()),
            prefix: prefix,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for RocksDBPrefixIterator<'a, K, V>
    where K: Serialize + Deserialize,
          V: Serialize + Deserialize,
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
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static,
{
    temporary: bool,
    path: String,
    db: Rocks,
    _k: PhantomData<fn() -> K>,
    _v: PhantomData<fn() -> V>,
}

impl<K, V> RocksDB<K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static,
{
    pub fn new(path: &str, temporary: bool) -> Self {
        if temporary {
            if std::path::Path::new(path).exists() {
                remove_dir_all(path).unwrap();
            }
        }
        let db = rocks_db(path);
        Self {
            temporary,
            path: path.to_string(),
            db: db,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
    pub fn get(&self, key: &K) -> Option<V> {
        match self.db.get(key.serialize()).unwrap() {
            Some(value) => Some(V::deserialize(&value)),
            None => None,
        }
    }
    pub fn put(&self, key: &K, value: &V) {
        self.db.put(key.serialize(), value.serialize()).unwrap();
    }
    pub fn delete(&self, key: &K) {
        self.db.delete(key.serialize()).unwrap();
    }
    pub fn iter(&self) -> RocksDBIterator<'_, K, V> {
        RocksDBIterator::new(&self)
    }
    pub fn prefix_iter(&self, prefix: Vec<u8>) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        Box::new(RocksDBPrefixIterator::new(&self, prefix))
    }
    pub fn purge(&self) {
        remove_dir_all(&self.path).unwrap();
    }
}

impl<K, V> Drop for RocksDB<K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static,
{
    fn drop(&mut self) {
        if self.temporary {
            self.purge();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn rocks_db() {
        let db = RocksDB::<String, Vec<u32>>::new("/tmp/chainseeker/test_rocks_db", true);
        let key1 = "bar".to_string();
        let value1 = vec![3939, 4649];
        let key2 = "foo".to_string();
        let value2 = vec![1234, 5678];
        db.put(&key1, &value1);
        db.put(&key2, &value2);
        assert_eq!(db.get(&key1), Some(value1.clone()));
        assert_eq!(db.get(&key2), Some(value2.clone()));
        assert_eq!(
            db.iter().collect::<Vec<(String, Vec<u32>)>>(),
            vec![(key1.clone(), value1.clone()), (key2.clone(), value2.clone())]);
        assert_eq!(
            db.prefix_iter(key1.as_bytes().to_vec()).collect::<Vec<(String, Vec<u32>)>>(),
            vec![(key1.clone(), value1.clone())]);
        db.delete(&key1);
        assert_eq!(db.get(&key1), None);
    }
}
