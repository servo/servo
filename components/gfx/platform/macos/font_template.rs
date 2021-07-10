/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_graphics::data_provider::CGDataProvider;
use core_graphics::font::CGFont;
use core_text::font::CTFont;
use core_text::font_collection;
use core_text::font_descriptor;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{Error as IoError, Read};
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use webrender_api::NativeFontHandle;

/// Platform specific font representation for mac.
/// The identifier is a PostScript font name. The
/// CTFont object is cached here for use by the
/// paint functions that create CGFont references.
#[derive(Deserialize, Serialize)]
pub struct FontTemplateData {
    // If you add members here, review the Debug impl below
    /// The `CTFont` object, if present. This is cached here so that we don't have to keep creating
    /// `CTFont` instances over and over. It can always be recreated from the `identifier` and/or
    /// `font_data` fields.
    ///
    /// When sending a `FontTemplateData` instance across processes, this will be cleared out on
    /// the other side, because `CTFont` instances cannot be sent across processes. This is
    /// harmless, however, because it can always be recreated.
    ctfont: CachedCTFont,

    pub identifier: Atom,
    pub font_data: RwLock<Option<Arc<Vec<u8>>>>,
}

impl fmt::Debug for FontTemplateData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("FontTemplateData")
            .field("ctfont", &self.ctfont)
            .field("identifier", &self.identifier)
            .field(
                "font_data",
                &self
                    .font_data
                    .read()
                    .unwrap()
                    .as_ref()
                    .map(|bytes| format!("[{} bytes]", bytes.len())),
            )
            .finish()
    }
}

unsafe impl Send for FontTemplateData {}
unsafe impl Sync for FontTemplateData {}

impl FontTemplateData {
    pub fn new(identifier: Atom, font_data: Option<Vec<u8>>) -> Result<FontTemplateData, IoError> {
        Ok(FontTemplateData {
            ctfont: CachedCTFont(Mutex::new(HashMap::new())),
            identifier: identifier.to_owned(),
            font_data: RwLock::new(font_data.map(Arc::new)),
        })
    }

    /// Retrieves the Core Text font instance, instantiating it if necessary.
    pub fn ctfont(&self, pt_size: f64) -> Option<CTFont> {
        let mut ctfonts = self.ctfont.lock().unwrap();
        let pt_size_key = Au::from_f64_px(pt_size);
        if !ctfonts.contains_key(&pt_size_key) {
            // If you pass a zero font size to one of the Core Text APIs, it'll replace it with
            // 12.0. We don't want that! (Issue #10492.)
            let clamped_pt_size = pt_size.max(0.01);
            let mut font_data = self.font_data.write().unwrap();
            let ctfont = match *font_data {
                Some(ref bytes) => {
                    let fontprov = CGDataProvider::from_buffer(bytes.clone());
                    let cgfont_result = CGFont::from_data_provider(fontprov);
                    match cgfont_result {
                        Ok(cgfont) => {
                            Some(core_text::font::new_from_CGFont(&cgfont, clamped_pt_size))
                        },
                        Err(_) => None,
                    }
                },
                None => {
                    // We can't rely on Core Text to load a font for us by postscript
                    // name here, due to https://github.com/servo/servo/issues/23290.
                    // The APIs will randomly load the wrong font, forcing us to use
                    // the roundabout route of creating a Core Graphics font from a
                    // a set of descriptors and then creating a Core Text font from
                    // that one.

                    let attributes: CFDictionary<CFString, CFType> =
                        CFDictionary::from_CFType_pairs(&[(
                            CFString::new("NSFontNameAttribute"),
                            CFString::new(&*self.identifier).as_CFType(),
                        )]);

                    let descriptor = font_descriptor::new_from_attributes(&attributes);
                    let descriptors = CFArray::from_CFTypes(&[descriptor]);
                    let collection = font_collection::new_from_descriptors(&descriptors);
                    collection.get_descriptors().and_then(|descriptors| {
                        let descriptor = descriptors.get(0).unwrap();
                        let font_path = Path::new(&descriptor.font_path().unwrap()).to_owned();
                        fs::read(&font_path).ok().and_then(|bytes| {
                            let font_bytes = Arc::new(bytes);
                            let fontprov = CGDataProvider::from_buffer(font_bytes.clone());
                            CGFont::from_data_provider(fontprov).ok().map(|cgfont| {
                                *font_data = Some(font_bytes);
                                core_text::font::new_from_CGFont(&cgfont, clamped_pt_size)
                            })
                        })
                    })
                },
            };
            if let Some(ctfont) = ctfont {
                ctfonts.insert(pt_size_key, ctfont);
            }
        }
        ctfonts.get(&pt_size_key).map(|ctfont| (*ctfont).clone())
    }

    /// Returns a clone of the data in this font. This may be a hugely expensive
    /// operation (depending on the platform) which performs synchronous disk I/O
    /// and should never be done lightly.
    pub fn bytes(&self) -> Vec<u8> {
        if let Some(font_data) = self.bytes_if_in_memory() {
            return font_data;
        }

        // This is spooky action at a distance, but getting a CTFont from this template
        // will (in the common case) bring the bytes into memory if they were not there
        // already. This also helps work around intermittent panics like
        // https://github.com/servo/servo/issues/24622 that occur for unclear reasons.
        let ctfont = self.ctfont(0.0);
        if let Some(font_data) = self.bytes_if_in_memory() {
            return font_data;
        }

        let path = ServoUrl::parse(
            &*ctfont
                .expect("No Core Text font available!")
                .url()
                .expect("No URL for Core Text font!")
                .get_string()
                .to_string(),
        )
        .expect("Couldn't parse Core Text font URL!")
        .as_url()
        .to_file_path()
        .expect("Core Text font didn't name a path!");
        let mut bytes = Vec::new();
        File::open(path)
            .expect("Couldn't open font file!")
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    }

    /// Returns a clone of the bytes in this font if they are in memory. This function never
    /// performs disk I/O.
    pub fn bytes_if_in_memory(&self) -> Option<Vec<u8>> {
        self.font_data
            .read()
            .unwrap()
            .as_ref()
            .map(|bytes| (**bytes).clone())
    }

    /// Returns the native font that underlies this font template, if applicable.
    pub fn native_font(&self) -> Option<NativeFontHandle> {
        self.ctfont(0.0)
            .map(|ctfont| NativeFontHandle(ctfont.copy_to_CGFont()))
    }
}

#[derive(Debug)]
pub struct CachedCTFont(Mutex<HashMap<Au, CTFont>>);

impl Deref for CachedCTFont {
    type Target = Mutex<HashMap<Au, CTFont>>;
    fn deref(&self) -> &Mutex<HashMap<Au, CTFont>> {
        &self.0
    }
}

impl Serialize for CachedCTFont {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for CachedCTFont {
    fn deserialize<D>(deserializer: D) -> Result<CachedCTFont, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NoneOptionVisitor;

        impl<'de> Visitor<'de> for NoneOptionVisitor {
            type Value = CachedCTFont;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "none")
            }

            #[inline]
            fn visit_none<E>(self) -> Result<CachedCTFont, E>
            where
                E: Error,
            {
                Ok(CachedCTFont(Mutex::new(HashMap::new())))
            }
        }

        deserializer.deserialize_option(NoneOptionVisitor)
    }
}
