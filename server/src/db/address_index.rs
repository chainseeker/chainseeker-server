use crate::*;
use bitcoin::{Block, Txid, Script};
use crate::rocks_db::{Serialize, Deserialize, ConstantSize};
use crate::db::utxo::UtxoEntry;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressIndexDBKey {
    pub script_pubkey: Script,
    pub txid: Txid,
}

impl Serialize for AddressIndexDBKey {
    fn serialize(&self) -> Vec<u8> {
        [consensus_encode(&self.script_pubkey), consensus_encode(&self.txid)].concat()
    }
}

impl Deserialize for AddressIndexDBKey {
    fn deserialize(buf: &[u8]) -> Self {
        let script_pubkey = consensus_decode(&buf[0..buf.len()-32]);
        let txid = consensus_decode(&buf[buf.len()-32..]);
        AddressIndexDBKey {
            script_pubkey,
            txid,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AddressIndexDBValue {
    confirmed_height: Option<u32>,
}

impl ConstantSize for AddressIndexDBValue {
    const LEN: usize = 4;
}

impl From<Option<u32>> for AddressIndexDBValue {
    fn from(confirmed_height: Option<u32>) -> Self {
        Self { confirmed_height }
    }
}

impl Serialize for AddressIndexDBValue {
    fn serialize(&self) -> Vec<u8> {
        let confirmed_height: i32 = self.confirmed_height.map_or(-1i32, |h| h as i32);
        confirmed_height.to_le_bytes().to_vec()
    }
}

impl Deserialize for AddressIndexDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let confirmed_height = bytes_to_i32(buf);
        Self {
            confirmed_height: if confirmed_height < 0 { None } else { Some(confirmed_height as u32) },
        }
    }
}

#[derive(Debug)]
pub struct AddressIndexDB {
    db: RocksDB<AddressIndexDBKey, AddressIndexDBValue>,
}

/// The database which stores (script_pubkey, txid) tuple.
impl AddressIndexDB {
    pub fn get_path(coin: &str) -> String {
        format!("{}/{}/address_index", data_dir(), coin)
    }
    pub fn new(coin: &str, temporary: bool) -> Self {
        let path = Self::get_path(coin);
        Self {
            db: RocksDB::new(&path, temporary),
        }
    }
    pub fn get(&self, script_pubkey: &Script) -> Vec<Txid> {
        let script_pubkey = consensus_encode(script_pubkey);
        let mut txids = self.db.prefix_iter(script_pubkey)
            .map(|(key, value)| (key.txid, value.confirmed_height))
            .collect::<Vec<(bitcoin::Txid, Option<u32>)>>();
        txids.sort_by(|a, b| {
            if a.1 == b.1 {
                return std::cmp::Ordering::Equal;
            }
            if a.1.is_none() {
                return std::cmp::Ordering::Less;
            }
            if b.1.is_none() {
                return std::cmp::Ordering::Greater;
            }
            b.1.unwrap().cmp(&a.1.unwrap())
        });
        txids.iter().map(|d| d.0).collect()
    }
    pub fn put(&self, script_pubkey: &Script, txid: &Txid, confirmed_height: Option<u32>) {
        let key = AddressIndexDBKey {
            script_pubkey: (*script_pubkey).clone(),
            txid: *txid,
        };
        self.db.put(&key, &confirmed_height.into());
    }
    pub fn process_tx(&self, tx: &bitcoin::Transaction, previous_utxos: &[UtxoEntry], height: Option<u32>) -> usize {
        let mut previous_utxo_index = 0;
        let txid = tx.txid();
        // Process vins.
        for vin in tx.input.iter() {
            if !vin.previous_output.is_null() {
                // Fetch transaction from `previous_output`.
                self.put(&previous_utxos[previous_utxo_index].script_pubkey, &txid, height);
                previous_utxo_index += 1;
            }
        }
        // Process vouts.
        for vout in tx.output.iter() {
            self.put(&vout.script_pubkey, &txid, height);
        }
        previous_utxo_index
    }
    pub fn process_block(&self, height: u32, block: &Block, previous_utxos: &[UtxoEntry]) {
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            previous_utxo_index += self.process_tx(&tx, &previous_utxos[previous_utxo_index..], Some(height));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::utxo::UtxoDB;
    use super::*;
    #[allow(dead_code)]
    fn print_addr_index_db(addr_index_db: &AddressIndexDB) {
        let mut entries = addr_index_db.db.iter().map(|(key, _value)| key).collect::<Vec<AddressIndexDBKey>>();
        entries.sort();
        for entry in entries.iter() {
            println!("        AddressIndexDBKey {{ script_pubkey: consensus_decode(&hex::decode(\"{}\").unwrap()), txid: consensus_decode(&hex::decode(\"{}\").unwrap()), }},",
                hex::encode(consensus_encode(&entry.script_pubkey)),
                hex::encode(consensus_encode(&entry.txid)));
        }
    }
    #[test]
    fn addr_index_db() {
        let addr_index_db = AddressIndexDB::new("test/address_index", true);
        let mut utxo_db = UtxoDB::new("test/address_index", true);
        for (height, block) in fixtures::regtest_blocks().iter().enumerate() {
            let prev_utxos = utxo_db.process_block(&block, false);
            addr_index_db.process_block(height as u32, &block, &prev_utxos);
        }
        print_addr_index_db(&addr_index_db);
        let mut entries_test = addr_index_db.db.iter().map(|(key, _value)| key).collect::<Vec<AddressIndexDBKey>>();
        entries_test.sort();
        let mut entries = fixtures::addr_index_db();
        entries.sort();
        assert_eq!(entries_test, entries);
        for entry in entries.iter() {
            let mut found = false;
            for txid in addr_index_db.get(&entry.script_pubkey).iter() {
                if *txid == entry.txid {
                    found = true;
                    break;
                }
            }
            assert!(found);
        }
    }
}
