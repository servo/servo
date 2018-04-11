/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::embedder_traits::resources::{self, Resource};

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
    }
}

pub fn init() {
    resources::set(Box::new(ResourceReader));
}

#[cfg(not(target_os = "android"))]
mod not_android {
    use servo::embedder_traits::resources::{self, Resource};
    use std::env;
    use std::fs::File;
    use std::io::{self, Read};
    use std::path::PathBuf;
    use std::sync::Mutex;

    lazy_static! {
        static ref CMD_RESOURCE_DIR: Mutex<Option<String>> = Mutex::new(None);
    }

    impl resources::ResourceReaderMethods for super::ResourceReader {
        fn read(&self, file: Resource) -> Vec<u8> {
            let file = super::filename(file);
            let mut path = resources_dir_path().expect("Can't find resources directory");
            path.push(file);
            let mut buffer = vec![];
            File::open(path).expect(&format!("Can't find file: {}", file))
                .read_to_end(&mut buffer).expect("Can't read file");
            buffer
        }
        fn sandbox_access_files_rec(&self) -> Vec<PathBuf> {
            vec![resources_dir_path().expect("Can't find resources directory")]
        }
        fn sandbox_access_files(&self) -> Vec<PathBuf> {
            vec![]
        }
    }

    fn resources_dir_path() -> io::Result<PathBuf> {
        // This needs to be called before the process is sandboxed
        // as we only give permission to read inside the resources directory,
        // not the permissions the "search" for the resources directory.
        let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
        if let Some(ref path) = *dir {
            return Ok(PathBuf::from(path));
        }

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
        *dir = Some(path.to_str().unwrap().to_owned());
        Ok(path)
    }
}

#[cfg(target_os = "android")]
mod android {
    use android_injected_glue::load_asset;
    use servo::embedder_traits::resources::{self, Resource};
    use std::path::PathBuf;

    impl resources::ResourceReaderMethods for super::ResourceReader {
        fn read(&self, file: Resource) -> Vec<u8> {
            let file = super::filename(file);
            load_asset(file).unwrap_or_else(|_| {
                panic!("Can't load asset");
            })
        }
        fn sandbox_access_files_rec(&self) -> Vec<PathBuf> {
            vec![]
        }
        fn sandbox_access_files(&self) -> Vec<PathBuf> {
            vec![]
        }
    }
}
