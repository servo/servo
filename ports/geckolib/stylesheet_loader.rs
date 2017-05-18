/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::gecko_bindings::bindings::Gecko_LoadStyleSheet;
use style::gecko_bindings::structs::{Loader, ServoStyleSheet, LoaderReusableStyleSheets};
use style::gecko_bindings::sugar::ownership::{HasArcFFI, FFIArcHelpers};
use style::media_queries::MediaList;
use style::shared_lock::Locked;
use style::stylearc::Arc;
use style::stylesheets::{ImportRule, Stylesheet, StylesheetLoader as StyleStylesheetLoader};

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
        media: Arc<Locked<MediaList>>,
        make_import: &mut FnMut(Arc<Locked<MediaList>>) -> ImportRule,
        make_arc: &mut FnMut(ImportRule) -> Arc<Locked<ImportRule>>,
    ) -> Arc<Locked<ImportRule>> {
        let import = make_import(media.clone());

        // After we get this raw pointer ImportRule will be moved into a lock and Arc
        // and so the Arc<Url> pointer inside will also move,
        // but the Url it points to or the allocating backing the String inside that Url won’t,
        // so this raw pointer will still be valid.
        let (spec_bytes, spec_len): (*const u8, usize) = import.url.as_slice_components();

        let base_url_data = import.url.extra_data.get();
        unsafe {
            Gecko_LoadStyleSheet(self.0,
                                 self.1,
                                 self.2,
                                 Stylesheet::arc_as_borrowed(&import.stylesheet),
                                 base_url_data,
                                 spec_bytes,
                                 spec_len as u32,
                                 media.into_strong())
        }
        make_arc(import)
    }
}
