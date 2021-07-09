use std::io::{Read, Write};
use num_format::{Locale, ToFormattedStr, ToFormattedString};
pub use bitcoin_rest::bitcoin;
use bitcoin::hashes::hex::FromHex;
use bitcoin::consensus::{Encodable, Decodable};
use bitcoin::{BlockHash, Block, BlockHeader, Address, Script, Network};
use bitcoin::util::uint::Uint256;
use bitcoin::util::address::Payload;
use bitcoin::util::base58;
use bitcoin::bech32;
use bitcoin::bech32::ToBase32;

use crate::db::Database;

pub mod rocks_db;
pub use rocks_db::*;
pub mod rocks_db_multi;
pub use rocks_db_multi::*;
pub mod db;
pub use db::*;
pub mod syncer;
pub use syncer::*;
pub mod rest;
pub use rest::*;
pub mod http_server;
pub use http_server::*;
pub mod web_socket_relay;
pub use web_socket_relay::*;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod integration_test;

const DEFAULT_DATA_DIR: &str = ".chainseeker";

pub fn parse_arguments(args: &[String]) -> Result<(String, Config), String> {
    // Read arguments.
    if args.len() < 2 {
        println!("usage: {} COIN", args[0]);
        return Err("Insufficient arguments.".to_string());
    }
    let coin = &args[1];
    // Load config.
    let config = load_config(coin);
    Ok((coin.to_string(), config))
}

pub async fn main(coin: &str, config: &Config) {
    let db = Database::new(coin, config);
    // Create Syncer instance.
    let mut syncer = Syncer::new(db.clone()).await;
    let mut handles = Vec::new();
    // Run HTTP server.
    {
        let server = HttpServer::new(db.clone());
        let http_ip = config.http_ip.clone();
        let http_port = config.http_port;
        handles.push(tokio::spawn(async move {
            server.run(&http_ip, http_port).await;
        }));
    }
    // Run WebSocketRelay.
    {
        let ws = WebSocketRelay::new(&config.zmq_endpoint, &config.ws_endpoint);
        handles.push(tokio::spawn(async move {
            ws.run().await;
        }));
    }
    // Do initial sync.
    syncer.initial_sync().await;
    // Run syncer.
    syncer.run().await;
    // Join for the threads.
    for handle in handles.iter_mut() {
        handle.await.expect("Failed to await a tokio JoinHandle.");
    }
}

pub fn flush_stdout() {
    std::io::stdout().flush().expect("Failed to flush.");
}

pub fn data_dir() -> String {
    let home = std::env::var("HOME").unwrap();
    format!("{}/{}", home, DEFAULT_DATA_DIR)
}

