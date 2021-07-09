//! chainseeker - Open-source cryptocurrency block explorer (client library).
//!

pub use serde;
pub use reqwest;
use serde::{Serialize, Deserialize};
#[cfg(feature = "bitcoin")]
pub use bitcoin;
#[cfg(feature = "bitcoin")]
use bitcoin::hashes::hex::FromHex;

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

#[cfg(feature = "bitcoin")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressType {
    P2wpkh,
}

#[cfg(feature = "bitcoin")]
#[derive(Debug, Clone)]
pub struct Wallet {
    pub address_type: AddressType,
    pub private_keys: Vec<bitcoin::PrivateKey>,
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
    /// Generate a new valid transaction from private keys and outputs.
    /// 
    /// # Arguments
    /// 
    /// * `wallet` - The collection of private keys.
    /// * `txouts` - The destinations.
    /// * `change` - The change address in `bitcoin::Script` format.
    /// * `fee_rate` - The fee rate in satoshi / weight.
    /// * `network` - The network.
    #[cfg(feature = "bitcoin")]
    pub async fn generate_tx(
        &self,
        wallet: &Wallet,
        txouts: &[bitcoin::TxOut],
        change: &bitcoin::Script,
        fee_rate: u64,
        network: bitcoin::Network,
    ) -> Result<bitcoin::Transaction, reqwest::Error> {
        // Compute address.
        let secp256k1 = bitcoin::secp256k1::Secp256k1::new();
        let public_keys = wallet.private_keys.iter()
            .map(|private_key| private_key.public_key(&secp256k1))
            .collect::<Vec<bitcoin::PublicKey>>();
        let addresses = match wallet.address_type {
            AddressType::P2wpkh =>
                public_keys.iter()
                    .map(|public_key| bitcoin::Address::p2wpkh(&public_key, network).unwrap())
                    .collect::<Vec<bitcoin::Address>>(),
        };
        // Get UTXOs.
        let mut utxos_list = Vec::with_capacity(addresses.len());
        for address in addresses.iter() {
            utxos_list.push(self.utxos(&address.to_string()).await?);
        }
        // Compute inputs.
        let mut input_value: u64 = 0;
        let inputs_list = utxos_list.iter().map(|utxos| {
            utxos.iter().map(|utxo| {
                input_value += utxo.value;
                bitcoin::TxIn {
                    previous_output: bitcoin::OutPoint {
                        txid: bitcoin::Txid::from_hex(&utxo.txid).unwrap(),
                        vout: utxo.vout,
                    },
                    script_sig: bitcoin::Script::new(),
                    sequence: 0xFFFFFFFF,
                    witness: vec![],
                }
            }).collect::<Vec<bitcoin::TxIn>>()
        }).collect::<Vec<Vec<bitcoin::TxIn>>>();
        // Add change output.
        let output_value = txouts.iter().map(|txout| txout.value).sum::<u64>();
        let mut outputs = txouts.to_vec();
        outputs.push(bitcoin::TxOut {
            script_pubkey: (*change).clone(),
            value: 0,
        });
        // Construct transaction.
        let mut tx = bitcoin::Transaction {
            version: 2,
            lock_time: 0,
            input: inputs_list.concat(),
            output: outputs,
        };
        // Sign transaction.
        let sign = |tx: &mut bitcoin::Transaction| {
            let mut sig_hasher = bitcoin::util::bip143::SigHashCache::new(tx);
            let mut input_index = 0;
            for (utxos, (public_key, private_key)) in utxos_list.iter().zip(public_keys.iter().zip(wallet.private_keys.iter())) {
                let script_code = bitcoin::Script::new_p2pkh(&public_key.pubkey_hash());
                for utxo in utxos.iter() {
                    let sighash = sig_hasher.signature_hash(input_index, &script_code, utxo.value, bitcoin::SigHashType::All);
                    let message = bitcoin::secp256k1::Message::from_slice(&sighash).unwrap();
                    let signature = secp256k1.sign(&message, &private_key.key);
                    let mut signature = signature.serialize_der().to_vec();
                    // Push SIGHASH_ALL.
                    signature.push(0x01);
                    let witness = sig_hasher.access_witness(input_index);
                    witness.clear();
                    witness.push(signature);
                    witness.push(public_key.to_bytes());
                    input_index += 1;
                }
            }
        };
        sign(&mut tx);
        let weight = tx.get_weight() as u64;
        let fee = fee_rate * weight;
        tx.output.last_mut().unwrap().value = input_value - output_value - fee;
        sign(&mut tx);
        Ok(tx)
    }
}

/// Create a new `chainseeker` client.
///
/// The `endpoint` will be the string like <https://btc-v3.chainseeker.info/api>
/// (Note: this string is available via `chainseeker::DEFAULT_ENDPOINT`).
pub fn new(endpoint: &str) -> Client {
    Client::new(endpoint)
}

#[cfg(test)]
mod tests {
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
    #[cfg(feature = "bitcoin")]
    use bitcoin::consensus::Encodable;
    #[cfg(feature = "bitcoin")]
    #[tokio::test]
    async fn put_tx() {
        let secp256k1 = bitcoin::secp256k1::Secp256k1::new();
        let client = new("https://tbtc-v3.chainseeker.info/api");
        let privkey = bitcoin::PrivateKey::from_wif(&std::env::var("CS_PRIVKEY").unwrap()).unwrap();
        let pubkey = privkey.public_key(&secp256k1);
        let wallet = Wallet {
            address_type: AddressType::P2wpkh,
            private_keys: vec![privkey],
        };
        let change = bitcoin::Script::new_v0_wpkh(&pubkey.wpubkey_hash().unwrap());
        let tx = client.generate_tx(&wallet, &[], &change, 100, bitcoin::Network::Testnet).await.unwrap();
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
