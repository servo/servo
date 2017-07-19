/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The stylesheet loader is the abstraction used to trigger network requests
//! for `@import` rules.

use cssparser::SourceLocation;
use media_queries::MediaList;
use parser::ParserContext;
use servo_arc::Arc;
use shared_lock::{Locked, SharedRwLock};
use stylesheets::import_rule::ImportRule;
use values::specified::url::SpecifiedUrl;

/// The stylesheet loader is the abstraction used to trigger network requests
/// for `@import` rules.
pub trait StylesheetLoader {
    /// Request a stylesheet after parsing a given `@import` rule, and return
    /// the constructed `@import` rule.
    fn request_stylesheet(
        &self,
        url: SpecifiedUrl,
        location: SourceLocation,
        context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
    ) -> Arc<Locked<ImportRule>>;
}
