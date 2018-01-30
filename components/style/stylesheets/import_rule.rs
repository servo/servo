/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@import`][import] at-rule.
//!
//! [import]: https://drafts.csswg.org/css-cascade-3/#at-import

use cssparser::SourceLocation;
use media_queries::MediaList;
use shared_lock::{DeepCloneWithLock, DeepCloneParams, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt::{self, Write};
use str::CssStringWriter;
use style_traits::{CssWriter, ToCss};
use stylesheets::{StylesheetContents, StylesheetInDocument};
use values::specified::url::SpecifiedUrl;

/// A sheet that is held from an import rule.
#[cfg(feature = "gecko")]
#[derive(Debug)]
pub struct ImportSheet(pub ::gecko::data::GeckoStyleSheet);

#[cfg(feature = "gecko")]
impl DeepCloneWithLock for ImportSheet {
    fn deep_clone_with_lock(
        &self,
        _lock: &SharedRwLock,
        _guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        use gecko::data::GeckoStyleSheet;
        use gecko_bindings::bindings;
        let clone = unsafe {
            bindings::Gecko_StyleSheet_Clone(
                self.0.raw() as *const _,
                params.reference_sheet
            )
        };
        ImportSheet(unsafe { GeckoStyleSheet::from_addrefed(clone) })
    }
}

/// A sheet that is held from an import rule.
#[cfg(feature = "servo")]
#[derive(Debug)]
pub struct ImportSheet(pub ::servo_arc::Arc<::stylesheets::Stylesheet>);

impl StylesheetInDocument for ImportSheet {
    /// Get the media associated with this stylesheet.
    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.0.media(guard)
    }

    fn contents(&self, guard: &SharedRwLockReadGuard) -> &StylesheetContents {
        self.0.contents(guard)
    }

    fn enabled(&self) -> bool {
        self.0.enabled()
    }
}

#[cfg(feature = "servo")]
impl DeepCloneWithLock for ImportSheet {
    fn deep_clone_with_lock(
        &self,
        _lock: &SharedRwLock,
        _guard: &SharedRwLockReadGuard,
        _params: &DeepCloneParams,
    ) -> Self {
        use servo_arc::Arc;

        ImportSheet(Arc::new((&*self.0).clone()))
    }
}

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
    pub stylesheet: ImportSheet,

    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl DeepCloneWithLock for ImportRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        ImportRule {
            url: self.url.clone(),
            stylesheet: self.stylesheet.deep_clone_with_lock(lock, guard, params),
            source_location: self.source_location.clone(),
        }
    }
}

impl ToCssWithGuard for ImportRule {
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@import ")?;
        self.url.to_css(&mut CssWriter::new(dest))?;

        match self.stylesheet.media(guard) {
            Some(media) if !media.is_empty() => {
                dest.write_str(" ")?;
                media.to_css(&mut CssWriter::new(dest))?;
            }
            _ => {},
        };

        dest.write_str(";")
    }
}
