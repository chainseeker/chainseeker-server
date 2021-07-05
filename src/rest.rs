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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn rest() {
        let tx_db = TxDB::new("test/rest", true);
        let regtest_blocks = fixtures::regtest_blocks();
        for (height, block) in regtest_blocks.iter().enumerate() {
            for tx in block.txdata.iter() {
                tx_db.put_tx(tx, Some(height as u32)).unwrap();
            }
        }
        let config = config_example("rbtc");
        let block_rest = RestBlockWithTxs::from_block_content(&tx_db, &BlockContentDBValue::new(102, &regtest_blocks[102]), &config);
        let block_rest_json = serde_json::to_string(&block_rest).unwrap();
        //println!("{}", block_rest_json);
        let block_json = r#"{"height":102,"header":"00000020df0fb031f6a92e8d1984f5371a6b6ff3b75258b88b8236c267eb5bd9eb1ac504fa67c6f3a210ff753118bc65a1c06ec9fde641d00fcee6cc441c2915694c81f2b226d960ffff7f2001000000","hash":"0df678a7cc20dc21d0859930daece5f2837798342d24c8f3dfdbaffe47012c31","version":536870912,"previousblockhash":"04c51aebd95beb67c236828bb85852b7f36f6b1a37f584198d2ea9f631b00fdf","merkleroot":"f2814c6915291c44cce6ce0fd041e6fdc96ec0a165bc183175ff10a2f3c667fa","time":1624843954,"bits":"207fffff","difficulty":4.6565423739069247e-10,"nonce":1,"size":472,"strippedsize":327,"weight":1453,"txs":[{"confirmedHeight":102,"hex":"020000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff0401660101ffffffff028df2052a01000000160014683ca4604908ebda57dfd97ff94eb5553b4e5aee0000000000000000266a24aa21a9ed623e8562941c757c06c39e2b1c11bc9e92aa9e6254dbf8d968d43acaab0408d60120000000000000000000000000000000000000000000000000000000000000000000000000","txid":"518bcac0eff203a1cbdcd28a330760c6906eaa3f08885d5b396fdd606e9e6ea4","hash":"8f0303643150caf821e66da0c63865134b83abab8d6bfe12b1f6b0a1656a02f8","size":169,"vsize":142,"weight":568,"version":2,"locktime":0,"vin":[{"txid":"0000000000000000000000000000000000000000000000000000000000000000","vout":4294967295,"scriptSig":{"asm":"OP_PUSHBYTES_1 66 OP_PUSHBYTES_1 01","hex":"01660101"},"txinwitness":["200000000000000000000000000000000000000000000000000000000000000000"],"sequence":4294967295,"value":0,"address":null}],"vout":[{"value":5000000141,"n":0,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 683ca4604908ebda57dfd97ff94eb5553b4e5aee","hex":"0014683ca4604908ebda57dfd97ff94eb5553b4e5aee","type":"witnesspubkeyhash","address":"bcrt1qdq72gczfpr4a547lm9lljn4425a5ukhwrfntw8"}},{"value":0,"n":1,"scriptPubKey":{"asm":"OP_RETURN OP_PUSHBYTES_36 aa21a9ed623e8562941c757c06c39e2b1c11bc9e92aa9e6254dbf8d968d43acaab0408d6","hex":"6a24aa21a9ed623e8562941c757c06c39e2b1c11bc9e92aa9e6254dbf8d968d43acaab0408d6","type":"unknown","address":null}}],"fee":-5000000141},{"confirmedHeight":102,"hex":"02000000000101f0081f8204df7696cc30cda9a8644fc6f6f30bb76a44f65691edcf821b7d100d0000000000fdffffff0273276bee000000001600143fb63f3b6d30f31ada79d7c14a995e52c4a6fdb300ca9a3b00000000160014ecac3ece9070b2b28ecfbc487b5ca575b1edb47a0247304402203a6c4b22bf0970bc4976f7dc51c20d55efe567c0b65d44011ca43e19dbbe6b25022001cf5af20bd916271120c957c8699c54f60711d44fcda81911718656318e47c7012102ae26b761393f8cebd89fbeeb783f190fc7b293a9c68060e10c076bc8df794d7f65000000","txid":"e5bdd0adc488e35e5441e7391d26808b40149031abe6d4014504b5f34a3e14e5","hash":"ccd68ec2c6b7f02fd29b5f88501761d11c2840b5accc07f269b867fdadb9f41e","size":222,"vsize":141,"weight":561,"version":2,"locktime":101,"vin":[{"txid":"0d107d1b82cfed9156f6446ab70bf3f6c64f64a8a9cd30cc9676df04821f08f0","vout":0,"scriptSig":{"asm":"","hex":""},"txinwitness":["47304402203a6c4b22bf0970bc4976f7dc51c20d55efe567c0b65d44011ca43e19dbbe6b25022001cf5af20bd916271120c957c8699c54f60711d44fcda81911718656318e47c701","2102ae26b761393f8cebd89fbeeb783f190fc7b293a9c68060e10c076bc8df794d7f"],"sequence":4294967293,"value":5000000000,"address":"bcrt1qdq72gczfpr4a547lm9lljn4425a5ukhwrfntw8"}],"vout":[{"value":3999999859,"n":0,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 3fb63f3b6d30f31ada79d7c14a995e52c4a6fdb3","hex":"00143fb63f3b6d30f31ada79d7c14a995e52c4a6fdb3","type":"witnesspubkeyhash","address":"bcrt1q87mr7wmdxre34kne6lq54x272tz2dldnxnk9el"}},{"value":1000000000,"n":1,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 ecac3ece9070b2b28ecfbc487b5ca575b1edb47a","hex":"0014ecac3ece9070b2b28ecfbc487b5ca575b1edb47a","type":"witnesspubkeyhash","address":"bcrt1qajkran5swzet9rk0h3y8kh99wkc7mdr6amyn3x"}}],"fee":141}]}"#;
        assert_eq!(block_rest_json, block_json);
    }
}
