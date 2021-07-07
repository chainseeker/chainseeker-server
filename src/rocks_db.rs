/// An abstraction struct for key-value store.
use std::fs::remove_dir_all;
use std::marker::PhantomData;
use rocksdb::{DBWithThreadMode, MultiThreaded, DBIteratorWithThreadMode, BoundColumnFamily, Options, DBPinnableSlice};

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

#[derive(Debug, Clone, Default)]
pub struct Empty {}

impl ConstantSize for Empty {
    const LEN: usize = 0;
}

impl Serialize for Empty {
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl Deserialize for Empty {
    fn deserialize(_buf: &[u8]) -> Self {
        Empty {}
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
        self.base.next().map(|(key, value)| (K::deserialize(&key), V::deserialize(&value)))
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
    pub fn new(base: DBIteratorWithThreadMode<'a, Rocks>, prefix: Vec<u8>) -> Self
    {
        Self {
            base,
            prefix,
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

pub struct RocksDBColumnFamily<'a, K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static,
{
    base: &'a RocksDB<Empty, Empty>,
    name: String,
    cf: BoundColumnFamily<'a>,
    _k: PhantomData<fn() -> K>,
    _v: PhantomData<fn() -> V>,
}

impl<'a, K, V> RocksDBColumnFamily<'a, K, V>
    where K: Serialize + Deserialize + 'static,
          V: Serialize + Deserialize + 'static,
{
    pub fn new(base: &'a RocksDB<Empty, Empty>, name: &str) -> Self {
        let cf = match base.db.cf_handle(name) {
            Some(cf) => cf,
            None => {
                let mut opts = Options::default();
                opts.set_max_open_files(100);
                opts.create_if_missing(true);
                base.db.create_cf(name, &opts).unwrap();
                base.db.cf_handle(name).unwrap()
            },
        };
        Self {
            base,
            name: name.to_string(),
            cf,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn get(&self, key: &K) -> Option<V> {
        self.base.db.get_pinned_cf(self.cf, key.serialize()).unwrap().map(|value| V::deserialize(&value))
    }
    pub fn put(&self, key: &K, value: &V) {
        self.base.db.put_cf(self.cf, key.serialize(), value.serialize()).unwrap();
    }
    pub fn delete(&self, key: &K) {
        self.base.db.delete_cf(self.cf, key.serialize()).unwrap();
    }
    pub fn iter(&self) -> RocksDBIterator<'_, K, V> {
        RocksDBIterator::new(self.base.db.iterator_cf(self.cf, rocksdb::IteratorMode::Start))
    }
    pub fn prefix_iter(&self, prefix: Vec<u8>) -> RocksDBPrefixIterator<'_, K, V> {
        RocksDBPrefixIterator::new(self.base.db.prefix_iterator_cf(self.cf, prefix.clone()), prefix)
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
        if temporary && std::path::Path::new(path).exists() {
            remove_dir_all(path).unwrap();
        }
        let mut opts = Options::default();
        opts.set_max_open_files(100);
        opts.create_if_missing(true);
        let db = Rocks::open(&opts, path).expect("Failed to open the database.");
        Self {
            temporary,
            path: path.to_string(),
            db,
            _k: PhantomData,
            _v: PhantomData,
        }
    }
    pub fn get(&self, key: &K) -> Option<V> {
        self.db.get_pinned(key.serialize()).unwrap().map(|value| V::deserialize(&value))
    }
    pub fn get_raw(&self, key: &K) -> Option<DBPinnableSlice<'_>> {
        self.db.get_pinned(key.serialize()).unwrap()
    }
    pub fn multi_get<I: IntoIterator<Item = K>>(&self, keys: I) -> Vec<Option<V>> {
        let keys: Vec<Vec<u8>> = keys.into_iter().map(|key| key.serialize()).collect();
        self.db.multi_get(keys).unwrap().iter().map(|value| {
            if value.is_empty() {
                None
            } else {
                Some(V::deserialize(value))
            }
        }).collect()
    }
    pub fn put(&self, key: &K, value: &V) {
        self.db.put(key.serialize(), value.serialize()).unwrap();
    }
    pub fn delete(&self, key: &K) {
        self.db.delete(key.serialize()).unwrap();
    }
    pub fn iter(&self) -> RocksDBIterator<'_, K, V> {
        RocksDBIterator::new(self.db.iterator(rocksdb::IteratorMode::Start))
    }
    pub fn prefix_iter(&self, prefix: Vec<u8>) -> RocksDBPrefixIterator<'_, K, V> {
        RocksDBPrefixIterator::new(self.db.prefix_iterator(prefix.clone()), prefix)
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
mod tests {
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
            vec![(key1.clone(), value1)]);
        db.delete(&key1);
        assert_eq!(db.get(&key1), None);
        assert_eq!(db.multi_get(vec![key1, key2]), vec![None, Some(value2)]);
    }
    #[test]
    fn rocks_db_cf() {
        let db = RocksDB::<Empty, Empty>::new("/tmp/chainseeker/test_rocks_db_cf", true);
        let db_cf1 = RocksDBColumnFamily::<u32, u32>::new(&db, "cf1");
        let db_cf2 = RocksDBColumnFamily::<u32, u32>::new(&db, "cf2");
        db_cf1.put(&114514, &12345);
        db_cf2.put(&114514, &67890);
        assert_eq!(db_cf1.get(&114514), Some(12345));
        assert_eq!(db_cf2.get(&114514), Some(67890));
    }
}
