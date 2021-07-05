use std::fs::create_dir_all;
use crate::*;

#[derive(Debug, Clone)]
pub struct SyncedHeightDB {
    coin: String,
    //synced_height: Option<u32>,
}

impl SyncedHeightDB {
    fn dir(coin: &str) -> String {
        format!("{}/{}", data_dir(), coin)
    }
    fn path(coin: &str) -> String {
        format!("{}/synced_height.txt", Self::dir(coin))
    }
    pub fn new(coin: &str) -> Self {
        /*
        let synced_height = match std::fs::read_to_string(&Self::path(coin)) {
            Ok(s) => Some(s.parse().unwrap()),
            Err(_) => None,
        };
        */
        Self {
            coin: coin.to_string(),
            //synced_height,
        }
    }
    pub fn get(&self) -> Option<u32> {
        match std::fs::read_to_string(&Self::path(&self.coin)) {
            Ok(s) => Some(s.parse().unwrap()),
            Err(_) => None,
        }
    }
    pub fn put(&self, synced_height: u32) {
        create_dir_all(&Self::dir(&self.coin)).unwrap();
        std::fs::write(&Self::path(&self.coin), synced_height.to_string()).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn synced_height() {
        let synced_height_db = SyncedHeightDB::new("test/synced_height");
        synced_height_db.put(123456);
        assert_eq!(synced_height_db.get(), Some(123456));
    }
}
