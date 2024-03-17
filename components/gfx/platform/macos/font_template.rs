/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{Error as IoError, Read};
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use app_units::Au;
use core_graphics::data_provider::CGDataProvider;
use core_graphics::font::CGFont;
use core_text::font::CTFont;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use webrender_api::NativeFontHandle;

use crate::font_cache_thread::FontIdentifier;

/// Platform specific font representation for MacOS. CTFont object is cached here for use
/// by the paint functions that create CGFont references.
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
    pub identifier: FontIdentifier,
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
    pub fn new(
        identifier: FontIdentifier,
        font_data: Option<Vec<u8>>,
    ) -> Result<FontTemplateData, IoError> {
        Ok(FontTemplateData {
            ctfont: CachedCTFont(Mutex::new(HashMap::new())),
            identifier,
            font_data: RwLock::new(font_data.map(Arc::new)),
        })
    }

    /// Retrieves the Core Text font instance, instantiating it if necessary.
    pub fn ctfont(&self, pt_size: f64) -> Option<CTFont> {
        let mut ctfonts = self.ctfont.lock().unwrap();

        let entry = ctfonts.entry(Au::from_f64_px(pt_size));
        match entry {
            Entry::Occupied(entry) => return Some(entry.get().clone()),
            Entry::Vacant(_) => {},
        }

        // If you pass a zero font size to one of the Core Text APIs, it'll replace it with
        // 12.0. We don't want that! (Issue #10492.)
        let clamped_pt_size = pt_size.max(0.01);

        let provider = CGDataProvider::from_buffer(self.bytes());
        let cgfont = CGFont::from_data_provider(provider).ok()?;
        let ctfont = core_text::font::new_from_CGFont(&cgfont, clamped_pt_size);

        // Cache the newly created CTFont font.
        entry.or_insert(ctfont.clone());

        Some(ctfont)
    }

    /// Returns a reference to the data in this font. This may be a hugely expensive
    /// operation (depending on the platform) which performs synchronous disk I/O
    /// and should never be done lightly.
    pub fn bytes(&self) -> Arc<Vec<u8>> {
        self.font_data
            .write()
            .unwrap()
            .get_or_insert_with(|| {
                let path = match &self.identifier {
                    FontIdentifier::Local(local) => local.path.clone(),
                    FontIdentifier::Web(_) => unreachable!("Web fonts should always have data."),
                };
                let mut bytes = Vec::new();
                File::open(Path::new(&*path))
                    .expect("Couldn't open font file!")
                    .read_to_end(&mut bytes)
                    .unwrap();
                Arc::new(bytes)
            })
            .clone()
    }

    /// Returns a reference to the bytes in this font if they are in memory.
    /// This function never performs disk I/O.
    pub fn bytes_if_in_memory(&self) -> Option<Arc<Vec<u8>>> {
        self.font_data.read().unwrap().as_ref().cloned()
    }

    /// Returns the native font that underlies this font template, if applicable.
    pub fn native_font(&self) -> Option<NativeFontHandle> {
        let local_identifier = match &self.identifier {
            FontIdentifier::Local(local_identifier) => local_identifier,
            FontIdentifier::Web(_) => return None,
        };
        Some(NativeFontHandle {
            name: local_identifier.postscript_name.to_string(),
            path: local_identifier.path.to_string(),
        })
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
