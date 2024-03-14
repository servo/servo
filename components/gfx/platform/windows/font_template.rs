/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, RwLock};
use std::{fmt, io};

use dwrote::{Font, FontCollection};
use serde::{Deserialize, Serialize};
use webrender_api::NativeFontHandle;

use crate::font_cache_thread::FontIdentifier;

#[derive(Deserialize, Serialize)]
pub struct FontTemplateData {
    /// The identifier for this font.
    pub identifier: FontIdentifier,
    /// The bytes of this font, lazily loaded if this is a local font.
    pub font_data: RwLock<Option<Arc<Vec<u8>>>>,
}

impl fmt::Debug for FontTemplateData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("FontTemplateData")
            .field(
                "font_data",
                &self
                    .font_data
                    .read()
                    .unwrap()
                    .as_ref()
                    .map(|bytes| format!("[{} bytes]", bytes.len())),
            )
            .field("identifier", &self.identifier)
            .finish()
    }
}

impl FontTemplateData {
    pub fn new(
        identifier: FontIdentifier,
        font_data: Option<Vec<u8>>,
    ) -> Result<FontTemplateData, io::Error> {
        Ok(FontTemplateData {
            identifier,
            font_data: RwLock::new(font_data.map(Arc::new)),
        })
    }

    /// Returns a reference to the data in this font. This may be a hugely expensive
    /// operation (depending on the platform) which performs synchronous disk I/O
    /// and should never be done lightly.
    pub fn bytes(&self) -> Arc<Vec<u8>> {
        self.font_data
            .write()
            .unwrap()
            .get_or_insert_with(|| {
                let font_descriptor = match &self.identifier {
                    FontIdentifier::Local(local_identifier) => {
                        local_identifier.font_descriptor.clone()
                    },
                    FontIdentifier::Web(_) => unreachable!("Created a web font without data."),
                };

                let font = FontCollection::system()
                    .get_font_from_descriptor(&font_descriptor)
                    .unwrap();
                let face = font.create_font_face();
                let files = face.get_files();
                assert!(!files.is_empty());
                Arc::new(files[0].get_font_file_bytes())
            })
            .clone()
    }

    /// Returns a reference to the bytes in this font if they are in memory.
    /// This function never performs disk I/O.
    pub fn bytes_if_in_memory(&self) -> Option<Arc<Vec<u8>>> {
        self.font_data.read().unwrap().as_ref().cloned()
    }

    /// Get a [`FontFace`] for this font if it is a local font or return `None` if it's a
    /// web font.
    pub fn get_font(&self) -> Option<Font> {
        let font_descriptor = match &self.identifier {
            FontIdentifier::Local(local_identifier) => local_identifier.font_descriptor.clone(),
            FontIdentifier::Web(_) => return None,
        };

        FontCollection::system().get_font_from_descriptor(&font_descriptor)
    }

    pub fn native_font(&self) -> Option<NativeFontHandle> {
        let face = self.get_font()?.create_font_face();
        let path = face.get_files().first()?.get_font_file_path()?;
        Some(NativeFontHandle {
            path,
            index: face.get_index(),
        })
    }
}
