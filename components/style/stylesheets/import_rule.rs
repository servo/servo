/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The [`@import`][import] at-rule.
//!
//! [import]: https://drafts.csswg.org/css-cascade-3/#at-import

use crate::context::QuirksMode;
use crate::media_queries::MediaList;
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::stylesheets::{CssRule, Origin, StylesheetInDocument};
use crate::values::CssUrl;
use cssparser::SourceLocation;
use std::fmt::{self, Write};
use std::mem::ManuallyDrop;
use style_traits::{CssWriter, ToCss};
use to_shmem::{SharedMemoryBuilder, ToShmem};

/// With asynchronous stylesheet parsing, we can't synchronously create a
/// GeckoStyleSheet. So we use this placeholder instead.
#[derive(Clone, Debug)]
pub struct PendingSheet {
    origin: Origin,
    quirks_mode: QuirksMode,
}

/// A sheet that is held from an import rule.
#[cfg(feature = "gecko")]
#[derive(Debug)]
pub enum ImportSheet {
    /// A bonafide stylesheet.
    Sheet(crate::gecko::data::GeckoStyleSheet),
    /// An @import created while parsing off-main-thread, whose Gecko sheet has
    /// yet to be created and attached.
    Pending(PendingSheet),
}

#[cfg(feature = "gecko")]
impl ImportSheet {
    /// Creates a new ImportSheet from a GeckoStyleSheet.
    pub fn new(sheet: crate::gecko::data::GeckoStyleSheet) -> Self {
        ImportSheet::Sheet(sheet)
    }

    /// Creates a pending ImportSheet for a load that has not started yet.
    pub fn new_pending(origin: Origin, quirks_mode: QuirksMode) -> Self {
        ImportSheet::Pending(PendingSheet {
            origin,
            quirks_mode,
        })
    }

    /// Returns a reference to the GeckoStyleSheet in this ImportSheet, if it
    /// exists.
    pub fn as_sheet(&self) -> Option<&crate::gecko::data::GeckoStyleSheet> {
        match *self {
            ImportSheet::Sheet(ref s) => Some(s),
            ImportSheet::Pending(_) => None,
        }
    }
}

#[cfg(feature = "gecko")]
impl DeepCloneWithLock for ImportSheet {
    fn deep_clone_with_lock(
        &self,
        _lock: &SharedRwLock,
        _guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        use crate::gecko::data::GeckoStyleSheet;
        use crate::gecko_bindings::bindings;
        match *self {
            ImportSheet::Sheet(ref s) => {
                let clone = unsafe {
                    bindings::Gecko_StyleSheet_Clone(s.raw() as *const _, params.reference_sheet)
                };
                ImportSheet::Sheet(unsafe { GeckoStyleSheet::from_addrefed(clone) })
            },
            ImportSheet::Pending(ref p) => ImportSheet::Pending(p.clone()),
        }
    }
}

#[cfg(feature = "gecko")]
impl StylesheetInDocument for ImportSheet {
    fn origin(&self, _guard: &SharedRwLockReadGuard) -> Origin {
        match *self {
            ImportSheet::Sheet(ref s) => s.contents().origin,
            ImportSheet::Pending(ref p) => p.origin,
        }
    }

    fn quirks_mode(&self, _guard: &SharedRwLockReadGuard) -> QuirksMode {
        match *self {
            ImportSheet::Sheet(ref s) => s.contents().quirks_mode,
            ImportSheet::Pending(ref p) => p.quirks_mode,
        }
    }

    fn enabled(&self) -> bool {
        match *self {
            ImportSheet::Sheet(ref s) => s.enabled(),
            ImportSheet::Pending(_) => true,
        }
    }

    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        match *self {
            ImportSheet::Sheet(ref s) => s.media(guard),
            ImportSheet::Pending(_) => None,
        }
    }

    fn rules<'a, 'b: 'a>(&'a self, guard: &'b SharedRwLockReadGuard) -> &'a [CssRule] {
        match *self {
            ImportSheet::Sheet(ref s) => s.contents().rules(guard),
            ImportSheet::Pending(_) => &[],
        }
    }
}

/// A sheet that is held from an import rule.
#[cfg(feature = "servo")]
#[derive(Debug)]
pub struct ImportSheet(pub ::servo_arc::Arc<crate::stylesheets::Stylesheet>);

#[cfg(feature = "servo")]
impl StylesheetInDocument for ImportSheet {
    fn origin(&self, guard: &SharedRwLockReadGuard) -> Origin {
        self.0.origin(guard)
    }

    fn quirks_mode(&self, guard: &SharedRwLockReadGuard) -> QuirksMode {
        self.0.quirks_mode(guard)
    }

    fn enabled(&self) -> bool {
        self.0.enabled()
    }

    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.0.media(guard)
    }

    fn rules<'a, 'b: 'a>(&'a self, guard: &'b SharedRwLockReadGuard) -> &'a [CssRule] {
        self.0.rules(guard)
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
    pub url: CssUrl,

    /// The stylesheet is always present. However, in the case of gecko async
    /// parsing, we don't actually have a Gecko sheet at first, and so the
    /// ImportSheet just has stub behavior until it appears.
    pub stylesheet: ImportSheet,

    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl ToShmem for ImportRule {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> ManuallyDrop<Self> {
        panic!("ToShmem failed for ImportRule: cannot handle imported style sheets")
    }
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
            },
            _ => {},
        };

        dest.write_str(";")
    }
}
