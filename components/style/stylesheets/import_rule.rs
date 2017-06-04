/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@import`][import] at-rule.
//!
//! [import]: https://drafts.csswg.org/css-cascade-3/#at-import

use cssparser::SourceLocation;
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::ToCss;
use stylearc::Arc;
use stylesheets::stylesheet::Stylesheet;
use values::specified::url::SpecifiedUrl;

/// The [`@import`][import] at-rule.
///
/// [import]: https://drafts.csswg.org/css-cascade-3/#at-import
#[derive(Debug)]
pub struct ImportRule {
    /// The `<url>` this `@import` rule is loading.
    pub url: SpecifiedUrl,

    /// The stylesheet is always present.
    ///
    /// It contains an empty list of rules and namespace set that is updated
    /// when it loads.
    pub stylesheet: Arc<Stylesheet>,

    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl Clone for ImportRule {
    fn clone(&self) -> ImportRule {
        let stylesheet: &Stylesheet = &*self.stylesheet;
        ImportRule {
            url: self.url.clone(),
            stylesheet: Arc::new(stylesheet.clone()),
            source_location: self.source_location.clone(),
        }
    }
}

impl ToCssWithGuard for ImportRule {
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        dest.write_str("@import ")?;
        self.url.to_css(dest)?;
        let media = self.stylesheet.media.read_with(guard);
        if !media.is_empty() {
            dest.write_str(" ")?;
            media.to_css(dest)?;
        }
        dest.write_str(";")
    }
}
