/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::{create_dir_all, File};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

use servo_url::ServoUrl;
use tempfile::NamedTempFile;
use uuid::Uuid;

pub fn create_temp_files() -> Option<(NamedTempFile, File)> {
    // Write the minified code to a temporary file and pass its path as an argument
    // to js-beautify to read from. Meanwhile, redirect the process' stdout into
    // another temporary file and read that into a string. This avoids some hangs
    // observed on macOS when using direct input/output pipes with very large
    // unminified content.
    let (input, output) = (NamedTempFile::new(), tempfile::tempfile());
    if let (Ok(input), Ok(output)) = (input, output) {
        Some((input, output))
    } else {
        log::warn!("Error creating input and output temp files");
        None
    }
}

#[derive(Debug)]
pub enum BeautifyFileType {
    Css,
    Js,
}

pub fn execute_js_beautify(input: &Path, output: File, file_type: BeautifyFileType) -> bool {
    let mut cmd = Command::new("npx");
    match file_type {
        BeautifyFileType::Js => (),
        BeautifyFileType::Css => {
            cmd.arg("js-beautify").arg("--type").arg("css");
        },
    }
    match cmd.arg(input).stdout(output).status() {
        Ok(status) => status.success(),
        _ => {
            log::warn!(
                "Failed to execute js-beautify --type {:?}, Will store unmodified script",
                file_type
            );
            false
        },
    }
}

pub fn create_output_file(
    unminified_dir: Option<String>,
    url: &ServoUrl,
    external: Option<bool>,
) -> Result<File, Error> {
    let path = match unminified_dir {
        Some(unminified_dir) => PathBuf::from(unminified_dir),
        None => {
            warn!("Unminified file directory not found");
            return Err(Error::new(
                ErrorKind::NotFound,
                "Unminified file directory not found",
            ));
        },
    };

    let (base, has_name) = match url.as_str().ends_with('/') {
        true => (
            path.join(&url[url::Position::BeforeHost..])
                .as_path()
                .to_owned(),
            false,
        ),
        false => (
            path.join(&url[url::Position::BeforeHost..])
                .parent()
                .unwrap()
                .to_owned(),
            true,
        ),
    };

    create_dir_all(&base)?;

    let path = if external.unwrap_or(true) && has_name {
        // External.
        path.join(&url[url::Position::BeforeHost..])
    } else {
        // Inline file or url ends with '/'
        base.join(Uuid::new_v4().to_string())
    };

    debug!("Unminified files will be stored in {:?}", path);

    Ok(File::create(path)?)
}
