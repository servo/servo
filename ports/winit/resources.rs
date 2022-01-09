/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo::embedder_traits::resources::{self, Resource};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static! {
    static ref CMD_RESOURCE_DIR: Mutex<Option<String>> = Mutex::new(None);
}

struct ResourceReader;

fn filename(file: Resource) -> &'static str {
    match file {
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
        Resource::MediaControlsCSS => "media-controls.css",
        Resource::MediaControlsJS => "media-controls.js",
    }
}

pub fn init() {
    resources::set(Box::new(ResourceReader));
}

fn resources_dir_path() -> io::Result<Option<PathBuf>> {
    // This needs to be called before the process is sandboxed
    // as we only give permission to read inside the resources directory,
    // not the permissions the "search" for the resources directory.
    let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
    if let Some(ref path) = *dir {
        return Ok(Some(PathBuf::from(path)));
    }

    let mut current_env_path = env::current_exe()?;
    // Follow symlink
    current_env_path = current_env_path.canonicalize()?;

    if let Some(path) = resources_dir_scan(current_env_path)? {
        *dir = Some(path.to_str().unwrap().to_owned());
        Ok(Some(path))
    } else if let Some(path) = resources_dir_scan(std::env::current_dir()?)? {
        *dir = Some(path.to_str().unwrap().to_owned());
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

fn resources_dir_scan(mut path: PathBuf) -> io::Result<Option<PathBuf>> {
    path = path.canonicalize()?;

    path.push("resources");
    if path.is_dir() {
        return Ok(Some(path));
    }

    while path.pop() {
        path.push("resources");
        if path.is_dir() {
            return Ok(Some(path));
        }
        path.pop();
        // Check for Resources on mac when using a case sensitive filesystem.
        path.push("Resources");
        if path.is_dir() {
            return Ok(Some(path));
        }
        path.pop();
    }

    Ok(None)
}

impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let file = filename(file);

        let mut path = resources_dir_path()
            .expect("Error when searching for resources directory")
            .expect("Can't find resources directory");

        path.push(file);
        fs::read(path).expect("Can't read file")
    }
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![resources_dir_path()
             .expect("Error when searching for resources directory")
             .expect("Can't find resources directory")]
    }
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}
