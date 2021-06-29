use serde::ser::{Serializer, SerializeStruct};
use bitcoin_hashes::hex::ToHex;
use bitcoin::{Txid, Block, BlockHeader, BlockHash};
use bitcoin::blockdata::constants::WITNESS_SCALE_FACTOR;
use bitcoin::network::constants::Network;

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
        match self.db.get(&height) {
            Some(value) => Some(value.block_hash),
            None => None,
        }
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
    fn new(height: u32, block: &Block) -> Self {
        let size = block.get_size() as u32;
        let weight = block.get_weight() as u32;
        Self {
            height,
            block_header: block.header,
            size,
            //strippedsize: block.get_strippedsize() as u32,
            strippedsize: (weight - size) / ((WITNESS_SCALE_FACTOR - 1) as u32),
            weight,
            txids: block.txdata.iter().map(|tx| tx.txid()).collect(),
        }
    }
}

impl serde::ser::Serialize for BlockContentDBValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("BlockContentDBValue", 14)?;
        state.serialize_field("height"           , &self.height)?;
        let block_header = &self.block_header;
        state.serialize_field("header"           , &hex::encode(consensus_encode(&block_header)))?;
        let mut hash = consensus_encode(&block_header.block_hash());
        hash.reverse();
        state.serialize_field("hash"             , &hex::encode(&hash))?;
        state.serialize_field("version"          , &block_header.version)?;
        let mut prev_blockhash = consensus_encode(&block_header.prev_blockhash);
        prev_blockhash.reverse();
        state.serialize_field("previousblockhash", &hex::encode(&prev_blockhash))?;
        let mut merkle_root = consensus_encode(&block_header.merkle_root);
        merkle_root.reverse();
        state.serialize_field("merkleroot"       , &hex::encode(&merkle_root))?;
        state.serialize_field("time"             , &block_header.time)?;
        state.serialize_field("bits"             , &format!("{:x}", block_header.bits))?;
        state.serialize_field("difficulty"       , &block_header.difficulty(Network::Bitcoin))?;
        state.serialize_field("nonce"            , &block_header.nonce)?;
        state.serialize_field("size"             , &self.size)?;
        state.serialize_field("strippedsize"     , &self.strippedsize)?;
        state.serialize_field("weight"           , &self.weight)?;
        state.serialize_field("txids"            , &self.txids.iter().map(|txid| txid.to_hex()).collect::<Vec<String>>())?;
        state.end()
    }
}

impl Serialize for BlockContentDBValue {
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.push(self.height.to_le_bytes().to_vec());
        let block_header = consensus_encode(&self.block_header);
        let block_header_len: u16 = block_header.len() as u16;
        ret.push(block_header_len.to_le_bytes().to_vec());
        ret.push(block_header);
        ret.push(self.size.to_le_bytes().to_vec());
        ret.push(self.strippedsize.to_le_bytes().to_vec());
        ret.push(self.weight.to_le_bytes().to_vec());
        for txid in self.txids.iter() {
            ret.push(consensus_encode(txid));
        }
        ret.concat()
    }
}

impl Deserialize for BlockContentDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let height = bytes_to_u32(&buf[0..4]);
        let block_header_len = bytes_to_u16(&buf[4..6]) as usize;
        let mut offset = 6usize;
        let block_header = consensus_decode(&buf[offset..block_header_len+offset]);
        offset += block_header_len as usize;
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
        self.db.get(&BlockHashDBValue { block_hash: (*block_hash).clone() })
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
    const HEIGHT: u32 = 500000;
    const BLOCK: &[u8] = include_bytes!("../../fixtures/mainnet/block_500000.bin");
    #[test]
    fn put_and_get_block() {
        let block = consensus_decode(BLOCK);
        let block_db = BlockDB::new("test/block", true);
        block_db.put(HEIGHT, &block);
        let value_test = block_db.get(HEIGHT);
        assert_eq!(value_test, Some(BlockContentDBValue::new(HEIGHT, &block)));
    }
}
