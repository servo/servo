/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The stylesheet loader is the abstraction used to trigger network requests
//! for `@import` rules.

use media_queries::MediaList;
use shared_lock::Locked;
use stylearc::Arc;
use stylesheets::ImportRule;

/// The stylesheet loader is the abstraction used to trigger network requests
/// for `@import` rules.
pub trait StylesheetLoader {
    /// Request a stylesheet after parsing a given `@import` rule.
    ///
    /// The called code is responsible to update the `stylesheet` rules field
    /// when the sheet is done loading.
    ///
    /// The convoluted signature allows impls to look at MediaList and
    /// ImportRule before they’re locked, while keeping the trait object-safe.
    fn request_stylesheet(
        &self,
        media: Arc<Locked<MediaList>>,
        make_import: &mut FnMut(Arc<Locked<MediaList>>) -> ImportRule,
        make_arc: &mut FnMut(ImportRule) -> Arc<Locked<ImportRule>>,
    ) -> Arc<Locked<ImportRule>>;
}

/// A dummy loader that just creates the import rule with the empty stylesheet.
pub struct NoOpLoader;

impl StylesheetLoader for NoOpLoader {
    fn request_stylesheet(
        &self,
        media: Arc<Locked<MediaList>>,
        make_import: &mut FnMut(Arc<Locked<MediaList>>) -> ImportRule,
        make_arc: &mut FnMut(ImportRule) -> Arc<Locked<ImportRule>>,
    ) -> Arc<Locked<ImportRule>> {
        make_arc(make_import(media))
    }
}
