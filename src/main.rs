use std::io::Write;
use std::time::Instant;

use chainseeker_syncer::*;
use chainseeker_syncer::address_index::*;
use chainseeker_syncer::utxo::*;

const UTXO_DELETE_THRESHOLD: u32 = 100;
const DEFAULT_UTXO_SAVE_INTERVAL: u32 = 1000;

struct Syncer {
    addr_index_db: AddressIndexDB,
    utxo_db: UtxoDB,
    rest: bitcoin_rest::Context,
    synced_height: Option<u32>,
}

impl Syncer {
    fn new() -> Self {
        let addr_index_db = AddressIndexDB::new();
        let synced_height = addr_index_db.get_synced_height();
        Syncer{
            addr_index_db,
            utxo_db: match synced_height {
                Some(h) => UtxoDB::load(h),
                None => UtxoDB::new(),
            },
            rest: get_rest(),
            synced_height,
        }
    }
    async fn process_block(&mut self, height: u32, save: bool) {
        let begin = Instant::now();
        print!("Syncing at block height = {:6}", height);
        let blockid = self.rest.blockhashbyheight(height).await
            .expect(&format!("Failed to fetch block at height = {}.", height));
        let block = self.rest.block(blockid).await.expect(&format!("Failed to fetch a block with blockid = {}", blockid));
        print!(", txs = {:5}", block.txdata.len());
        let previous_pubkeys = self.utxo_db.process_block(&block);
        self.addr_index_db.process_block(&block, previous_pubkeys);
        let mut vins: usize = 0;
        let mut vouts: usize = 0;
        for tx in block.txdata.iter() {
            vins += tx.input.len();
            vouts += tx.output.len();
        }
        println!(
            ", #vins = {:5}, #vouts = {:5}, #utxos = {:9} ({:4}ms)",
            vins, vouts, self.utxo_db.len(), begin.elapsed().as_millis());
        if save {
            let begin = Instant::now();
            print!("Saving...");
            std::io::stdout().flush().expect("Failed to flush to stdout.");
            self.utxo_db.save(height);
            self.addr_index_db.put_synced_height(height);
            self.synced_height = Some(height);
            UtxoDB::delete_older_than(height - UTXO_DELETE_THRESHOLD);
            println!(" ({:4}ms)", begin.elapsed().as_millis());
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
            let save = (height - start_height) % utxo_save_interval == 0;
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
