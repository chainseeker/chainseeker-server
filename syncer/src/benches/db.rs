use criterion::{criterion_group, criterion_main, Criterion};
use criterion::async_executor::FuturesExecutor;

use bitcoin::consensus::Decodable;
use bitcoin::Block;

use chainseeker_syncer::*;

const COIN: &str = "bench";
const BLOCK: &[u8] = include_bytes!("block_500000.bin");

async fn run_utxo_server_in_memory(utxo: &Utxo) {
    let _print_gag = gag::Gag::stdout().unwrap();
    UtxoServerInMemory::from(utxo).await;
}

async fn run_utxo_server_in_storage(utxo: &Utxo) {
    let _print_gag = gag::Gag::stdout().unwrap();
    UtxoServerInStorage::from(utxo).await;
}

fn bench_db(c: &mut Criterion) {
    let block = Block::consensus_decode(BLOCK).expect("Failed to decode block.");
    let mut utxo_db = UtxoDB::new(COIN);
    c.bench_function("utxo_db", |b| b.iter(|| {
        utxo_db.process_block(&block, true);
    }));
    utxo_db.process_block(&block, true);
    let utxo = Utxo::from(&utxo_db);
    c.bench_function("utxo_server_in_memory", |b| b.to_async(FuturesExecutor).iter(|| {
        run_utxo_server_in_memory(&utxo)
    }));
    c.bench_function("utxo_server_in_storage", |b| b.to_async(FuturesExecutor).iter(|| {
        run_utxo_server_in_storage(&utxo)
    }));
    c.bench_function("rich_list", |b| b.iter(|| {
        let _print_gag = gag::Gag::stdout().unwrap();
        RichList::from(&utxo);
    }));
    // Construct dummy data.
    let mut previous_pubkeys = Vec::new();
    for tx in block.txdata.iter() {
        for _vin in tx.input.iter() {
            previous_pubkeys.push(block.txdata[0].output[0].script_pubkey.clone());
        }
    }
    let addr_index_db = AddressIndexDB::new(COIN);
    c.bench_function("address_index_db", |b| b.iter(|| {
        addr_index_db.process_block(&block, &previous_pubkeys);
    }));
}

criterion_group!(benches, bench_db);
criterion_main!(benches);
