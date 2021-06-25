use bitcoin::{Block, Txid, Script};

use rayon::prelude::*;

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
        buf.push(serialize_script(script));
        buf.push(serialize_txid(&txid).to_vec());
        buf.concat()
    }
    pub fn get(&self, script: &Script) -> Vec<Txid> {
        let mut ret = Vec::new();
        let script_vec = serialize_script(script);
        for (key, _val) in self.db.prefix_iterator(&script_vec) {
            if script_vec != key[0..script.len()] {
                break;
            }
            if key.len() != script.len() + 32 {
                continue;
            }
            let txid = deserialize_txid(&key[script.len()..]);
            ret.push(txid);
        }
        ret
    }
    pub fn put(&self, script: &Script, txid: &Txid) {
        let key = Self::serialize_key(script, txid);
        self.db.put(key, Vec::new()).expect("Failed to put a database element.");
    }
    pub fn process_block(&self, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
        let mut previous_utxo_index = 0;
        let txids: Vec<Txid> = block.txdata.iter().map(|tx| {
            tx.txid()
        }).collect();
        let mut elems = Vec::new();
        for i in 0..block.txdata.len() {
            let tx = &block.txdata[i];
            let txid = &txids[i];
            // Process vins.
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                // Fetch transaction from `previous_output`.
                elems.push((&previous_utxos[previous_utxo_index].script_pubkey, txid));
                previous_utxo_index += 1;
            }
            // Process vouts.
            for vout in tx.output.iter() {
                elems.push((&vout.script_pubkey, txid));
            }
        }
        let elems: Vec<Vec<u8>> = elems.par_iter().map(|elem| {
            Self::serialize_key(elem.0, elem.1)
        }).collect();
        for elem in elems.iter() {
            self.db.put(elem, Vec::new()).unwrap();
        }
    }
}
