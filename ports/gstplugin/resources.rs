/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// This is a copy of the resource loader from the winit port
// TODO: move this to somewhere where it can be shared.
// https://github.com/servo/servo/issues/24853

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{env, fs};

use servo::embedder_traits::resources::{self, Resource};

lazy_static::lazy_static! {
    static ref CMD_RESOURCE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

struct ResourceReader;

pub fn init() {
    resources::set(Box::new(ResourceReader));
}

fn resources_dir_path() -> PathBuf {
    // This needs to be called before the process is sandboxed
    // as we only give permission to read inside the resources directory,
    // not the permissions the "search" for the resources directory.
    let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
    if let Some(ref path) = *dir {
        return PathBuf::from(path);
    }

    let path = env::current_exe().unwrap();
    let path = path.parent().unwrap();
    let mut path = path.canonicalize().unwrap();
    path.push("resources");
    if path.is_dir() {
        *dir = Some(path);
        return dir.clone().unwrap();
    }

    // Check for Resources on mac when using a case sensitive filesystem.
    path.pop();
    path.push("Resources");
    if path.is_dir() {
        *dir = Some(path);
        return dir.clone().unwrap();
    }

    if cfg!(servo_production) {
        panic!("Can't find resources directory")
    } else {
        // Try to find resources in the current working directory too.
        // Not to be used in production builds without considering the security implications!
        const _: () = assert!(cfg!(servo_do_not_use_in_production));
        let path = Path::new("resources").to_owned();
        *dir = Some(path);
        dir.clone().unwrap()
    }
}

impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let mut path = resources_dir_path();
        path.push(file.filename());
        fs::read(path).expect("Can't read file")
    }
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![resources_dir_path()]
    }
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}
