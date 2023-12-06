/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The stylesheet loader is the abstraction used to trigger network requests
//! for `@import` rules.

use crate::media_queries::MediaList;
use crate::parser::ParserContext;
use crate::shared_lock::{Locked, SharedRwLock};
use crate::stylesheets::import_rule::{ImportLayer, ImportRule, ImportSupportsCondition};
use crate::values::CssUrl;
use cssparser::SourceLocation;
use servo_arc::Arc;

/// The stylesheet loader is the abstraction used to trigger network requests
/// for `@import` rules.
pub trait StylesheetLoader {
    /// Request a stylesheet after parsing a given `@import` rule, and return
    /// the constructed `@import` rule.
    fn request_stylesheet(
        &self,
        url: CssUrl,
        location: SourceLocation,
        context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
        supports: Option<ImportSupportsCondition>,
        layer: ImportLayer,
    ) -> Arc<Locked<ImportRule>>;
}
