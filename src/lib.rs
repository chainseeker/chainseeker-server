use std::io::{Read, Write};
use num_format::{Locale, ToFormattedStr, ToFormattedString};
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use bitcoin::consensus::{Encodable, Decodable};
use bitcoin::{BlockHash, Block, BlockHeader, Address, Script, Network};
use bitcoin::util::uint::Uint256;
use bitcoin::util::address::Payload;
use bitcoin::util::base58;
use bitcoin::bech32;
use bitcoin::bech32::ToBase32;

pub mod rocks_db;
pub use rocks_db::*;
pub mod rocks_db_multi;
pub use rocks_db_multi::*;
pub mod rocks_db_lazy;
pub use rocks_db_lazy::*;
pub mod db;
pub use db::*;
pub mod syncer;
pub use syncer::*;
pub mod http_server;
pub use http_server::*;
pub mod web_socket_relay;
pub use web_socket_relay::*;
#[cfg(test)]
pub mod fixtures;

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

fn synced_height_path(coin: &str) -> String {
    format!("{}/{}/synced_height.txt", data_dir(), coin)
}

pub async fn fetch_block(rest: &bitcoin_rest::Context, height: u32) -> (BlockHash, Block) {
    let block_hash = rest.blockhashbyheight(height).await
        .expect(&format!("Failed to fetch block at height = {}.", height));
    let block = rest.block(&block_hash).await.expect(&format!("Failed to fetch a block with blockid = {}", block_hash));
    (block_hash, block)
}

pub fn get_synced_height(coin: &str) -> Option<u32> {
    match std::fs::read_to_string(&synced_height_path(coin)) {
        Ok(s) => Some(s.parse().unwrap()),
        Err(_) => None,
    }
}

pub fn put_synced_height(coin: &str, height: u32) {
    std::fs::write(&synced_height_path(coin), height.to_string()).unwrap()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CoinConfig {
    pub p2pkh_version: u8,
    pub p2sh_version: u8,
    pub segwit_hrp: String,
    pub rpc_endpoint: String,
    pub rpc_user: String,
    pub rpc_pass: String,
    pub rest_endpoint: String,
    pub zmq_endpoint: String,
    pub http_ip: String,
    pub http_port: u16,
    pub ws_endpoint: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub coins: std::collections::HashMap<String, CoinConfig>,
}

pub fn load_config(coin: &str) -> CoinConfig {
    let config_path = format!("{}/config.toml", data_dir());
    let mut config_file = std::fs::File::open(&config_path)
        .expect("Failed to open config file.\nPlease copy \"config.example.toml\" to \"~/.chainseeker/config.toml\".");
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str).expect("Failed to read config file.");
    let mut config: Config = toml::from_str(&config_str).expect("Failed to parse config file.");
    match config.coins.remove(coin) {
        Some(coin_config) => coin_config,
        None => panic!("Cannot find the specified coin in your config."),
    }
}

pub fn consensus_encode<E>(enc: &E) -> Vec<u8>
    where E: Encodable,
{
    let mut vec = Vec::new();
    enc.consensus_encode(&mut vec).unwrap();
    vec
}

pub fn consensus_decode<D>(dec: &[u8]) -> D
    where D: Decodable,
{
    D::consensus_decode(dec).unwrap()
}

fn address_to_string(addr: &Address, config: &CoinConfig) -> String {
    match addr.payload {
        Payload::PubkeyHash(ref hash) => {
            let mut prefixed = [0; 21];
            prefixed[0] = config.p2pkh_version;
            prefixed[1..].copy_from_slice(&hash[..]);
            base58::check_encode_slice(&prefixed[..])
        }
        Payload::ScriptHash(ref hash) => {
            let mut prefixed = [0; 21];
            prefixed[0] = config.p2sh_version;
            prefixed[1..].copy_from_slice(&hash[..]);
            base58::check_encode_slice(&prefixed[..])
        }
        Payload::WitnessProgram {
            version: ver,
            program: ref prog,
        } => {
            let vec = vec![vec![ver.to_u8()], prog.clone()].concat();
            bech32::encode(&config.segwit_hrp, &vec.to_base32()).unwrap()
        }
    }
}

pub fn script_to_address_string(script: &Script, config: &CoinConfig) -> Option<String> {
    let addr = Address::from_script(script, Network::Bitcoin /* any */);
    match addr {
        Some(addr) => Some(address_to_string(&addr, config)),
        None => None,
    }
}

pub fn uint256_as_f64(num: &Uint256) -> f64 {
    let be = num.to_be_bytes();
    let mut ret = 0f64;
    for i in 0..32 {
        ret += (be[31 - i] as f64) * 2f64.powi(8 * i as i32);
    }
    ret
}

pub fn get_difficulty(block_header: &BlockHeader, _config: &CoinConfig) -> f64 {
    let max_target = Uint256::from_u64(0xFFFF).unwrap() << 208;
    uint256_as_f64(&max_target) / uint256_as_f64(&block_header.target())
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

pub fn bytes_to_i32(buf: &[u8]) -> i32 {
    assert_eq!(buf.len(), 4);
    let mut tmp: [u8; 4] = [0; 4];
    tmp.copy_from_slice(&buf);
    i32::from_le_bytes(tmp)
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

pub fn to_locale_string<T>(num: T) -> String
    where T: ToFormattedStr,
{
    num.to_formatted_string(&Locale::en)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn uint256_as_f64_12345() {
        assert_eq!(uint256_as_f64(&Uint256::from_u64(12345).unwrap()), 12345f64);
    }
    #[test]
    fn synced_height() {
        put_synced_height("test", 123456);
        assert_eq!(get_synced_height("test"), Some(123456));
    }
}
