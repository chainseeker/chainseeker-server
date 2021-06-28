use rayon::prelude::*;
use bitcoin::{Block, Txid, Script, Transaction};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtxoEntry {
    pub script_pubkey: Script,
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
}

#[derive(Debug, Clone)]
pub struct UtxoDBKey {
    txid: Txid,
    vout: u32,
}

impl Serialize for UtxoDBKey {
    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.txid.to_vec());
        buf.push(self.vout.to_le_bytes().to_vec());
        buf.concat()
    }
}

impl Deserialize for UtxoDBKey {
    fn deserialize(buf: &[u8]) -> Self {
        let txid = deserialize_txid(&buf[0..32]);
        let vout = bytes_to_u32(&buf[32..36]);
        Self {
            txid,
            vout,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UtxoDBValue {
    script_pubkey: Script,
    value: u64,
}

impl Serialize for UtxoDBValue {
    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.script_pubkey.to_bytes());
        buf.push(self.value.to_le_bytes().to_vec());
        buf.concat()
    }
}

impl Deserialize for UtxoDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let script_pubkey_len = buf.len() - 8;
        let script_pubkey = deserialize_script(&buf[0..script_pubkey_len]);
        let value = bytes_to_u64(&buf[script_pubkey_len..]);
        Self {
            script_pubkey,
            value,
        }
    }
}

impl From<(UtxoDBKey, UtxoDBValue)> for UtxoEntry {
    fn from(data: (UtxoDBKey, UtxoDBValue)) -> Self {
        UtxoEntry {
            script_pubkey: data.1.script_pubkey,
            txid: data.0.txid,
            vout: data.0.vout,
            value: data.1.value,
        }
    }
}

pub struct UtxoDBIterator<'a> {
    iter: RocksDBIterator<'a, UtxoDBKey, UtxoDBValue>,
}

impl<'a> Iterator for UtxoDBIterator<'a> {
    type Item = UtxoEntry;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();
        match next {
            Some((key, value)) => {
                let utxo: UtxoEntry = (key, value).into();
                Some(utxo)
            },
            None => None,
        }
    }
}

pub struct UtxoDB {
    /// Stores:
    ///     key   = txid || vout
    ///     value = script_pubkey || value
    pub db: RocksDB<UtxoDBKey, UtxoDBValue>,
}

