/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use malloc_size_of::MallocSizeOfOps;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
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

    pub fn scripts(&self) -> &[UserScript] {
        &self.user_scripts
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
