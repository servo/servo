/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use app_units::Au;
use core_graphics::data_provider::CGDataProvider;
use core_graphics::font::CGFont;
use core_text::font::CTFont;
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
        let identifier_cache = cache
            .entry(font_identifier)
            .or_insert_with(Default::default);

        // It could be that between the time of the cache miss above and now, after the write lock
        // on the cache has been acquired, the cache was populated with the data that we need. Thus
        // check again and return the CTFont if it is is already cached.
        if let Some(core_text_font) = identifier_cache.get(&au_size) {
            return Some(core_text_font.clone());
        }

        let provider = CGDataProvider::from_buffer(data);
        let cgfont = CGFont::from_data_provider(provider).ok()?;
        let core_text_font = core_text::font::new_from_CGFont(&cgfont, clamped_pt_size);
        identifier_cache.insert(au_size, core_text_font.clone());
        Some(core_text_font)
    }

    pub(crate) fn clear_core_text_font_cache() {
        let cache = CACHE.0.get_or_init(Default::default);
        cache.write().clear();
    }
}
