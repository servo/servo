/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::{File, IoResult};
use std::io::fs::PathExtensions;
use std::os;
use std::path::Path;


pub fn resources_dir_path() -> Path {
    // FIXME: Find a way to not rely on the executable being under `<servo source>/target`.
    let mut path = os::self_exe_path().expect("can't get exe path");
    path.pop();
    path.push("resources");
    if !path.is_dir() {
        path.pop();
        path.pop();
        path.push("resources");
    }
    path
}


pub fn read_resource_file(relative_path_components: &[&str]) -> IoResult<Vec<u8>> {
    let mut path = resources_dir_path();
    path.push_many(relative_path_components);
    let mut file = try!(File::open(&path));
    file.read_to_end()
}
