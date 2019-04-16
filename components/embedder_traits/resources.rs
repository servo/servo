/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::{Once, RwLock};

lazy_static! {
    static ref RES: RwLock<Option<Box<ResourceReaderMethods + Sync + Send>>> = RwLock::new(None);
}

pub fn set(reader: Box<ResourceReaderMethods + Sync + Send>) {
    *RES.write().unwrap() = Some(reader);
}

pub fn set_for_tests() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| set(resources_for_tests()));
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
    SSLCertificates,
    BadCertHTML,
    NetErrorHTML,
    UserAgentCSS,
    ServoCSS,
    PresentationalHintsCSS,
    QuirksModeCSS,
    RippyPNG,
    MediaControlsCSS,
    MediaControlsJS,
}

pub trait ResourceReaderMethods {
    fn read(&self, res: Resource) -> Vec<u8>;
    fn sandbox_access_files(&self) -> Vec<PathBuf>;
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf>;
}

fn resources_for_tests() -> Box<ResourceReaderMethods + Sync + Send> {
    use std::env;
    use std::fs::File;
    use std::io::Read;
    struct ResourceReader;
    impl ResourceReaderMethods for ResourceReader {
        fn sandbox_access_files(&self) -> Vec<PathBuf> {
            vec![]
        }
        fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
            vec![]
        }
        fn read(&self, file: Resource) -> Vec<u8> {
            let file = match file {
                Resource::Preferences => "prefs.json",
                Resource::BluetoothBlocklist => "gatt_blocklist.txt",
                Resource::DomainList => "public_domains.txt",
                Resource::HstsPreloadList => "hsts_preload.json",
                Resource::SSLCertificates => "certs",
                Resource::BadCertHTML => "badcert.html",
                Resource::NetErrorHTML => "neterror.html",
                Resource::UserAgentCSS => "user-agent.css",
                Resource::ServoCSS => "servo.css",
                Resource::PresentationalHintsCSS => "presentational-hints.css",
                Resource::QuirksModeCSS => "quirks-mode.css",
                Resource::RippyPNG => "rippy.png",
                Resource::MediaControlsCSS => "media_controls.css",
                Resource::MediaControlsJS => "media_controls.js",
            };
            let mut path = env::current_exe().unwrap();
            path = path.canonicalize().unwrap();
            while path.pop() {
                path.push("resources");
                if path.is_dir() {
                    break;
                }
                path.pop();
            }
            path.push(file);
            let mut buffer = vec![];
            File::open(path)
                .expect(&format!("Can't find file: {}", file))
                .read_to_end(&mut buffer)
                .expect("Can't read file");
            buffer
        }
    }
    Box::new(ResourceReader)
}
