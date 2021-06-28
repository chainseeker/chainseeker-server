use bitcoin::BlockHash;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDBValue {
    pub block_hash: BlockHash,
}

impl BlockDBValue {
    fn new(block: &Block) -> Self {
        Self {
            block_hash: block.block_hash(),
        }
    }
}

impl Serialize for BlockDBValue {
    fn serialize(&self) -> Vec<u8> {
        serialize_block_hash(&self.block_hash).to_vec()
    }
}

impl Deserialize for BlockDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let block_hash = deserialize_block_hash(buf);
        Self {
            block_hash,
        }
    }
}

pub struct BlockDB {
    db: RocksDB<u32, BlockDBValue>,
}

impl BlockDB {
    pub fn path(coin: &str) -> String {
        format!("{}/{}/block", data_dir(), coin)
    }
    pub fn new(coin: &str, temporary: bool) -> Self {
        let path = Self::path(coin);
        Self {
            db: RocksDB::new(&path, temporary),
        }
    }
    pub fn put(&self, height: u32, block: &Block) {
        self.db.put(&height, &BlockDBValue::new(&block));
    }
    pub fn get(&self, height: u32) -> Option<BlockDBValue> {
        self.db.get(&height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const BLOCK: &[u8] = include_bytes!("../../fixtures/mainnet/block_500000.bin");
    #[test]
    fn path() {
        let path = BlockDB::path("test");
        assert_eq!(path, format!("{}/.chainseeker/test/block", std::env::var("HOME").unwrap()));
    }
    #[test]
    fn put_and_get_block() {
        let height = 123456;
        let block = deserialize_block(BLOCK);
        let block_db = BlockDB::new("test", true);
        block_db.put(height, &block);
        let value_test = block_db.get(height);
        assert_eq!(value_test, Some(BlockDBValue::new(&block)));
    }
}
