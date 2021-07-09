use std::sync::Arc;
use tokio::sync::RwLock;
use crate::Config;

pub mod synced_height;
pub use synced_height::*;
pub mod block;
pub use block::*;
pub mod tx;
pub use tx::*;
pub mod address_index;
pub use address_index::*;
pub mod utxo;
pub use utxo::*;
pub mod utxo_server;
pub use utxo_server::*;
pub mod rich_list;
pub use rich_list::*;

#[derive(Debug, Clone)]
pub struct Database {
    pub coin: String,
    pub config: Config,
    pub synced_height_db: Arc<RwLock<SyncedHeightDB>>,
    pub block_db: Arc<RwLock<BlockDB>>,
    pub tx_db: Arc<RwLock<TxDB>>,
    pub addr_index_db: Arc<RwLock<AddressIndexDB>>,
    pub utxo_server: Arc<RwLock<UtxoServer>>,
    pub rich_list: Arc<RwLock<RichList>>,
}

impl Database {
    pub fn new(coin: &str, config: &Config) -> Self {
        Self {
            coin: coin.to_string(),
            config: (*config).clone(),
            synced_height_db: Arc::new(RwLock::new(SyncedHeightDB::new(coin))),
            block_db        : Arc::new(RwLock::new(BlockDB::new(coin, false))),
            tx_db           : Arc::new(RwLock::new(TxDB::new(coin, false))),
            addr_index_db   : Arc::new(RwLock::new(AddressIndexDB::new(coin, false))),
            utxo_server     : Arc::new(RwLock::new(UtxoServer::new())),
            rich_list       : Arc::new(RwLock::new(RichList::new())),
        }
    }
}
