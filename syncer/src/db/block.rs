use std::convert::TryInto;
use bitcoin::BlockHash;

use crate::*;

const SYNCED_HEIGHT_KEY: &str = "synced_height";

pub struct BlockDB {
    db: RocksDBBase,
}

impl BlockDB {
    pub fn path(coin: &str) -> String {
        format!("{}/{}/block", get_data_dir_path().expect("Failed to get the data directory path."), coin)
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
    pub fn get_synced_height(&self) -> Option<u32> {
        match self.db.get(SYNCED_HEIGHT_KEY).expect("Failed to get the synced height.") {
            Some(synced_height) => {
                let buf: [u8; 4] = synced_height.try_into().unwrap();
                Some(u32::from_le_bytes(buf))
            },
            None => None,
        }
    }
    pub fn put_synced_height(&self, height: u32) {
        let height = height.to_le_bytes();
        self.db.put(SYNCED_HEIGHT_KEY, height).expect("Failed to put the synced height.");
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
    #[test]
    fn put_and_get_synced_height() {
        let height = 123456;
        let coin = "test/synced_height";
        let block_db = BlockDB::new(&coin);
        block_db.put_synced_height(height);
        let height_test = block_db.get_synced_height().unwrap();
        assert_eq!(height_test, height);
        remove_dir_all(BlockDB::path(&coin)).unwrap();
    }
}
