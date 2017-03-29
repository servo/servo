/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use style::gecko_bindings::bindings::Gecko_LoadStyleSheet;
use style::gecko_bindings::structs::{Loader, ServoStyleSheet};
use style::gecko_bindings::sugar::ownership::HasArcFFI;
use style::media_queries::MediaList;
use style::shared_lock::Locked;
use style::stylesheets::{ImportRule, StylesheetLoader as StyleStylesheetLoader};
use style_traits::ToCss;

pub struct StylesheetLoader(*mut Loader, *mut ServoStyleSheet);

impl StylesheetLoader {
    pub fn new(loader: *mut Loader, parent: *mut ServoStyleSheet) -> Self {
        StylesheetLoader(loader, parent)
    }
}

impl StyleStylesheetLoader for StylesheetLoader {
    fn request_stylesheet(
        &self,
        media: MediaList,
        make_import: &mut FnMut(MediaList) -> ImportRule,
        make_arc: &mut FnMut(ImportRule) -> Arc<Locked<ImportRule>>,
    ) -> Arc<Locked<ImportRule>> {
        // TODO(emilio): We probably want to share media representation with
        // Gecko in Stylo.
        //
        // This also allows us to get rid of a bunch of extra work to evaluate
        // and ensure parity, and shouldn't be much Gecko work given we always
        // evaluate them on the main thread.
        //
        // Meanwhile, this works.
        let media_string = media.to_css_string();

        let import = make_import(media);

        // After we get this raw pointer ImportRule will be moved into a lock and Arc
        // and so the Arc<Url> pointer inside will also move,
        // but the Url it points to or the allocating backing the String inside that Url won’t,
        // so this raw pointer will still be valid.
        let (spec_bytes, spec_len): (*const u8, usize) = import.url.as_slice_components();

        let base_uri = import.url.base.mRawPtr;
        let arc = make_arc(import);
        unsafe {
            Gecko_LoadStyleSheet(self.0,
                                 self.1,
                                 HasArcFFI::arc_as_borrowed(&arc),
                                 base_uri,
                                 spec_bytes,
                                 spec_len as u32,
                                 media_string.as_bytes().as_ptr(),
                                 media_string.len() as u32);
        }
        arc
    }
}
