use std::io::{Read, Write};
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use bitcoin::consensus::{Encodable, Decodable};
use bitcoin::{Script, Txid, BlockHash, Block, BlockHeader};

pub mod rocks_db;
pub use rocks_db::*;
pub mod rocks_db_multi;
pub use rocks_db_multi::*;
pub mod rocks_db_lazy;
pub use rocks_db_lazy::*;
pub mod db;
pub use db::*;
pub mod block_fetcher;
pub use block_fetcher::*;
pub mod syncer;
pub use syncer::*;
pub mod http_server;
pub use http_server::*;
#[cfg(test)]
pub mod test_fixtures;

const DEFAULT_DATA_DIR: &str = ".chainseeker";

type RocksDBBase = DBWithThreadMode<MultiThreaded>;

pub fn flush_stdout() {
    std::io::stdout().flush().expect("Failed to flush.");
}

pub fn data_dir() -> String {
    let home = std::env::var("HOME").unwrap();
    format!("{}/{}", home, DEFAULT_DATA_DIR)
}

pub fn get_rest(config: &CoinConfig) -> bitcoin_rest::Context {
    bitcoin_rest::new(&config.rest_endpoint)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CoinConfig {
    pub rest_endpoint: String,
    pub zmq_endpoint: String,
    pub http_port: u16,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub http_ip: String,
    pub coins: std::collections::HashMap<String, CoinConfig>,
}

pub fn load_config() -> Config {
    let config_path = format!("{}/config.toml", data_dir());
    let mut config_file = std::fs::File::open(&config_path)
        .expect("Failed to open config file.\nPlease copy \"config.example.toml\" to \"~/.chainseeker/config.toml\".");
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str).expect("Failed to read config file.");
    toml::from_str(&config_str).expect("Failed to parse config file.")
}

pub fn serialize_script(script: &Script) -> Vec<u8> {
    script.to_bytes()
}

pub fn deserialize_script(script_vec: &[u8]) -> Script {
    Script::from(script_vec.to_vec())
}

pub fn serialize_txid(txid: &Txid) -> [u8; 32] {
    let mut txid_vec: [u8; 32] = [0; 32];
    txid.consensus_encode(&mut txid_vec as &mut [u8]).expect("Failed to encode txid.");
    txid_vec
}

pub fn deserialize_txid(txid_vec: &[u8]) -> Txid {
    Txid::consensus_decode(&txid_vec[..]).expect("Failed to decode txid.")
}

pub fn deserialize_block(block_vec: &[u8]) -> Block {
    Block::consensus_decode(block_vec).unwrap()
}

pub fn serialize_block_hash(block_hash: &BlockHash) -> [u8; 32] {
    let mut block_hash_vec: [u8; 32] = [0; 32];
    block_hash.consensus_encode(&mut block_hash_vec as &mut [u8]).expect("Failed to encode block hash.");
    block_hash_vec
}

pub fn deserialize_block_hash(block_hash_vec: &[u8]) -> BlockHash {
    BlockHash::consensus_decode(&block_hash_vec[..]).expect("Failed to decode block hash.")
}

pub fn serialize_block_header(block_header: &BlockHeader) -> Vec<u8> {
    let mut block_header_vec = Vec::new();
    block_header.consensus_encode(&mut block_header_vec).expect("Failed to encode block header.");
    block_header_vec
}

pub fn deserialize_block_header(block_header_vec: &[u8]) -> BlockHeader {
    BlockHeader::consensus_decode(&block_header_vec[..]).expect("Failed to decode block hash.")
}

pub fn bytes_to_u16(buf: &[u8]) -> u16 {
    assert_eq!(buf.len(), 2);
    let mut tmp: [u8; 2] = [0; 2];
    tmp.copy_from_slice(&buf);
    u16::from_le_bytes(tmp)
}

pub fn bytes_to_u32(buf: &[u8]) -> u32 {
    assert_eq!(buf.len(), 4);
    let mut tmp: [u8; 4] = [0; 4];
    tmp.copy_from_slice(&buf);
    u32::from_le_bytes(tmp)
}

pub fn bytes_to_u64(buf: &[u8]) -> u64 {
    assert_eq!(buf.len(), 8);
    let mut tmp: [u8; 8] = [0; 8];
    tmp.copy_from_slice(&buf);
    u64::from_le_bytes(tmp)
}

pub fn write_u32<W>(w: &mut W, n: u32)
    where W: Write
{
    w.write_all(&n.to_le_bytes()).expect("Failed to write u32.");
}

pub fn read_u32<R>(r: &mut R) -> u32
    where R: Read
{
    let mut buf: [u8; 4] = [0; 4];
    r.read_exact(&mut buf).expect("Failed to read u32.");
    u32::from_le_bytes(buf)
}

pub fn write_u64<W>(w: &mut W, n: u64)
    where W: Write
{
    w.write_all(&n.to_le_bytes()).expect("Failed to write u64.");
}

pub fn read_u64<R>(r: &mut R) -> u64
    where R: Read
{
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf).expect("Failed to read u64.");
    u64::from_le_bytes(buf)
}

pub fn write_usize<W>(w: &mut W, n: usize)
    where W: Write
{
    w.write_all(&n.to_le_bytes()).expect("Failed to write usize.");
}

pub fn read_usize<R>(r: &mut R) -> usize
    where R: Read
{
    const BYTES: usize = std::mem::size_of::<usize>();
    let mut buf: [u8; BYTES] = [0; BYTES];
    r.read_exact(&mut buf).expect("Failed to read usize.");
    usize::from_le_bytes(buf)
}

pub fn write_arr<W>(w: &mut W, arr: &[u8])
    where W: Write
{
    w.write_all(&arr).expect("Failed to write arr.");
}

pub fn read_vec<R>(r: &mut R, len: usize) -> Vec<u8>
    where R: Read
{
    let mut vec = Vec::with_capacity(len);
    vec.resize(len, 0);
    r.read_exact(&mut vec).expect("Failed to read vec.");
    vec
}
