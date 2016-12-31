/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parking_lot::RwLock;
use std::sync::Arc;
use style::gecko_bindings::bindings::Gecko_LoadStyleSheet;
use style::gecko_bindings::structs::{Loader, ServoStyleSheet};
use style::gecko_bindings::sugar::ownership::HasArcFFI;
use style::stylesheets::{ImportRule, StylesheetLoader as StyleStylesheetLoader};
use style_traits::ToCss;

pub struct StylesheetLoader(*mut Loader, *mut ServoStyleSheet);

impl StylesheetLoader {
    pub fn new(loader: *mut Loader, parent: *mut ServoStyleSheet) -> Self {
        StylesheetLoader(loader, parent)
    }
}

impl StyleStylesheetLoader for StylesheetLoader {
    fn request_stylesheet(&self, import_rule: &Arc<RwLock<ImportRule>>) {
        let import = import_rule.read();
        let (spec_bytes, spec_len) = import.url.as_slice_components()
            .expect("Import only loads valid URLs");

        // TODO(emilio): We probably want to share media representation with
        // Gecko in Stylo.
        //
        // This also allows us to get rid of a bunch of extra work to evaluate
        // and ensure parity, and shouldn't be much Gecko work given we always
        // evaluate them on the main thread.
        //
        // Meanwhile, this works.
        let media = import.stylesheet.media.read().to_css_string();

        unsafe {
            Gecko_LoadStyleSheet(self.0,
                                 self.1,
                                 HasArcFFI::arc_as_borrowed(import_rule),
                                 spec_bytes,
                                 spec_len as u32,
                                 media.as_bytes().as_ptr(),
                                 media.len() as u32);
        }
    }
}
