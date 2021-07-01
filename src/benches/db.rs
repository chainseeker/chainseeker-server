use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use bitcoin::consensus::Decodable;
use bitcoin::Block;

use chainseeker_syncer::*;

const COIN: &str = "bench";
const BLOCK: &[u8] = include_bytes!("../fixtures/mainnet/block_500000.bin");

async fn run_utxo_server_in_memory(utxos: &Vec<UtxoEntry>) {
    let _print_gag = gag::Gag::stdout().unwrap();
    let mut utxo_server = UtxoServerInMemory::new(COIN);
    for utxo in utxos {
        utxo_server.push(&utxo).await;
    }
}

async fn run_utxo_server_in_storage(utxos: &Vec<UtxoEntry>) {
    let _print_gag = gag::Gag::stdout().unwrap();
    let mut utxo_server = UtxoServerInStorage::new(COIN);
    for utxo in utxos {
        utxo_server.push(&utxo).await;
    }
}

async fn run_utxo_server_in_storage_lazy(utxos: &Vec<UtxoEntry>) {
    let _print_gag = gag::Gag::stdout().unwrap();
    let mut utxo_server = UtxoServerInStorageLazy::new(COIN);
    for utxo in utxos {
        utxo_server.push(&utxo).await;
    }
}

fn bench_db(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let block = Block::consensus_decode(BLOCK).expect("Failed to decode block.");
    let mut utxo_db = UtxoDB::new(COIN, true);
    c.bench_function("UtxoDB", |b| b.iter(|| {
        utxo_db.process_block(&block, true);
    }));
    let utxos = utxo_db.process_block(&block, true);
    c.bench_function("UtxoServerInMemory", |b| b.iter(|| {
        rt.block_on(async {
            run_utxo_server_in_memory(&utxos).await;
        });
    }));
    c.bench_function("UtxoServerInStorage", |b| b.iter(|| {
        rt.block_on(async {
            run_utxo_server_in_storage(&utxos).await;
        });
    }));
    c.bench_function("UtxoServerInStorageLazy", |b| b.iter(|| {
        rt.block_on(async {
            run_utxo_server_in_storage_lazy(&utxos).await;
        });
    }));
    // Construct dummy data.
    let mut previous_utxos = Vec::new();
    for tx in block.txdata.iter() {
        for _vin in tx.input.iter() {
            previous_utxos.push(UtxoEntry {
                script_pubkey: block.txdata[0].output[0].script_pubkey.clone(),
                txid: tx.txid(),
                vout: 0,
                value: 12345678u64,
            });
        }
    }
    c.bench_function("RichList", |b| b.iter(|| {
        let _print_gag = gag::Gag::stdout().unwrap();
        let mut rich_list_builder = RichListBuilder::new();
        rich_list_builder.process_block(&block, &previous_utxos);
        let _rich_list = rich_list_builder.finalize();
    }));
    let addr_index_db = AddressIndexDB::new(COIN, true);
    c.bench_function("AddressIndexDB", |b| b.iter(|| {
        addr_index_db.process_block(&block, &previous_utxos);
    }));
}

criterion_group!(benches, bench_db);
criterion_main!(benches);