pub fn get_rest(config: &Config) -> bitcoin_rest::Context {
    bitcoin_rest::new(&config.rest_endpoint)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub genesis_block_hash: BlockHash,
    pub p2pkh_version: u8,
    pub p2sh_version : u8,
    pub segwit_hrp   : String,
    pub rpc_endpoint : String,
    pub rpc_user     : String,
    pub rpc_pass     : String,
    pub rest_endpoint: String,
    pub zmq_endpoint : String,
    pub http_ip      : String,
    pub http_port    : u16,
    pub ws_endpoint  : String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlConfigEntry {
    genesis_block_hash: Option<String>,
    p2pkh_version     : Option<u8>,
    p2sh_version      : Option<u8>,
    segwit_hrp        : Option<String>,
    rpc_endpoint      : Option<String>,
    rpc_user          : Option<String>,
    rpc_pass          : Option<String>,
    rest_endpoint     : Option<String>,
    zmq_endpoint      : Option<String>,
    http_ip           : Option<String>,
    http_port         : Option<u16>,
    ws_endpoint       : Option<String>,
}

pub fn default_genesis_block_hash() -> String {
    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f".to_string()
}
pub fn default_p2pkh_version() -> u8 {
    0
}
pub fn default_p2sh_version() -> u8 {
    5
}
pub fn default_segwit_hrp() -> String {
    "bc".to_string()
}
pub fn default_rpc_endpoint() -> String {
    "http://localhost:8332".to_string()
}
pub fn default_rpc_user() -> String {
    "bitcoin".to_string()
}
pub fn default_rpc_pass() -> String {
    "bitcoinrpc".to_string()
}
pub fn default_rest_endpoint() -> String {
    bitcoin_rest::DEFAULT_ENDPOINT.to_string()
}
pub fn default_zmq_endpoint() -> String {
    "tcp://localhost:28332".to_string()
}
pub fn default_http_ip() -> String {
    "127.0.0.1".to_string()
}
pub fn default_http_port() -> u16 {
    8000
}
pub fn default_ws_endpoint() -> String {
    "127.0.0.1:8001".to_string()
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlConfig {
    #[serde(default = "default_genesis_block_hash")]
    genesis_block_hash: String,
    #[serde(default = "default_p2pkh_version")]
    p2pkh_version     : u8,
    #[serde(default = "default_p2sh_version")]
    p2sh_version      : u8,
    #[serde(default = "default_segwit_hrp")]
    segwit_hrp        : String,
    #[serde(default = "default_rpc_endpoint")]
    rpc_endpoint      : String,
    #[serde(default = "default_rpc_user")]
    rpc_user          : String,
    #[serde(default = "default_rpc_pass")]
    rpc_pass          : String,
    #[serde(default = "default_rest_endpoint")]
    rest_endpoint     : String,
    #[serde(default = "default_zmq_endpoint")]
    zmq_endpoint      : String,
    #[serde(default = "default_http_ip")]
    http_ip           : String,
    #[serde(default = "default_http_port")]
    http_port         : u16,
    #[serde(default = "default_ws_endpoint")]
    ws_endpoint       : String,
    coins             : std::collections::HashMap<String, TomlConfigEntry>,
}

pub fn load_config_from_str(config_str: &str, coin: &str) -> Config {
    let mut config: TomlConfig = toml::from_str(&config_str).expect("Failed to parse config file.");
    let coin_config = config.coins.remove(coin);
    if coin_config.is_none() {
        panic!("Cannot find the specified coin in your config.");
    }
    let coin_config = coin_config.unwrap();
    let genesis_block_hash = BlockHash::from_hex(&coin_config.genesis_block_hash.unwrap_or(config.genesis_block_hash)).unwrap();
    Config {
        genesis_block_hash,
        p2pkh_version: coin_config.p2pkh_version.unwrap_or(config.p2pkh_version),
        p2sh_version : coin_config.p2sh_version .unwrap_or(config.p2sh_version ),
        segwit_hrp   : coin_config.segwit_hrp   .unwrap_or(config.segwit_hrp   ),
        rpc_endpoint : coin_config.rpc_endpoint .unwrap_or(config.rpc_endpoint ),
        rpc_user     : coin_config.rpc_user     .unwrap_or(config.rpc_user     ),
        rpc_pass     : coin_config.rpc_pass     .unwrap_or(config.rpc_pass     ),
        rest_endpoint: coin_config.rest_endpoint.unwrap_or(config.rest_endpoint),
        zmq_endpoint : coin_config.zmq_endpoint .unwrap_or(config.zmq_endpoint ),
        http_ip      : coin_config.http_ip      .unwrap_or(config.http_ip      ),
        http_port    : coin_config.http_port    .unwrap_or(config.http_port    ),
        ws_endpoint  : coin_config.ws_endpoint  .unwrap_or(config.ws_endpoint  ),
    }
}

pub fn load_config(coin: &str) -> Config {
    let mut config_file = std::fs::File::open(&format!("{}/config.toml", data_dir()))
        .expect("Failed to open config file.\nPlease copy \"config.example.toml\" to \"~/.chainseeker/config.toml\".");
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str).expect("Failed to read config file.");
    load_config_from_str(&config_str, coin)
}

pub fn config_example(coin: &str) -> Config {
    load_config_from_str(std::str::from_utf8(include_bytes!("../config.example.toml")).unwrap(), coin)
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

pub fn address_to_string(addr: &Address, config: &Config) -> String {
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
            let vec = vec![vec![ver], prog.to_base32()].concat();
            bech32::encode(&config.segwit_hrp, &vec).unwrap()
        }
    }
}

pub fn script_to_address_string(script: &Script, config: &Config) -> Option<String> {
    Address::from_script(script, Network::Bitcoin /* any */).map(|addr| address_to_string(&addr, config))
}

pub fn uint256_as_f64(num: &Uint256) -> f64 {
    let be = num.to_be_bytes();
    let mut ret = 0f64;
    for i in 0..32 {
        ret += (be[31 - i] as f64) * 2f64.powi(8 * i as i32);
    }
    ret
}

pub fn get_difficulty(block_header: &BlockHeader, _config: &Config) -> f64 {
    let max_target = Uint256::from_u64(0xFFFF).unwrap() << 208;
    uint256_as_f64(&max_target) / uint256_as_f64(&block_header.target())
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
    let mut vec = vec![0; len];
    r.read_exact(&mut vec).expect("Failed to read vec.");
    vec
}

pub fn to_locale_string<T>(num: T) -> String
    where T: ToFormattedStr,
{
    num.to_formatted_string(&Locale::en)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;
    #[test]
    fn parse_arguments() {
        assert_eq!(super::parse_arguments(&[std::env::args().next().unwrap()]).unwrap_err(), "Insufficient arguments.");
    }
    fn script_or_address_to_string(address: &str, script_pubkey: &str) {
        let config = config_example("btc");
        assert_eq!(address_to_string(&Address::from_str(address).unwrap(), &config), address);
        assert_eq!(script_to_address_string(&Script::from_str(script_pubkey).unwrap(), &config).unwrap(), address);
    }
    #[test]
    fn script_or_address_to_string_p2pkh() {
        script_or_address_to_string(
            "1P5ZEDWTKTFGxQjZphgWPQUpe554WKDfHQ",
            "76a914f22f5563839ba6ba5aa8d3726fcbc675cb3e4c9e88ac",
        );
    }
    #[test]
    fn script_or_address_to_string_p2sh() {
        script_or_address_to_string(
            "34xp4vRoCGJym3xR7yCVPFHoCNxv4Twseo",
            "a91423e522dfc6656a8fda3d47b4fa53f7585ac758cd87",
        );
    }
    #[test]
    fn script_or_address_to_string_p2wpkh() {
        // Test vectors come from https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki.
        script_or_address_to_string(
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
            "0014751e76e8199196d454941c45d1b3a323f1433bd6",
        );
    }
    #[test]
    fn uint256_as_f64_12345() {
        assert!((uint256_as_f64(&Uint256::from_u64(12345).unwrap()) - 12345f64).abs() < f64::EPSILON);
    }
}
