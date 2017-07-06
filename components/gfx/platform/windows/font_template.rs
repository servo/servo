/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::windows::font_list::{descriptor_from_atom, font_from_atom};
use servo_atoms::Atom;
use std::io;
use webrender_api::NativeFontHandle;

#[derive(Deserialize, Serialize, Debug)]
pub struct FontTemplateData {
    pub bytes: Option<Vec<u8>>,
    pub identifier: Atom,
}

impl FontTemplateData {
    pub fn new(identifier: Atom,
               font_data: Option<Vec<u8>>) -> Result<FontTemplateData, io::Error> {
        Ok(FontTemplateData {
            bytes: font_data,
            identifier: identifier,
        })
    }

    pub fn bytes(&self) -> Vec<u8> {
        if self.bytes.is_some() {
            self.bytes.as_ref().unwrap().clone()
        } else {
            let font = font_from_atom(&self.identifier);
            let face = font.create_font_face();
            let files = face.get_files();
            assert!(files.len() > 0);

            files[0].get_font_file_bytes()
        }
    }

    pub fn bytes_if_in_memory(&self) -> Option<Vec<u8>> {
        self.bytes.clone()
    }

    pub fn native_font(&self) -> Option<NativeFontHandle> {
        if self.bytes.is_none() {
            Some(descriptor_from_atom(&self.identifier))
        } else {
            None
        }
    }
}
