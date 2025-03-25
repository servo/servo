use std::path::PathBuf;

use malloc_size_of::MallocSizeOfOps;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, MallocSizeOf)]
pub struct UserContentManager {
    user_scripts: Vec<UserScript>,
}

impl UserContentManager {
    pub fn new() -> Self {
        UserContentManager::default()
    }

    pub fn add_script(&mut self, script: impl Into<UserScript>) {
        self.user_scripts.push(script.into());
    }

    pub fn get_scripts(&self) -> &Vec<UserScript> {
        &self.user_scripts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScript {
    pub script: String,
    pub source_file: Option<PathBuf>,
}

// Maybe we should implement `MallocSizeOf` for `PathBuf` in `malloc_size_of` crate?
impl malloc_size_of::MallocSizeOf for UserScript {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut sum = 0;
        sum += self.script.size_of(ops);
        if let Some(path) = &self.source_file {
            sum += unsafe { ops.malloc_size_of(path.as_path()) };
        }
        sum
    }
}

impl<T: Into<String>> From<T> for UserScript {
    fn from(script: T) -> Self {
        UserScript {
            script: script.into(),
            source_file: None,
        }
    }
}
