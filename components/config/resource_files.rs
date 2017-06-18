/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(target_os = "android")]
use android_injected_glue;
#[cfg(not(target_os = "android"))]
use std::env;
#[cfg(target_os = "android")]
use std::ffi::CStr;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
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
#[allow(unsafe_code)]
pub fn resources_dir_path() -> io::Result<PathBuf> {
    let dir = unsafe {
        CStr::from_ptr((*android_injected_glue::get_app().activity).externalDataPath)
    };
    Ok(PathBuf::from(dir.to_str().unwrap()))
}

#[cfg(not(target_os = "android"))]
pub fn resources_dir_path() -> io::Result<PathBuf> {
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

pub fn read_resource_file<P: AsRef<Path>>(relative_path: P) -> io::Result<Vec<u8>> {
    let mut path = resources_dir_path()?;
    path.push(relative_path);
    let mut file = File::open(&path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}
