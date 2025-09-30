mod bincode;

use std::collections::HashMap;

pub use bincode::BincodeEngine;

use crate::webstorage::webstorage_thread::OriginEntry;

pub trait WebStorageEngine {
    fn load(&self) -> HashMap<String, OriginEntry>;
    fn save(&self, data: HashMap<String, OriginEntry>);
}

pub struct MemoryEngine;

impl WebStorageEngine for MemoryEngine {
    fn load(&self) -> HashMap<String, OriginEntry> {
        HashMap::new()
    }

    fn save(&self, _data: HashMap<String, OriginEntry>) {}
}
