use core::ops::Range;
use std::collections::HashMap;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use rayon::prelude::*;
use bitcoin::Script;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichListEntry {
    script_pubkey: Script,
    value: u64,
}

impl Serialize for RichListEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state = serializer.serialize_struct("RichListEntry", 2)?;
        state.serialize_field("script_pubkey", &hex::encode(serialize_script(&self.script_pubkey)))?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
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
    pub fn get_in_range(&self, range: Range<usize>) -> Vec<RichListEntry> {
        self.entries[range].to_vec()
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
    pub fn push(&mut self, utxo: &UtxoEntry) {
        let value = self.map.get(&utxo.script_pubkey).unwrap_or(&0u64) + utxo.value;
        self.map.insert(utxo.script_pubkey.clone(), value);
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
