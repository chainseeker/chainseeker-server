use serde::Serialize;
use bitcoin_hashes::hex::ToHex;
use bitcoin::{Script, TxIn, TxOut, Address, Network, AddressType, Transaction};
use bitcoin::blockdata::constants::WITNESS_SCALE_FACTOR;

use super::*;

#[derive(Debug, Clone, Serialize)]
pub struct RestScriptSig {
    asm: String,
    hex: String,
}

impl RestScriptSig {
    pub fn new(script: &Script) -> Self {
        Self {
            asm: script.asm(),
            hex: hex::encode(script.as_bytes()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestVin {
    txid: String,
    vout: u32,
    script_sig: RestScriptSig,
    txinwitness: Vec<String>,
    sequence: u32,
    value: u64,
    address: Option<String>,
}

impl RestVin {
    pub fn new(txin: &TxIn, previous_txout: &Option<TxOut>, config: &Config) -> Self {
        Self {
            txid: txin.previous_output.txid.to_string(),
            vout: txin.previous_output.vout,
            script_sig: RestScriptSig::new(&txin.script_sig),
            txinwitness: txin.witness.iter().map(|witness| hex::encode(consensus_encode(witness))).collect(),
            sequence: txin.sequence,
            value: match previous_txout {
                Some(pt) => pt.value,
                None => 0,
            },
            address: match previous_txout {
                Some(previous_txout) => script_to_address_string(&previous_txout.script_pubkey, config),
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestScriptPubKey {
    asm: String,
    hex: String,
    r#type: String,
    address: Option<String>,
}

impl RestScriptPubKey {
    pub fn new(script_pubkey: &Script, config: &Config) -> Self {
        let address = Address::from_script(&script_pubkey, Network::Bitcoin /* any */);
        let address_str = script_to_address_string(&script_pubkey, config);
        Self {
            asm: script_pubkey.asm(),
            hex: hex::encode(script_pubkey.as_bytes()),
            r#type: match address.clone() {
                Some(address) => match address.address_type() {
                    Some(address_type) => match address_type {
                        AddressType::P2pkh  => "pubkeyhash",
                        AddressType::P2sh   => "scripthash",
                        AddressType::P2wpkh => "witnesspubkeyhash",
                        AddressType::P2wsh  => "witnessscripthash",
                    },
                    None => "unknown",
                }
                None => "unknown",
            }.to_string(),
            address: address_str,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestUtxo {
    txid: String,
    vout: u32,
    script_pub_key: RestScriptPubKey,
    value: u64,
}

impl RestUtxo {
    pub fn new(value: &UtxoServerValue, tx: &Transaction, config: &Config) -> Self {
        let mut txid = consensus_encode(&value.txid);
        txid.reverse();
        let previous_output = &tx.output[value.vout as usize];
        Self {
            txid: hex::encode(txid),
            vout: value.vout,
            script_pub_key: RestScriptPubKey::new(&previous_output.script_pubkey, config),
            value: previous_output.value,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestVout {
    value: u64,
    n: usize,
    script_pub_key: RestScriptPubKey,
}

impl RestVout {
    pub fn new(txout: &TxOut, n: usize, config: &Config) -> Self {
        Self {
            value: txout.value,
            n,
            script_pub_key: RestScriptPubKey::new(&txout.script_pubkey, config),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestTx {
    confirmed_height: Option<u32>,
    hex: String,
    txid: String,
    hash: String,
    size: usize,
    vsize: usize,
    weight: usize,
    version: i32,
    locktime: u32,
    vin: Vec<RestVin>,
    vout: Vec<RestVout>,
    fee: i64,
    //counterparty: ,
}

impl RestTx {
    pub fn from_tx_db_value(value: &TxDBValue, config: &Config) -> Self {
        let tx = &value.tx;
        let mut input_value = 0;
        let mut vin = Vec::new();
        let mut previous_txout_index = 0;
        for input in tx.input.iter() {
            if input.previous_output.is_null() {
                vin.push(RestVin::new(input, &None, config));
            } else {
                input_value += value.previous_txouts[previous_txout_index].value;
                vin.push(RestVin::new(input, &Some(value.previous_txouts[previous_txout_index].clone()), config));
                previous_txout_index += 1;
            }
        }
        let output_value: u64 = tx.output.iter().map(|output| output.value).sum();
        Self {
            confirmed_height: value.confirmed_height,
            hex: hex::encode(&consensus_encode(tx)),
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
            vout: tx.output.iter().enumerate().map(|(n, vout)| RestVout::new(vout, n, config)).collect(),
            // TODO: compute for coinbase transactions!
            fee: (input_value as i64) - (output_value as i64),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockHeader {
    pub height: u32,
    pub header: String,
    pub hash: String,
    pub version: i32,
    pub previousblockhash: String,
    pub merkleroot: String,
    pub time: u32,
    pub bits: String,
    pub difficulty: f64,
    pub nonce: u32,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub ntxs: usize,
}

impl RestBlockHeader {
    pub fn from_block_content(block_content: &BlockContentDBValue, config: &Config) -> Self {
        let block_header = &block_content.block_header;
        let mut hash = consensus_encode(&block_header.block_hash());
        hash.reverse();
        let mut prev_blockhash = consensus_encode(&block_header.prev_blockhash);
        prev_blockhash.reverse();
        let mut merkle_root = consensus_encode(&block_header.merkle_root);
        merkle_root.reverse();
        Self {
            height           : block_content.height,
            header           : hex::encode(consensus_encode(&block_header)),
            hash             : hex::encode(&hash),
            version          : block_header.version,
            previousblockhash: hex::encode(&prev_blockhash),
            merkleroot       : hex::encode(&merkle_root),
            time             : block_header.time,
            bits             : format!("{:x}", block_header.bits),
            difficulty       : get_difficulty(block_header, config),
            nonce            : block_header.nonce,
            size             : block_content.size,
            strippedsize     : block_content.strippedsize,
            weight           : block_content.weight,
            ntxs             : block_content.txids.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockWithTxids {
    pub height: u32,
    pub header: String,
    pub hash: String,
    pub version: i32,
    pub previousblockhash: String,
    pub merkleroot: String,
    pub time: u32,
    pub bits: String,
    pub difficulty: f64,
    pub nonce: u32,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub txids: Vec<String>,
}

impl RestBlockWithTxids {
    pub fn from_block_content(block_content: &BlockContentDBValue, config: &Config) -> Self {
        let rest_block_header = RestBlockHeader::from_block_content(block_content, config);
        Self {
            height           : rest_block_header.height,
            header           : rest_block_header.header,
            hash             : rest_block_header.hash,
            version          : rest_block_header.version,
            previousblockhash: rest_block_header.previousblockhash,
            merkleroot       : rest_block_header.merkleroot,
            time             : rest_block_header.time,
            bits             : rest_block_header.bits,
            difficulty       : rest_block_header.difficulty,
            nonce            : rest_block_header.nonce,
            size             : rest_block_header.size,
            strippedsize     : rest_block_header.strippedsize,
            weight           : rest_block_header.weight,
            txids            : block_content.txids.iter().map(|txid| txid.to_hex()).collect::<Vec<String>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockWithTxs {
    pub height: u32,
    pub header: String,
    pub hash: String,
    pub version: i32,
    pub previousblockhash: String,
    pub merkleroot: String,
    pub time: u32,
    pub bits: String,
    pub difficulty: f64,
    pub nonce: u32,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub txs: Vec<RestTx>,
}

impl RestBlockWithTxs {
    pub fn from_block_content(tx_db: &TxDB, block_content: &BlockContentDBValue, config: &Config) -> Self {
        let rest_block_header = RestBlockHeader::from_block_content(block_content, config);
        let txs = block_content.txids.iter().map(|txid| {
            let tx = tx_db.get(txid).unwrap();
            RestTx::from_tx_db_value(&tx, config)
        }).collect::<Vec<RestTx>>();
        Self {
            height           : rest_block_header.height,
            header           : rest_block_header.header,
            hash             : rest_block_header.hash,
            version          : rest_block_header.version,
            previousblockhash: rest_block_header.previousblockhash,
            merkleroot       : rest_block_header.merkleroot,
            time             : rest_block_header.time,
            bits             : rest_block_header.bits,
            difficulty       : rest_block_header.difficulty,
            nonce            : rest_block_header.nonce,
            size             : rest_block_header.size,
            strippedsize     : rest_block_header.strippedsize,
            weight           : rest_block_header.weight,
            txs,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestRichListEntry {
    pub script_pub_key: RestScriptPubKey,
    pub value: u64,
}

impl RestRichListEntry {
    pub fn from_rich_list_entry(entry: &RichListEntry, config: &Config) -> Self {
        let script_pub_key = RestScriptPubKey::new(&entry.script_pubkey, config);
        Self {
            script_pub_key,
            value: entry.value,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestBlockSummary {
    hash        : String,
    time        : u32,
    nonce       : u32,
    size        : u32,
    strippedsize: u32,
    weight      : u32,
    txcount     : usize,
}

impl RestBlockSummary {
    pub fn new(block: &BlockContentDBValue) -> Self {
        Self {
            hash        : block.block_header.block_hash().to_string(),
            time        : block.block_header.time,
            nonce       : block.block_header.nonce,
            size        : block.size,
            strippedsize: block.strippedsize,
            weight      : block.weight,
            txcount     : block.txids.len(),
        }
    }
}
