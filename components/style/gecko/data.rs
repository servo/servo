/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Data needed to style a Gecko document.

use crate::dom::TElement;
use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::{self, RawServoStyleSet, ServoStyleSetSizes};
use crate::gecko_bindings::structs::{StyleSheet as DomStyleSheet, StyleSheetInfo};
use crate::gecko_bindings::sugar::ownership::{HasArcFFI, HasBoxFFI, HasFFI, HasSimpleFFI};
use crate::invalidation::media_queries::{MediaListKey, ToMediaListKey};
use crate::media_queries::{Device, MediaList};
use crate::properties::ComputedValues;
use crate::selector_parser::SnapshotMap;
use crate::shared_lock::{Locked, SharedRwLockReadGuard, StylesheetGuards};
use crate::stylesheets::{StylesheetContents, StylesheetInDocument};
use crate::stylist::Stylist;
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use malloc_size_of::MallocSizeOfOps;
use servo_arc::Arc;
use std::fmt;

/// Little wrapper to a Gecko style sheet.
#[derive(Eq, PartialEq)]
pub struct GeckoStyleSheet(*const DomStyleSheet);

impl fmt::Debug for GeckoStyleSheet {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let contents = self.contents();
        formatter
            .debug_struct("GeckoStyleSheet")
            .field("origin", &contents.origin)
            .field("url_data", &*contents.url_data.read())
            .finish()
    }
}

impl ToMediaListKey for crate::gecko::data::GeckoStyleSheet {
    fn to_media_list_key(&self) -> MediaListKey {
        use std::mem;
        unsafe { MediaListKey::from_raw(mem::transmute(self.0)) }
    }
}

impl GeckoStyleSheet {
    /// Create a `GeckoStyleSheet` from a raw `DomStyleSheet` pointer.
    #[inline]
    pub unsafe fn new(s: *const DomStyleSheet) -> Self {
        debug_assert!(!s.is_null());
        bindings::Gecko_StyleSheet_AddRef(s);
        Self::from_addrefed(s)
    }

    /// Create a `GeckoStyleSheet` from a raw `DomStyleSheet` pointer that
    /// already holds a strong reference.
    #[inline]
    pub unsafe fn from_addrefed(s: *const DomStyleSheet) -> Self {
        assert!(!s.is_null());
        GeckoStyleSheet(s)
    }

    /// HACK(emilio): This is so that we can avoid crashing release due to
    /// bug 1719963 and can hopefully get a useful report from fuzzers.
    #[inline]
    pub fn hack_is_null(&self) -> bool {
        self.0.is_null()
    }

    /// Get the raw `StyleSheet` that we're wrapping.
    pub fn raw(&self) -> &DomStyleSheet {
        unsafe { &*self.0 }
    }

    fn inner(&self) -> &StyleSheetInfo {
        unsafe { &*(self.raw().mInner as *const StyleSheetInfo) }
    }
}

impl Drop for GeckoStyleSheet {
    fn drop(&mut self) {
        unsafe { bindings::Gecko_StyleSheet_Release(self.0) };
    }
}

impl Clone for GeckoStyleSheet {
    fn clone(&self) -> Self {
        unsafe { bindings::Gecko_StyleSheet_AddRef(self.0) };
        GeckoStyleSheet(self.0)
    }
}

impl StylesheetInDocument for GeckoStyleSheet {
    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        use crate::gecko_bindings::structs::mozilla::dom::MediaList as DomMediaList;
        use std::mem;

        unsafe {
            let dom_media_list = self.raw().mMedia.mRawPtr as *const DomMediaList;
            if dom_media_list.is_null() {
                return None;
            }
            let raw_list = &*(*dom_media_list).mRawList.mRawPtr;
            let list = Locked::<MediaList>::as_arc(mem::transmute(&raw_list));
            Some(list.read_with(guard))
        }
    }

    // All the stylesheets Servo knows about are enabled, because that state is
    // handled externally by Gecko.
    #[inline]
    fn enabled(&self) -> bool {
        true
    }

    #[inline]
    fn contents(&self) -> &StylesheetContents {
        debug_assert!(!self.inner().mContents.mRawPtr.is_null());
        unsafe {
            let contents =
                (&**StylesheetContents::as_arc(&&*self.inner().mContents.mRawPtr)) as *const _;
            &*contents
        }
    }
}

/// The container for data that a Servo-backed Gecko document needs to style
/// itself.
pub struct PerDocumentStyleDataImpl {
    /// Rule processor.
    pub stylist: Stylist,
}

/// The data itself is an `AtomicRefCell`, which guarantees the proper semantics
/// and unexpected races while trying to mutate it.
pub struct PerDocumentStyleData(AtomicRefCell<PerDocumentStyleDataImpl>);

impl PerDocumentStyleData {
    /// Create a `PerDocumentStyleData`.
    pub fn new(document: *const structs::Document) -> Self {
        let device = Device::new(document);
        let quirks_mode = device.document().mCompatMode;

        PerDocumentStyleData(AtomicRefCell::new(PerDocumentStyleDataImpl {
            stylist: Stylist::new(device, quirks_mode.into()),
        }))
    }

    /// Get an immutable reference to this style data.
    pub fn borrow(&self) -> AtomicRef<PerDocumentStyleDataImpl> {
        self.0.borrow()
    }

    /// Get an mutable reference to this style data.
    pub fn borrow_mut(&self) -> AtomicRefMut<PerDocumentStyleDataImpl> {
        self.0.borrow_mut()
    }
}

impl PerDocumentStyleDataImpl {
    /// Recreate the style data if the stylesheets have changed.
    pub fn flush_stylesheets<E>(
        &mut self,
        guard: &SharedRwLockReadGuard,
        document_element: Option<E>,
        snapshots: Option<&SnapshotMap>,
    ) -> bool
    where
        E: TElement,
    {
        self.stylist
            .flush(&StylesheetGuards::same(guard), document_element, snapshots)
    }

    /// Get the default computed values for this document.
    pub fn default_computed_values(&self) -> &Arc<ComputedValues> {
        self.stylist.device().default_computed_values_arc()
    }

    /// Returns whether visited styles are enabled.
    #[inline]
    pub fn visited_styles_enabled(&self) -> bool {
        unsafe { bindings::Gecko_VisitedStylesEnabled(self.stylist.device().document()) }
    }

    /// Measure heap usage.
    pub fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        self.stylist.add_size_of(ops, sizes);
    }
}

unsafe impl HasFFI for PerDocumentStyleData {
    type FFIType = RawServoStyleSet;
}
unsafe impl HasSimpleFFI for PerDocumentStyleData {}
unsafe impl HasBoxFFI for PerDocumentStyleData {}
