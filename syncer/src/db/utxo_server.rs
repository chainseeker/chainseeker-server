use std::collections::HashMap;
use rand::Rng;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use bitcoin::{Txid, Script};

use crate::*;

//pub type UtxoServer = UtxoServerInMemory;
pub type UtxoServer = UtxoServerInStorage;

#[derive(Debug, Clone)]
pub struct UtxoServerValue {
    pub txid: Txid,  // +32 = 32
    pub vout: u32,   // + 4 = 36
    pub value: u64,  // + 8 = 44
}

impl Serialize for UtxoServerValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("UtxoServerValue", 3)?;
        let mut txid = serialize_txid(&self.txid);
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

impl From<&[u8]> for UtxoServerValue {
    fn from(buf: &[u8]) -> UtxoServerValue {
        assert_eq!(buf.len(), 44);
        let txid = deserialize_txid(&buf[0..32]);
        let vout = bytes_to_u32(&buf[32..36]);
        let value = bytes_to_u64(&buf[36..44]);
        UtxoServerValue {
            txid,
            vout,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UtxoServerElement {
    pub values: Vec<UtxoServerValue>
}

impl UtxoServerElement {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }
}

impl From<&UtxoServerElement> for Vec<u8> {
    fn from(element: &UtxoServerElement) -> Self {
        let mut vec = Vec::new();
        for value in element.values.iter() {
            let ser = Vec::<u8>::from(value);
            vec.push(ser);
        }
        vec.concat()
    }
}

impl From<&[u8]> for UtxoServerElement {
    fn from(buf: &[u8]) -> Self {
        let mut values = Vec::new();
        for i in 0..(buf.len() / 44) {
            let buf = &buf[(44 * i)..(44 * (i + 1))];
            values.push(buf.into());
        }
        Self {
            values
        }
    }
}

#[derive(Debug, Clone)]
pub struct UtxoServerInMemory {
    db: HashMap<Script, UtxoServerElement>,
}

impl UtxoServerInMemory {
    pub fn new() -> Self {
        Self {
            db: HashMap::new(),
        }
    }
    pub async fn get(&self, script_pubkey: &Script) -> UtxoServerElement {
        match self.db.get(script_pubkey) {
            Some(element) => (*element).clone(),
            None => UtxoServerElement::new(),
        }
    }
    pub async fn balance(&self, script_pubkey: &Script) -> u64 {
        let element = self.get(script_pubkey).await;
        let mut value = 0u64;
        for v in element.values.iter() {
            value += v.value;
        }
        value
    }
    pub async fn push(&mut self, utxo: &UtxoEntry) {
        let element = match self.db.get_mut(&utxo.script_pubkey) {
            Some(element) => element,
            None => {
                self.db.insert(utxo.script_pubkey.clone(), UtxoServerElement::new());
                self.db.get_mut(&utxo.script_pubkey).unwrap()
            },
        };
        let v = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
        };
        element.values.push(v);
    }
    pub fn remove(&mut self, script_pubkey: &Script, txid: &Txid, vout: u32) {
        let element = self.db.get_mut(script_pubkey).unwrap();
        element.values = element.values.iter().filter(|&utxo_value| {
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

#[derive(Debug)]
pub struct UtxoServerInStorage {
    path: String,
    db: RocksDBLazy,
}

impl UtxoServerInStorage {
    fn get_random_path() -> String {
        let rand_string: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        format!("/tmp/chainseeker/utxo/{}", rand_string)
    }
    fn get_path() -> String {
        loop {
            let path = Self::get_random_path();
            if !std::path::Path::new(&path).exists() {
                return path;
            }
        }
    }
    pub fn new() -> Self {
        let path = Self::get_path();
        if std::path::Path::new(&path).exists() {
            std::fs::remove_dir_all(&path).expect("Failed to remove directory.");
        }
        let db = RocksDBLazy::new(&path);
        let server = Self {
            path,
            db,
        };
        server.run();
        server
    }
    pub fn run(&self) {
        self.db.run();
    }
    pub async fn get(&self, script_pubkey: &Script) -> UtxoServerElement {
        self.db.get(script_pubkey).await
    }
    pub async fn push(&mut self, utxo: &UtxoEntry) {
        let value = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
        };
        self.db.push(&utxo.script_pubkey, value).await;
    }
    pub async fn remove(&mut self, script_pubkey: &Script, txid: &Txid, vout: u32) {
        self.db.remove(script_pubkey, txid, vout).await;
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
                self.remove(&utxo.script_pubkey, &utxo.txid, utxo.vout).await;
                previous_utxo_index += 1;
            }
        }
    }
}

impl Drop for UtxoServerInStorage {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("Failed to remove UtxoServerInStorage directory.");
    }
}
