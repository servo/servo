/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use app_units::Au;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation::url::{kCFURLPOSIXPathStyle, CFURL};
use core_graphics::data_provider::CGDataProvider;
use core_graphics::display::CFDictionary;
use core_graphics::font::CGFont;
use core_text::font::CTFont;
use core_text::font_descriptor::kCTFontURLAttribute;
use parking_lot::RwLock;

use crate::font_cache_thread::FontIdentifier;

/// A cache of `CTFont` to avoid having to create `CTFont` instances over and over. It is
/// always possible to create a `CTFont` using a `FontTemplate` even if it isn't in this
/// cache.
pub(crate) struct CoreTextFontCache(OnceLock<RwLock<HashMap<FontIdentifier, CachedCTFont>>>);

/// The global [`CoreTextFontCache`].
static CACHE: CoreTextFontCache = CoreTextFontCache(OnceLock::new());

/// A [`HashMap`] of cached [`CTFont`] for a single [`FontIdentifier`]. There is one [`CTFont`]
/// for each cached font size.
type CachedCTFont = HashMap<Au, CTFont>;

impl CoreTextFontCache {
    pub(crate) fn core_text_font(
        font_identifier: FontIdentifier,
        data: Arc<Vec<u8>>,
        pt_size: f64,
    ) -> Option<CTFont> {
        //// If you pass a zero font size to one of the Core Text APIs, it'll replace it with
        //// 12.0. We don't want that! (Issue #10492.)
        let clamped_pt_size = pt_size.max(0.01);
        let au_size = Au::from_f64_px(clamped_pt_size);

        let cache = CACHE.0.get_or_init(Default::default);
        {
            let cache = cache.read();
            if let Some(core_text_font) = cache
                .get(&font_identifier)
                .and_then(|identifier_cache| identifier_cache.get(&au_size))
            {
                return Some(core_text_font.clone());
            }
        }

        let mut cache = cache.write();
        let identifier_cache = cache.entry(font_identifier.clone()).or_default();

        // It could be that between the time of the cache miss above and now, after the write lock
        // on the cache has been acquired, the cache was populated with the data that we need. Thus
        // check again and return the CTFont if it is is already cached.
        if let Some(core_text_font) = identifier_cache.get(&au_size) {
            return Some(core_text_font.clone());
        }

        let core_text_font = match font_identifier {
            FontIdentifier::Local(local_font_identifier) => {
                // Other platforms can instantiate a platform font by loading the data
                // from a file and passing an index in the case the file is a TTC bundle.
                // The only way to reliably load the correct font from a TTC bundle on
                // macOS is to create the font using a descriptor with both the PostScript
                // name and path.
                let cf_name = CFString::new(&local_font_identifier.postscript_name);
                let mut descriptor = core_text::font_descriptor::new_from_postscript_name(&cf_name);

                let cf_path = CFString::new(&local_font_identifier.path);
                let url_attribute = unsafe { CFString::wrap_under_get_rule(kCTFontURLAttribute) };
                let attributes = CFDictionary::from_CFType_pairs(&[(
                    url_attribute,
                    CFURL::from_file_system_path(cf_path, kCFURLPOSIXPathStyle, false),
                )]);
                if let Ok(descriptor_with_path) =
                    descriptor.create_copy_with_attributes(attributes.to_untyped())
                {
                    descriptor = descriptor_with_path;
                }

                core_text::font::new_from_descriptor(&descriptor, clamped_pt_size)
            },
            FontIdentifier::Web(_) => {
                let provider = CGDataProvider::from_buffer(data);
                let cgfont = CGFont::from_data_provider(provider).ok()?;
                core_text::font::new_from_CGFont(&cgfont, clamped_pt_size)
            },
        };

        identifier_cache.insert(au_size, core_text_font.clone());
        Some(core_text_font)
    }
}
