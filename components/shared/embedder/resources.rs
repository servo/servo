/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::RwLock;

use cfg_if::cfg_if;
use lazy_static::lazy_static;

lazy_static! {
    static ref RES: RwLock<Option<Box<dyn ResourceReaderMethods + Sync + Send>>> = {
        cfg_if! {
            if #[cfg(servo_production)] {
                RwLock::new(None)
            } else {
                // Static assert that this is really a non-production build, rather
                // than a failure of the build script’s production check.
                const _: () = assert!(cfg!(servo_do_not_use_in_production));

                RwLock::new(Some(resources_for_tests()))
            }
        }
    };
}

pub fn set(reader: Box<dyn ResourceReaderMethods + Sync + Send>) {
    *RES.write().unwrap() = Some(reader);
}

pub fn read_bytes(res: Resource) -> Vec<u8> {
    RES.read()
        .unwrap()
        .as_ref()
        .expect("Resource reader not set.")
        .read(res)
}

pub fn read_string(res: Resource) -> String {
    String::from_utf8(read_bytes(res)).unwrap()
}

pub fn sandbox_access_files() -> Vec<PathBuf> {
    RES.read()
        .unwrap()
        .as_ref()
        .expect("Resource reader not set.")
        .sandbox_access_files()
}

pub fn sandbox_access_files_dirs() -> Vec<PathBuf> {
    RES.read()
        .unwrap()
        .as_ref()
        .expect("Resource reader not set.")
        .sandbox_access_files_dirs()
}

pub enum Resource {
    Preferences,
    BluetoothBlocklist,
    DomainList,
    HstsPreloadList,
    BadCertHTML,
    NetErrorHTML,
    UserAgentCSS,
    ServoCSS,
    PresentationalHintsCSS,
    QuirksModeCSS,
    RippyPNG,
    MediaControlsCSS,
    MediaControlsJS,
    CrashHTML,
    DirectoryListingHTML,
}

impl Resource {
    pub fn filename(&self) -> &'static str {
        match self {
            Resource::Preferences => "prefs.json",
            Resource::BluetoothBlocklist => "gatt_blocklist.txt",
            Resource::DomainList => "public_domains.txt",
            Resource::HstsPreloadList => "hsts_preload.json",
            Resource::BadCertHTML => "badcert.html",
            Resource::NetErrorHTML => "neterror.html",
            Resource::UserAgentCSS => "user-agent.css",
            Resource::ServoCSS => "servo.css",
            Resource::PresentationalHintsCSS => "presentational-hints.css",
            Resource::QuirksModeCSS => "quirks-mode.css",
            Resource::RippyPNG => "rippy.png",
            Resource::MediaControlsCSS => "media-controls.css",
            Resource::MediaControlsJS => "media-controls.js",
            Resource::CrashHTML => "crash.html",
            Resource::DirectoryListingHTML => "directory-listing.html",
        }
    }
}

pub trait ResourceReaderMethods {
    fn read(&self, res: Resource) -> Vec<u8>;
    fn sandbox_access_files(&self) -> Vec<PathBuf>;
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf>;
}

/// Bake all of our resources into this crate for tests, unless we are `cfg!(servo_production)`.
///
/// Local non-production embedder builds (e.g. servoshell) can still override these with [`set`],
/// if runtime loading of prefs.json and other resources is needed.
///
/// In theory this can be `#[cfg(servo_production)]`, but omitting the attribute ensures that the
/// code is always checked by the compiler, even if it later gets optimised out as dead code.
fn resources_for_tests() -> Box<dyn ResourceReaderMethods + Sync + Send> {
    struct ResourceReader;
    impl ResourceReaderMethods for ResourceReader {
        fn sandbox_access_files(&self) -> Vec<PathBuf> {
            vec![]
        }
        fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
            vec![]
        }
        fn read(&self, file: Resource) -> Vec<u8> {
            match file {
                Resource::Preferences => &include_bytes!("../../../resources/prefs.json")[..],
                Resource::BluetoothBlocklist => {
                    &include_bytes!("../../../resources/gatt_blocklist.txt")[..]
                },
                Resource::DomainList => {
                    &include_bytes!("../../../resources/public_domains.txt")[..]
                },
                Resource::HstsPreloadList => {
                    &include_bytes!("../../../resources/hsts_preload.json")[..]
                },
                Resource::BadCertHTML => &include_bytes!("../../../resources/badcert.html")[..],
                Resource::NetErrorHTML => &include_bytes!("../../../resources/neterror.html")[..],
                Resource::UserAgentCSS => &include_bytes!("../../../resources/user-agent.css")[..],
                Resource::ServoCSS => &include_bytes!("../../../resources/servo.css")[..],
                Resource::PresentationalHintsCSS => {
                    &include_bytes!("../../../resources/presentational-hints.css")[..]
                },
                Resource::QuirksModeCSS => {
                    &include_bytes!("../../../resources/quirks-mode.css")[..]
                },
                Resource::RippyPNG => &include_bytes!("../../../resources/rippy.png")[..],
                Resource::MediaControlsCSS => {
                    &include_bytes!("../../../resources/media-controls.css")[..]
                },
                Resource::MediaControlsJS => {
                    &include_bytes!("../../../resources/media-controls.js")[..]
                },
                Resource::CrashHTML => &include_bytes!("../../../resources/crash.html")[..],
                Resource::DirectoryListingHTML => {
                    &include_bytes!("../../../resources/directory-listing.html")[..]
                },
            }
            .to_owned()
        }
    }
    Box::new(ResourceReader)
}
