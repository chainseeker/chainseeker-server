use std::time::Instant;
use std::collections::HashMap;
use rand::Rng;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use bitcoin::{Txid, Script};

use crate::*;

pub type UtxoServer = UtxoServerInMemory;

#[derive(Debug, Clone)]
pub struct UtxoServerValue {
    txid: Txid,  // +32 = 32
    vout: u32,   // + 4 = 36
    value: u64,  // + 8 = 44
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
pub struct UtxoServerInMemory {
    db: HashMap<Script, Vec<UtxoServerValue>>,
}

impl UtxoServerInMemory {
    pub fn new() -> Self {
        Self {
            db: HashMap::new(),
        }
    }
    pub fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        match self.db.get(script_pubkey) {
            Some(values) => (*values).clone(),
            None => Vec::new(),
        }
    }
}

impl From<&Utxo> for UtxoServerInMemory {
    fn from(utxo_db: &Utxo) -> Self {
        let mut db = HashMap::new();
        let begin = Instant::now();
        let print_stat = |i: u32, force: bool| {
            if i % 10_000_000 == 0 || force {
                println!("UtxoServer: processed {} entries...", i);
            }
        };
        let mut i = 0;
        for utxo in utxo_db.utxos.iter() {
            let cur = match db.get_mut(&utxo.script_pubkey) {
                Some(cur) => cur,
                None => {
                    let vec = Vec::new();
                    db.insert(utxo.script_pubkey.clone(), vec);
                    db.get_mut(&utxo.script_pubkey).unwrap()
                },
            };
            let v = UtxoServerValue {
                txid: utxo.txid,
                vout: utxo.vout,
                value: utxo.value,
            };
            cur.push(v);
            i += 1;
            print_stat(i, false);
        }
        print_stat(i, true);
        println!("UtxoServer: processed in {}ms.", begin.elapsed().as_millis());
        Self {
            db,
        }
    }
}

#[derive(Debug)]
pub struct UtxoServerInStorage {
    path: String,
    db: RocksDB,
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
        let db = rocks_db(&path);
        Self {
            path,
            db,
        }
    }
    fn deserialize_values(buf: &[u8]) -> Vec<UtxoServerValue> {
        let mut ret = Vec::new();
        for i in 0..(buf.len() / 44) {
            let buf = &buf[(44 * i)..(44 * (i + 1))];
            ret.push(buf.into());
        }
        ret
    }
    pub fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        let script_pubkey = serialize_script(script_pubkey);
        let ser = self.db.get(script_pubkey).expect("Failed to get from UTXO server DB.");
        match ser {
            Some(ser) => {
                Self::deserialize_values(&ser[..])
            },
            None => Vec::new(),
        }
    }
    fn push(&self, script_pubkey: &Script, value: UtxoServerValue) {
        let script_pubkey = serialize_script(script_pubkey);
        let values = self.db.get(script_pubkey.clone()).expect("Failed to get from UTXO server DB.");
        match values {
            Some(mut values) => {
                values.append(&mut (&value).into());
                self.db.put(script_pubkey, values)
            },
            None => self.db.put(script_pubkey, Vec::<u8>::from(&value)),
        }.expect("Failed to put to DB.");
    }
}

impl Drop for UtxoServerInStorage {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("Failed to remove UtxoServerInStorage directory.");
    }
}

impl From<&Utxo> for UtxoServerInStorage {
    fn from(utxos: &Utxo) -> UtxoServerInStorage {
        let begin = Instant::now();
        let server = UtxoServerInStorage::new();
        let len = utxos.utxos.len();
        let mut i = 0;
        for utxo in utxos.utxos.iter() {
            i += 1;
            if i % 10_000_000 == 0 || i == len {
                println!("UtxoServerInStorage: constructing ({} of {})...", i, len);
            }
            let value = UtxoServerValue {
                txid: utxo.txid,
                vout: utxo.vout,
                value: utxo.value,
            };
            server.push(&utxo.script_pubkey, value);
        }
        println!("UtxoServerInStorage: constructed in {}ms.", begin.elapsed().as_millis());
        server
    }
}
