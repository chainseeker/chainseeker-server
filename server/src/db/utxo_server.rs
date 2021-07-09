use crate::*;
use indexmap::IndexMap;
use bitcoin::{Txid, Script, Block, WScriptHash};
use crate::db::utxo::UtxoEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct UtxoServerValue {
    pub txid: Txid,  // +32 = 32
    pub vout: u32,   // + 4 = 36
}

#[derive(Debug, Clone)]
pub struct UtxoServer {
    db: IndexMap<WScriptHash, Vec<UtxoServerValue>>,
}

impl Default for UtxoServer {
    fn default() -> Self {
        Self::new()
    }
}

impl UtxoServer {
    pub fn new() -> Self {
        Self {
            db: IndexMap::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.db.is_empty()
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
    pub fn get(&self, script_pubkey: &Script) -> Vec<UtxoServerValue> {
        self.db.get(&script_pubkey.wscript_hash()).map_or_else(Vec::new, |values| (*values).clone())
    }
    pub fn push(&mut self, utxo: &UtxoEntry) {
        let v = UtxoServerValue {
            txid: utxo.txid,
            vout: utxo.vout,
        };
        let wscript_hash = utxo.script_pubkey.wscript_hash();
        match self.db.get_mut(&wscript_hash) {
            Some(values) => values.push(v),
            None => assert!(self.db.insert(wscript_hash, vec![v]).is_none()),
        }
    }
    fn remove(&mut self, script_pubkey: &Script, txid: &Txid, vout: u32) {
        let values = self.db.get_mut(&script_pubkey.wscript_hash()).unwrap();
        *values = values.iter().filter(|&utxo_value| {
            !(utxo_value.txid == *txid && utxo_value.vout == vout)
        }).cloned().collect();
    }
    pub fn process_block(&mut self, block: &Block, previous_utxos: &[UtxoEntry]) {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for (vout, output) in tx.output.iter().enumerate() {
                self.push(&UtxoEntry {
                    script_pubkey: output.script_pubkey.clone(),
                    txid,
                    vout: vout as u32,
                    value: output.value,
                });
            }
        }
        // Process vins.
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if !vin.previous_output.is_null() {
                    let utxo = &previous_utxos[previous_utxo_index];
                    self.remove(&utxo.script_pubkey, &utxo.txid, utxo.vout);
                    previous_utxo_index += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::utxo::UtxoDB;
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
        let mut utxo_server: UtxoServer = Default::default();
        assert!(utxo_server.is_empty());
        let mut utxo_db = UtxoDB::new("test/utxo_server", true);
        let blocks = fixtures::regtest_blocks();
        for block in blocks.iter() {
            let prev_utxos = utxo_db.process_block(&block, false);
            utxo_server.process_block(&block, &prev_utxos);
        }
        utxo_server.shrink_to_fit();
        print_utxo_server(&utxo_server);
        assert!(!utxo_server.is_empty());
        let entries = fixtures::utxo_server_entries();
        assert_eq!(utxo_server.len(), entries.len());
        assert_eq!(utxo_server.capacity(), entries.len());
        assert_eq!(utxo_server.size(), 7672);
        for (i, (wscript_hash, value)) in utxo_server.iter().enumerate() {
            assert_eq!(*wscript_hash, entries[i].0);
            assert_eq!(*value, entries[i].1);
        }
        assert_eq!(utxo_server.get(&blocks[0].txdata[0].output[0].script_pubkey).len(), 1);
    }
}
