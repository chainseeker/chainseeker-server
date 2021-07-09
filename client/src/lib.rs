//! chainseeker - Open-source cryptocurrency block explorer (client library).
//!

pub use serde;
pub use reqwest;
use serde::{Serialize, Deserialize};

pub const DEFAULT_ENDPOINT: &str = "https://btc-v3.chainseeker.info/api";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub blocks: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSig {
    pub asm: String,
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vin {
    pub txid: String,
    pub vout: u32,
    pub script_sig: ScriptSig,
    pub txinwitness: Vec<String>,
    pub sequence: u32,
    pub value: u64,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPubKey {
    pub asm: String,
    pub hex: String,
    pub r#type: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub script_pub_key: ScriptPubKey,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vout {
    pub value: u64,
    pub n: usize,
    pub script_pub_key: ScriptPubKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub confirmed_height: Option<u32>,
    pub hex: String,
    pub txid: String,
    pub hash: String,
    pub size: usize,
    pub vsize: usize,
    pub weight: usize,
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub fee: i64,
    //counterparty: ,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Txid {
    pub txid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockWithTxids {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockWithTxs {
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
    pub txs: Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichListCount {
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichListRank {
    pub rank: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichListEntry {
    pub script_pub_key: ScriptPubKey,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockSummary {
    pub hash        : String,
    pub time        : u32,
    pub nonce       : u32,
    pub size        : u32,
    pub strippedsize: u32,
    pub weight      : u32,
    pub txcount     : usize,
}

#[derive(Debug, Clone)]
pub struct Client {
    endpoint: String,
    reqwest_client: reqwest::Client,
}

impl Client {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            reqwest_client: reqwest::Client::new(),
        }
    }
    /// GET the REST endpoint and parse it as a JSON.
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, reqwest::Error> {
        let url = format!("{}/v1/{}", &self.endpoint, path);
        let result = self.reqwest_client.get(url)
            .send().await?
            .json::<T>().await?;
        Ok(result)
    }
    /// PUT the REST endpoint and parse it as a JSON.
    pub async fn put<T: for<'de> Deserialize<'de>>(&self, path: &str, s: String) -> Result<T, reqwest::Error> {
        let url = format!("{}/v1/{}", &self.endpoint, path);
        let result = self.reqwest_client.put(url)
            .body(s)
            .send().await?
            .json::<T>().await?;
        Ok(result)
    }
    pub async fn status(&self) -> Result<Status, reqwest::Error> {
        self.get("status").await
    }
    pub async fn tx(&self, txid: &str) -> Result<Transaction, reqwest::Error> {
        self.get(&["tx", txid].join("/")).await
    }
    pub async fn put_tx(&self, hex: String) -> Result<Txid, reqwest::Error> {
        self.put("tx/broadcast", hex).await
    }
    pub async fn block_summary(&self, offset: u32, limit: u32) -> Result<Vec<BlockSummary>, reqwest::Error> {
        self.get(&["block_summary", &offset.to_string(), &limit.to_string()].join("/")).await
    }
    pub async fn block_with_txids<T: ToString>(&self, hash_or_height: T) -> Result<BlockWithTxids, reqwest::Error> {
        self.get(&["block_with_txids", &hash_or_height.to_string()].join("/")).await
    }
    pub async fn block_with_txs<T: ToString>(&self, hash_or_height: T) -> Result<BlockWithTxs, reqwest::Error> {
        self.get(&["block_with_txs", &hash_or_height.to_string()].join("/")).await
    }
    pub async fn block_header<T: ToString>(&self, hash_or_height: T) -> Result<BlockHeader, reqwest::Error> {
        self.get(&["block", &hash_or_height.to_string()].join("/")).await
    }
    pub async fn txids(&self, script_or_address: &str) -> Result<Vec<String>, reqwest::Error> {
        self.get(&["txids", script_or_address].join("/")).await
    }
    pub async fn txs(&self, script_or_address: &str) -> Result<Vec<Transaction>, reqwest::Error> {
        self.get(&["txs", script_or_address].join("/")).await
    }
    pub async fn utxos(&self, script_or_address: &str) -> Result<Vec<Utxo>, reqwest::Error> {
        self.get(&["utxos", script_or_address].join("/")).await
    }
    pub async fn rich_list_count(&self) -> Result<RichListCount, reqwest::Error> {
        self.get("rich_list_count").await
    }
    pub async fn rich_list_addr_rank(&self, script_or_address: &str) -> Result<RichListRank, reqwest::Error> {
        self.get(&["rich_list_addr_rank", script_or_address].join("/")).await
    }
    pub async fn rich_list(&self, offset: u32, limit: u32) -> Result<Vec<Option<RichListEntry>>, reqwest::Error> {
        self.get(&["rich_list", &offset.to_string(), &limit.to_string()].join("/")).await
    }
}

/// Create a new `chainseeker` client.
///
/// The `endpoint` will be the string like "https://btc-v3.chainseeker.info/api"
/// (Note: this string is available via `bitcoin_rest::DEFAULT_ENDPOINT`).
pub fn new(endpoint: &str) -> Client {
    Client::new(endpoint)
}

#[cfg(test)]
mod tests {
    use bitcoin::*;
    use bitcoin::consensus::Encodable;
    use bitcoin::hashes::hex::FromHex;
    use super::*;
    #[tokio::test]
    async fn status() {
        let client = new(DEFAULT_ENDPOINT);
        assert!(client.status().await.unwrap().blocks > 0);
    }
    #[tokio::test]
    async fn tx() {
        const TXID: &str = "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098";
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.tx(TXID).await.unwrap().txid, TXID);
    }
    #[tokio::test]
    async fn put_tx() {
        let client = new("https://tbtc-v3.chainseeker.info/api");
        // Compute address.
        let secp256k1 = bitcoin::secp256k1::Secp256k1::new();
        let privkey = std::env::var("CS_PRIVKEY").unwrap();
        let privkey = PrivateKey::from_wif(&privkey).unwrap();
        let pubkey = privkey.public_key(&secp256k1);
        let address = Address::p2wpkh(&pubkey, Network::Testnet).unwrap();
        // Get UTXOs.
        let utxos = client.utxos(&address.to_string()).await.unwrap();
        println!("UTXOs: {:#?}", utxos);
        // Construct transaction.
        let mut value: u64 = 0;
        let input = utxos.iter().map(|utxo| {
            value += utxo.value;
            TxIn {
                previous_output: OutPoint {
                    txid: bitcoin::Txid::from_hex(&utxo.txid).unwrap(),
                    vout: utxo.vout,
                },
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: vec![],
            }
        }).collect();
        let output = TxOut {
            script_pubkey: Script::new_v0_wpkh(&pubkey.wpubkey_hash().unwrap()),
            value: value - 300 * (utxos.len() as u64),
        };
        let mut tx = bitcoin::Transaction {
            version: 2,
            lock_time: 0,
            input,
            output: vec![output],
        };
        // Sign transaction.
        let mut sig_hasher = bitcoin::util::bip143::SigHashCache::new(&mut tx);
        for (i, utxo) in utxos.iter().enumerate() {
            let script_code = Script::new_p2pkh(&pubkey.pubkey_hash());
            let sighash = sig_hasher.signature_hash(utxo.vout as usize, &script_code, utxo.value, SigHashType::All);
            let signature = secp256k1.sign(&bitcoin::secp256k1::Message::from_slice(&sighash).unwrap(), &privkey.key);
            let mut signature = signature.serialize_der().to_vec();
            // Push SIGHASH_ALL.
            signature.push(0x01);
            sig_hasher.access_witness(i).push(signature);
            sig_hasher.access_witness(i).push(pubkey.to_bytes());
        }
        println!("Txid: {}", tx.txid());
        let mut tx_raw = Vec::new();
        tx.consensus_encode(&mut tx_raw).unwrap();
        let tx_hex = hex::encode(tx_raw);
        println!("Raw tx: {}", tx_hex);
        assert_eq!(client.put_tx(tx_hex).await.unwrap().txid, tx.txid().to_string());
    }
    #[tokio::test]
    async fn block_summary() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.block_summary(0, 10).await.unwrap().len(), 10);
    }
    const BLOCK_HASH: &str = "00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048";
    const BLOCK_HEIGHT: u32 = 1;
    const TXID: &str = "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098";
    #[tokio::test]
    async fn block_with_txids_from_hash() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.block_with_txids(BLOCK_HASH).await.unwrap().txids, [TXID]);
    }
    #[tokio::test]
    async fn block_with_txids_from_height() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.block_with_txids(BLOCK_HEIGHT).await.unwrap().txids, [TXID]);
    }
    #[tokio::test]
    async fn block_with_txs_from_hash() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.block_with_txs(BLOCK_HASH).await.unwrap().txs[0].txid, TXID);
    }
    #[tokio::test]
    async fn block_with_txs_from_height() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.block_with_txs(BLOCK_HEIGHT).await.unwrap().txs[0].txid, TXID);
    }
    #[tokio::test]
    async fn block_header_from_hash() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.block_header(BLOCK_HASH).await.unwrap().hash, BLOCK_HASH);
    }
    #[tokio::test]
    async fn block_header_from_height() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.block_header(BLOCK_HEIGHT).await.unwrap().hash, BLOCK_HASH);
    }
    const ADDRESS: &str = "1CounterpartyXXXXXXXXXXXXXXXUWLpVr";
    #[tokio::test]
    async fn txids() {
        let client = new(DEFAULT_ENDPOINT);
        assert!(!client.txids(ADDRESS).await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn txs() {
        let client = new(DEFAULT_ENDPOINT);
        assert!(!client.txs(ADDRESS).await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn utxos() {
        let client = new(DEFAULT_ENDPOINT);
        assert!(!client.utxos(ADDRESS).await.unwrap().is_empty());
    }
    #[tokio::test]
    async fn rich_list_count() {
        let client = new(DEFAULT_ENDPOINT);
        assert!(client.rich_list_count().await.unwrap().count > 0);
    }
    #[tokio::test]
    async fn rich_list_addr_rank() {
        let client = new(DEFAULT_ENDPOINT);
        assert!(client.rich_list_addr_rank(ADDRESS).await.unwrap().rank > 0);
    }
    #[tokio::test]
    async fn rich_list() {
        let client = new(DEFAULT_ENDPOINT);
        assert_eq!(client.rich_list(0, 100).await.unwrap().len(), 100);
    }
}
