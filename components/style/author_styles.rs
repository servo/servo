/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A set of author stylesheets and their computed representation, such as the
//! ones used for ShadowRoot and XBL.

use crate::context::QuirksMode;
use crate::dom::TElement;
#[cfg(feature = "gecko")]
use crate::gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use crate::invalidation::media_queries::ToMediaListKey;
use crate::media_queries::Device;
use crate::shared_lock::SharedRwLockReadGuard;
use crate::stylesheet_set::AuthorStylesheetSet;
use crate::stylesheets::StylesheetInDocument;
use crate::stylist::CascadeData;

/// A set of author stylesheets and their computed representation, such as the
/// ones used for ShadowRoot and XBL.
pub struct AuthorStyles<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// The sheet collection, which holds the sheet pointers, the invalidations,
    /// and all that stuff.
    pub stylesheets: AuthorStylesheetSet<S>,
    /// The actual cascade data computed from the stylesheets.
    pub data: CascadeData,
    /// The quirks mode of the last stylesheet flush, used because XBL sucks and
    /// we should really fix it, see bug 1406875.
    pub quirks_mode: QuirksMode,
}

impl<S> AuthorStyles<S>
where
    S: StylesheetInDocument + PartialEq + 'static,
{
    /// Create an empty AuthorStyles.
    #[inline]
    pub fn new() -> Self {
        Self {
            stylesheets: AuthorStylesheetSet::new(),
            data: CascadeData::new(),
            quirks_mode: QuirksMode::NoQuirks,
        }
    }

    /// Flush the pending sheet changes, updating `data` as appropriate.
    ///
    /// TODO(emilio): Need a host element and a snapshot map to do invalidation
    /// properly.
    #[inline]
    pub fn flush<E>(
        &mut self,
        device: &Device,
        quirks_mode: QuirksMode,
        guard: &SharedRwLockReadGuard,
    ) where
        E: TElement,
        S: ToMediaListKey,
    {
        let flusher = self
            .stylesheets
            .flush::<E>(/* host = */ None, /* snapshot_map = */ None);

        if flusher.sheets.dirty() {
            self.quirks_mode = quirks_mode;
        }

        // Ignore OOM.
        let _ = self
            .data
            .rebuild(device, quirks_mode, flusher.sheets, guard);
    }
}

#[cfg(feature = "gecko")]
unsafe impl HasFFI for AuthorStyles<crate::gecko::data::GeckoStyleSheet> {
    type FFIType = crate::gecko_bindings::bindings::RawServoAuthorStyles;
}
#[cfg(feature = "gecko")]
unsafe impl HasSimpleFFI for AuthorStyles<crate::gecko::data::GeckoStyleSheet> {}
#[cfg(feature = "gecko")]
unsafe impl HasBoxFFI for AuthorStyles<crate::gecko::data::GeckoStyleSheet> {}
