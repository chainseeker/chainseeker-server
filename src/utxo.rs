use std::io::{Read, Write};
use std::fs::File;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use bitcoin::hash_types::Txid;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::script::Script;
use bitcoin::consensus::{Encodable, Decodable};

use super::*;

#[derive(Serialize, Deserialize)]
struct UtxoEntry {
    script_pubkey: Vec<u8>,
    txid: [u8; 32],
    vout: u32,
    value: u64,
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
    utxos: HashMap<UtxoKey, UtxoValue>,
}

impl UtxoDB {
    pub fn new() -> Self {
        Self{
            utxos: HashMap::new(),
        }
    }
    pub fn get_dir() -> String {
        format!("{}/utxo", get_data_dir_path().expect("Failed to get the data directory path."))
    }
    pub fn get_path(height: u32) -> String {
        format!("{}/{}.bin", Self::get_dir(), height)
    }
    pub fn len(&self) -> usize {
        self.utxos.len()
    }
    /// Save UTXO database to a file.
    pub fn save(&self, height: u32) {
        let path = Self::get_path(height);
        std::fs::create_dir_all(Self::get_dir()).expect("Failed to create the UTXO data directory.");
        let mut entries = Vec::with_capacity(self.utxos.len());
        for (key, value) in self.utxos.iter() {
            let mut script_pubkey = Vec::with_capacity(value.script_pubkey.len());
            script_pubkey.resize(value.script_pubkey.len(), 0);
            value.script_pubkey.consensus_encode(&mut script_pubkey).expect("Failed to encode script_pubkey.");
            let mut txid: [u8; 32] = [0; 32];
            key.txid.consensus_encode(&mut txid as &mut [u8]).expect("Failed to encode txid.");
            entries.push(UtxoEntry{
                script_pubkey,
                txid,
                vout: key.vout,
                value: value.value,
            });
        }
        let ser: Vec<u8> = bincode::serialize(&entries).expect("Failed to serialize UtxoDB.");
        let mut file = File::create(&path).expect(&format!("Failed to craete a file: {}", path));
        file.write_all(&ser).expect(&format!("Failed to write to a file: {}", path));
    }
    /// Load UTXO database from a file.
    pub fn load(height: u32) -> Self {
        let path = Self::get_path(height);
        let mut file = File::open(&path).expect(&format!("Failed to open a file: {}", path));
        let mut ser = Vec::new();
        file.read_to_end(&mut ser).expect(&format!("Failed to read from a file: {}", path));
        let entries: Vec<UtxoEntry> = bincode::deserialize(&ser).expect("Failed to deserialize UtxoDB.");
        let mut utxos = HashMap::new();
        for entry in entries.iter() {
            let script_pubkey = Script::consensus_decode(&entry.script_pubkey[..]).expect("Failed to decode script_pubkey.");
            let txid = Txid::consensus_decode(&entry.txid[..]).expect("Failed to decode txid.");
            utxos.insert(
                UtxoKey{
                    txid,
                    vout: entry.vout,
                },
                UtxoValue{
                    script_pubkey,
                    value: entry.value,
                },
            );
        }
        UtxoDB{
            utxos,
        }
    }
    pub fn delete(height: u32) -> std::io::Result<()> {
        std::fs::remove_file(Self::get_path(height))
    }
    pub fn delete_older_than(height: u32) -> u32 {
        let mut delete_cnt = 0;
        for h in (0..height).rev() {
            let result = Self::delete(h);
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
                self.utxos.insert(
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
                let val = self.utxos.remove(&key).expect("Failed to remove UTXO.");
                previous_script_pubkeys.push(val.script_pubkey);
            }
        }
        previous_script_pubkeys
    }
}
