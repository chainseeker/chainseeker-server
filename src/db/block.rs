use bitcoin::{Txid, Block, BlockHeader, BlockHash};
use bitcoin::blockdata::constants::WITNESS_SCALE_FACTOR;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHashDBValue {
    pub block_hash: BlockHash,
}

impl Serialize for BlockHashDBValue {
    fn serialize(&self) -> Vec<u8> {
        consensus_encode(&self.block_hash)
    }
}

impl Deserialize for BlockHashDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let block_hash = consensus_decode(buf);
        Self {
            block_hash,
        }
    }
}

#[derive(Debug)]
pub struct BlockHashDB {
    /// Stores (block_height, block_hash).
    db: RocksDB<u32, BlockHashDBValue>,
}

impl BlockHashDB {
    pub fn path(coin: &str) -> String {
        format!("{}/{}/block_hash", data_dir(), coin)
    }
    pub fn new(coin: &str, temporary: bool) -> Self {
        let path = Self::path(coin);
        Self {
            db: RocksDB::new(&path, temporary),
        }
    }
    pub fn put(&self, height: u32, block: &Block) {
        self.db.put(&height, &BlockHashDBValue { block_hash: block.block_hash() });
    }
    pub fn get(&self, height: u32) -> Option<BlockHash> {
        self.db.get(&height).map(|value| value.block_hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockContentDBValue {
    pub height: u32,
    pub block_header: BlockHeader,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub txids: Vec<Txid>,
}

impl BlockContentDBValue {
    pub fn new(height: u32, block: &Block) -> Self {
        let size = block.get_size() as u32;
        let weight = block.get_weight() as u32;
        Self {
            height,
            block_header: block.header,
            size,
            // TODO: wating for upstream merge.
            //strippedsize: block.get_strippedsize() as u32,
            strippedsize: (weight - size) / ((WITNESS_SCALE_FACTOR - 1) as u32),
            weight,
            txids: block.txdata.iter().map(|tx| tx.txid()).collect(),
        }
    }
}

const BLOCK_HEADER_LEN: usize = 80;

impl Serialize for BlockContentDBValue {
    fn serialize(&self) -> Vec<u8> {
        let mut ret = vec![
            self.height.to_le_bytes().to_vec(),
            consensus_encode(&self.block_header),
            self.size.to_le_bytes().to_vec(),
            self.strippedsize.to_le_bytes().to_vec(),
            self.weight.to_le_bytes().to_vec(),
        ];
        for txid in self.txids.iter() {
            ret.push(consensus_encode(txid));
        }
        ret.concat()
    }
}

impl Deserialize for BlockContentDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let height = bytes_to_u32(&buf[0..4]);
        let mut offset = 4usize;
        let block_header = consensus_decode(&buf[offset..BLOCK_HEADER_LEN+offset]);
        offset += BLOCK_HEADER_LEN;
        let size = bytes_to_u32(&buf[offset..offset+4]);
        offset += 4;
        let strippedsize = bytes_to_u32(&buf[offset..offset+4]);
        offset += 4;
        let weight = bytes_to_u32(&buf[offset..offset+4]);
        offset += 4;
        let mut txids = Vec::new();
        while offset < buf.len() {
            txids.push(consensus_decode(&buf[offset..offset+32]));
            offset += 32;
        }
        Self {
            height,
            block_header,
            size,
            strippedsize,
            weight,
            txids,
        }
    }
}

#[derive(Debug)]
pub struct BlockContentDB {
    db: RocksDB<BlockHashDBValue, BlockContentDBValue>,
}

impl BlockContentDB {
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
        self.db.put(&BlockHashDBValue { block_hash: block.block_hash() }, &BlockContentDBValue::new(height, &block));
    }
    pub fn get(&self, block_hash: &BlockHash) -> Option<BlockContentDBValue> {
        self.db.get(&BlockHashDBValue { block_hash: *block_hash })
    }
}

#[derive(Debug)]
pub struct BlockDB {
    hash_db: BlockHashDB,
    content_db: BlockContentDB,
}

impl BlockDB {
    pub fn new(coin: &str, temporary: bool) -> Self {
        Self {
            hash_db: BlockHashDB::new(coin, temporary),
            content_db: BlockContentDB::new(coin, temporary),
        }
    }
    pub fn put(&self, height: u32, block: &Block) {
        self.hash_db.put(height, block);
        self.content_db.put(height, block);
    }
    pub fn get(&self, height: u32) -> Option<BlockContentDBValue> {
        let block_hash = match self.hash_db.get(height) {
            Some(block_hash) => block_hash,
            None => return None,
        };
        self.get_by_hash(&block_hash)
    }
    pub fn get_by_hash(&self, block_hash: &BlockHash) -> Option<BlockContentDBValue> {
        self.content_db.get(block_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn put_and_get_block() {
        let block_db = BlockDB::new("test/block", true);
        let blocks = fixtures::regtest_blocks();
        for (height, block) in blocks.iter().enumerate() {
            block_db.put(height as u32, &block);
        }
        for height in 0..blocks.len() {
            assert_eq!(block_db.get(height as u32), Some(BlockContentDBValue::new(height as u32, &blocks[height])));
        }
        assert_eq!(block_db.get(blocks.len() as u32), None);
    }
}
