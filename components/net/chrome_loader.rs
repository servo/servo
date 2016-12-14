/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_config::resource_files::resources_dir_path;
use servo_url::ServoUrl;
use std::fs::canonicalize;
use url::percent_encoding::percent_decode;

pub fn resolve_chrome_url(url: &ServoUrl) -> Result<ServoUrl, ()> {
    assert_eq!(url.scheme(), "chrome");
    if url.host_str() != Some("resources") {
        return Err(())
    }
    let resources = canonicalize(resources_dir_path().expect("Error finding resource folder"))
        .expect("Error canonicalizing path to the resources directory");
    let mut path = resources.clone();
    for segment in url.path_segments().unwrap() {
        match percent_decode(segment.as_bytes()).decode_utf8() {
            // Check ".." to prevent access to files outside of the resources directory.
            Ok(segment) => path.push(&*segment),
            _ => return Err(())
        }
    }
    match canonicalize(path) {
        Ok(ref path) if path.starts_with(&resources) && path.exists() => {
            Ok(ServoUrl::from_file_path(path).unwrap())
        }
        _ => Err(())
    }
}
