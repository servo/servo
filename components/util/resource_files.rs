/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

#[cfg(target_os = "android")]
pub fn resources_dir_path() -> PathBuf {
    PathBuf::from("/sdcard/servo/")
}

#[cfg(not(target_os = "android"))]
pub fn resources_dir_path() -> PathBuf {
    use opts;
    use std::env;
    use std::fs::PathExt;

    match opts::get().resources_path {
        Some(ref path) => PathBuf::from(path),
        None => {
            // FIXME: Find a way to not rely on the executable being
            // under `<servo source>/components/servo/target`
            // or `<servo source>/components/servo/target/release`.
            let mut path = env::current_exe().ok().expect("can't get exe path");
            path.pop();
            path.push("resources");
            if !path.is_dir() {   // resources dir not in same dir as exe?
                path.pop();
                path.pop();
                path.pop();
                path.pop();
                path.push("resources");
                if !path.is_dir() {  // self_exe_path() is probably in .../target/release
                    path.pop();
                    path.pop();
                    path.push("resources");
                    if !path.is_dir() { // self_exe_path() is probably in .../target/release
                        path.pop();
                        path.pop();
                        path.push("resources");
                    }
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
