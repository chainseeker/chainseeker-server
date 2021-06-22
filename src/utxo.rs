use std::fs::File;
use std::io::{stdout, BufReader, BufWriter};
use std::time::Instant;
use std::collections::HashMap;

use serde::ser::{Serialize, Serializer, SerializeStruct};

use bitcoin::hash_types::{Txid, BlockHash};
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
        let mut i = 0;
        for (key, value) in utxos.db.iter() {
            i += 1;
            if i % 100_000 == 0 || i == utxos.db.len() {
                print!("\rConstructing UTXO server ({} of {})...", i, utxos.db.len());
                stdout().flush().expect("Failed to flush.");
            }
            let script_pubkey = value.script_pubkey.clone();
            let cur = match db.get_mut(&script_pubkey) {
                Some(cur) => cur,
                None => {
                    let vec = Vec::new();
                    db.insert(script_pubkey.clone(), vec);
                    db.get_mut(&script_pubkey).unwrap()
                },
            };
            let v = UtxoServerValue {
                txid: key.txid,
                vout: key.vout,
                value: value.value,
            };
            cur.push(v);
        }
        println!(" ({}ms).", begin.elapsed().as_millis());
        Self {
            db,
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
struct UtxoKey {
    txid: Txid,
    vout: u32,
}

struct UtxoValue {
    script_pubkey: Script,
    value: u64,
}

pub struct UtxoDB {
    db: HashMap<UtxoKey, UtxoValue>,
    pub block_hash: Option<BlockHash>,
}

impl UtxoDB {
    pub fn new() -> Self {
        Self{
            db: HashMap::new(),
            block_hash: None,
        }
    }
    fn get_dir(coin: &str) -> String {
        format!("{}/{}/utxo", get_data_dir_path().expect("Failed to get the data directory path."), coin)
    }
    fn get_path(coin: &str, height: u32) -> String {
        format!("{}/{}.bin", Self::get_dir(coin), height)
    }
    pub fn len(&self) -> usize {
        self.db.len()
    }
    /// Save UTXO database to a file.
    pub fn save(&self, coin: &str, height: u32) {
        let begin = Instant::now();
        let path = Self::get_path(coin, height);
        std::fs::create_dir_all(Self::get_dir(coin)).expect("Failed to create the UTXO data directory.");
        let file = File::create(&path).expect(&format!("Failed to craete a file: {}", path));
        let mut writer = BufWriter::new(file);
        // Write block hash.
        write_arr(&mut writer, &serialize_block_hash(&self.block_hash.unwrap()));
        // Write the number of entries.
        write_usize(&mut writer, self.db.len());
        let mut i = 0;
        for (key, value) in self.db.iter() {
            i += 1;
            if i % 100_000 == 0 || i == self.len() {
                print!("\rSaving UTXO database ({} of {})...", i, self.db.len());
                stdout().flush().expect("Failed to flush.");
            }
            let script_pubkey = serialize_script(&value.script_pubkey);
            // Write the byte length of script_pubkey.
            write_usize(&mut writer, script_pubkey.len());
            // Write script_pubkey.
            write_arr(&mut writer, &script_pubkey);
            // Write txid.
            let txid = serialize_txid(&key.txid);
            write_arr(&mut writer, &txid);
            // Write vout.
            write_u32(&mut writer, key.vout);
            // Write value.
            write_u64(&mut writer, value.value);
        }
        println!(" ({}ms)", begin.elapsed().as_millis());
    }
    /// Load UTXO database from a file.
    pub fn load(coin: &str, height: u32) -> Self {
        let begin = Instant::now();
        let path = Self::get_path(coin, height);
        let file = File::open(&path).expect(&format!("Failed to open a file: {}", path));
        let mut reader = BufReader::new(file);
        // Read block hash.
        let block_hash = deserialize_block_hash(&read_vec(&mut reader, 32));
        // Read the number of entries.
        let n_entries = read_usize(&mut reader);
        let mut db = HashMap::new();
        for i in 0..n_entries {
            if (i + 1) % 100_000 == 0 || (i + 1) == n_entries {
                print!("\rLoading UTXO database ({} of {})...", i + 1, n_entries);
                stdout().flush().expect("Failed to flush.");
            }
            // Read the byte length of script_pubkey.
            let script_pubkey_len = read_usize(&mut reader);
            // Read script_pubkey.
            let script_pubkey_vec = read_vec(&mut reader, script_pubkey_len);
            let script_pubkey = deserialize_script(&script_pubkey_vec);
            // Read txid.
            let txid = deserialize_txid(&read_vec(&mut reader, 32));
            // Read vout.
            let vout = read_u32(&mut reader);
            // Read value.
            let value = read_u64(&mut reader);
            db.insert(
                UtxoKey{
                    txid,
                    vout,
                },
                UtxoValue{
                    script_pubkey,
                    value,
                },
            );
        }
        println!(" ({}ms).", begin.elapsed().as_millis());
        UtxoDB{
            db,
            block_hash: Some(block_hash),
        }
    }
    pub fn delete(coin: &str, height: u32) -> std::io::Result<()> {
        std::fs::remove_file(Self::get_path(coin, height))
    }
    pub fn delete_older_than(coin: &str, height: u32) -> u32 {
        let mut delete_cnt = 0;
        for h in (0..height).rev() {
            let result = Self::delete(coin, h);
            if result.is_ok() {
                delete_cnt += 1;
            }
        }
        delete_cnt
    }
    pub fn process_block(&mut self, block: &Block) -> Vec<Script> {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for i in 0..tx.output.len() {
                self.db.insert(
                    UtxoKey{
                        txid,
                        vout: i as u32,
                    },
                    UtxoValue{
                        script_pubkey: tx.output[i].script_pubkey.clone(),
                        value: tx.output[i].value,
                    }
                );
            }
        }
        // Process vins.
        let mut previous_script_pubkeys = Vec::new();
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let key = UtxoKey{
                    txid: vin.previous_output.txid,
                    vout: vin.previous_output.vout,
                };
                let val = self.db.remove(&key).expect("Failed to remove UTXO.");
                previous_script_pubkeys.push(val.script_pubkey);
            }
        }
        self.block_hash = Some(block.block_hash());
        previous_script_pubkeys
    }
    pub fn reorg(&mut self, coin: &str, height: u32) -> u32 {
        for height in (0..height).rev() {
            if !std::path::Path::new(&Self::get_path(coin, height)).exists() {
                continue;
            }
            *self = Self::load(coin, height);
            return height;
        }
        panic!("Failed to reorg because no older UTXO database exists.");
    }
}
