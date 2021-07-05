use std::mem::size_of;
use std::cmp::min;
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

impl RichList {
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
        }
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
    pub fn iter(&self) -> indexmap::map::Iter<'_, Script, u64> {
        self.map.iter()
    }
    pub fn push(&mut self, utxo: &UtxoEntry) {
        let value = self.map.get(&utxo.script_pubkey).unwrap_or(&0u64) + utxo.value;
        self.map.insert(utxo.script_pubkey.clone(), value);
    }
    pub fn remove(&mut self, script_pubkey: &Script, value: u64) {
        let v = self.map.get_mut(script_pubkey).unwrap();
        *v -= value;
    }
    pub fn get_index_of(&self, script_pubkey: &Script) -> Option<usize> {
        self.map.get_index_of(script_pubkey)
    }
    pub fn get_in_range(&self, range: Range<usize>) -> Vec<RichListEntry> {
        if self.map.len() < 1 {
            return Vec::new();
        }
        if range.start >= self.map.len() {
            return Vec::new();
        }
        let start = range.start;
        let end = min(range.end, self.map.len());
        let mut ret = Vec::with_capacity(end - start);
        for i in start..end {
            let data = self.map.get_index(i);
            if data.is_none() {
                break;
            }
            let (script_pubkey, value) = data.unwrap();
            ret.push(RichListEntry {
                script_pubkey: (*script_pubkey).clone(),
                value: *value,
            });
        }
        ret
    }
    pub fn process_block(&mut self, block: &Block, previous_utxos: &Vec<UtxoEntry>) {
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
                self.push(&utxo);
            }
        }
        // Process vins.
        let mut previous_utxo_index = 0;
        for tx in block.txdata.iter() {
            for vin in tx.input.iter() {
                if vin.previous_output.is_null() {
                    continue;
                }
                let previous_utxo = &previous_utxos[previous_utxo_index];
                self.remove(&previous_utxo.script_pubkey, previous_utxo.value);
                previous_utxo_index += 1;
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
    fn entries() -> [RichListEntry; 6] {
        [
            RichListEntry { script_pubkey: consensus_decode(&hex::decode("160014683ca4604908ebda57dfd97ff94eb5553b4e5aee").unwrap()), value: 505000000141, },
            RichListEntry { script_pubkey: consensus_decode(&hex::decode("434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac").unwrap()), value: 5000000000, },
            RichListEntry { script_pubkey: consensus_decode(&hex::decode("1600143fb63f3b6d30f31ada79d7c14a995e52c4a6fdb3").unwrap()), value: 3999999859, },
            RichListEntry { script_pubkey: consensus_decode(&hex::decode("160014ecac3ece9070b2b28ecfbc487b5ca575b1edb47a").unwrap()), value: 1000000000, },
            RichListEntry { script_pubkey: consensus_decode(&hex::decode("266a24aa21a9ede2f61c3f71d1defd3fa999dfa36953755c690689799962b48bebd836974e8cf9").unwrap()), value: 0, },
            RichListEntry { script_pubkey: consensus_decode(&hex::decode("266a24aa21a9ed623e8562941c757c06c39e2b1c11bc9e92aa9e6254dbf8d968d43acaab0408d6").unwrap()), value: 0, },
        ]
    }
    #[allow(dead_code)]
    fn print_rich_list(rich_list: &RichList) {
        for (script_pubkey, value) in rich_list.iter() {
            println!("            RichListEntry {{ script_pubkey: consensus_decode(&hex::decode(\"{}\").unwrap()), value: {}, }},",
                hex::encode(consensus_encode(&script_pubkey)),
                value);
        }
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
        //print_rich_list(&rich_list);
        let entries = entries();
        for (i, (script_pubkey, value)) in rich_list.iter().enumerate() {
            assert_eq!(*script_pubkey, entries[i].script_pubkey);
            assert_eq!(*value, entries[i].value);
        }
    }
}
