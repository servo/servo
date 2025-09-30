use std::collections::HashMap;
use std::path::PathBuf;

use crate::webstorage::engines::WebStorageEngine;
use crate::webstorage::webstorage_thread::OriginEntry;

pub struct BincodeEngine(PathBuf);

impl BincodeEngine {
    pub fn new(dir: &PathBuf) -> Self {
        Self(dir.join("local_data.bin"))
    }
}

impl WebStorageEngine for BincodeEngine {
    fn load(&self) -> HashMap<String, OriginEntry> {
        if let Ok(data) = std::fs::read(&self.0) {
            if let Ok(map) = bincode::deserialize(&data) {
                return map;
            }
        }
        HashMap::new()
    }

    fn save(&self, data: HashMap<String, OriginEntry>) {
        if let Ok(encoded) = bincode::serialize(&data) {
            if let Some(parent) = self.0.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&self.0, encoded);
        }
    }
}
