use std::fs::File;
use std::time::Instant;
use std::collections::{HashSet, HashMap};
use bitcoin::hash_types::Txid;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::script::Script;

use super::*;

pub struct AddressIndexDB {
    db: HashMap<Script, HashSet<Txid>>,
    pub synced_height: Option<u32>,
}

/// The database which stores (script_pubkey, txid) tuple.
impl AddressIndexDB {
    pub fn get_path() -> String {
        format!("{}/address_index.bin", get_data_dir_path().expect("Failed to get the data directory path."))
    }
    pub fn get_path_tmp() -> String {
        format!("{}/address_index.tmp.bin", get_data_dir_path().expect("Failed to get the data directory path."))
    }
    pub fn save(&self) {
        let begin = Instant::now();
        let path = Self::get_path_tmp();
        std::fs::create_dir_all(get_data_dir_path().expect("Failed to get the data directory path."))
            .expect("Failed to create the UTXO data directory.");
        let mut file = File::create(&path).expect(&format!("Failed to craete a file: {}", path));
        // Write synced_height.
        write_u32(&mut file, self.synced_height.unwrap());
        // Write the number of entries.
        write_usize(&mut file, self.db.len());
        let mut i = 0;
        for (script_pubkey, txids) in self.db.iter() {
            i += 1;
            print!("\rSaving address index database ({} of {})...", i, self.db.len());
            // Write the byte length of script_pubkey.
            write_usize(&mut file, script_pubkey.len());
            // Write script_pubkey.
            let script_pubkey = serialize_script(&script_pubkey);
            write_arr(&mut file, &script_pubkey);
            // Write the number of txids.
            write_usize(&mut file, txids.len());
            // Write txids.
            for txid in txids.iter() {
                let txid_vec = serialize_txid(&txid);
                write_arr(&mut file, &txid_vec);
            }
        }
        println!(" ({}ms)", begin.elapsed().as_millis());
        std::fs::rename(Self::get_path_tmp(), Self::get_path()).expect("Failed to rename address index DB tmp file.");
    }
    pub fn load() -> Self {
        let begin = Instant::now();
        let path = Self::get_path();
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_err) => return AddressIndexDB{
                synced_height: None,
                db: HashMap::new(),
            },
        };
        // Read synced_height.
        let synced_height = read_u32(&mut file);
        // Read the number of entries.
        let n_entries = read_usize(&mut file);
        let mut db = HashMap::new();
        for i in 0..n_entries {
            print!("\rLoading address index database ({} of {})...", i + 1, n_entries);
            // Read the byte length of script_pubkey.
            let script_pubkey_len = read_usize(&mut file);
            // Read script_pubkey.
            let script_pubkey_vec = read_vec(&mut file, script_pubkey_len);
            let script_pubkey = deserialize_script(&script_pubkey_vec);
            // Read the number of txids.
            let n_txids = read_usize(&mut file);
            // Read txids.
            let mut txids = HashSet::new();
            for _ in 0..n_txids {
                txids.insert(deserialize_txid(&read_vec(&mut file, 32)));
            }
            db.insert(script_pubkey, txids);
        }
        println!(" ({}ms).", begin.elapsed().as_millis());
        AddressIndexDB{
            synced_height: Some(synced_height),
            db,
        }
    }
    pub fn put(&mut self, script_pubkey: Script, txid: Txid) {
        match self.db.get_mut(&script_pubkey) {
            Some(txids) => {
                txids.insert(txid);
            },
            None => {
                let mut txids = HashSet::new();
                txids.insert(txid);
                self.db.insert(script_pubkey, txids);
            },
        };
    }
    pub fn process_block(&mut self, block: &Block, previous_pubkeys: Vec<Script>) {
        let mut previous_pubkey_index = 0;
        // Process vins.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                self.put(previous_pubkeys[previous_pubkey_index].clone(), txid);
                previous_pubkey_index += 1;
            }
        }
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in tx.output.iter() {
                self.put(vout.script_pubkey.clone(), txid);
            }
        }
        self.synced_height = match self.synced_height {
            Some(h) => Some(h + 1),
            None => Some(0),
        };
    }
}
