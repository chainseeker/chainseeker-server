use serde::{Serialize, Deserialize};
use bitcoin::hashes::hex::ToHex;
use bitcoin::{Script, TxIn, TxOut, Address, Network, AddressType, Transaction};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestStatus {
    pub blocks: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestScriptSig {
    pub asm: String,
    pub hex: String,
}

impl RestScriptSig {
    pub fn new(script: &Script) -> Self {
        Self {
            asm: script.asm(),
            hex: hex::encode(script.as_bytes()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestVin {
    pub txid: String,
    pub vout: u32,
    pub script_sig: RestScriptSig,
    pub txinwitness: Vec<String>,
    pub sequence: u32,
    pub value: u64,
    pub address: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestScriptPubKey {
    pub asm: String,
    pub hex: String,
    pub r#type: String,
    pub address: Option<String>,
}

impl RestScriptPubKey {
    pub fn new(script_pubkey: &Script, config: &Config) -> Self {
        let address = Address::from_script(&script_pubkey, Network::Bitcoin /* any */);
        let address_str = script_to_address_string(&script_pubkey, config);
        Self {
            asm: script_pubkey.asm(),
            hex: hex::encode(script_pubkey.as_bytes()),
            r#type: address.map_or("unknown", |address| address.address_type().map_or("unknown", |address_type| {
                match address_type {
                    AddressType::P2pkh  => "pubkeyhash",
                    AddressType::P2sh   => "scripthash",
                    AddressType::P2wpkh => "witnesspubkeyhash",
                    AddressType::P2wsh  => "witnessscripthash",
                }
            })).to_string(),
            address: address_str,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestUtxo {
    pub txid: String,
    pub vout: u32,
    pub script_pub_key: RestScriptPubKey,
    pub value: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestVout {
    pub value: u64,
    pub n: usize,
    pub script_pub_key: RestScriptPubKey,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestTx {
    pub confirmed_height: Option<u32>,
    pub hex: String,
    pub txid: String,
    pub hash: String,
    pub size: usize,
    pub vsize: usize,
    pub weight: usize,
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<RestVin>,
    pub vout: Vec<RestVout>,
    pub fee: i64,
    //counterparty: ,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        // TODO: waiting upstream fix: https://github.com/rust-rocksdb/rust-rocksdb/issues/536
        //let txs = tx_db.multi_get_as_rest(block_content.txids.clone());
        let txs = block_content.txids.iter().map(|txid| {
            tx_db.get_as_rest(txid, config).unwrap()
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestRichListEntry {
    pub script_pub_key: RestScriptPubKey,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestBlockSummary {
    pub hash        : String,
    pub time        : u32,
    pub nonce       : u32,
    pub size        : u32,
    pub strippedsize: u32,
    pub weight      : u32,
    pub txcount     : usize,
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
mod tests {
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
        println!("{}", block_rest_json);
        let block_json = r#"{"height":102,"header":"00000020f4a34bc39e46acbf6ad1cd786d978718b4ef94002eb080e86f383b77798b8b1e197bc47e4d72c8cc02c78a6c89a45db03f9906c1a60448bbeecf7b84566fc66eca41e560ffff7f2001000000","hash":"25263a195c89fae46d08558b1b501617aac630a6854000e19eb7ada6c39e6e0b","version":536870912,"previousblockhash":"1e8b8b79773b386fe880b02e0094efb41887976d78cdd16abfac469ec34ba3f4","merkleroot":"6ec66f56847bcfeebb4804a6c106993fb05da4896c8ac702ccc8724d7ec47b19","time":1625637322,"bits":"207fffff","difficulty":4.6565423739069247e-10,"nonce":1,"size":842,"strippedsize":481,"weight":2285,"txs":[{"confirmedHeight":102,"hex":"020000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff0401660101ffffffff02547a062a0100000016001497033ca70d45fe6d49310859e132a9df98f976250000000000000000266a24aa21a9edaaed4e9155661fb1a2bfc87c1458ab0eebc25bf8f0b8126ccc9214c407ebd34f0120000000000000000000000000000000000000000000000000000000000000000000000000","txid":"b903046c3fb719a2a404e3b749609414b4f914b9cbf2eadf289c8ed2120aaebc","hash":"cb01519e3eee67593f7f0eb955a82a3aa6fc1c83364c50e7c6891b14320d88c7","size":169,"vsize":142,"weight":568,"version":2,"locktime":0,"vin":[{"txid":"0000000000000000000000000000000000000000000000000000000000000000","vout":4294967295,"scriptSig":{"asm":"OP_PUSHBYTES_1 66 OP_PUSHBYTES_1 01","hex":"01660101"},"txinwitness":["200000000000000000000000000000000000000000000000000000000000000000"],"sequence":4294967295,"value":0,"address":null}],"vout":[{"value":5000034900,"n":0,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 97033ca70d45fe6d49310859e132a9df98f97625","hex":"001497033ca70d45fe6d49310859e132a9df98f97625","type":"witnesspubkeyhash","address":"bcrt1qjupnefcdghlx6jf3ppv7zv4fm7v0ja39dzzwvd"}},{"value":0,"n":1,"scriptPubKey":{"asm":"OP_RETURN OP_PUSHBYTES_36 aa21a9edaaed4e9155661fb1a2bfc87c1458ab0eebc25bf8f0b8126ccc9214c407ebd34f","hex":"6a24aa21a9edaaed4e9155661fb1a2bfc87c1458ab0eebc25bf8f0b8126ccc9214c407ebd34f","type":"unknown","address":null}}],"fee":-5000034900},{"confirmedHeight":102,"hex":"02000000000101592f96fe043aaa22cdcb6f6e710946aa0af25dff4536759f6873965555a6660c0000000000fdffffff02ecd90f2401000000160014629ef06211f8e75e223b6338228a025bd15b0db900e1f50500000000160014f0e9ede24bceb0c16fdce952279d8094dc6b3f5802473044022057f0dde0e8d094034f47a136ae22a433e0216d5b1cfa3a03d8d259660148744702204377d2dbf3e442128c03097af4d4fa56b52b0cc7d9c24511b99d45a47e8df2d7012102b934d90a0aa5e0b73e04d00df3a8633ee16d24dad9ab21697f1dc5feb43fdbed65000000","txid":"e6a6a7db6faadbfc0815fd3a78c3b5d15cb733dde8aca0e911b6f565ea24ee73","hash":"4fdf235851b72bb7ddc8f6d86e43797bb42e3601ee1eb1e8d8a0e157b3c81af6","size":222,"vsize":141,"weight":561,"version":2,"locktime":101,"vin":[{"txid":"0c66a655559673689f753645ff5df20aaa4609716e6fcbcd22aa3a04fe962f59","vout":0,"scriptSig":{"asm":"","hex":""},"txinwitness":["473044022057f0dde0e8d094034f47a136ae22a433e0216d5b1cfa3a03d8d259660148744702204377d2dbf3e442128c03097af4d4fa56b52b0cc7d9c24511b99d45a47e8df2d701","2102b934d90a0aa5e0b73e04d00df3a8633ee16d24dad9ab21697f1dc5feb43fdbed"],"sequence":4294967293,"value":5000000000,"address":"bcrt1qjupnefcdghlx6jf3ppv7zv4fm7v0ja39dzzwvd"}],"vout":[{"value":4899985900,"n":0,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 629ef06211f8e75e223b6338228a025bd15b0db9","hex":"0014629ef06211f8e75e223b6338228a025bd15b0db9","type":"witnesspubkeyhash","address":"bcrt1qv200qcs3lrn4ug3mvvuz9zszt0g4krde3uqyzy"}},{"value":100000000,"n":1,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 f0e9ede24bceb0c16fdce952279d8094dc6b3f58","hex":"0014f0e9ede24bceb0c16fdce952279d8094dc6b3f58","type":"witnesspubkeyhash","address":"bcrt1q7r57mcjte6cvzm7ua9fz08vqjnwxk06c2v6jdv"}}],"fee":14100},{"confirmedHeight":102,"hex":"0200000000010273ee24ea65f5b611e9a0ace8dd33b75cd1b5c3783afd1508fcdbaa6fdba7a6e60000000000fdffffff73ee24ea65f5b611e9a0ace8dd33b75cd1b5c3783afd1508fcdbaa6fdba7a6e60100000000fdffffff02001110240100000016001413bb0bcb776f3e15fa8800877552044d4db27b96ac58f50500000000160014261b6555a3cb5d3593c0275ff46f31c36e42a4c70247304402207a89cf2b2d7620ace221894746c4a72f5fd5dd5bbe9e56018b182eaf046e9766022050a14136c20402b281c244a918856b9e866b5fc77182334dd6242993e2144b7d0121034968df50370db27d51b294cba769ec47e0accb3a582c312549303c555ca834d102473044022058a1f0d9b8bde83c28954a1cfac6a3f43f8f8b53529df47d0ef2f7d66f8f42bb022026993b9ba88bedff19ec3f70d32bab177d35e0570f8c86a9b2eac5b359189c1d012103d0d5b793a8a23ff2e92b6204c15f72fd51765ad6c01f6ebd9e712adee11bda4065000000","txid":"29704e049abbacf3222cd8af9654afe6c7c91d5d9f2faa5d2f0f3cd5de3a812d","hash":"fb3551865d89fae25b8bcc5a02da25ad91d6fe1c0d4fd391b4c7b1b69e9ece63","size":370,"vsize":208,"weight":832,"version":2,"locktime":101,"vin":[{"txid":"e6a6a7db6faadbfc0815fd3a78c3b5d15cb733dde8aca0e911b6f565ea24ee73","vout":0,"scriptSig":{"asm":"","hex":""},"txinwitness":["47304402207a89cf2b2d7620ace221894746c4a72f5fd5dd5bbe9e56018b182eaf046e9766022050a14136c20402b281c244a918856b9e866b5fc77182334dd6242993e2144b7d01","21034968df50370db27d51b294cba769ec47e0accb3a582c312549303c555ca834d1"],"sequence":4294967293,"value":4899985900,"address":"bcrt1qv200qcs3lrn4ug3mvvuz9zszt0g4krde3uqyzy"},{"txid":"e6a6a7db6faadbfc0815fd3a78c3b5d15cb733dde8aca0e911b6f565ea24ee73","vout":1,"scriptSig":{"asm":"","hex":""},"txinwitness":["473044022058a1f0d9b8bde83c28954a1cfac6a3f43f8f8b53529df47d0ef2f7d66f8f42bb022026993b9ba88bedff19ec3f70d32bab177d35e0570f8c86a9b2eac5b359189c1d01","2103d0d5b793a8a23ff2e92b6204c15f72fd51765ad6c01f6ebd9e712adee11bda40"],"sequence":4294967293,"value":100000000,"address":"bcrt1q7r57mcjte6cvzm7ua9fz08vqjnwxk06c2v6jdv"}],"vout":[{"value":4900000000,"n":0,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 13bb0bcb776f3e15fa8800877552044d4db27b96","hex":"001413bb0bcb776f3e15fa8800877552044d4db27b96","type":"witnesspubkeyhash","address":"bcrt1qzwashjmhdulpt75gqzrh25syf4xmy7uk6clm0p"}},{"value":99965100,"n":1,"scriptPubKey":{"asm":"OP_0 OP_PUSHBYTES_20 261b6555a3cb5d3593c0275ff46f31c36e42a4c7","hex":"0014261b6555a3cb5d3593c0275ff46f31c36e42a4c7","type":"witnesspubkeyhash","address":"bcrt1qycdk24dredwnty7qya0lgme3cdhy9fx83qc9wd"}}],"fee":20800}]}"#;
        assert_eq!(block_rest_json, block_json);
    }
}
