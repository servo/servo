/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::Mutex;
use std::{env, fs};

use cfg_if::cfg_if;
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

    // Try ./resources and ./Resources relative to the directory containing the
    // canonicalised executable path, then each of its ancestors.
    let mut path = env::current_exe().unwrap().canonicalize().unwrap();
    while path.pop() {
        path.push("resources");
        if path.is_dir() {
            *dir = Some(path);
            return dir.clone().unwrap();
        }
        path.pop();

        // Check for Resources on mac when using a case sensitive filesystem.
        path.push("Resources");
        if path.is_dir() {
            *dir = Some(path);
            return dir.clone().unwrap();
        }
        path.pop();
    }

    cfg_if! {
        if #[cfg(servo_production)] {
            panic!("Can't find resources directory")
        } else {
            // Static assert that this is really a non-production build, rather
            // than a failure of the build script’s production check.
            const _: () = assert!(cfg!(servo_do_not_use_in_production));

            // Try ./resources in the current directory, then each of its ancestors.
            // Not to be used in production builds without considering the security implications!
            let mut path = std::env::current_dir().unwrap();
            loop {
                path.push("resources");
                if path.is_dir() {
                    *dir = Some(path);
                    return dir.clone().unwrap();
                }
                path.pop();

                if !path.pop() {
                    panic!("Can't find resources directory")
                }
            }
        }
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
