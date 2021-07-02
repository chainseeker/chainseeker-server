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
    pub fn push(&mut self, utxo: &UtxoEntry) {
        let value = self.map.get(&utxo.script_pubkey).unwrap_or(&0u64) + utxo.value;
        self.map.insert(utxo.script_pubkey.clone(), value);
    }
    pub fn remove(&mut self, script_pubkey: &Script, value: u64) {
        let v = self.map.get_mut(script_pubkey).unwrap();
        *v -= value;
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
