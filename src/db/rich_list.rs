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
        // Remove the entry if the value is zero.
        if *v <= 0 {
            self.map.remove(script_pubkey);
        }
    }
    pub fn get_index_of(&self, script_pubkey: &Script) -> Option<usize> {
        self.map.get_index_of(script_pubkey)
    }
    pub fn get_in_range(&self, range: Range<usize>) -> Vec<Option<RichListEntry>> {
        let mut ret = Vec::with_capacity(range.end - range.start);
        for i in range {
            ret.push(match self.map.get_index(i) {
                Some((script_pubkey, value)) => Some(RichListEntry {
                    script_pubkey: (*script_pubkey).clone(),
                    value: *value,
                }),
                None => None,
            });
        }
        ret
    }
    pub fn get_in_range_as_rest(&self, range: Range<usize>, config: &Config) -> Vec<Option<RestRichListEntry>> {
        let mut ret = Vec::with_capacity(range.end - range.start);
        for i in range {
            ret.push(match self.map.get_index(i) {
                Some((script_pubkey, value)) => Some(RestRichListEntry {
                    script_pub_key: RestScriptPubKey::new(script_pubkey, config),
                    value: *value,
                }),
                None => None,
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
                // Ignore zero values.
                if output.value <= 0 {
                    continue;
                }
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
    fn entries() -> [Option<RichListEntry>; 4] {
        [
            Some(RichListEntry { script_pubkey: consensus_decode(&hex::decode("160014683ca4604908ebda57dfd97ff94eb5553b4e5aee").unwrap()), value: 505000000141, }),
            Some(RichListEntry { script_pubkey: consensus_decode(&hex::decode("434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac").unwrap()), value: 5000000000, }),
            Some(RichListEntry { script_pubkey: consensus_decode(&hex::decode("1600143fb63f3b6d30f31ada79d7c14a995e52c4a6fdb3").unwrap()), value: 3999999859, }),
            Some(RichListEntry { script_pubkey: consensus_decode(&hex::decode("160014ecac3ece9070b2b28ecfbc487b5ca575b1edb47a").unwrap()), value: 1000000000, }),
        ]
    }
    #[allow(dead_code)]
    fn print_rich_list(rich_list: &RichList) {
        for (script_pubkey, value) in rich_list.iter() {
            println!("            Some(RichListEntry {{ script_pubkey: consensus_decode(&hex::decode(\"{}\").unwrap()), value: {}, }}),",
                hex::encode(consensus_encode(&script_pubkey)),
                value);
        }
    }
    #[test]
    fn rich_list() {
        let mut rich_list = RichList::new();
        assert_eq!(rich_list.get_in_range(0..1), vec![None]);
        let mut utxo_db = UtxoDB::new("test/rich_list", true);
        for block in fixtures::regtest_blocks().iter() {
            let prev_utxos = utxo_db.process_block(&block, false);
            rich_list.process_block(&block, &prev_utxos);
        }
        rich_list.finalize();
        rich_list.shrink_to_fit();
        print_rich_list(&rich_list);
        let entries = entries();
        assert_eq!(rich_list.len(), entries.len());
        assert_eq!(rich_list.capacity(), entries.len());
        assert_eq!(rich_list.size(), 165);
        assert_eq!(rich_list.get_in_range(0..entries.len()), entries);
        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(rich_list.get_index_of(&entry.as_ref().unwrap().script_pubkey), Some(i));
        }
        assert_eq!(rich_list.get_in_range(0..0), Vec::new());
        assert_eq!(rich_list.get_in_range(entries.len()..entries.len()+1), vec![None]);
    }
}
