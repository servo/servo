/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref CMD_RESOURCE_DIR: Arc<Mutex<Option<String>>> = {
        Arc::new(Mutex::new(None))
    };
}

pub fn set_resources_path(path: String) {
    let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
    *dir = Some(path);
}

pub fn resources_dir_path() -> io::Result<PathBuf> {
    match *CMD_RESOURCE_DIR.lock().unwrap() {
        Some(ref path) => Ok(PathBuf::from(path)),
        None => Err(io::Error::new(io::ErrorKind::Other, "Resources path not initialized")),
    }
}

pub fn read_resource_file<P: AsRef<Path>>(relative_path: P) -> io::Result<Vec<u8>> {
    let mut path = resources_dir_path()?;
    path.push(relative_path);
    let mut file = File::open(&path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}
