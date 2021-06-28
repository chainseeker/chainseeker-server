use bitcoin::BlockHash;

use crate::*;

pub struct BlockDB {
    db: RocksDBBase,
}

impl BlockDB {
    pub fn path(coin: &str) -> String {
        format!("{}/{}/block", data_dir(), coin)
    }
    pub fn new(coin: &str) -> Self {
        let path = Self::path(coin);
        Self {
            db: rocks_db(&path),
        }
    }
    pub fn put_block_hash(&self, height: u32, block_hash: &BlockHash) {
        let height = height.to_le_bytes();
        let block_hash = serialize_block_hash(block_hash);
        self.db.put(height, &block_hash).expect("Failed to put block hash.");
    }
    pub fn get_block_hash(&self, height: u32) -> Option<BlockHash> {
        let height = height.to_le_bytes();
        let block_hash = self.db.get(height).expect("Failed to get block hash");
        match block_hash {
            Some(block_hash) => Some(deserialize_block_hash(&block_hash)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_dir_all;
    use super::*;
    #[test]
    fn path() {
        let path = BlockDB::path("test");
        assert_eq!(path, format!("{}/.chainseeker/test/block", std::env::var("HOME").unwrap()));
    }
    #[test]
    fn put_and_get_block_hash() {
        let height = 123456;
        let block_hash_arr = hex::decode("00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048").unwrap();
        let block_hash = deserialize_block_hash(&block_hash_arr);
        let coin = "test/block_hash";
        let block_db = BlockDB::new(&coin);
        block_db.put_block_hash(height, &block_hash);
        let block_hash_test = block_db.get_block_hash(height).unwrap();
        assert_eq!(block_hash_test, block_hash);
        remove_dir_all(BlockDB::path(&coin)).unwrap();
    }
}
