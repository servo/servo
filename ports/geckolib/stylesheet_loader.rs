/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use servo_arc::Arc;
use style::gecko::data::GeckoStyleSheet;
use style::gecko_bindings::bindings::Gecko_LoadStyleSheet;
use style::gecko_bindings::structs::{Loader, ServoStyleSheet, LoaderReusableStyleSheets};
use style::gecko_bindings::sugar::ownership::FFIArcHelpers;
use style::media_queries::MediaList;
use style::parser::ParserContext;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::{ImportRule, StylesheetLoader as StyleStylesheetLoader};
use style::stylesheets::import_rule::ImportSheet;
use style::values::specified::url::SpecifiedUrl;

pub struct StylesheetLoader(*mut Loader, *mut ServoStyleSheet, *mut LoaderReusableStyleSheets);

impl StylesheetLoader {
    pub fn new(loader: *mut Loader,
               parent: *mut ServoStyleSheet,
               reusable_sheets: *mut LoaderReusableStyleSheets) -> Self {
        StylesheetLoader(loader, parent, reusable_sheets)
    }
}

impl StyleStylesheetLoader for StylesheetLoader {
    fn request_stylesheet(
        &self,
        url: SpecifiedUrl,
        source_location: SourceLocation,
        _context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
    ) -> Arc<Locked<ImportRule>> {
        // After we get this raw pointer ImportRule will be moved into a lock and Arc
        // and so the Arc<Url> pointer inside will also move,
        // but the Url it points to or the allocating backing the String inside that Url won’t,
        // so this raw pointer will still be valid.

        let child_sheet = unsafe {
            let (spec_bytes, spec_len) = url.as_slice_components();
            let base_url_data = url.extra_data.get();
            Gecko_LoadStyleSheet(self.0,
                                 self.1,
                                 self.2,
                                 base_url_data,
                                 spec_bytes,
                                 spec_len as u32,
                                 media.into_strong())
        };

        debug_assert!(!child_sheet.is_null(),
                      "Import rules should always have a strong sheet");
        let stylesheet = unsafe {
            ImportSheet(GeckoStyleSheet::from_addrefed(child_sheet))
        };
        Arc::new(lock.wrap(ImportRule { url, source_location, stylesheet }))
    }
}
