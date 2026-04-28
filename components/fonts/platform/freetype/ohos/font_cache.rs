/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::error::Error;
use std::ffi::CStr;
use std::fs::File;
use std::io::Write;
use std::{fs, thread};

use log::error;
use ohos_deviceinfo_sys::OH_GetIncrementalVersion;
use postcard::{from_bytes, to_stdvec};
use servo_config::opts;

use crate::platform::freetype::ohos::font_list::FontList;

/// Checks if the font file has been cached on the disk. If no such file is found,
/// or for whatever reason Servo fails to parse the file path, return false.
/// Additionally, this function also spawns a detached thread to execute `remove_redundant_cache_files`.
pub fn font_file_cached_on_disk() -> bool {
    thread::spawn(remove_redundant_cache_files); // do clean up of the directory

    match parse_file_path() {
        Ok(file_path) => {
            let Ok(res) = fs::exists(file_path) else {
                return false;
            };
            return res;
        },
        Err(e) => {
            error!(
                "Servo fails to parse the file path of the OHOS FontList cache file (if exists). Error message: {:?}",
                e
            );
            return false;
        },
    }
}

/// This is a wrapper to `serialize_and_write_to_disk_wrapper`. The reason this wrapper is used is because
/// serialization will be performed on a separate detached thread, and so the spawning thread won't be receiving the result.
/// Instead, this wrapper will receive the result and log an error, if it exists.
pub fn serialize_and_write_to_disk_wrapper(input_data: FontList) {
    if let Err(e) = serialize_and_write_to_disk(input_data) {
        error!(
            "Servo fails to serialize font list to disk. Error message: {:?}",
            e
        );
    }
}

/// Reads the OHOS FontList cache file. Returns a `Result` so the caller can know if this function fails
/// and that the caller needs to find another way to get the FontList.
pub fn read_from_disk() -> Result<FontList, Box<dyn Error>> {
    let file_path = parse_file_path()?;
    let data = fs::read(file_path)?;

    let font_list: FontList = from_bytes(&data)?;
    Ok(font_list)
}

/// Traverses the directory where the font cache file is stored and removes redundant font cache files.
/// A font cache file becomes redundant when there is an OS update (because the system fonts may be updated as well).
fn remove_redundant_cache_files() {
    let base_dir = get_directory().unwrap();
    let expected_cache_filename = parse_filename().unwrap();
    let cache_filename_components: Vec<&str> = expected_cache_filename.split('_').collect();

    if let Ok(entries) = fs::read_dir(&base_dir) {
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(filename) = entry.file_name().into_string() else {
                continue;
            };
            let filename_components: Vec<&str> = filename.split('_').collect();

            // check if file is font cache file, and whether or not it is outdated.
            if (filename_components.len() == 2) && // the font cache file only has one `_`. So the vector length from splitting must be 2.
                        (filename_components[1] == cache_filename_components[1]) && // check if the suffix is the same
                        (filename_components[0] != cache_filename_components[0])
            {
                if let Err(e) = fs::remove_file(format!("{}{}", &base_dir, filename).to_string()) {
                    error!(
                        "Obsolete font cache file found; but Servo fails to remove it. Error : {:?}",
                        e
                    );
                };
            }
        }
    }
}

/// Helper function to parse the filepath of the cache file.
fn parse_file_path() -> Result<String, Box<dyn Error>> {
    let base_dir = get_directory()?;
    let cache_filename = parse_filename()?;

    Ok(format!("{}{}", base_dir, cache_filename).to_string())
}

/// Helper function to obtain the path to the directory where we'll eventually store our cache file in.
fn get_directory() -> Result<String, Box<dyn Error>> {
    let binding = opts::get().config_dir.clone().unwrap();
    let base_dir = binding
        .to_str()
        .ok_or("Failed to parse base directory's path")?;

    Ok(base_dir.to_string())
}

/// Helper function to parse the filename.
/// Currently, the naming format is <OS_VERSION>_font-cache.bin"
fn parse_filename() -> Result<String, Box<dyn Error>> {
    let os_version = unsafe {
        let os_version_c_str = CStr::from_ptr(OH_GetIncrementalVersion());
        os_version_c_str.to_str()?
    };

    Ok(format!("{}{}", os_version, "_font-cache.bin").to_string())
}

/// This function serializes `FontList` and caches its result into disk.
fn serialize_and_write_to_disk(input_data: FontList) -> Result<(), Box<dyn Error>> {
    let serialized_data = to_stdvec(&input_data).unwrap();

    let file_path = parse_file_path()?;

    let mut file = File::create(file_path)?;
    file.write_all(serialized_data.as_slice())?;
    Ok(())
}