impl UtxoDB {
    pub fn new(coin: &str, temporary: bool) -> Self {
        let path = Self::get_path(coin);
        Self {
            db: RocksDB::new(&path, temporary),
        }
    }
    fn get_path(coin: &str) -> String {
        format!("{}/{}/utxo", data_dir(), coin)
    }
    pub fn iter(&self) -> UtxoDBIterator {
        UtxoDBIterator {
            iter: self.db.iter(),
        }
    }
    pub fn process_block(&mut self, block: &Block, no_panic: bool) -> Vec<UtxoEntry> {
        // Process vouts.
        let inserts = block.txdata.par_iter().map(|tx| {
            let mut inserts = Vec::new();
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let output = &tx.output[vout];
                let key = UtxoDBKey {
                    txid,
                    vout: vout as u32,
                };
                let value = UtxoDBValue {
                    script_pubkey: output.script_pubkey.clone(),
                    value: output.value,
                };
                inserts.push((key ,value));
            }
            inserts
        }).collect::<Vec<Vec<(UtxoDBKey, UtxoDBValue)>>>().concat();
        for insert in inserts.iter() {
            self.db.put(&insert.0, &insert.1);
        }
        // Process vins.
        let keys = block.txdata.par_iter().map(|tx| {
            let mut keys = Vec::new();
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let txid = vin.previous_output.txid;
                let vout = vin.previous_output.vout;
                let key = UtxoDBKey {
                    txid,
                    vout,
                };
                keys.push(key);
            }
            keys
        }).collect::<Vec<Vec<UtxoDBKey>>>().concat();
        let mut previous_utxos = Vec::new();
        for key in keys.iter() {
            let value = self.db.get(&key);
            match value {
                Some(value) => {
                    self.db.delete(&key);
                    let utxo = UtxoEntry {
                        script_pubkey: value.script_pubkey,
                        txid: key.txid,
                        vout: key.vout,
                        value: value.value,
                    };
                    previous_utxos.push(utxo);
                },
                None => {
                    if !no_panic {
                        panic!("Failed to find UTXO entry.");
                    }
                },
            }
        }
        previous_utxos
    }
    pub fn reorg_block(&mut self, block: &Block, prev_txs: &Vec<Transaction>) {
        // Process vins.
        let mut prev_tx_offset = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let txid = &vin.previous_output.txid;
                let vout = vin.previous_output.vout;
                let key = UtxoDBKey {
                    txid: *txid,
                    vout,
                };
                let prev_tx = &prev_txs[prev_tx_offset];
                prev_tx_offset += 1;
                let prev_out = &prev_tx.output[vout as usize];
                let script_pubkey = &prev_out.script_pubkey;
                let value = prev_out.value;
                let value = UtxoDBValue {
                    script_pubkey: (*script_pubkey).clone(),
                    value,
                };
                self.db.put(&key, &value);
            }
        }
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let key = UtxoDBKey {
                    txid,
                    vout: vout as u32,
                };
                self.db.delete(&key);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(dead_code)]
    fn print_utxo_db(utxo_db: &UtxoDB) {
        let mut utxos = utxo_db.iter().collect::<Vec<UtxoEntry>>();
        utxos.sort();
        for utxo in utxos.iter() {
            if utxo.value == 0 {
                continue;
            }
            println!("        UtxoEntry {{ script_pubkey: deserialize_script(&hex::decode(\"{}\").unwrap()), txid: deserialize_txid(&hex::decode(\"{}\").unwrap()), vout: {}, value: {}u64, }},",
            hex::encode(serialize_script(&utxo.script_pubkey)),
            hex::encode(serialize_txid(&utxo.txid)),
            utxo.vout,
            utxo.value);
        }
    }
    fn find_tx(blocks: &Vec<Block>, txid: &Txid) -> Transaction {
        for block in blocks.iter() {
            for tx in block.txdata.iter() {
                if tx.txid() == *txid {
                    return (*tx).clone();
                }
            }
        }
        panic!("Failed to find the transaction with txid = {}.", txid);
    }
    #[test]
    fn utxo_db() {
        let blocks = test_fixtures::regtest_blocks().to_vec();
        let mut utxo_db = UtxoDB::new("test", true);
        for h in 0..(blocks.len()-1) {
            utxo_db.process_block(&blocks[h], false);
        }
        // Test UTXO database BEFORE reorg.
        let mut utxos_test = utxo_db.iter().filter(|utxo| utxo.value != 0).collect::<Vec<UtxoEntry>>();
        utxos_test.sort();
        let mut utxos = test_fixtures::utxos_before_reorg();
        utxos.sort();
        assert_eq!(utxos_test, utxos);
        // Test UTXO database AFTER reorg.
        // Find previous transactions.
        let mut prev_txs = Vec::new();
        for tx in blocks[blocks.len()-2].txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let txid = &vin.previous_output.txid;
                let prev_tx = find_tx(&blocks, txid);
                prev_txs.push(prev_tx);
            }
        }
        utxo_db.reorg_block(&blocks[blocks.len()-2], &prev_txs);
        utxo_db.process_block(&blocks[blocks.len()-1], false);
        let mut utxos_test = utxo_db.iter().filter(|utxo| utxo.value != 0).collect::<Vec<UtxoEntry>>();
        utxos_test.sort();
        let mut utxos = test_fixtures::utxos_after_reorg();
        utxos.sort();
        assert_eq!(utxos_test, utxos);
    }
}
