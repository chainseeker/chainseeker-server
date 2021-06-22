use std::io::stdout;
use std::time::Instant;
use std::collections::HashMap;

use serde::ser::{Serialize, Serializer, SerializeStruct};

use bitcoin::hash_types::Txid;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::script::Script;

use super::*;

pub struct UtxoServerValue {
    txid: Txid,
    vout: u32,
    value: u64,
}

impl Serialize for UtxoServerValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("UtxoServerValue", 3)?;
        let mut txid: [u8; 32] = [0; 32];
        self.txid.consensus_encode(&mut txid[..]).expect("Failed to encode txid.");
        state.serialize_field("txid", &hex::encode(txid))?;
        state.serialize_field("vout", &self.vout)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

pub struct UtxoServer {
    db: HashMap<Script, Vec<UtxoServerValue>>,
}

impl UtxoServer {
    pub fn new() -> Self {
        Self {
            db: HashMap::new(),
        }
    }
    pub fn get(&self, script_pubkey: &Script) -> Option<&Vec<UtxoServerValue>> {
        self.db.get(script_pubkey)
    }
}

impl From<&UtxoDB> for UtxoServer {
    fn from(utxos: &UtxoDB) -> Self {
        let begin = Instant::now();
        let mut db: HashMap<Script, Vec<UtxoServerValue>> = HashMap::new();
        let len = utxos.len();
        let mut i = 0;
        for (key, value) in utxos.db.full_iterator(rocksdb::IteratorMode::Start) {
            i += 1;
            if i % 100_000 == 0 || i == len {
                print!("\rConstructing UTXO server ({} of {})...", i, len);
                stdout().flush().expect("Failed to flush.");
            }
            let (txid, vout) = UtxoDB::deserialize_key(&key);
            let (script_pubkey, value) = UtxoDB::deserialize_value(&value);
            let cur = match db.get_mut(&script_pubkey) {
                Some(cur) => cur,
                None => {
                    let vec = Vec::new();
                    db.insert(script_pubkey.clone(), vec);
                    db.get_mut(&script_pubkey).unwrap()
                },
            };
            let v = UtxoServerValue {
                txid,
                vout,
                value,
            };
            cur.push(v);
        }
        println!(" ({}ms).", begin.elapsed().as_millis());
        Self {
            db,
        }
    }
}

pub struct UtxoDB {
    /// Stores:
    ///     key   = txid || vout
    ///     value = script_pubkey || value
    db: RocksDB,
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
    fn serialize_key(txid: &Txid, vout: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(txid.to_vec());
        buf.push(vout.to_le_bytes().to_vec());
        buf.concat()
    }
    fn deserialize_key(buf: &[u8]) -> (Txid, u32) {
        let txid = deserialize_txid(&buf[0..32]);
        let mut vout_vec: [u8; 4] = [0; 4];
        vout_vec.copy_from_slice(&buf[32..36]);
        let vout = u32::from_le_bytes(vout_vec);
        (txid, vout)
    }
    fn serialize_value(script_pubkey: &Script, value: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(script_pubkey.to_bytes());
        buf.push(value.to_le_bytes().to_vec());
        buf.concat()
    }
    fn deserialize_value(buf: &[u8]) -> (Script, u64) {
        let script_pubkey_len = buf.len() - 8;
        let script_pubkey = deserialize_script(&buf[0..script_pubkey_len]);
        let mut value_vec: [u8; 8] = [0; 8];
        value_vec.copy_from_slice(&buf[script_pubkey_len..]);
        let value = u64::from_le_bytes(value_vec);
        (script_pubkey, value)
    }
    pub fn process_block(&mut self, block: &Block) -> Vec<Script> {
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
                    None => panic!("Failed to find UTXO entry."),
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
