use std::collections::HashMap;

use serde::ser::{Serializer, SerializeStruct};
use bitcoin::{Txid, Script, Block};

use crate::*;

pub type UtxoServer = UtxoServerInMemory;
//pub type UtxoServer = UtxoServerInStorageLazy;

#[derive(Debug, Clone, PartialEq)]
pub struct UtxoServerValue {
    pub txid: Txid,  // +32 = 32
    pub vout: u32,   // + 4 = 36
    pub value: u64,  // + 8 = 44
}

impl ConstantSize for UtxoServerValue {
    const LEN: usize = 44;
}

impl serde::ser::Serialize for UtxoServerValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("UtxoServerValue", 3)?;
        let mut txid = consensus_encode(&self.txid);
        txid.reverse();
        state.serialize_field("txid", &hex::encode(txid))?;
        state.serialize_field("vout", &self.vout)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

impl From<&UtxoServerValue> for Vec<u8> {
    fn from(value: &UtxoServerValue) -> Self {
        let mut buf: [u8; 44] = [0; 44];
        value.txid.consensus_encode(&mut buf[0..32]).expect("Failed to encode txid.");
        buf[32..36].copy_from_slice(&value.vout.to_le_bytes());
        buf[36..44].copy_from_slice(&value.value.to_le_bytes());
        buf.to_vec()
    }
}

impl Serialize for UtxoServerValue {
    fn serialize(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&[u8]> for UtxoServerValue {
    fn from(buf: &[u8]) -> UtxoServerValue {
        assert_eq!(buf.len(), 44);
        let txid = consensus_decode(&buf[0..32]);
        let vout = bytes_to_u32(&buf[32..36]);
        let value = bytes_to_u64(&buf[36..44]);
        UtxoServerValue {
            txid,
            vout,
            value,
        }
    }
}

impl Deserialize for UtxoServerValue {
    fn deserialize(buf: &[u8]) -> Self {
        buf.into()
    }
}

#[derive(Debug, Clone)]
pub struct UtxoServerInMemory {
    db: HashMap<Script, Vec<UtxoServerValue>>,
}

impl UtxoServerInMemory {
    pub fn new(_coin: &str) -> Self {
        Self {
            db: HashMap::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.db.len()
    }
    pub fn capacity(&self) -> usize {
        self.db.capacity()
    }
    pub fn size(&self) -> usize {
        self.db.iter().map(|(script, value)| script.len() + 44 * value.len()).sum()
    }
    pub fn shrink_to_fit(&mut self) {
        self.db.shrink_to_fit();
    }
    pub async fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        match self.db.get(script_pubkey) {
            Some(values) => (*values).clone(),
            None => Vec::new(),
        }
    }
    pub async fn push(&mut self, utxo: &UtxoEntry) {
        let values = match self.db.get_mut(&utxo.script_pubkey) {
            Some(values) => values,
            None => {
                self.db.insert(utxo.script_pubkey.clone(), Vec::new());
                self.db.get_mut(&utxo.script_pubkey).unwrap()
            },
        };
        let v = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
        };
        values.push(v);
    }
    fn remove(&mut self, script_pubkey: &Script, txid: &Txid, vout: u32) {
        let values = self.db.get_mut(script_pubkey).unwrap();
        *values = values.iter().filter(|&utxo_value| {
            !(utxo_value.txid == *txid && utxo_value.vout == vout)
        }).cloned().collect();
    }
    pub async fn process_block(&mut self, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let output = &tx.output[vout];
                let utxo = UtxoEntry {
                    script_pubkey: output.script_pubkey.clone(),
                    txid,
                    vout: vout as u32,
                    value: output.value,
                };
                self.push(&utxo).await;
            }
        }
        // Process vins.
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let utxo = &previous_utxos[previous_utxo_index];
                self.remove(&utxo.script_pubkey, &utxo.txid, utxo.vout);
                previous_utxo_index += 1;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
struct UtxoServerInStorageKey {
    script_pubkey: Script,
}

impl From<&Script> for UtxoServerInStorageKey {
    fn from(script_pubkey: &Script) -> Self {
        Self {
            script_pubkey: (*script_pubkey).clone(),
        }
    }
}

impl Serialize for UtxoServerInStorageKey {
    fn serialize(&self) -> Vec<u8> {
        consensus_encode(&self.script_pubkey)
    }
}

impl Deserialize for UtxoServerInStorageKey {
    fn deserialize(buf: &[u8]) -> Self {
        Self {
            script_pubkey: consensus_decode(buf),
        }
    }
}

#[derive(Debug)]
pub struct UtxoServerInStorage {
    db: RocksDBMulti<UtxoServerInStorageKey, UtxoServerValue>,
}

impl UtxoServerInStorage {
    fn path(coin: &str) -> String {
        format!("/tmp/chainseeker/{}/utxo", coin)
    }
    pub fn new(coin: &str) -> Self {
        let db = RocksDBMulti::new(&Self::path(coin), true);
        Self {
            db,
        }
    }
    pub async fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        self.db.get(&script_pubkey.into())
    }
    pub async fn push(&mut self, utxo: &UtxoEntry) {
        let value = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
        };
        self.db.push(&(&utxo.script_pubkey).into(), value);
    }
    pub async fn remove(&mut self, utxo: &UtxoEntry) {
        let key = UtxoServerInStorageKey {
            script_pubkey: utxo.script_pubkey.clone()
        };
        let value = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
        };
        self.db.pop(&key, &value);
    }
    pub async fn process_block(&mut self, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let output = &tx.output[vout];
                let utxo = UtxoEntry {
                    script_pubkey: output.script_pubkey.clone(),
                    txid,
                    vout: vout as u32,
                    value: output.value,
                };
                self.push(&utxo).await;
            }
        }
        // Process vins.
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let utxo = &previous_utxos[previous_utxo_index];
                self.remove(utxo).await;
                previous_utxo_index += 1;
            }
        }
    }
}

#[derive(Debug)]
pub struct UtxoServerInStorageLazy {
    db: RocksDBLazy<UtxoServerInStorageKey, UtxoServerValue>,
}

impl UtxoServerInStorageLazy {
    fn path(coin: &str) -> String {
        format!("/tmp/chainseeker/{}/utxo", coin)
    }
    pub fn new(coin: &str) -> Self {
        let db = RocksDBLazy::new(&Self::path(coin), true);
        let server = Self {
            db,
        };
        server
    }
    pub async fn flush(&mut self) {
        self.db.flush().await;
    }
    pub async fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        self.db.get(&script_pubkey.into()).await
    }
    pub async fn push(&mut self, utxo: &UtxoEntry) {
        let value = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
        };
        self.db.push(&(&utxo.script_pubkey).into(), &value).await;
    }
    pub async fn remove(&mut self, utxo: &UtxoEntry) {
        let key = UtxoServerInStorageKey {
            script_pubkey: utxo.script_pubkey.clone()
        };
        let value = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
        };
        self.db.remove(&key, &value).await;
    }
    pub async fn process_block(&mut self, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let output = &tx.output[vout];
                let utxo = UtxoEntry {
                    script_pubkey: output.script_pubkey.clone(),
                    txid,
                    vout: vout as u32,
                    value: output.value,
                };
                self.push(&utxo).await;
            }
        }
        // Process vins.
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let utxo = &previous_utxos[previous_utxo_index];
                self.remove(utxo).await;
                previous_utxo_index += 1;
            }
        }
    }
}
