use bitcoin::hash_types::Txid;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::script::Script;
use bitcoin::consensus::{Encodable, Decodable};

use super::*;

type DB = rocksdb::DBWithThreadMode<rocksdb::MultiThreaded>;

const SYNCED_HEIGHT_KEY: &str = "synced_height";

pub struct AddressIndexDB {
    db: DB,
}

/// The database which stores (script_pubkey, txid) tuple.
impl AddressIndexDB {
    pub fn new() -> Self {
        let path = get_data_dir_path().expect("Failed to get the data directory path.") + "/address_index";
        let mut db_options = rocksdb::Options::default();
        db_options.create_if_missing(true);
        db_options.increase_parallelism(num_cpus::get() as i32);
        db_options.set_db_write_buffer_size(128 * 1024 * 1024);
        let db = DB::open(&db_options, path).expect("Failed to open the database.");
        AddressIndexDB {
            db,
        }
    }
    pub fn get(&self, script: &Script) -> Vec<Txid> {
        let txid_vec_option = self.db.get(script.as_bytes()).expect("Failed to get a database element.");
        if let Some(txid_vec) = txid_vec_option {
            let mut txids = Vec::new();
            for i in 0..(txid_vec.len() / 32) {
                let txid = Txid::consensus_decode(&txid_vec[(i * 32)..((i + 1) * 32)]).expect("Failed to decode a txid.");
                txids.push(txid);
            }
            return txids;
        }
        Vec::new()
    }
    pub fn put(&self, script: &Script, txid: &Txid) {
        let mut txids = self.get(&script);
        let mut found = false;
        for txid2 in txids.iter() {
            if txid == txid2 {
                found = true;
                break;
            }
        }
        if found {
            return;
        }
        txids.push(*txid);
        let mut txids_ser = Vec::with_capacity(txids.len() * 32);
        txids_ser.resize(txids.len() * 32, 0);
        for i in 0..txids.len() {
            txids[i].consensus_encode(&mut txids_ser[(i * 32)..((i + 1) * 32)]).expect("Failed to encode a txid.");
        }
        //println!("{}", txid);
        self.db.put(script.as_bytes(), txids_ser).expect("Failed to put a database element.");
    }
    pub fn flush(&self) {
        self.db.flush().expect("Failed to flush to address index DB.");
    }
    pub fn process_block(&self, block: &Block, previous_pubkeys: Vec<Script>) {
        let mut previous_pubkey_index = 0;
        // Process vins.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                self.put(&previous_pubkeys[previous_pubkey_index], &txid);
                previous_pubkey_index += 1;
            }
        }
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in tx.output.iter() {
                self.put(&vout.script_pubkey, &txid);
            }
        }
    }
    pub fn get_synced_height(&self) -> Option<u32> {
        let synced_height_vec_option = self.db.get(SYNCED_HEIGHT_KEY).expect("Failed to get the synced height.");
        if let Some(synced_height_vec) = synced_height_vec_option {
            if synced_height_vec.len() != 4 {
                return None;
            }
            let synced_height: u32 =
                ((synced_height_vec[0] as u32) <<  0) |
                ((synced_height_vec[1] as u32) <<  8) |
                ((synced_height_vec[2] as u32) << 16) |
                ((synced_height_vec[3] as u32) << 24);
            return Some(synced_height);
        }
        None
    }
    pub fn put_synced_height(&self, height: u32) {
        let height_arr: [u8; 4] = [
            ((height >>  0) & 0xff) as u8,
            ((height >>  8) & 0xff) as u8,
            ((height >> 16) & 0xff) as u8,
            ((height >> 24) & 0xff) as u8,
        ];
        self.db.put(SYNCED_HEIGHT_KEY, height_arr).expect("Failed to put the synced height.");
    }
}
