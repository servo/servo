/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::PathBuf;

use servo::embedder_traits::resources::{Resource, ResourceReaderMethods};

pub(crate) struct ResourceReaderInstance;

impl ResourceReaderInstance {
    pub(crate) fn new() -> ResourceReaderInstance {
        ResourceReaderInstance
    }
}

impl ResourceReaderMethods for ResourceReaderInstance {
    fn read(&self, res: Resource) -> Vec<u8> {
        Vec::from(match res {
            Resource::Preferences => &include_bytes!("../../../../resources/prefs.json")[..],
            Resource::HstsPreloadList => {
                &include_bytes!("../../../../resources/hsts_preload.json")[..]
            },
            Resource::BadCertHTML => &include_bytes!("../../../../resources/badcert.html")[..],
            Resource::NetErrorHTML => &include_bytes!("../../../../resources/neterror.html")[..],
            Resource::UserAgentCSS => &include_bytes!("../../../../resources/user-agent.css")[..],
            Resource::ServoCSS => &include_bytes!("../../../../resources/servo.css")[..],
            Resource::PresentationalHintsCSS => {
                &include_bytes!("../../../../resources/presentational-hints.css")[..]
            },
            Resource::QuirksModeCSS => &include_bytes!("../../../../resources/quirks-mode.css")[..],
            Resource::RippyPNG => &include_bytes!("../../../../resources/rippy.png")[..],
            Resource::DomainList => &include_bytes!("../../../../resources/public_domains.txt")[..],
            Resource::BluetoothBlocklist => {
                &include_bytes!("../../../../resources/gatt_blocklist.txt")[..]
            },
            Resource::MediaControlsCSS => {
                &include_bytes!("../../../../resources/media-controls.css")[..]
            },
            Resource::MediaControlsJS => {
                &include_bytes!("../../../../resources/media-controls.js")[..]
            },
            Resource::CrashHTML => &include_bytes!("../../../../resources/crash.html")[..],
            Resource::DirectoryListingHTML => {
                &include_bytes!("../../../../resources/directory-listing.html")[..]
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
