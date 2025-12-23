/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use malloc_size_of::MallocSizeOfOps;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

static USER_CONTENT_MANAGER_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct UserContentManagerId(u32);

impl UserContentManagerId {
    pub fn next() -> Self {
        Self(USER_CONTENT_MANAGER_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct UserContents {
    pub scripts: Vec<UserScript>,
}

impl UserContents {
    pub fn new() -> Self {
        UserContents::default()
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
