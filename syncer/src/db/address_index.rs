use bitcoin::{Block, Txid, Script};

use super::super::*;

pub struct AddressIndexDB {
    db: RocksDB,
}

/// The database which stores (script_pubkey, txid) tuple.
impl AddressIndexDB {
    pub fn get_path(coin: &str) -> String {
        format!("{}/{}/address_index", get_data_dir_path().expect("Failed to get the data directory path."), coin)
    }
    pub fn new(coin: &str) -> Self {
        let path = Self::get_path(coin);
        Self {
            db: rocks_db(&path),
        }
    }
    fn serialize_key(script: &Script, txid: &Txid) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(script.to_bytes());
        buf.push(txid.to_vec());
        buf.concat()
    }
    pub fn get(&self, script: &Script) -> Vec<Txid> {
        let mut ret = Vec::new();
        for (key, _val) in self.db.prefix_iterator(script.as_bytes()) {
            if *script.as_bytes() != key[0..script.len()] {
                break;
            }
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
}
