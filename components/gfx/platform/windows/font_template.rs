/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use servo_atoms::Atom;
use serde;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::io;
use std::ops::Deref;
use std::sync::Mutex;
use webrender_traits::NativeFontHandle;
use dwrote::{Font};
use platform::windows::font_list::{descriptor_from_atom, font_from_atom};

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
        if self.bytes.is_some() {
            panic!("Can't create fonts yet");
        }

        let descriptor = descriptor_from_atom(&self.identifier);
        Some(descriptor)
    }
}

#[derive(Debug)]
pub struct CachedDWFont(Mutex<HashMap<Au, Font>>);

impl Deref for CachedDWFont {
    type Target = Mutex<HashMap<Au, Font>>;
    fn deref(&self) -> &Mutex<HashMap<Au, Font>> {
        &self.0
    }
}

impl Serialize for CachedDWFont {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_none()
    }
}

impl Deserialize for CachedDWFont {
    fn deserialize<D>(deserializer: &mut D) -> Result<CachedDWFont, D::Error>
                      where D: Deserializer {
        struct NoneOptionVisitor;

        impl serde::de::Visitor for NoneOptionVisitor {
            type Value = CachedDWFont;

            #[inline]
            fn visit_none<E>(&mut self) -> Result<CachedDWFont, E> where E: serde::de::Error {
                Ok(CachedDWFont(Mutex::new(HashMap::new())))
            }
        }

        deserializer.deserialize_option(NoneOptionVisitor)
    }
}
