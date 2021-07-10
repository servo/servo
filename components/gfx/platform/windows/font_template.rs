/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::platform::windows::font_list::font_from_atom;
use servo_atoms::Atom;
use std::fmt;
use std::io;
use webrender_api::NativeFontHandle;

#[derive(Deserialize, Serialize)]
pub struct FontTemplateData {
    // If you add members here, review the Debug impl below
    pub bytes: Option<Vec<u8>>,
    pub identifier: Atom,
}

impl fmt::Debug for FontTemplateData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("FontTemplateData")
            .field(
                "bytes",
                &self
                    .bytes
                    .as_ref()
                    .map(|bytes| format!("[{} bytes]", bytes.len())),
            )
            .field("identifier", &self.identifier)
            .finish()
    }
}

impl FontTemplateData {
    pub fn new(
        identifier: Atom,
        font_data: Option<Vec<u8>>,
    ) -> Result<FontTemplateData, io::Error> {
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
        if self.bytes.is_some() {
            return None;
        }
        let font = font_from_atom(&self.identifier);
        let face = font.create_font_face();
        let files = face.get_files();
        let path = files.iter().next()?.get_font_file_path()?;
        Some(NativeFontHandle {
            path,
            index: face.get_index(),
        })
    }
}
