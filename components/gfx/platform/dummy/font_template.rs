/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use string_cache::Atom;
use webrender_traits::NativeFontHandle;

#[derive(Deserialize, Serialize, Debug)]
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
                unimplemented!()
            }
        };

        FontTemplateData {
            bytes: bytes,
            identifier: identifier,
        }
    }
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    pub fn bytes_if_in_memory(&self) -> Option<Vec<u8>> {
        Some(self.bytes())
    }
    pub fn native_font(&self) -> Option<NativeFontHandle> {
        None
    }
}
