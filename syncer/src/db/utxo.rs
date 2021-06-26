use bitcoin::{Block, Txid, Script};

use crate::*;

#[derive(Debug, Clone)]
pub struct UtxoEntry {
    pub script_pubkey: Script,
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
}

pub struct UtxoDBKey {
    txid: Txid,
    vout: u32,
}

impl Serialize for UtxoDBKey {
    fn serialize(&self) -> Vec<u8> {
        UtxoDB::serialize_key(&self.txid, self.vout)
    }
}

impl Deserialize for UtxoDBKey {
    fn deserialize(buf: &[u8]) -> Self {
        let (txid, vout) = UtxoDB::deserialize_key(buf);
        Self {
            txid,
            vout,
        }
    }
}

pub struct UtxoDBValue {
    script_pubkey: Script,
    value: u64,
}

impl Serialize for UtxoDBValue {
    fn serialize(&self) -> Vec<u8> {
        UtxoDB::serialize_value(&self.script_pubkey, self.value)
    }
}

impl Deserialize for UtxoDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let (script_pubkey, value) = UtxoDB::deserialize_value(buf);
        Self {
            script_pubkey,
            value,
        }
    }
}

impl From<(UtxoDBKey, UtxoDBValue)> for UtxoEntry {
    fn from(data: (UtxoDBKey, UtxoDBValue)) -> Self {
        UtxoEntry {
            script_pubkey: data.1.script_pubkey,
            txid: data.0.txid,
            vout: data.0.vout,
            value: data.1.value,
        }
    }
}

pub struct UtxoDB {
    /// Stores:
    ///     key   = txid || vout
    ///     value = script_pubkey || value
    pub db: kvs::RocksDB<UtxoDBKey, UtxoDBValue>,
}

impl UtxoDB {
    pub fn new(coin: &str) -> Self {
        let path = Self::get_path(coin);
        Self {
            db: kvs::RocksDB::new(&path),
        }
    }
    fn get_path(coin: &str) -> String {
        format!("{}/{}/utxo", get_data_dir_path().expect("Failed to get the data directory path."), coin)
    }
    pub fn len(&self) -> usize {
        self.db.iter().count()
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
    pub fn process_block(&mut self, block: &Block, no_panic: bool) -> Vec<UtxoEntry> {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let output = &tx.output[vout];
                let key = UtxoDBKey {
                    txid,
                    vout: vout as u32,
                };
                let value = UtxoDBValue {
                    script_pubkey: output.script_pubkey.clone(),
                    value: output.value,
                };
                self.db.put(&key, &value);
            }
        }
        // Process vins.
        let mut previous_utxos = Vec::new();
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let txid = vin.previous_output.txid;
                let vout = vin.previous_output.vout;
                let key = UtxoDBKey {
                    txid,
                    vout,
                };
                let value = self.db.get(&key);
                match value {
                    Some(value) => {
                        self.db.delete(&key);
                        let (script_pubkey, value) = Self::deserialize_value(&value);
                        let utxo = UtxoEntry {
                            script_pubkey,
                            txid,
                            vout,
                            value,
                        };
                        previous_utxos.push(utxo);
                    },
                    None => {
                        if !no_panic {
                            panic!("Failed to find UTXO entry.");
                        }
                    },
                }
            }
        }
        previous_utxos
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
                let key = UtxoDBKey {
                    txid: *txid,
                    vout,
                };
                let prev_tx = rest.tx(txid).await.expect("Failed to fetch the previous transaction.");
                let prev_out = &prev_tx.output[vout as usize];
                let script_pubkey = &prev_out.script_pubkey;
                let value = prev_out.value;
                let value = UtxoDBValue {
                    script_pubkey: (*script_pubkey).clone(),
                    value,
                };
                self.db.put(&key, &value);
            }
        }
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let key = UtxoDBKey {
                    txid,
                    vout: vout as u32,
                };
                self.db.delete(&key);
            }
        }
    }
}
