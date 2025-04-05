/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::{File, create_dir_all};
use std::io::{Error, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use servo_url::ServoUrl;
use tempfile::NamedTempFile;
use uuid::Uuid;

use crate::dom::bindings::str::DOMString;

pub(crate) trait ScriptSource {
    fn unminified_dir(&self) -> Option<String>;
    fn extract_bytes(&self) -> &[u8];
    fn rewrite_source(&mut self, source: Rc<DOMString>);
    fn url(&self) -> ServoUrl;
    fn is_external(&self) -> bool;
}

pub(crate) fn create_temp_files() -> Option<(NamedTempFile, File)> {
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
pub(crate) enum BeautifyFileType {
    Css,
    Js,
}

pub(crate) fn execute_js_beautify(input: &Path, output: File, file_type: BeautifyFileType) -> bool {
    let mut cmd = Command::new("js-beautify");
    match file_type {
        BeautifyFileType::Js => (),
        BeautifyFileType::Css => {
            cmd.arg("--type").arg("css");
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

pub(crate) fn create_output_file(
    unminified_dir: String,
    url: &ServoUrl,
    external: Option<bool>,
) -> Result<File, Error> {
    let path = PathBuf::from(unminified_dir);

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

    File::create(path)
}

pub(crate) fn unminify_js(script: &mut dyn ScriptSource) {
    let Some(unminified_dir) = script.unminified_dir() else {
        return;
    };

    if let Some((mut input, mut output)) = create_temp_files() {
        input.write_all(script.extract_bytes()).unwrap();

        if execute_js_beautify(
            input.path(),
            output.try_clone().unwrap(),
            BeautifyFileType::Js,
        ) {
            let mut script_content = String::new();
            output.seek(std::io::SeekFrom::Start(0)).unwrap();
            output.read_to_string(&mut script_content).unwrap();
            script.rewrite_source(Rc::new(DOMString::from(script_content)));
        }
    }

    match create_output_file(unminified_dir, &script.url(), Some(script.is_external())) {
        Ok(mut file) => file.write_all(script.extract_bytes()).unwrap(),
        Err(why) => warn!("Could not store script {:?}", why),
    }
}

pub(crate) fn unminified_path(dir: &str) -> String {
    let mut path = env::current_dir().unwrap();
    path.push(dir);
    path.into_os_string().into_string().unwrap()
}
