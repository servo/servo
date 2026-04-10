/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::ffi::CStr;
use std::fs;
use std::fs::File;
use std::io::Write;

use log::error;
use ohos_deviceinfo_sys::OH_GetBuildRootHash;
use serde_json;

use crate::platform::freetype::ohos::font_list::FontList;

/// When testing the ohos font code on linux, we can pass the fonts directory of the SDK
/// via an environment variable.
#[cfg(ohos_mock)]
static OHOS_FONTS_DIR: &str = env!("OHOS_SDK_FONTS_DIR");

/// On OpenHarmony devices the fonts are always located here.
#[cfg(not(ohos_mock))]
static OHOS_FONTS_DIR: &str = "/system/fonts/";

/// Checks if the font file has been cached on the disk. If no such file is found,
/// or for whatever reason Servo fails to parse the file path, return false.
pub fn font_file_cached_on_disk() -> bool {
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

/// This function serializes `FontList` and caches its result into disk.
pub fn serialize_and_write_to_disk(input_data: FontList) -> Result<(), Box<dyn std::error::Error>> {
    let serialized_data = serde_json::to_string(&input_data).unwrap();

    let file_path = parse_file_path()?;

    let mut file = File::create(file_path)?;
    file.write_all(serialized_data.as_bytes())?;
    Ok(())
}

/// Reads the OHOS FontList cache file. Returns a `Result` so the caller can know if this function fails
/// and that the caller needs to find another way to get the FontList.
pub fn read_from_disk() -> Result<FontList, Box<dyn std::error::Error>> {
    let file_path = parse_file_path()?;
    let data = fs::read_to_string(file_path)?;

    let font_list: FontList = serde_json::from_str(&data).unwrap();
    Ok(font_list)
}

/// Helper function to parse the filepath of the cache file.
/// The format is /path/to/directory/font_cache_<OS_VERSION>.json
fn parse_file_path() -> Result<String, Box<dyn std::error::Error>> {
    let os_version = unsafe {
        let os_version_c_str = CStr::from_ptr(OH_GetBuildRootHash());
        os_version_c_str.to_str()?
    };

    Ok(format!(
        "{}{}{}{}",
        OHOS_FONTS_DIR, "font_cache_", os_version, ".json"
    )
    .to_string())
}
