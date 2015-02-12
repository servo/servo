/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::old_io::{File, IoResult};
use std::old_path::Path;

#[cfg(not(target_os = "android"))]
use opts;

#[cfg(not(target_os = "android"))]
use std::old_io::fs::PathExtensions;
#[cfg(not(target_os = "android"))]
use std::os;

#[cfg(target_os = "android")]
pub fn resources_dir_path() -> Path {
    Path::new("/sdcard/servo/")
}

#[cfg(not(target_os = "android"))]
pub fn resources_dir_path() -> Path {
    match opts::get().resources_path {
        Some(ref path) => Path::new(path),
        None => {
            // FIXME: Find a way to not rely on the executable being
            // under `<servo source>/components/servo/target`
            // or `<servo source>/components/servo/target/release`.
            let mut path = os::self_exe_path().expect("can't get exe path");
            path.pop();
            path.pop();
            path.pop();
            path.push("resources");
            if !path.is_dir() {  // self_exe_path() is probably in .../target/release
                path.pop();
                path.pop();
                path.push("resources");
            }
            path
        }
    }
}


pub fn read_resource_file(relative_path_components: &[&str]) -> IoResult<Vec<u8>> {
    let mut path = resources_dir_path();
    path.push_many(relative_path_components);
    let mut file = try!(File::open(&path));
    file.read_to_end()
}
