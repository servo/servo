/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{env, fs, io};

use servo::embedder_traits::resources::{self, Resource};

lazy_static::lazy_static! {
    static ref CMD_RESOURCE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

struct ResourceReader;

pub fn init() {
    resources::set(Box::new(ResourceReader));
}

fn resources_dir_path() -> io::Result<PathBuf> {
    // This needs to be called before the process is sandboxed
    // as we only give permission to read inside the resources directory,
    // not the permissions the "search" for the resources directory.
    let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
    if let Some(ref path) = *dir {
        return Ok(PathBuf::from(path));
    }

    let mut path = env::current_exe()?.parent().unwrap().to_owned();
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

impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let mut path = resources_dir_path().expect("Can't find resources directory");
        path.push(file.filename());
        fs::read(path).expect("Can't read file")
    }
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![resources_dir_path().expect("Can't find resources directory")]
    }
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}
