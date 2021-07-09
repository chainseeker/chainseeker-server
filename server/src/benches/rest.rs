use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use bitcoin::consensus::Decodable;
use bitcoin::Block;

use chainseeker_server::*;

const COIN: &str = "bench/rest";

fn bench_rest(c: &mut Criterion) {
    let blocks = fixtures::regtest_blocks();
    let mut utxo_db = UtxoDB::new(COIN, true);
    let addr_index_db = AddressIndexDB::new(COIN, true);
    for block in blocks.iter() {
        let prev_utxos = utxo_db.process_block(&block, false);
        addr_index_db.process_block(&block, &prev_utxos);
    }
    c.bench_function("Utxo", |b| b.iter(|| {
        create_utxo(utxo_db.get(blocks.last().txdata[0].vout[0].script_pubkey));
    }));
}

criterion_group!(benches, bench_rest);
criterion_main!(benches);
