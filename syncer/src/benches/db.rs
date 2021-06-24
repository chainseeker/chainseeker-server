use criterion::{criterion_group, criterion_main, Criterion};

use bitcoin::consensus::Decodable;
use bitcoin::Block;

use chainseeker_syncer::*;

const COIN: &str = "bench";
const BLOCK: &[u8] = include_bytes!("block_500000.bin");

fn bench_db(c: &mut Criterion) {
    let block = Block::consensus_decode(BLOCK).expect("Failed to decode block.");
    let mut utxo_db = UtxoDB::new(COIN);
    c.bench_function("utxo", |b| b.iter(|| {
        utxo_db.process_block(&block, true);
    }));
    // Construct dummy data.
    let mut previous_pubkeys = Vec::new();
    for tx in block.txdata.iter() {
        for _vin in tx.input.iter() {
            previous_pubkeys.push(block.txdata[0].output[0].script_pubkey.clone());
        }
    }
    let addr_index_db = AddressIndexDB::new(COIN);
    c.bench_function("address_index", |b| b.iter(|| {
        addr_index_db.process_block(&block, &previous_pubkeys);
    }));
}

criterion_group!(benches, bench_db);
criterion_main!(benches);
