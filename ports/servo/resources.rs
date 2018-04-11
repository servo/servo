/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(target_os = "android")]
use android_injected_glue::load_asset;
use servo::embedder_traits::resources::{self, Resource};
#[cfg(not(target_os = "android"))]
use std::env;
#[cfg(not(target_os = "android"))]
use std::fs::File;
#[cfg(not(target_os = "android"))]
use std::io::{self, Read};
#[cfg(not(target_os = "android"))]
use std::path::PathBuf;

struct ResourceReader;

#[cfg(not(target_os = "android"))]
impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let file = filename(file);
        let mut path = resources_dir_path().expect("Can't find resources directory");
        path.push(file);
        let mut buffer = vec![];
        File::open(path).expect(&format!("Can't find file: {}", file)).read_to_end(&mut buffer).expect("Can't read file");
        buffer
    }
}

#[cfg(target_os = "android")]
impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let file = filename(file);
        load_asset(file).unwrap_or_else(|_| {
            panic!("Can't load asset");
        })
    }
}

pub fn init() {
    resources::set(Box::new(ResourceReader));
}

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
    }
}

#[cfg(not(target_os = "android"))]
fn resources_dir_path() -> io::Result<PathBuf> {
    // FIXME: Find a way to not rely on the executable being
    // under `<servo source>[/$target_triple]/target/debug`
    // or `<servo source>[/$target_triple]/target/release`.
    let mut path = env::current_exe()?;
    // Follow symlink
    path = path.canonicalize()?;

    while path.pop() {
        path.push("resources");
        if path.is_dir() {
            break;
        }
        path.pop();
        // Check for Resources on mac when using a case sensitive filesystem.
        path.push("Resources");
        if path.is_dir() {
            break;
        }
        path.pop();
    }
    Ok(path)
}
