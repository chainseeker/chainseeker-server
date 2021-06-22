use bitcoin::hash_types::Txid;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::script::Script;
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};

use super::*;

type DB = DBWithThreadMode<MultiThreaded>;

const SYNCED_HEIGHT_KEY: &str = "synced_height";

pub struct AddressIndexDB {
    db: DB,
}

/// The database which stores (script_pubkey, txid) tuple.
impl AddressIndexDB {
    pub fn get_path(coin: &str) -> String {
        format!("{}/{}/address_index", get_data_dir_path().expect("Failed to get the data directory path."), coin)
    }
    pub fn new(coin: &str) -> Self {
        let path = Self::get_path(coin);
        let mut opts = Options::default();
        opts.set_max_open_files(1000);
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).expect("Failed to open the database.");
        AddressIndexDB {
            db,
        }
    }
    fn serialize_key(script: &Script, txid: &Txid) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(script.to_bytes());
        buf.push(txid.to_vec());
        buf.concat()
    }
    /*
    fn deserialize_key(buf: &Vec<u8>) -> (Script, Txid) {
        let mut script_buf = buf.clone();
        let txid_buf = script_buf.split_off(buf.len() - 32);
        let script = Script::from(script_buf);
        let txid = Txid::consensus_decode(&txid_buf[..]).expect("Failed to decode txid.");
        (script, txid)
    }
    */
    pub fn get(&self, script: &Script) -> Vec<Txid> {
        let mut ret = Vec::new();
        for (key, _val) in self.db.prefix_iterator(script.as_bytes()) {
            if key.len() != script.len() + 32 {
                continue;
            }
            let txid = Txid::consensus_decode(&key[script.len()..]).expect("Failed to decode txid.");
            ret.push(txid);
        }
        ret
    }
    pub fn put(&self, script: &Script, txid: &Txid) {
        let key = Self::serialize_key(script, txid);
        self.db.put(key, Vec::new()).expect("Failed to put a database element.");
    }
    pub fn process_block(&self, block: &Block, previous_pubkeys: Vec<Script>) {
        let mut previous_pubkey_index = 0;
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            // Process vins.
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                // Fetch transaction from `previous_output`.
                self.put(&previous_pubkeys[previous_pubkey_index], &txid);
                previous_pubkey_index += 1;
            }
            // Process vouts.
            for vout in tx.output.iter() {
                self.put(&vout.script_pubkey, &txid);
            }
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
