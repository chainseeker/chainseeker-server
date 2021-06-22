use bitcoin::hash_types::BlockHash;

use super::*;

const SYNCED_HEIGHT_KEY: &str = "synced_height";

pub struct BlockDB {
    db: RocksDB,
}

impl BlockDB {
    pub fn get_path(coin: &str) -> String {
        format!("{}/{}/block", get_data_dir_path().expect("Failed to get the data directory path."), coin)
    }
    pub fn new(coin: &str) -> Self {
        let path = Self::get_path(coin);
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
        let synced_height_vec_option = self.db.get(SYNCED_HEIGHT_KEY).expect("Failed to get the synced height.");
        if let Some(synced_height_vec) = synced_height_vec_option {
            if synced_height_vec.len() != 4 {
                return None;
            }
            let synced_height: u32 =
                ((synced_height_vec[0] as u32) <<  0) |
                ((synced_height_vec[1] as u32) <<  8) |
                ((synced_height_vec[2] as u32) << 16) |
                ((synced_height_vec[3] as u32) << 24);
            return Some(synced_height);
        }
        None
    }
    pub fn put_synced_height(&self, height: u32) {
        let height_arr: [u8; 4] = [
            ((height >>  0) & 0xff) as u8,
            ((height >>  8) & 0xff) as u8,
            ((height >> 16) & 0xff) as u8,
            ((height >> 24) & 0xff) as u8,
        ];
        self.db.put(SYNCED_HEIGHT_KEY, height_arr).expect("Failed to put the synced height.");
    }
}

