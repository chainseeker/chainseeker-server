use std::io::{Read, Write};
use std::fs::File;
use bitcoin::hash_types::{Txid, BlockHash};
use bitcoin::blockdata::script::Script;
use bitcoin::consensus::{Encodable, Decodable};

pub mod address_index;
pub mod utxo;

const REST_ENDPOINT_ENV_NAME: &str = "BITCOIN_REST_ENDPOINT";
const DEFAULT_DATA_DIR: &str = ".chainseeker";

pub fn get_rest() -> bitcoin_rest::Context {
    let endpoint = std::env::var(REST_ENDPOINT_ENV_NAME).unwrap_or(bitcoin_rest::DEFAULT_ENDPOINT.to_string());
    bitcoin_rest::new(&endpoint)
}

pub fn get_data_dir_path() -> Result<String, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    Ok(home + "/" + DEFAULT_DATA_DIR)
}

pub fn serialize_script(script: &Script) -> Vec<u8> {
    script.to_bytes()
}

pub fn deserialize_script(script_vec: &Vec<u8>) -> Script {
    Script::from(script_vec.clone())
}

pub fn serialize_txid(txid: &Txid) -> [u8; 32] {
    let mut txid_vec: [u8; 32] = [0; 32];
    txid.consensus_encode(&mut txid_vec as &mut [u8]).expect("Failed to encode txid.");
    txid_vec
}

pub fn deserialize_txid(txid_vec: &[u8]) -> Txid {
    Txid::consensus_decode(&txid_vec[..]).expect("Failed to decode txid.")
}

pub fn serialize_block_hash(block_hash: &BlockHash) -> [u8; 32] {
    let mut block_hash_vec: [u8; 32] = [0; 32];
    block_hash.consensus_encode(&mut block_hash_vec as &mut [u8]).expect("Failed to encode block hash.");
    block_hash_vec
}

pub fn deserialize_block_hash(block_hash_vec: &[u8]) -> BlockHash {
    BlockHash::consensus_decode(&block_hash_vec[..]).expect("Failed to decode block hash.")
}

pub fn write_u32(file: &mut File, n: u32) {
    assert_eq!(file.write(&n.to_le_bytes()).unwrap(), 4);
}

pub fn read_u32(file: &mut File) -> u32 {
    let mut buf: [u8; 4] = [0; 4];
    assert_eq!(file.read(&mut buf).unwrap(), 4);
    u32::from_le_bytes(buf)
}

pub fn write_u64(file: &mut File, n: u64) {
    assert_eq!(file.write(&n.to_le_bytes()).unwrap(), 8);
}

pub fn read_u64(file: &mut File) -> u64 {
    let mut buf: [u8; 8] = [0; 8];
    assert_eq!(file.read(&mut buf).unwrap(), 8);
    u64::from_le_bytes(buf)
}

pub fn write_usize(file: &mut File, n: usize) {
    const BYTES: usize = std::mem::size_of::<usize>();
    assert_eq!(file.write(&n.to_le_bytes()).unwrap(), BYTES);
}

pub fn read_usize(file: &mut File) -> usize {
    const BYTES: usize = std::mem::size_of::<usize>();
    let mut buf: [u8; BYTES] = [0; BYTES];
    assert_eq!(file.read(&mut buf).unwrap(), BYTES);
    usize::from_le_bytes(buf)
}

pub fn write_arr(file: &mut File, arr: &[u8]) {
    assert_eq!(file.write(&arr).unwrap(), arr.len());
}

pub fn read_vec(file: &mut File, len: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(len);
    vec.resize(len, 0);
    assert_eq!(file.read(&mut vec).unwrap(), len);
    vec
}
