use crate::*;

pub struct SyncedHeightDB {
    coin: String,
}

impl SyncedHeightDB {
    fn path(coin: &str) -> String {
        format!("{}/{}/synced_height.txt", data_dir(), coin)
    }
    pub fn new(coin: &str) -> Self {
        Self {
            coin: coin.to_string(),
        }
    }
    pub fn get(&self) -> Option<u32> {
        match std::fs::read_to_string(&Self::path(&self.coin)) {
            Ok(s) => Some(s.parse().unwrap()),
            Err(_) => None,
        }
    }
    pub fn put(&self, height: u32) {
        std::fs::write(&Self::path(&self.coin), height.to_string()).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn synced_height_db() {
        let db = SyncedHeightDB::new("test");
        db.put(123456);
        assert_eq!(db.get(), Some(123456));
    }
}
