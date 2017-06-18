/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_atoms::Atom;
use std::fs::File;
use std::io::{Read, Error};
use webrender_traits::NativeFontHandle;

/// Platform specific font representation for Linux.
/// The identifier is an absolute path, and the bytes
/// field is the loaded data that can be passed to
/// freetype and azure directly.
#[derive(Deserialize, Serialize, Debug)]
pub struct FontTemplateData {
    pub bytes: Vec<u8>,
    pub identifier: Atom,
}

impl FontTemplateData {
    pub fn new(identifier: Atom, font_data: Option<Vec<u8>>) -> Result<FontTemplateData, Error> {
        let bytes = match font_data {
            Some(bytes) => {
                bytes
            },
            None => {
                // TODO: Handle file load failure!
                let mut file = File::open(&*identifier)?;
                let mut buffer = vec![];
                file.read_to_end(&mut buffer).unwrap();
                buffer
            },
        };

        Ok(FontTemplateData {
            bytes: bytes,
            identifier: identifier,
        })
    }

    /// Returns a clone of the data in this font. This may be a hugely expensive
    /// operation (depending on the platform) which performs synchronous disk I/O
    /// and should never be done lightly.
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Returns a clone of the bytes in this font if they are in memory. This function never
    /// performs disk I/O.
    pub fn bytes_if_in_memory(&self) -> Option<Vec<u8>> {
        Some(self.bytes())
    }

    /// Returns the native font that underlies this font template, if applicable.
    pub fn native_font(&self) -> Option<NativeFontHandle> {
        None
    }
}
