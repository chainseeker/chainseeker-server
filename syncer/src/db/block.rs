use bitcoin::BlockHeader;
use bitcoin::blockdata::constants::WITNESS_SCALE_FACTOR;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDBValue {
    pub block_header: BlockHeader,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub txids: Vec<Txid>,
}

impl BlockDBValue {
    fn new(block: &Block) -> Self {
        Self {
            block_header: block.header,
            size: block.get_size() as u32,
            //strippedsize: block.get_strippedsize() as u32,
            strippedsize: block.txdata.iter().map(|tx| {
                let size = tx.get_size();
                let weight = tx.get_weight();
                (weight - size) / (WITNESS_SCALE_FACTOR - 1)
            }).sum::<usize>() as u32,
            weight: block.get_weight() as u32,
            txids: block.txdata.iter().map(|tx| tx.txid()).collect(),
        }
    }
}

impl Serialize for BlockDBValue {
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        let block_header = serialize_block_header(&self.block_header);
        let block_header_len: u16 = block_header.len() as u16;
        ret.push(block_header_len.to_le_bytes().to_vec());
        ret.push(block_header);
        ret.push(self.size.to_le_bytes().to_vec());
        ret.push(self.strippedsize.to_le_bytes().to_vec());
        ret.push(self.weight.to_le_bytes().to_vec());
        for txid in self.txids.iter() {
            ret.push(serialize_txid(txid).to_vec());
        }
        ret.concat()
    }
}

impl Deserialize for BlockDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let block_header_len = bytes_to_u16(&buf[0..2]) as usize;
        let mut offset = 2usize;
        let block_header = deserialize_block_header(&buf[offset..block_header_len+offset]);
        offset += block_header_len as usize;
        let size = bytes_to_u32(&buf[offset..offset+4]);
        offset += 4;
        let strippedsize = bytes_to_u32(&buf[offset..offset+4]);
        offset += 4;
        let weight = bytes_to_u32(&buf[offset..offset+4]);
        offset += 4;
        let mut txids = Vec::new();
        while offset < buf.len() {
            txids.push(deserialize_txid(&buf[offset..offset+32]));
            offset += 32;
        }
        Self {
            block_header,
            size,
            strippedsize,
            weight,
            txids,
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
        let block_db = BlockDB::new("test/block", true);
        block_db.put(height, &block);
        let value_test = block_db.get(height);
        assert_eq!(value_test, Some(BlockDBValue::new(&block)));
    }
}
