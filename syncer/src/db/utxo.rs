use bitcoin::{Block, Txid, Script};

use crate::*;

#[derive(Debug, Clone)]
pub struct UtxoEntry {
    pub script_pubkey: Script,
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
}

#[derive(Debug)]
pub struct UtxoDB {
    /// Stores:
    ///     key   = txid || vout
    ///     value = script_pubkey || value
    pub db: RocksDB,
}

impl UtxoDB {
    pub fn new(coin: &str) -> Self {
        let path = Self::get_path(coin);
        Self {
            db: rocks_db(&path),
        }
    }
    fn get_path(coin: &str) -> String {
        format!("{}/{}/utxo", get_data_dir_path().expect("Failed to get the data directory path."), coin)
    }
    pub fn len(&self) -> usize {
        self.db.full_iterator(rocksdb::IteratorMode::Start).count()
    }
    pub fn serialize_key(txid: &Txid, vout: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(txid.to_vec());
        buf.push(vout.to_le_bytes().to_vec());
        buf.concat()
    }
    pub fn deserialize_key(buf: &[u8]) -> (Txid, u32) {
        let txid = deserialize_txid(&buf[0..32]);
        let vout = bytes_to_u32(&buf[32..36]);
        (txid, vout)
    }
    pub fn serialize_value(script_pubkey: &Script, value: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(script_pubkey.to_bytes());
        buf.push(value.to_le_bytes().to_vec());
        buf.concat()
    }
    pub fn deserialize_value(buf: &[u8]) -> (Script, u64) {
        let script_pubkey_len = buf.len() - 8;
        let script_pubkey = deserialize_script(&buf[0..script_pubkey_len]);
        let value = bytes_to_u64(&buf[script_pubkey_len..]);
        (script_pubkey, value)
    }
    pub fn process_block(&mut self, block: &Block, no_panic: bool) -> Vec<Script> {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let key = Self::serialize_key(&txid,  vout as u32);
                let output = &tx.output[vout];
                let value = Self::serialize_value(&output.script_pubkey, output.value);
                self.db.put(key, value).expect("Failed to put to DB.");
            }
        }
        // Process vins.
        let mut previous_script_pubkeys = Vec::new();
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let key = Self::serialize_key(&vin.previous_output.txid, vin.previous_output.vout);
                let value = self.db.get(&key).expect("Failed to get UTXO entry.");
                match value {
                    Some(value) => {
                        self.db.delete(&key).expect("Failed to delete UTXO entry.");
                        let (script_pubkey, _value) = Self::deserialize_value(&value);
                        previous_script_pubkeys.push(script_pubkey);
                    },
                    None => {
                        if !no_panic {
                            panic!("Failed to find UTXO entry.");
                        }
                    },
                }
            }
        }
        previous_script_pubkeys
    }
    pub async fn reorg_block(&mut self, rest: &bitcoin_rest::Context, block: &Block) {
        // Process vins.
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let txid = &vin.previous_output.txid;
                let vout = vin.previous_output.vout;
                let key = Self::serialize_key(txid, vout);
                let prev_tx = rest.tx(txid).await.expect("Failed to fetch the previous transaction.");
                let prev_out = &prev_tx.output[vout as usize];
                let script_pubkey = &prev_out.script_pubkey;
                let value = prev_out.value;
                let value = Self::serialize_value(script_pubkey, value);
                self.db.put(&key, &value).expect("Failed to put to DB.");
            }
        }
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let key = Self::serialize_key(&txid,  vout as u32);
                self.db.delete(&key).expect("Failed to delete UTXO entry.");
            }
        }
    }
}
