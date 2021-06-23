use std::io::stdout;
use std::time::Instant;
use std::collections::HashMap;

use serde::ser::{Serialize, Serializer, SerializeStruct};

use bitcoin::{Block, Txid, Script};

use super::*;

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
        let mut txid: [u8; 32] = [0; 32];
        self.txid.consensus_encode(&mut txid[..]).expect("Failed to encode txid.");
        state.serialize_field("txid", &hex::encode(txid))?;
        state.serialize_field("vout", &self.vout)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

fn serialize_value(value: &UtxoServerValue) -> Vec<u8> {
    let mut buf: [u8; 44] = [0; 44];
    value.txid.consensus_encode(&mut buf[0..32]).expect("Failed to encode txid.");
    buf[32..36].copy_from_slice(&value.vout.to_le_bytes());
    buf[36..44].copy_from_slice(&value.value.to_le_bytes());
    buf.to_vec()
}

fn deserialize_values(buf: &[u8]) -> Vec<UtxoServerValue> {
    let mut ret = Vec::new();
    for i in 0..(buf.len() / 44) {
        let buf = &buf[(44 * i)..(44 * (i + 1))];
        let txid = deserialize_txid(&buf[0..32]);
        let vout = bytes_to_u32(&buf[32..36]);
        let value = bytes_to_u64(&buf[36..44]);
        ret.push(UtxoServerValue {
            txid,
            vout,
            value,
        });
    }
    ret
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
    pub fn load_from_db(&mut self, utxos: &UtxoDB) {
        self.db.clear();
        let begin = Instant::now();
        let mut i = 0;
        for (key, value) in utxos.db.full_iterator(rocksdb::IteratorMode::Start) {
            i += 1;
            if i % 100_000 == 0 {
                print!("\rConstructing UTXO server ({})...", i);
                stdout().flush().expect("Failed to flush.");
            }
            let (txid, vout) = UtxoDB::deserialize_key(&key);
            let (script_pubkey, value) = UtxoDB::deserialize_value(&value);
            let cur = match self.db.get_mut(&script_pubkey) {
                Some(cur) => cur,
                None => {
                    let vec = Vec::new();
                    self.db.insert(script_pubkey.clone(), vec);
                    self.db.get_mut(&script_pubkey).unwrap()
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
    }
}

impl From<&UtxoDB> for UtxoServer {
    fn from(utxos: &UtxoDB) -> Self {
        let mut ret = Self::new();
        ret.load_from_db(utxos);
        ret
    }
}

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
                values.append(&mut serialize_value(&value));
                self.db.put(script_pubkey, values)
            },
            None => self.db.put(script_pubkey, serialize_value(&value)),
        }.expect("Failed to put to DB.");
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

/*
pub mod sqlite {
    use super::*;
    use diesel::prelude::*;
    use super::schema::utxos::dsl::*;
    use super::schema::utxos;
    
    #[derive(Queryable)]
    pub struct Utxo {
        id: i32,
        script_pubkey: Vec<u8>,
        txid: Vec<u8>,
        vout: i32,
        value: i64,
    }
    
    #[derive(Insertable)]
    #[table_name="utxos"]
    pub struct NewUtxo<'a> {
        pub script_pubkey: &'a Vec<u8>,
        pub txid: &'a Vec<u8>,
        pub vout: &'a i32,
        pub value: &'a i64,
    }
    
    pub struct UtxoDBSQLite {
        conn: SqliteConnection,
    }
    
    impl UtxoDBSQLite {
        pub fn new(coin: &str) -> Self {
            let path = Self::get_path(coin);
            Self {
                conn: SqliteConnection::establish(&path).expect("Failed to open SQLite database."),
            }
        }
        fn get_path(coin: &str) -> String {
            format!("{}/{}/utxo.sqlite", get_data_dir_path().expect("Failed to get the data directory path."), coin)
        }
        pub fn len(&self) -> usize {
            self.db.full_iterator(rocksdb::IteratorMode::Start).count()
        }
        pub fn get(&self, my_script_pubkey: &Script) -> Vec<UtxoServerValue> {
            let my_utxos = utxos.filter(script_pubkey.eq(my_script_pubkey.as_bytes()))
                .load::<Utxo>(&self.conn)
                .expect("Failed to get UTXO entries.");
            my_utxos.iter().map(|utxo| {
                UtxoServerValue {
                    txid: deserialize_txid(&utxo.txid),
                    vout: utxo.vout as u32,
                    value: utxo.value as u64,
                }
            }).collect()
        }
        pub fn process_block(&mut self, block: &Block) -> Vec<Script> {
            // Process vouts.
            for tx in block.txdata.iter() {
                let my_txid = tx.txid();
                for my_vout in 0..tx.output.len() {
                    let output = &tx.output[my_vout];
                    let new_utxo = NewUtxo {
                        script_pubkey: &serialize_script(&output.script_pubkey),
                        txid: &serialize_txid(&my_txid).to_vec(),
                        vout: &(my_vout as i32),
                        value: &(output.value as i64),
                    };
                    diesel::insert_into(utxos::table)
                        .values(&new_utxo)
                        .execute(&self.conn)
                        .expect("Failed to insert UTXO.");
                }
            }
            // Process vins.
            let mut previous_script_pubkeys = Vec::new();
            for tx in block.txdata.iter() {
                for vin in tx.input.iter() {
                    if vin.previous_output.is_null() {
                        continue;
                    }
                    let utxo = utxos
                        .filter(txid.eq(serialize_txid(&vin.previous_output.txid).to_vec()).and(vout.eq(vout)))
                        .limit(1)
                        .load::<Utxo>(&self.conn)
                        .expect("Failed to fetch UTXO from DB.");
                    if utxo.len() == 0 {
                        panic!("Failed to find UTXO entry.");
                    }
                    let utxo = &utxo[0];
                    let my_script_pubkey = deserialize_script(&utxo.script_pubkey[..]);
                    diesel::delete(utxos.filter(txid.eq(serialize_txid(&vin.previous_output.txid).to_vec()).and(vout.eq(vout))))
                        .execute(&self.conn)
                        .expect("Failed to delte UTXO entry.");
                    previous_script_pubkeys.push(my_script_pubkey);
                }
            }
            previous_script_pubkeys
        }
        pub async fn reorg_block(&mut self, rest: &bitcoin_rest::Context, block: &Block) {
            panic!("Not implemented!");
            //// Process vins.
            //for tx in block.txdata.iter() {
            //    for vin in tx.input.iter() {
            //        if vin.previous_output.is_null() {
            //            continue;
            //        }
            //        let txid = &vin.previous_output.txid;
            //        let vout = vin.previous_output.vout;
            //        let key = Self::serialize_key(txid, vout);
            //        let prev_tx = rest.tx(txid).await.expect("Failed to fetch the previous transaction.");
            //        let prev_out = &prev_tx.output[vout as usize];
            //        let script_pubkey = &prev_out.script_pubkey;
            //        let value = prev_out.value;
            //        let value = Self::serialize_value(script_pubkey, value);
            //        self.db.put(&key, &value).expect("Failed to put to DB.");
            //    }
            //}
            //// Process vouts.
            //for tx in block.txdata.iter() {
            //    let txid = tx.txid();
            //    for vout in 0..tx.output.len() {
            //        let key = Self::serialize_key(&txid,  vout as u32);
            //        self.db.delete(&key).expect("Failed to delete UTXO entry.");
            //    }
            //}
        }
    }
}
*/
