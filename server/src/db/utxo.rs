use crate::*;
use bitcoin::{Block, Txid, Script, Transaction};
use crate::rocks_db::{Serialize, Deserialize, RocksDBIterator};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtxoEntry {
    pub script_pubkey: Script,
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UtxoDBKey {
    txid: Txid,
    vout: u32,
}

impl Serialize for UtxoDBKey {
    fn serialize(&self) -> Vec<u8> {
        [self.txid.to_vec(), self.vout.to_le_bytes().to_vec()].concat()
    }
}

impl Deserialize for UtxoDBKey {
    fn deserialize(buf: &[u8]) -> Self {
        let txid = consensus_decode(&buf[0..32]);
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
        [consensus_encode(&self.script_pubkey), self.value.to_le_bytes().to_vec()].concat()
    }
}

impl Deserialize for UtxoDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let script_pubkey_len = buf.len() - 8;
        let script_pubkey = consensus_decode(&buf[0..script_pubkey_len]);
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
        let mut inserts = std::collections::HashMap::new();
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for (vout, output) in tx.output.iter().enumerate() {
                let key = UtxoDBKey {
                    txid,
                    vout: vout as u32,
                };
                let value = UtxoDBValue {
                    script_pubkey: output.script_pubkey.clone(),
                    value: output.value,
                };
                inserts.insert(key, value);
            }
        }
        // Process vins.
        let mut previous_utxos = Vec::new();
        let mut batch = rocks_db::WriteBatch::<UtxoDBKey, UtxoDBValue>::default();
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if !vin.previous_output.is_null() {
                    let txid = vin.previous_output.txid;
                    let vout = vin.previous_output.vout;
                    let key = UtxoDBKey {
                        txid,
                        vout,
                    };
                    let value = inserts.remove(&key).unwrap_or_else(|| {
                        match self.db.get(&key) {
                            Some(value) => {
                                batch.delete(&key);
                                value
                            },
                            None => {
                                if !no_panic {
                                    panic!("Failed to find UTXO entry.");
                                }
                                // Construct a dummy data.
                                UtxoDBValue {
                                    script_pubkey: Script::new(),
                                    value: 0,
                                }
                            },
                        }
                    });
                    previous_utxos.push((key, value).into());
                }
            }
        }
        for (key, value) in inserts.iter() {
            batch.put(&key, &value);
        }
        self.db.write(batch).unwrap();
        previous_utxos
    }
    pub fn reorg_block(&mut self, block: &Block, prev_txs: &[Transaction]) {
        // Process vins.
        let mut prev_tx_offset = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if !vin.previous_output.is_null() {
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
mod tests {
    use super::*;
    #[allow(dead_code)]
    fn print_utxo_db(utxo_db: &UtxoDB) {
        let mut utxos = utxo_db.iter().collect::<Vec<UtxoEntry>>();
        utxos.sort();
        for utxo in utxos.iter() {
            println!("        UtxoEntry {{ script_pubkey: consensus_decode(&Vec::from_hex(\"{}\").unwrap()), txid: consensus_decode(&Vec::from_hex(\"{}\").unwrap()), vout: {}, value: {}u64, }},",
            hex::encode(consensus_encode(&utxo.script_pubkey)),
            hex::encode(consensus_encode(&utxo.txid)),
            utxo.vout,
            utxo.value);
        }
    }
    fn find_tx(blocks: &[Block], txid: &Txid) -> Transaction {
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
        let blocks = fixtures::regtest_blocks();
        let mut utxo_db = UtxoDB::new("test/utxo", true);
        for block in blocks.iter() {
            utxo_db.process_block(&block, false);
        }
        println!("BEFORE");
        print_utxo_db(&utxo_db);
        // Test UTXO database BEFORE reorg.
        let mut utxos_test = utxo_db.iter().collect::<Vec<UtxoEntry>>();
        utxos_test.sort();
        let utxos = fixtures::utxos_before_reorg();
        assert_eq!(utxos_test, utxos);
        // Test UTXO database AFTER reorg.
        let reorged_block = fixtures::regtest_reorged_block();
        // Find previous transactions.
        let mut prev_txs = Vec::new();
        for tx in blocks.last().unwrap().txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let txid = &vin.previous_output.txid;
                let prev_tx = find_tx(&blocks, txid);
                prev_txs.push(prev_tx);
            }
        }
        utxo_db.reorg_block(&blocks.last().unwrap(), &prev_txs);
        utxo_db.process_block(&reorged_block, false);
        println!("AFTER");
        print_utxo_db(&utxo_db);
        let mut utxos_test = utxo_db.iter().collect::<Vec<UtxoEntry>>();
        utxos_test.sort();
        let utxos = fixtures::utxos_after_reorg();
        assert_eq!(utxos_test, utxos);
    }
}
