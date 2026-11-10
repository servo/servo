/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fs::File;
use std::path::PathBuf;
use std::sync::OnceLock;

use servo::resources::{Resource, ResourceBytes, ResourceReaderMethods};

pub(crate) struct ResourceReaderImpl {
    resource_dir: OnceLock<PathBuf>,
}

static RESOURCE_READER: ResourceReaderImpl = ResourceReaderImpl {
    resource_dir: OnceLock::new(),
};

servo::submit_resource_reader!(&RESOURCE_READER);

pub(crate) fn set_resource_dir(resource_dir: PathBuf) {
    if let Err(e) = RESOURCE_READER.resource_dir.set(resource_dir) {
        log::warn!("Failed to set resource dir: {:?}", e);
    }
}

impl ResourceReaderMethods for ResourceReaderImpl {
    fn read(&self, res: Resource) -> ResourceBytes {
        let file_path = RESOURCE_READER
            .resource_dir
            .get()
            .expect("Attempted to read resources before reader initialized")
            .join("named_resources")
            .join(res.filename());
        let file = File::open(file_path).expect("Could not read file");
        // Safety: The resource directory on ohos is read-only, so it's safe to mmap
        // the file, since the contents can't change.
        let mmap = unsafe { memmap2::Mmap::map(&file).expect("Could not memory map resource") };
        ResourceBytes::from_owner(mmap)
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
}
