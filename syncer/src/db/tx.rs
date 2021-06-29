use bitcoin::{Transaction, Txid, TxOut, Block};

use crate::*;

#[derive(Debug, Clone)]
pub struct TxDBKey {
    txid: Txid,
}

impl Serialize for TxDBKey {
    fn serialize(&self) -> Vec<u8> {
        consensus_encode(&self.txid)
    }
}

impl Deserialize for TxDBKey {
    fn deserialize(buf: &[u8]) -> Self {
        Self {
            txid: consensus_decode(buf)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxDBValue {
    confirmed_height: u32,
    tx: Transaction,
    previous_txouts: Vec<TxOut>,
}

impl Serialize for TxDBValue {
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.push(self.confirmed_height.to_le_bytes().to_vec());
        let tx = consensus_encode(&self.tx);
        let tx_len = tx.len() as u32;
        ret.push(tx_len.to_le_bytes().to_vec());
        ret.push(tx);
        for txout in self.previous_txouts.iter() {
            let txout = consensus_encode(txout);
            let txout_len = txout.len() as u32;
            ret.push(txout_len.to_le_bytes().to_vec());
            ret.push(txout);
        }
        ret.concat()
    }
}

impl Deserialize for TxDBValue {
    fn deserialize(buf: &[u8]) -> Self {
        let confirmed_height = bytes_to_u32(&buf[0..4]);
        let tx_len = bytes_to_u32(&buf[4..8]) as usize;
        let tx = consensus_decode(&buf[8..tx_len+8]);
        let mut offset: usize = tx_len + 8;
        let mut previous_txouts = Vec::new();
        while offset < buf.len() {
            let txout_len = bytes_to_u32(&buf[offset..offset+4]) as usize;
            offset += 4;
            let txout = consensus_decode(&buf[offset..txout_len+offset]);
            offset += txout_len;
            previous_txouts.push(txout);
        }
        Self {
            confirmed_height,
            tx,
            previous_txouts,
        }
    }
}

#[derive(Debug)]
pub struct TxDB {
    db: RocksDB<TxDBKey, TxDBValue>,
}

impl TxDB {
    pub fn path(coin: &str) -> String {
        format!("{}/{}/tx", data_dir(), coin)
    }
    pub fn new(coin: &str, temporary: bool) -> Self {
        let path = Self::path(coin);
        Self {
            db: RocksDB::new(&path, temporary),
        }
    }
    pub fn put(&self, txid: &Txid, value: &TxDBValue) {
        self.db.put(&TxDBKey { txid: (*txid).clone() }, value);
    }
    pub fn get(&self, txid: &Txid) -> Option<TxDBValue> {
        self.db.get(&TxDBKey { txid: (*txid).clone() })
    }
    pub fn process_block(&self, confirmed_height: u32, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            // Process vins.
            let mut previous_txouts = Vec::new();
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let txout = TxOut {
                    value: previous_utxos[previous_utxo_index].value,
                    script_pubkey: previous_utxos[previous_utxo_index].script_pubkey.clone(),
                };
                previous_txouts.push(txout);
                previous_utxo_index += 1;
            }
            let value = TxDBValue {
                confirmed_height,
                tx: (*tx).clone(),
                previous_txouts,
            };
            self.put(&tx.txid(), &value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn put_and_get_transactions() {
        let blocks = test_fixtures::regtest_blocks().to_vec();
        let mut utxo_db = UtxoDB::new("test/tx", true);
        let tx_db = TxDB::new("test/tx", true);
        let mut previous_utxos_vec = Vec::new();
        for height in 0..blocks.len()-1 {
            let previous_utxos = utxo_db.process_block(&blocks[height], true);
            tx_db.process_block(height as u32, &blocks[height], &previous_utxos);
            previous_utxos_vec.push(previous_utxos);
        }
        for height in 0..blocks.len()-1 {
            let mut previous_utxo_index = 0;
            for tx in blocks[height].txdata.iter() {
                let value = tx_db.get(&tx.txid()).unwrap();
                assert_eq!(value.confirmed_height, height as u32);
                assert_eq!(value.tx, *tx);
                for vin in tx.input.iter() {
                    if vin.previous_output.is_null() {
                        continue;
                    }
                    let txout = TxOut {
                        value: previous_utxos_vec[height][previous_utxo_index].value,
                        script_pubkey: previous_utxos_vec[height][previous_utxo_index].script_pubkey.clone(),
                    };
                    assert_eq!(value.previous_txouts[previous_utxo_index], txout);
                    previous_utxo_index += 1;
                }
            }
        }
    }
}
