use rayon::prelude::*;
use bitcoin::{Block, Txid, Script};

use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressIndexDBKey {
    pub script_pubkey: Script,
    pub txid: Txid,
}

impl Serialize for AddressIndexDBKey {
    fn serialize(&self) -> Vec<u8> {
        AddressIndexDB::serialize_key(&self.script_pubkey, &self.txid)
    }
}

impl Deserialize for AddressIndexDBKey {
    fn deserialize(buf: &[u8]) -> Self {
        let script_pubkey = deserialize_script(&buf[0..buf.len()-32]);
        let txid = deserialize_txid(&buf[buf.len()-32..]);
        AddressIndexDBKey {
            script_pubkey,
            txid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddressIndexDBValue {
}

impl AddressIndexDBValue {
    pub fn new() -> Self {
        Self {}
    }
}

impl Serialize for AddressIndexDBValue {
    fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl Deserialize for AddressIndexDBValue {
    fn deserialize(_buf: &[u8]) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct AddressIndexDB {
    db: RocksDBBase,
}

/// The database which stores (script_pubkey, txid) tuple.
impl AddressIndexDB {
    pub fn get_path(coin: &str) -> String {
        format!("{}/{}/address_index", data_dir(), coin)
    }
    pub fn new(coin: &str) -> Self {
        let path = Self::get_path(coin);
        Self {
            db: rocks_db(&path),
        }
    }
    fn serialize_key(script: &Script, txid: &Txid) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(serialize_script(script));
        buf.push(serialize_txid(&txid).to_vec());
        buf.concat()
    }
    pub fn get(&self, script: &Script) -> Vec<Txid> {
        let mut ret = Vec::new();
        let script_vec = serialize_script(script);
        for (key, _val) in self.db.prefix_iterator(&script_vec) {
            if script_vec != key[0..script.len()] {
                break;
            }
            if key.len() != script.len() + 32 {
                continue;
            }
            let txid = deserialize_txid(&key[script.len()..]);
            ret.push(txid);
        }
        ret
    }
    pub fn put(&self, script: &Script, txid: &Txid) {
        let key = Self::serialize_key(script, txid);
        self.db.put(key, Vec::new()).expect("Failed to put a database element.");
    }
    pub fn process_block(&self, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
        let mut previous_utxo_index = 0;
        let txids: Vec<Txid> = block.txdata.iter().map(|tx| {
            tx.txid()
        }).collect();
        let mut elems = Vec::new();
        for i in 0..block.txdata.len() {
            let tx = &block.txdata[i];
            let txid = &txids[i];
            // Process vins.
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                // Fetch transaction from `previous_output`.
                elems.push((&previous_utxos[previous_utxo_index].script_pubkey, txid));
                previous_utxo_index += 1;
            }
            // Process vouts.
            for vout in tx.output.iter() {
                elems.push((&vout.script_pubkey, txid));
            }
        }
        let elems: Vec<Vec<u8>> = elems.par_iter().map(|elem| {
            Self::serialize_key(elem.0, elem.1)
        }).collect();
        for elem in elems.iter() {
            self.db.put(elem, Vec::new()).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(dead_code)]
    fn print_addr_index_db(addr_index_db: &AddressIndexDB) {
        let mut entries = addr_index_db.db.iterator(rocksdb::IteratorMode::Start).map(|(key, _value)| {
            AddressIndexDBKey::deserialize(&key)
        }).collect::<Vec<AddressIndexDBKey>>();
        entries.sort();
        for entry in entries.iter() {
            println!("        AddressIndexDBKey {{ script_pubkey: deserialize_script(&hex::decode(\"{}\").unwrap()), txid: deserialize_txid(&hex::decode(\"{}\").unwrap()), }},",
            hex::encode(serialize_script(&entry.script_pubkey)),
            hex::encode(serialize_txid(&entry.txid)));
        }
    }
    #[test]
    fn addr_index_db() {
        let blocks = test_fixtures::regtest_blocks().to_vec();
        let addr_index_db = AddressIndexDB::new("test");
        let mut utxo_db = UtxoDB::new("test", true);
        for h in 0..(blocks.len()-1) {
            let block = &blocks[h];
            let prev_utxos = utxo_db.process_block(&block, false);
            addr_index_db.process_block(&block, &prev_utxos);
        }
        //print_addr_index_db(&addr_index_db);
        let mut entries_test = addr_index_db.db.iterator(rocksdb::IteratorMode::Start).map(|(key, _value)| {
            AddressIndexDBKey::deserialize(&key)
        }).collect::<Vec<AddressIndexDBKey>>();
        entries_test.sort();
        let mut entries = test_fixtures::addr_index_db();
        entries.sort();
        assert_eq!(entries_test, entries);
    }
}
