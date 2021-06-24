use core::ops::Range;
use std::time::Instant;
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

impl From<&Utxo> for RichList {
    fn from(utxo: &Utxo) -> Self {
        let begin_acc = Instant::now();
        // Accumulate balances.
        let mut map: HashMap<Script, u64> = HashMap::new();
        let print_stat = |i: u32, force: bool| {
            if i % 10_000_000 == 0 || force {
                println!("RichList: processed {} entries...", i);
            }
        };
        let mut i = 0;
        for utxo in utxo.utxos.iter() {
            let value = map.get(&utxo.script_pubkey).unwrap_or(&0u64) + utxo.value;
            map.insert(utxo.script_pubkey.clone(), value);
            i += 1;
            print_stat(i, false);
        }
        print_stat(i, true);
        println!("RichList: processed in {}ms.", begin_acc.elapsed().as_millis());
        // Construct RichList instance.
        let begin_construct = Instant::now();
        let mut entries = map.par_iter().map(|(script_pubkey, value)| {
            RichListEntry {
                script_pubkey: (*script_pubkey).clone(),
                value: *value,
            }
        }).collect::<Vec<RichListEntry>>();
        println!("RichList: constructed in {}ms.", begin_construct.elapsed().as_millis());
        let begin_sort = Instant::now();
        entries.par_sort_unstable_by(|a, b| b.cmp(a));
        println!("RichList: sorted in {}ms.", begin_sort.elapsed().as_millis());
        let rich_list = RichList {
            entries,
        };
        rich_list
    }
}
