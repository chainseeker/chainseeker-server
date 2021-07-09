use crate::*;
use bitcoin::{Transaction, Txid, TxOut, Block};
use bitcoin::blockdata::constants::WITNESS_SCALE_FACTOR;
use crate::db::utxo::UtxoEntry;
use crate::rocks_db::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub confirmed_height: Option<u32>,
    pub tx: Transaction,
    pub previous_txouts: Vec<TxOut>,
}

impl TxDBValue {
    pub fn deserialize_as_rawtx(buf: &[u8]) -> (Option<u32>, Vec<u8>, Vec<TxOut>) {
        let confirmed_height = bytes_to_i32(&buf[0..4]);
        let confirmed_height = if confirmed_height >= 0 {
            Some(confirmed_height as u32)
        } else {
            None
        };
        let tx_len = bytes_to_u32(&buf[4..8]) as usize;
        let tx = buf[8..tx_len+8].to_vec();
        let mut offset: usize = tx_len + 8;
        let mut previous_txouts = Vec::new();
        while offset < buf.len() {
            let txout_len = bytes_to_u32(&buf[offset..offset+4]) as usize;
            offset += 4;
            let txout = consensus_decode(&buf[offset..txout_len+offset]);
            offset += txout_len;
            previous_txouts.push(txout);
        }
        (confirmed_height, tx, previous_txouts)
    }
}

