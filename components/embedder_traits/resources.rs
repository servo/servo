/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::sync::{Once, RwLock};

use lazy_static::lazy_static;

lazy_static! {
    static ref RES: RwLock<Option<Box<dyn ResourceReaderMethods + Sync + Send>>> =
        RwLock::new(None);
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

pub fn filename(file: Resource) -> &'static str {
    match file {
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
    }
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
}

pub trait ResourceReaderMethods {
    fn read(&self, res: Resource) -> Vec<u8>;
    fn sandbox_access_files(&self) -> Vec<PathBuf>;
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf>;
}

// Can’t #[cfg(test)] the following because it breaks tests in dependent crates.

pub fn set_for_tests() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| set(resources_for_tests()));
}

lazy_static::lazy_static! {
    static ref CMD_RESOURCE_DIR: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);
}

fn resources_dir_path_for_tests() -> std::io::Result<PathBuf> {
    // This needs to be called before the process is sandboxed
    // as we only give permission to read inside the resources directory,
    // not the permissions the "search" for the resources directory.
    let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
    if let Some(ref path) = *dir {
        return Ok(PathBuf::from(path));
    }

    let mut path = std::env::current_exe()?.parent().unwrap().to_owned();
    path.push("resources");
    if path.is_dir() {
        *dir = Some(path);
        return Ok(dir.clone().unwrap());
    }

    // Check for Resources on mac when using a case sensitive filesystem.
    path.pop();
    path.push("Resources");
    if path.is_dir() {
        *dir = Some(path);
        return Ok(dir.clone().unwrap());
    }

    let path = Path::new("resources").to_owned();
    *dir = Some(path);
    Ok(dir.clone().unwrap())
}

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
            let file = filename(file);
            let mut path = resources_dir_path_for_tests().expect("Can't find resources directory");
            path.push(file);
            std::fs::read(path).expect("Can't read file")
        }
    }
    Box::new(ResourceReader)
}
