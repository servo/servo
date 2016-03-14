/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core_graphics::data_provider::CGDataProvider;
use core_graphics::font::CGFont;
use core_text;
use core_text::font::CTFont;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::ToOwned;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::sync::Mutex;
use string_cache::Atom;
use url::Url;

/// Platform specific font representation for mac.
/// The identifier is a PostScript font name. The
/// CTFont object is cached here for use by the
/// paint functions that create CGFont references.
#[derive(Deserialize, Serialize, Debug)]
pub struct FontTemplateData {
    /// The `CTFont` object, if present. This is cached here so that we don't have to keep creating
    /// `CTFont` instances over and over. It can always be recreated from the `identifier` and/or
    /// `font_data` fields.
    ///
    /// When sending a `FontTemplateData` instance across processes, this will be set to `None` on
    /// the other side, because `CTFont` instances cannot be sent across processes. This is
    /// harmless, however, because it can always be recreated.
    ctfont: CachedCTFont,

    pub identifier: Atom,
    pub font_data: Option<Vec<u8>>
}

unsafe impl Send for FontTemplateData {}
unsafe impl Sync for FontTemplateData {}

impl FontTemplateData {
    pub fn new(identifier: Atom, font_data: Option<Vec<u8>>) -> FontTemplateData {
        FontTemplateData {
            ctfont: CachedCTFont(Mutex::new(None)),
            identifier: identifier.to_owned(),
            font_data: font_data
        }
    }

    /// Retrieves the Core Text font instance, instantiating it if necessary.
    pub fn ctfont(&self) -> Option<CTFont> {
        let mut ctfont = self.ctfont.lock().unwrap();
        if ctfont.is_none() {
            *ctfont = match self.font_data {
                Some(ref bytes) => {
                    let fontprov = CGDataProvider::from_buffer(bytes);
                    let cgfont_result = CGFont::from_data_provider(fontprov);
                    match cgfont_result {
                        Ok(cgfont) => Some(core_text::font::new_from_CGFont(&cgfont, 0.0)),
                        Err(_) => None
                    }
                }
                None => core_text::font::new_from_name(&*self.identifier, 0.0).ok(),
            }
        }
        ctfont.as_ref().map(|ctfont| (*ctfont).clone())
    }

    /// Returns a clone of the data in this font. This may be a hugely expensive
    /// operation (depending on the platform) which performs synchronous disk I/O
    /// and should never be done lightly.
    pub fn bytes(&self) -> Vec<u8> {
        match self.bytes_if_in_memory() {
            Some(font_data) => return font_data,
            None => {}
        }

        let path = Url::parse(&*self.ctfont()
                                    .expect("No Core Text font available!")
                                    .url()
                                    .expect("No URL for Core Text font!")
                                    .get_string()
                                    .to_string()).expect("Couldn't parse Core Text font URL!")
                                                 .to_file_path()
                                                 .expect("Core Text font didn't name a path!");
        let mut bytes = Vec::new();
        File::open(path).expect("Couldn't open font file!").read_to_end(&mut bytes).unwrap();
        bytes
    }

    /// Returns a clone of the bytes in this font if they are in memory. This function never
    /// performs disk I/O.
    pub fn bytes_if_in_memory(&self) -> Option<Vec<u8>> {
        self.font_data.clone()
    }

    /// Returns the native font that underlies this font template, if applicable.
    pub fn native_font(&self) -> Option<CGFont> {
        self.ctfont().map(|ctfont| ctfont.copy_to_CGFont())
    }
}

#[derive(Debug)]
pub struct CachedCTFont(Mutex<Option<CTFont>>);

impl Deref for CachedCTFont {
    type Target = Mutex<Option<CTFont>>;
    fn deref(&self) -> &Mutex<Option<CTFont>> {
        &self.0
    }
}

impl Serialize for CachedCTFont {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_none()
    }
}

impl Deserialize for CachedCTFont {
    fn deserialize<D>(deserializer: &mut D) -> Result<CachedCTFont, D::Error>
                      where D: Deserializer {
        struct NoneOptionVisitor;

        impl Visitor for NoneOptionVisitor {
            type Value = CachedCTFont;

            #[inline]
            fn visit_none<E>(&mut self) -> Result<CachedCTFont, E> where E: Error {
                Ok(CachedCTFont(Mutex::new(None)))
            }
        }

        deserializer.deserialize_option(NoneOptionVisitor)
    }
}

