use std::mem::size_of;
use std::cmp::min;
use core::ops::Range;
use std::collections::HashMap;
use rayon::prelude::*;
use bitcoin::{Block, Script};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichListEntry {
    pub script_pubkey: Script,
    pub value: u64,
}

impl PartialOrd for RichListEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for RichListEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[derive(Debug, Clone)]
pub struct RichList {
    /// (script_pubkey, value)
    entries: Vec<RichListEntry>,
}

impl RichList {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }
    /// Estimate the allocated size of this struct.
    pub fn size(&self) -> usize {
        self.entries.iter().map(|entry| entry.script_pubkey.len() + size_of::<u64>()).sum()
    }
    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
    }
    pub fn get_in_range(&self, range: Range<usize>) -> Vec<RichListEntry> {
        if self.entries.len() < 1 {
            return Vec::new();
        }
        let start = min(range.start, self.entries.len() - 1);
        let end = min(range.end, self.entries.len());
        self.entries[start..end].to_vec()
    }
}

#[derive(Debug, Clone)]
pub struct RichListBuilder {
    map: HashMap<Script, u64>,
}

impl RichListBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
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
    pub fn push(&mut self, utxo: &UtxoEntry) {
        let value = self.map.get(&utxo.script_pubkey).unwrap_or(&0u64) + utxo.value;
        self.map.insert(utxo.script_pubkey.clone(), value);
    }
    pub fn remove(&mut self, script_pubkey: &Script, value: u64) {
        let v = self.map.get_mut(script_pubkey).unwrap();
        *v -= value;
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
    }
    pub fn finalize(&self) -> RichList {
        // Construct RichList instance.
        let mut entries = self.map.par_iter().map(|(script_pubkey, value)| {
            RichListEntry {
                script_pubkey: (*script_pubkey).clone(),
                value: *value,
            }
        }).collect::<Vec<RichListEntry>>();
        entries.par_sort_unstable_by(|a, b| b.cmp(a));
        RichList {
            entries,
        }
    }
}
