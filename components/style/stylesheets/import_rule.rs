/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The [`@import`][import] at-rule.
//!
//! [import]: https://drafts.csswg.org/css-cascade-3/#at-import

use crate::media_queries::MediaList;
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::stylesheets::{CssRule, StylesheetInDocument};
use crate::stylesheets::layer_rule::LayerName;
use crate::values::CssUrl;
use cssparser::SourceLocation;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use to_shmem::{self, SharedMemoryBuilder, ToShmem};

/// With asynchronous stylesheet parsing, we can't synchronously create a
/// GeckoStyleSheet. So we use this placeholder instead.
#[cfg(feature = "gecko")]
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
            ImportSheet::Sheet(ref s) => {
                debug_assert!(!s.hack_is_null());
                if s.hack_is_null() {
                    return None;
                }
                Some(s)
            },
            ImportSheet::Pending(_) => None,
        }
    }

    /// Returns the media list for this import rule.
    pub fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.as_sheet().and_then(|s| s.media(guard))
    }

    /// Returns the rule list for this import rule.
    pub fn rules<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a [CssRule] {
        match self.as_sheet() {
            Some(s) => s.rules(guard),
            None => &[],
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

/// A sheet that is held from an import rule.
#[cfg(feature = "servo")]
#[derive(Debug)]
pub struct ImportSheet(pub ::servo_arc::Arc<crate::stylesheets::Stylesheet>);

#[cfg(feature = "servo")]
impl ImportSheet {
    /// Returns the media list for this import rule.
    pub fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.0.media(guard)
    }

    /// Returns the rules for this import rule.
    pub fn rules<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a [CssRule] {
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

/// The layer keyword or function in an import rule.
#[derive(Debug)]
pub struct ImportLayer {
    /// Whether the layer is anonymous.
    pub is_anonymous: bool,
    /// The layer name.
    pub name: LayerName,
}


impl ToCss for ImportLayer {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_anonymous {
            dest.write_str("layer")
        } else {
            dest.write_str("layer(")?;
            self.name.to_css(dest)?;
            dest.write_char(')')
        }
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

    /// A `layer()` function name.
    pub layer: Option<ImportLayer>,

    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl ToShmem for ImportRule {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> to_shmem::Result<Self> {
        Err(String::from(
            "ToShmem failed for ImportRule: cannot handle imported style sheets",
        ))
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
            layer: self.layer.as_ref().map(|layer| {
                ImportLayer {
                    is_anonymous: layer.is_anonymous,
                    name: if layer.is_anonymous {
                        LayerName::new_anonymous()
                    } else {
                        layer.name.clone()
                    },
                }
            }),
            source_location: self.source_location.clone(),
        }
    }
}

impl ToCssWithGuard for ImportRule {
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@import ")?;
        self.url.to_css(&mut CssWriter::new(dest))?;

        if let Some(media) = self.stylesheet.media(guard) {
            if !media.is_empty() {
                dest.write_char(' ')?;
                media.to_css(&mut CssWriter::new(dest))?;
            }
        }

        if let Some(ref layer) = self.layer {
            dest.write_char(' ')?;
            layer.to_css(&mut CssWriter::new(dest))?;
        }

        dest.write_char(';')
    }
}
