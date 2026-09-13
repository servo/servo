/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::error::Error;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::{fs, thread};

use log::error;
use postcard::{from_io, to_io};
use servo_config::opts;

use crate::platform::freetype::ohos::font_list::FontList;

const CACHE_FILENAME_SUFFIX: &str = "_font-cache.bin";

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
            res
        },
        Err(e) => {
            error!(
                "Failed to parse the file path of the OHOS FontList cache file: {:?}",
                e
            );
            false
        },
    }
}

/// This is a wrapper to `serialize_and_write_to_disk_wrapper`. The reason this wrapper is used is because
/// serialization will be performed on a separate detached thread, and so the spawning thread won't be receiving the result.
/// Instead, this wrapper will receive the result and log an error, if it exists.
pub fn serialize_and_write_to_disk_wrapper(input_data: FontList) {
    if let Err(e) = serialize_and_write_to_disk(&input_data) {
        error!("Failed to serialize font list to disk: {:?}", e);
    }
}

/// Reads the OHOS FontList cache file. Returns a `Result` so the caller can know if this function fails
/// and that the caller needs to find another way to get the FontList.
pub fn read_from_disk() -> Result<FontList, Box<dyn Error>> {
    let file_path = parse_file_path()?;
    let mut file_handler = File::open(file_path)?;
    let mut buffer = [0u8; 1024]; // NB: This simply means that the deserializer will be deserializing 1024B of data at a time.

    let (font_list, (_, _)) = from_io((&mut file_handler, &mut buffer))?;

    Ok(font_list)
}

/// Traverses the directory where the font cache file is stored and removes redundant font cache files.
/// A font cache file becomes redundant when there is an OS update (because the system fonts may be updated as well).
fn remove_redundant_cache_files() {
    let base_dir = match get_directory() {
        Ok(dir) => dir,
        Err(error) => {
            log::debug!(
                "Couldn't determine font cache directory: {:?}. Skipping cleanup",
                error
            );
            return;
        },
    };
    let Ok(expected_cache_filename) = parse_filename() else {
        log::debug!("Could not determine font cache filename: {:?} Skipping cleanup");
        return;
    };

    if let Ok(entries) = fs::read_dir(&base_dir) {
        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let filename = entry.file_name();

            // A cache file with a mismatching prefix is obsolete.
            if filename
                .as_bytes()
                .ends_with(CACHE_FILENAME_SUFFIX.as_bytes()) &&
                filename.as_bytes() != expected_cache_filename.as_bytes() &&
                let Err(e) = fs::remove_file(entry.path())
            {
                error!(
                    "Obsolete font cache file found; but failed to remove it: {:?}",
                    e
                );
            };
        }
    }
}

/// Helper function to parse the filepath of the cache file.
fn parse_file_path() -> Result<PathBuf, Box<dyn Error>> {
    let base_dir = get_directory()?;
    let cache_filename = parse_filename()?;

    Ok(base_dir.join(cache_filename))
}

/// Helper function to obtain the path to the directory where we'll eventually store our cache file in.
fn get_directory() -> Result<PathBuf, Box<dyn Error>> {
    let base_dir = opts::get()
        .config_dir
        .clone()
        .ok_or("Failed to get config dir")?;

    Ok(base_dir)
}

/// Helper function to parse the filename.
/// Currently, the naming format is <OS_VERSION>_font-cache.bin"
fn parse_filename() -> Result<String, Box<dyn Error>> {
    let filename = ohos_deviceinfo::get_incremental_version()
        .map(|os_version| [os_version, CACHE_FILENAME_SUFFIX].concat())
        .ok_or("OH_get_incremental_version failed")?;
    Ok(filename)
}

/// This function serializes `FontList` and caches its result into disk.
fn serialize_and_write_to_disk(input_data: &FontList) -> Result<(), Box<dyn Error>> {
    let file_path = parse_file_path()?;

    let file = File::create(file_path)?;
    to_io(input_data, &file)?;
    Ok(())
}
