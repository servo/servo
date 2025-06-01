/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::PathBuf;

use servo::resources::{Resource, ResourceReaderMethods};

pub(crate) struct ResourceReaderInstance;

impl ResourceReaderInstance {
    pub(crate) fn new() -> ResourceReaderInstance {
        ResourceReaderInstance
    }
}

impl ResourceReaderMethods for ResourceReaderInstance {
    fn read(&self, res: Resource) -> Vec<u8> {
        Vec::from(match res {
            Resource::HstsPreloadList => {
                &include_bytes!("../../../../resources/hsts_preload.fstmap")[..]
            },
            Resource::BadCertHTML => &include_bytes!("../../../../resources/badcert.html")[..],
            Resource::NetErrorHTML => &include_bytes!("../../../../resources/neterror.html")[..],
            Resource::RippyPNG => &include_bytes!("../../../../resources/rippy.png")[..],
            Resource::DomainList => &include_bytes!("../../../../resources/public_domains.txt")[..],
            Resource::BluetoothBlocklist => {
                &include_bytes!("../../../../resources/gatt_blocklist.txt")[..]
            },
            Resource::CrashHTML => &include_bytes!("../../../../resources/crash.html")[..],
            Resource::DirectoryListingHTML => {
                &include_bytes!("../../../../resources/directory-listing.html")[..]
            },
            Resource::AboutMemoryHTML => {
                &include_bytes!("../../../../resources/about-memory.html")[..]
            },
        })
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
}
