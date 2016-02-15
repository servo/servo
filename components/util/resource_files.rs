/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref CMD_RESOURCE_DIR: Arc<Mutex<Option<String>>> = {
        Arc::new(Mutex::new(None))
    };
}

pub fn set_resources_path(path: Option<String>) {
    let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
    *dir = path;
}

#[cfg(target_os = "android")]
pub fn resources_dir_path() -> PathBuf {
    PathBuf::from("/sdcard/servo/")
}

#[cfg(not(target_os = "android"))]
pub fn resources_dir_path() -> PathBuf {
    use std::env;

    match *CMD_RESOURCE_DIR.lock().unwrap() {
        Some(ref path) => PathBuf::from(path),
        None => {
            // FIXME: Find a way to not rely on the executable being
            // under `<servo source>[/$target_triple]/target/debug`
            // or `<servo source>[/$target_triple]/target/release`.
            let mut path = env::current_exe().expect("can't get exe path");

            // Follow symlink until it's not a link anymore.
            // Example:
            // /usr/local/bin/servo links to ../opt/servo/bin/servo which links to ../servo
            loop {
                match path.read_link() {
                    Ok(resolved_path) => {
                        // Remove file name (./servo)
                        path.pop();
                        // expand with the resolved path, which includes the file name (./servo)
                        path.push(resolved_path);
                    },
                    _ => break,
                }
            }

            path.pop();
            path.push("resources");
            if !path.is_dir() {   // resources dir not in same dir as exe?
                // exe is probably in target/{debug,release} so we need to go back to topdir
                path.pop();
                path.pop();
                path.pop();
                path.push("resources");
                if !path.is_dir() {
                    // exe is probably in target/$target_triple/{debug,release} so go back one more
                    path.pop();
                    path.pop();
                    path.push("resources");
                }
            }
            path
        }
    }
}

pub fn read_resource_file(relative_path_components: &[&str]) -> io::Result<Vec<u8>> {
    let mut path = resources_dir_path();
    for component in relative_path_components {
        path.push(component);
    }
    let mut file = try!(File::open(&path));
    let mut data = Vec::new();
    try!(file.read_to_end(&mut data));
    Ok(data)
}