impl Serialize for TxDBValue {
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        let confirmed_height = self.confirmed_height.map_or_else(|| -1i32, |confirmed_height| confirmed_height as i32);
        ret.push(confirmed_height.to_le_bytes().to_vec());
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
        let (confirmed_height, tx, previous_txouts) = Self::deserialize_as_rawtx(buf);
        Self {
            confirmed_height,
            tx: consensus_decode(&tx),
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
        self.db.put(&TxDBKey { txid: *txid }, value);
    }
    pub fn put_tx(&self, tx: &Transaction, confirmed_height: Option<u32>) -> Result<TxDBValue, Txid> {
        let mut previous_txouts = Vec::new();
        for vin in tx.input.iter() {
            if !vin.previous_output.is_null() {
                let previous_txid = &vin.previous_output.txid;
                match self.get(previous_txid) {
                    Some(previous_tx) => previous_txouts.push(previous_tx.tx.output[vin.previous_output.vout as usize].clone()),
                    None => return Err(*previous_txid),
                }
            }
        }
        let value = TxDBValue {
            confirmed_height,
            tx: (*tx).clone(),
            previous_txouts,
        };
        self.put(&tx.txid(), &value);
        Ok(value)
    }
    pub fn get(&self, txid: &Txid) -> Option<TxDBValue> {
        self.db.get(&TxDBKey { txid: *txid })
    }
    pub fn get_as_rest(&self, txid: &Txid, config: &Config) -> Option<chainseeker::Transaction> {
        //let begin_get = std::time::Instant::now();
        let buf = self.db.get_raw(&TxDBKey { txid: *txid });
        //println!("Transaction got in {}us.", begin_get.elapsed().as_micros());
        buf.map_or_else(|| None, |buf| {
            //let begin_convert = std::time::Instant::now();
            let (confirmed_height, rawtx, previous_txouts) = TxDBValue::deserialize_as_rawtx(&buf);
            let tx: Transaction = consensus_decode(&rawtx);
            let mut input_value = 0;
            let mut vin = Vec::new();
            let mut previous_txout_index = 0;
            for input in tx.input.iter() {
                if input.previous_output.is_null() {
                    vin.push(create_vin(input, &None, config));
                } else {
                    input_value += previous_txouts[previous_txout_index].value;
                    vin.push(create_vin(input, &Some(previous_txouts[previous_txout_index].clone()), config));
                    previous_txout_index += 1;
                }
            }
            let output_value: u64 = tx.output.iter().map(|output| output.value).sum();
            let tx = chainseeker::Transaction {
                confirmed_height,
                hex: hex::encode(&rawtx),
                txid: tx.txid().to_string(),
                hash: tx.wtxid().to_string(),
                size: tx.get_size(),
                // TODO: waiting for upstream merge.
                //vsize: tx.get_vsize(),
                vsize: (tx.get_weight() + WITNESS_SCALE_FACTOR - 1) / WITNESS_SCALE_FACTOR,
                weight: tx.get_weight(),
                version: tx.version,
                locktime: tx.lock_time,
                vin,
                vout: tx.output.iter().enumerate().map(|(n, vout)| create_vout(vout, n, config)).collect(),
                // TODO: compute for coinbase transactions!
                fee: (input_value as i64) - (output_value as i64),
            };
            //println!("Transaction converted in {}us.", begin_convert.elapsed().as_micros());
            Some(tx)
        })
    }
    /*
    pub fn multi_get<I: IntoIterator<Item = Txid>>(&self, txids: I) -> Vec<Option<TxDBValue>> {
        let txids: Vec<TxDBKey> = txids.into_iter().map(|txid| TxDBKey { txid }).collect();
        self.db.multi_get(txids)
    }
    */
    pub fn process_block(&self, confirmed_height: u32, block: &Block, previous_utxos: &[UtxoEntry]) {
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            // Process vins.
            let mut previous_txouts = Vec::new();
            for vin in tx.input.iter() {
                if !vin.previous_output.is_null() {
                    let txout = TxOut {
                        value: previous_utxos[previous_utxo_index].value,
                        script_pubkey: previous_utxos[previous_utxo_index].script_pubkey.clone(),
                    };
                    previous_txouts.push(txout);
                    previous_utxo_index += 1;
                }
            }
            let value = TxDBValue {
                confirmed_height: Some(confirmed_height),
                tx: (*tx).clone(),
                previous_txouts,
            };
            self.put(&tx.txid(), &value);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::db::utxo::UtxoDB;
    use super::*;
    const TXID: &str = "503e4e9824282eb06f1a328484e2b367b5f4f93a405d6e7b97261bafabfb53d5";
    #[test]
    fn key_deserialize() {
        assert_eq!(
            TxDBKey {
                txid: Txid::from_str(TXID).unwrap(),
            },
            TxDBKey::deserialize(&Txid::from_hex(TXID).unwrap()),
        );
    }
    #[test]
    fn put_unconfirmed() {
        let tx = &fixtures::regtest_blocks()[0].txdata[0];
        let tx_db = TxDB::new("test/tx/unconfirmed", true);
        tx_db.put_tx(&tx, None).unwrap();
        assert_eq!(
            tx_db.get(&tx.txid()).unwrap(),
            TxDBValue {
                confirmed_height: None,
                tx: (*tx).clone(),
                previous_txouts: Vec::new(),
            },
        );
    }
    #[test]
    fn put_confirmed() {
        let blocks = fixtures::regtest_blocks();
        let mut utxo_db = UtxoDB::new("test/tx/confirmed", true);
        let tx_db = TxDB::new("test/tx/confirmed", true);
        let mut previous_utxos_vec = Vec::new();
        for (height, block) in blocks.iter().enumerate() {
            let previous_utxos = utxo_db.process_block(&block, true);
            tx_db.process_block(height as u32, &block, &previous_utxos);
            previous_utxos_vec.push(previous_utxos);
        }
        // txid = fe6c48bbfdc025670f4db0340650ba5a50f9307b091d9aaa19aa44291961c69f.
        assert_eq!(
            tx_db.put_tx(&consensus_decode(&hex::decode("01000000000101d553fbabaf1b26977b6e5d403af9f4b567b3e28484321a6fb02e2824984e3e5000000000171600142b2296c588ec413cebd19c3cbc04ea830ead6e78ffffffff01be1611020000000017a91487e4e5a7ff7bf78b8a8972a49381c8a673917f3e870247304402205f39ccbab38b644acea0776d18cb63ce3e37428cbac06dc23b59c61607aef69102206b8610827e9cb853ea0ba38983662034bd3575cc1ab118fb66d6a98066fa0bed01210304c01563d46e38264283b99bb352b46e69bf132431f102d4bd9a9d8dab075e7f00000000").unwrap()), Some(500_000)),
            Result::<TxDBValue, Txid>::Err(Txid::from_str(TXID).unwrap()),
        );
        for (height, block) in blocks.iter().enumerate() {
            let mut previous_utxo_index = 0;
            for tx in block.txdata.iter() {
                let mut previous_txout_index = 0;
                let value = tx_db.get(&tx.txid()).unwrap();
                assert_eq!(value.confirmed_height, Some(height as u32));
                assert_eq!(value.tx, *tx);
                for vin in tx.input.iter() {
                    if !vin.previous_output.is_null() {
                        let txout = TxOut {
                            value: previous_utxos_vec[height][previous_utxo_index].value,
                            script_pubkey: previous_utxos_vec[height][previous_utxo_index].script_pubkey.clone(),
                        };
                        assert_eq!(value.previous_txouts[previous_txout_index], txout);
                        previous_utxo_index += 1;
                        previous_txout_index += 1;
                    }
                }
            }
        }
    }
}
