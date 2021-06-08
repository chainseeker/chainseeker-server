use std::io::Write;
use std::time::Instant;

use chainseeker_syncer::*;
use chainseeker_syncer::address_index::*;
use chainseeker_syncer::utxo::*;

const UTXO_DELETE_THRESHOLD: u32 = 20;
const DEFAULT_UTXO_SAVE_INTERVAL: u32 = 100;

struct Syncer {
    addr_index_db: AddressIndexDB,
    utxo_db: UtxoDB,
    rest: bitcoin_rest::Context,
    synced_height: Option<u32>,
}

impl Syncer {
    fn new() -> Self {
        let addr_index_db = AddressIndexDB::load();
        let synced_height = addr_index_db.synced_height();
        Syncer{
            addr_index_db,
            utxo_db: match synced_height {
                Some(h) => {
                    print!("Loading UTXO database...");
                    std::io::stdout().flush().expect("Failed to flush to stdout.");
                    let begin = Instant::now();
                    let db = UtxoDB::load(h);
                    println!(" {}ms.", begin.elapsed().as_millis());
                    db
                },
                None => UtxoDB::new(),
            },
            rest: get_rest(),
            synced_height,
        }
    }
    async fn process_block(&mut self, height: u32, save: bool) {
        let begin = Instant::now();
        print!("Height={:6}", height);
        let begin_rest = Instant::now();
        let blockid = self.rest.blockhashbyheight(height).await
            .expect(&format!("Failed to fetch block at height = {}.", height));
        let block = self.rest.block(blockid).await.expect(&format!("Failed to fetch a block with blockid = {}", blockid));
        let rest_elapsed = begin_rest.elapsed();
        print!(", #tx={:4}", block.txdata.len());
        let begin_utxo = Instant::now();
        let previous_pubkeys = self.utxo_db.process_block(&block);
        let utxo_elapsed = begin_utxo.elapsed();
        let begin_addr_index = Instant::now();
        self.addr_index_db.process_block(&block, previous_pubkeys);
        let addr_index_elapsed = begin_addr_index.elapsed();
        let mut vins: usize = 0;
        let mut vouts: usize = 0;
        for tx in block.txdata.iter() {
            vins += tx.input.len();
            vouts += tx.output.len();
        }
        println!(
            ", #vin={:4}, #vout={:4}, #utxo={:9} (rest:{:3}ms, utxo:{:3}ms, addr:{:3}ms, total:{:3}ms)",
            vins, vouts, self.utxo_db.len(),
            rest_elapsed.as_millis(), utxo_elapsed.as_millis(), addr_index_elapsed.as_millis(), begin.elapsed().as_millis());
        if save {
            let begin = Instant::now();
            std::io::stdout().flush().expect("Failed to flush to stdout.");
            self.utxo_db.save(height);
            self.addr_index_db.save();
            self.synced_height = Some(height);
            if height > UTXO_DELETE_THRESHOLD {
                UtxoDB::delete_older_than(height - UTXO_DELETE_THRESHOLD);
            }
            println!("Saved in {}ms.", begin.elapsed().as_millis());
        }
    }
    async fn run(&mut self, utxo_save_interval: u32) -> u32 {
        let start_height = match self.synced_height {
            Some(h) => h + 1,
            None => 0,
        };
        let chaininfo = self.rest.chaininfo().await.expect("Failed to fetch chaininfo.");
        let target_height = chaininfo.blocks;
        for height in start_height..(target_height + 1) {
            let save = ((height - start_height) % utxo_save_interval == utxo_save_interval - 1) || height == target_height;
            self.process_block(height, save).await;
        }
        target_height - start_height + 1
    }
}

#[tokio::main]
async fn main() {
    let mut syncer = Syncer::new();
    syncer.run(DEFAULT_UTXO_SAVE_INTERVAL).await;
}
