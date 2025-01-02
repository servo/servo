/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fs;
use std::path::PathBuf;

use servo::embedder_traits::resources::{Resource, ResourceReaderMethods};

pub(crate) struct ResourceReaderInstance {
    resource_dir: PathBuf,
}

impl ResourceReaderInstance {
    pub(crate) fn new(resource_dir: PathBuf) -> Self {
        assert!(resource_dir.is_dir());
        Self { resource_dir }
    }
}

impl ResourceReaderMethods for ResourceReaderInstance {
    fn read(&self, res: Resource) -> Vec<u8> {
        let file_path = self.resource_dir.join(res.filename());
        fs::read(&file_path).expect("failed to read resource file")
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
}
