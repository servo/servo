/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::{ImportRule, StylesheetLoader as StyleStylesheetLoader};

pub struct StylesheetLoader;

impl StylesheetLoader {
    pub fn new() -> Self {
        StylesheetLoader
    }
}

impl StyleStylesheetLoader for StylesheetLoader {
    fn request_stylesheet(&self, _import: &Arc<RwLock<ImportRule>>) {
        // FIXME(emilio): Implement `@import` in stylo.
    }
}
