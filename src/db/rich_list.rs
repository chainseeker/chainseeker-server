use std::mem::size_of;
use core::ops::Range;
use indexmap::IndexMap;
use bitcoin::{Block, Script};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichListEntry {
    pub script_pubkey: Script,
    pub value: u64,
}

#[derive(Debug, Clone)]
pub struct RichList {
    map: IndexMap<Script, u64>,
}

impl Default for RichList {
    fn default() -> Self {
        Self::new()
    }
}

impl RichList {
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
    pub fn size(&self) -> usize {
        self.map.iter().map(|(script, _val)| script.len() + size_of::<u64>()).sum()
    }
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }
    pub fn push(&mut self, utxo: &UtxoEntry) {
        let value = self.map.get(&utxo.script_pubkey).unwrap_or(&0u64) + utxo.value;
        self.map.insert(utxo.script_pubkey.clone(), value);
    }
    pub fn remove(&mut self, script_pubkey: &Script, value: u64) {
        let v = self.map.get_mut(script_pubkey).unwrap();
        *v -= value;
        // Remove the entry if the value is zero.
        if *v == 0 {
            self.map.remove(script_pubkey);
        }
    }
    pub fn get_index_of(&self, script_pubkey: &Script) -> Option<usize> {
        self.map.get_index_of(script_pubkey)
    }
    pub fn get_in_range_as_rest(&self, range: Range<usize>, config: &Config) -> Vec<Option<RestRichListEntry>> {
        range.map(|i| {
            self.map.get_index(i).map(|(script_pubkey, value)| {
                RestRichListEntry {
                    script_pub_key: RestScriptPubKey::new(script_pubkey, config),
                    value: *value,
                }
            })
        }).collect()
    }
    pub fn process_block(&mut self, block: &Block, previous_utxos: &[UtxoEntry]) {
        // Process vouts.
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for vout in 0..tx.output.len() {
                let output = &tx.output[vout];
                // Ignore zero values.
                if output.value > 0 {
                    let vout = vout as u32;
                    let utxo = UtxoEntry {
                        script_pubkey: output.script_pubkey.clone(),
                        txid,
                        vout,
                        value: output.value,
                    };
                    self.push(&utxo);
                }
            }
        }
        // Process vins.
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if !vin.previous_output.is_null() {
                    let previous_utxo = &previous_utxos[previous_utxo_index];
                    self.remove(&previous_utxo.script_pubkey, previous_utxo.value);
                    previous_utxo_index += 1;
                }
            }
        }
        self.finalize();
    }
    pub fn finalize(&mut self) {
        self.map.par_sort_by(|_k1, v1, _k2, v2| v2.cmp(v1));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const JSON: &str = r#"[{"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 683ca4604908ebda57dfd97ff94eb5553b4e5aee","hex":"0014683ca4604908ebda57dfd97ff94eb5553b4e5aee","type":"witnesspubkeyhash","address":"bc1qdq72gczfpr4a547lm9lljn4425a5ukhwtx34za"},"value":505000000141},{"scriptPubKey":{"asm":"OP_PUSHBYTES_65 04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f OP_CHECKSIG","hex":"4104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac","type":"unknown","address":null},"value":5000000000},{"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 3fb63f3b6d30f31ada79d7c14a995e52c4a6fdb3","hex":"00143fb63f3b6d30f31ada79d7c14a995e52c4a6fdb3","type":"witnesspubkeyhash","address":"bc1q87mr7wmdxre34kne6lq54x272tz2dldnwu5m49"},"value":3999999859},{"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 ecac3ece9070b2b28ecfbc487b5ca575b1edb47a","hex":"0014ecac3ece9070b2b28ecfbc487b5ca575b1edb47a","type":"witnesspubkeyhash","address":"bc1qajkran5swzet9rk0h3y8kh99wkc7mdr645xdau"},"value":1000000000}]"#;
    #[allow(dead_code)]
    fn print_rich_list(rich_list: &RichList) {
        let entries = rich_list.get_in_range_as_rest(0..rich_list.len(), &config_example("btc"));
        println!("{}", serde_json::to_string(&entries).unwrap());
    }
    #[test]
    fn rich_list() {
        let mut rich_list = RichList::new();
        let mut utxo_db = UtxoDB::new("test/rich_list", true);
        for block in fixtures::regtest_blocks().iter() {
            let prev_utxos = utxo_db.process_block(&block, false);
            rich_list.process_block(&block, &prev_utxos);
        }
        rich_list.finalize();
        rich_list.shrink_to_fit();
        print_rich_list(&rich_list);
        let entries: Vec<Option<RestRichListEntry>> = serde_json::from_str(JSON).unwrap();
        assert_eq!(rich_list.len(), entries.len());
        assert_eq!(rich_list.capacity(), entries.len());
        assert_eq!(rich_list.size(), 165);
        assert_eq!(rich_list.get_in_range_as_rest(0..entries.len(), &config_example("btc")), entries);
        for (i, entry) in entries.iter().enumerate() {
            let script_pubkey = Script::from_hex(&entry.as_ref().unwrap().script_pub_key.hex).unwrap();
            assert_eq!(rich_list.get_index_of(&script_pubkey), Some(i));
        }
    }
}
