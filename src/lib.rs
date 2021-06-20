use std::io::{Read, Write};
use bitcoin::hash_types::{Txid, BlockHash};
use bitcoin::blockdata::script::Script;
use bitcoin::consensus::{Encodable, Decodable};

pub mod address_index;
pub mod utxo;

const DEFAULT_DATA_DIR: &str = ".chainseeker";

pub fn get_rest(coin: &str) -> bitcoin_rest::Context {
    let port = match coin {
        "btc" => 8332,
        "tbtc" => 18332,
        _ => panic!("COIN not supported."),
    };
    let host = std::env::var("BITCOIN_REST_HOST").unwrap_or("localhost".to_string());
    let endpoint = format!("http://{}:{}/rest/", host, port);
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
