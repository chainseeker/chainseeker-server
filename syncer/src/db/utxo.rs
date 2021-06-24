use core::ops::Range;
use std::io::stdout;
use std::time::Instant;
use std::collections::HashMap;

use serde::ser::{Serialize, Serializer, SerializeStruct};

use rayon::prelude::*;

use bitcoin::{Block, Txid, Script};

use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichListEntry {
    script_pubkey: Script,
    value: u64,
}

impl Serialize for RichListEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("RichListEntry", 2)?;
        state.serialize_field("script_pubkey", &hex::encode(serialize_script(&self.script_pubkey)))?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

impl PartialOrd for RichListEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for RichListEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[derive(Debug, Clone)]
pub struct RichList {
    /// (script_pubkey, value)
    entries: Vec<RichListEntry>,
}

impl RichList {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn get_in_range(&self, range: Range<usize>) -> Vec<RichListEntry> {
        self.entries[range].to_vec()
    }
}

impl From<&Utxo> for RichList {
    fn from(utxo: &Utxo) -> Self {
        let begin_acc = Instant::now();
        // Accumulate balances.
        let mut map: HashMap<Script, u64> = HashMap::new();
        let print_stat = |i: u32, force: bool| {
            if i % 10_000_000 == 0 || force {
                println!("RichList: processed {} entries...", i);
            }
        };
        let mut i = 0;
        for utxo in utxo.utxos.iter() {
            let value = map.get(&utxo.script_pubkey).unwrap_or(&0u64) + utxo.value;
            map.insert(utxo.script_pubkey.clone(), value);
            i += 1;
            print_stat(i, false);
        }
        print_stat(i, true);
        println!("RichList: processed in {}ms.", begin_acc.elapsed().as_millis());
        // Construct RichList instance.
        let begin_construct = Instant::now();
        let mut entries = map.par_iter().map(|(script_pubkey, value)| {
            RichListEntry {
                script_pubkey: (*script_pubkey).clone(),
                value: *value,
            }
        }).collect::<Vec<RichListEntry>>();
        println!("RichList: constructed in {}ms.", begin_construct.elapsed().as_millis());
        let begin_sort = Instant::now();
        entries.par_sort_unstable_by(|a, b| b.cmp(a));
        println!("RichList: sorted in {}ms.", begin_sort.elapsed().as_millis());
        let rich_list = RichList {
            entries,
        };
        rich_list
    }
}

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

fn deserialize_values(buf: &[u8]) -> Vec<UtxoServerValue> {
    let mut ret = Vec::new();
    for i in 0..(buf.len() / 44) {
        let buf = &buf[(44 * i)..(44 * (i + 1))];
        ret.push(buf.into());
    }
    ret
}

#[derive(Debug, Clone)]
pub struct UtxoServer {
    db: HashMap<Script, Vec<UtxoServerValue>>,
}

impl UtxoServer {
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

impl From<&Utxo> for UtxoServer {
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
    db: RocksDB,
}

impl UtxoServerInStorage {
    fn get_path(coin: &str) -> String {
        format!("/tmp/chainseeker/{}/utxo", coin)
    }
    pub fn new(coin: &str) -> Self {
        let path = Self::get_path(coin);
        if std::path::Path::new(&path).exists() {
            std::fs::remove_dir_all(&path).expect("Failed to remove directory.");
        }
        Self {
            db: rocks_db(&path),
        }
    }
    pub fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        let script_pubkey = serialize_script(script_pubkey);
        let ser = self.db.get(script_pubkey).expect("Failed to get from UTXO server DB.");
        match ser {
            Some(ser) => {
                deserialize_values(&ser[..])
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

#[derive(Debug, Clone)]
pub struct UtxoEntry {
    script_pubkey: Script,
    txid: Txid,
    vout: u32,
    value: u64,
}

#[derive(Debug, Clone)]
pub struct Utxo {
    utxos: Vec<UtxoEntry>
}

impl From<&UtxoDB> for Utxo {
    fn from(utxo_db: &UtxoDB) -> Self {
        let begin = Instant::now();
        let mut utxos = Vec::new();
        let print_stat = |i: u32, force: bool| {
            if i % 100_000 == 0 || force {
                print!("\rExtracting UTXOs ({} entries processed)...", i);
                stdout().flush().expect("Failed to flush.");
            }
        };
        let mut i = 0;
        for (key, value) in utxo_db.db.full_iterator(rocksdb::IteratorMode::Start) {
            print_stat(i, false);
            i += 1;
            let (txid, vout) = UtxoDB::deserialize_key(&key);
            let (script_pubkey, value) = UtxoDB::deserialize_value(&value);
            utxos.push(UtxoEntry {
                script_pubkey,
                txid,
                vout,
                value,
            });
        }
        print_stat(i, true);
        println!(" ({}ms).", begin.elapsed().as_millis());
        Utxo {
            utxos,
        }
    }
}

#[derive(Debug)]
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
        let vout = bytes_to_u32(&buf[32..36]);
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
        let value = bytes_to_u64(&buf[script_pubkey_len..]);
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
    pub fn create_server_in_storage(&self, coin: &str) -> UtxoServerInStorage {
        let begin = Instant::now();
        let server = UtxoServerInStorage::new(coin);
        let len = self.len();
        let mut i = 0;
        for (key, value) in self.db.full_iterator(rocksdb::IteratorMode::Start) {
            i += 1;
            if i % 100_000 == 0 || i == len {
                print!("\rConstructing UTXO server ({} of {})...", i, len);
                stdout().flush().expect("Failed to flush.");
            }
            let (txid, vout) = UtxoDB::deserialize_key(&key);
            let (script_pubkey, value) = UtxoDB::deserialize_value(&value);
            let value = UtxoServerValue {
                txid,
                vout,
                value,
            };
            server.push(&script_pubkey, value);
        }
        println!(" ({}ms).", begin.elapsed().as_millis());
        server
    }
}
