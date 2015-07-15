/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;
use string_cache::Atom;

/// Platform specific font representation for Linux.
/// The identifier is an absolute path, and the bytes
/// field is the loaded data that can be passed to
/// freetype and azure directly.
pub struct FontTemplateData {
    pub bytes: Vec<u8>,
    pub identifier: Atom,
}

impl FontTemplateData {
    pub fn new(identifier: Atom, font_data: Option<Vec<u8>>) -> FontTemplateData {
        let bytes = match font_data {
            Some(bytes) => {
                bytes
            },
            None => {
                // TODO: Handle file load failure!
                let mut file = File::open(identifier.as_slice()).unwrap();
                let mut buffer = vec![];
                file.read_to_end(&mut buffer).unwrap();
                buffer
            },
        };

        FontTemplateData {
            bytes: bytes,
            identifier: identifier,
        }
    }
}
