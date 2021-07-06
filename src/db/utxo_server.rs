use indexmap::IndexMap;
use bitcoin::{Txid, Script, Block, WScriptHash};

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UtxoServerValue {
    pub txid: Txid,  // +32 = 32
    pub vout: u32,   // + 4 = 36
}

impl ConstantSize for UtxoServerValue {
    const LEN: usize = 36;
}

impl From<&UtxoServerValue> for Vec<u8> {
    fn from(value: &UtxoServerValue) -> Self {
        let mut buf: [u8; 36] = [0; 36];
        value.txid.consensus_encode(&mut buf[0..32]).expect("Failed to encode txid.");
        buf[32..36].copy_from_slice(&value.vout.to_le_bytes());
        buf.to_vec()
    }
}

impl Serialize for UtxoServerValue {
    fn serialize(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&[u8]> for UtxoServerValue {
    fn from(buf: &[u8]) -> UtxoServerValue {
        assert_eq!(buf.len(), 36);
        let txid = consensus_decode(&buf[0..32]);
        let vout = bytes_to_u32(&buf[32..36]);
        UtxoServerValue {
            txid,
            vout,
        }
    }
}

impl Deserialize for UtxoServerValue {
    fn deserialize(buf: &[u8]) -> Self {
        buf.into()
    }
}

#[derive(Debug, Clone)]
pub struct UtxoServer {
    db: IndexMap<WScriptHash, Vec<UtxoServerValue>>,
}

impl UtxoServer {
    pub fn new() -> Self {
        Self {
            db: IndexMap::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.db.len()
    }
    pub fn capacity(&self) -> usize {
        self.db.capacity()
    }
    pub fn size(&self) -> usize {
        self.db.iter().map(|(script, value)| script.len() + 36 * value.len()).sum()
    }
    pub fn shrink_to_fit(&mut self) {
        self.db.shrink_to_fit();
    }
    pub fn iter(&self) -> indexmap::map::Iter<WScriptHash, Vec<UtxoServerValue>> {
        self.db.iter()
    }
    pub async fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        match self.db.get(&script_pubkey.wscript_hash()) {
            Some(values) => (*values).clone(),
            None => Vec::new(),
        }
    }
    pub async fn push(&mut self, utxo: &UtxoEntry) {
        let wscript_hash = utxo.script_pubkey.wscript_hash();
        let values = match self.db.get_mut(&wscript_hash) {
            Some(values) => values,
            None => {
                self.db.insert(wscript_hash.clone(), Vec::with_capacity(1));
                self.db.get_mut(&wscript_hash).unwrap()
            },
        };
        let v = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
        };
        values.push(v);
    }
    fn remove(&mut self, script_pubkey: &Script, txid: &Txid, vout: u32) {
        let values = self.db.get_mut(&script_pubkey.wscript_hash()).unwrap();
        *values = values.iter().filter(|&utxo_value| {
            !(utxo_value.txid == *txid && utxo_value.vout == vout)
        }).cloned().collect();
    }
    pub async fn process_block(&mut self, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let output = &tx.output[vout];
                let utxo = UtxoEntry {
                    script_pubkey: output.script_pubkey.clone(),
                    txid,
                    vout: vout as u32,
                    value: output.value,
                };
                self.push(&utxo).await;
            }
        }
        // Process vins.
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let utxo = &previous_utxos[previous_utxo_index];
                self.remove(&utxo.script_pubkey, &utxo.txid, utxo.vout);
                previous_utxo_index += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(dead_code)]
    fn print_utxo_server(utxo_server: &UtxoServer) {
        for (wscript_hash, values) in utxo_server.iter() {
            println!("        (WScriptHash::from_hex(\"{}\").unwrap(), vec![", hex::encode(wscript_hash));
            for value in values.iter() {
                println!("            UtxoServerValue {{ txid: consensus_decode(&hex::decode(\"{}\").unwrap()), vout: {}, }},",
                    hex::encode(consensus_encode(&value.txid)),
                    value.vout);
            }
            println!("        ]),");
        }
    }
    #[tokio::test]
    async fn utxo_server() {
        let mut utxo_server = UtxoServer::new();
        let mut utxo_db = UtxoDB::new("test/utxo_server", true);
        for block in fixtures::regtest_blocks().iter() {
            let prev_utxos = utxo_db.process_block(&block, false);
            utxo_server.process_block(&block, &prev_utxos).await;
        }
        utxo_server.shrink_to_fit();
        //print_utxo_server(&utxo_server);
        let entries = fixtures::utxo_server_entries();
        assert_eq!(utxo_server.len(), entries.len());
        assert_eq!(utxo_server.capacity(), entries.len());
        assert_eq!(utxo_server.size(), 7608);
        for (i, (wscript_hash, value)) in utxo_server.iter().enumerate() {
            assert_eq!(*wscript_hash, entries[i].0);
            assert_eq!(*value, entries[i].1);
        }
    }
}
