use std::fs::create_dir_all;
use crate::*;

#[derive(Debug, Clone)]
pub struct SyncedHeightDB {
    coin: String,
    synced_height: Option<u32>,
}

impl SyncedHeightDB {
    fn dir(coin: &str) -> String {
        format!("{}/{}", data_dir(), coin)
    }
    fn path(coin: &str) -> String {
        format!("{}/synced_height.txt", Self::dir(coin))
    }
    pub fn new(coin: &str) -> Self {
        let synced_height = std::fs::read_to_string(&Self::path(coin)).map_or_else(|_| None, |s| Some(s.parse().unwrap()));
        Self {
            coin: coin.to_string(),
            synced_height,
        }
    }
    pub fn get(&self) -> Option<u32> {
        self.synced_height
    }
    pub fn put(&mut self, synced_height: u32) {
        create_dir_all(&Self::dir(&self.coin)).unwrap();
        std::fs::write(&Self::path(&self.coin), synced_height.to_string()).unwrap();
        self.synced_height = Some(synced_height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synced_height() {
        let path = SyncedHeightDB::path("test/synced_height");
        if std::path::Path::new(&path).exists() {
            std::fs::remove_file(&path).unwrap();
        }
        let mut synced_height_db = SyncedHeightDB::new("test/synced_height");
        assert_eq!(synced_height_db.get(), None);
        synced_height_db.put(123456);
        assert_eq!(synced_height_db.get(), Some(123456));
    }
}
