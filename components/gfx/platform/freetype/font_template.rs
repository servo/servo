/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::old_io as io;
use std::old_io::File;

/// Platform specific font representation for Linux.
/// The identifier is an absolute path, and the bytes
/// field is the loaded data that can be passed to
/// freetype and azure directly.
pub struct FontTemplateData {
    pub bytes: Vec<u8>,
    pub identifier: String,
}

impl FontTemplateData {
    pub fn new(identifier: &str, font_data: Option<Vec<u8>>) -> FontTemplateData {
        let bytes = match font_data {
            Some(bytes) => {
                bytes
            },
            None => {
                // TODO: Handle file load failure!
                let mut file = File::open_mode(&Path::new(identifier), io::Open, io::Read).unwrap();
                file.read_to_end().unwrap()
            },
        };

        FontTemplateData {
            bytes: bytes,
            identifier: identifier.to_owned(),
        }
    }
}
