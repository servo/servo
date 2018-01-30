/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed to style a Gecko document.

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use context::TraversalStatistics;
use dom::TElement;
use gecko_bindings::bindings::{self, RawServoStyleSet};
use gecko_bindings::structs::{RawGeckoPresContextOwned, ServoStyleSetSizes, ServoStyleSheet};
use gecko_bindings::structs::{StyleSheetInfo, ServoStyleSheetInner, nsIDocument, self};
use gecko_bindings::sugar::ownership::{HasArcFFI, HasBoxFFI, HasFFI, HasSimpleFFI};
use invalidation::media_queries::{MediaListKey, ToMediaListKey};
use malloc_size_of::MallocSizeOfOps;
use media_queries::{Device, MediaList};
use properties::ComputedValues;
use selector_parser::SnapshotMap;
use servo_arc::Arc;
use shared_lock::{Locked, StylesheetGuards, SharedRwLockReadGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use stylesheets::{StylesheetContents, StylesheetInDocument};
use stylist::Stylist;

/// Little wrapper to a Gecko style sheet.
#[derive(Debug, Eq, PartialEq)]
pub struct GeckoStyleSheet(*const ServoStyleSheet);

impl ToMediaListKey for ::gecko::data::GeckoStyleSheet {
    fn to_media_list_key(&self) -> MediaListKey {
        use std::mem;
        unsafe {
            MediaListKey::from_raw(mem::transmute(self.0))
        }
    }
}

impl GeckoStyleSheet {
    /// Create a `GeckoStyleSheet` from a raw `ServoStyleSheet` pointer.
    #[inline]
    pub unsafe fn new(s: *const ServoStyleSheet) -> Self {
        debug_assert!(!s.is_null());
        bindings::Gecko_StyleSheet_AddRef(s);
        Self::from_addrefed(s)
    }

    /// Create a `GeckoStyleSheet` from a raw `ServoStyleSheet` pointer that
    /// already holds a strong reference.
    #[inline]
    pub unsafe fn from_addrefed(s: *const ServoStyleSheet) -> Self {
        debug_assert!(!s.is_null());
        GeckoStyleSheet(s)
    }

    /// Get the raw `ServoStyleSheet` that we're wrapping.
    pub fn raw(&self) -> &ServoStyleSheet {
        unsafe { &*self.0 }
    }

    fn inner(&self) -> &ServoStyleSheetInner {
        unsafe {
            &*(self.raw()._base.mInner as *const StyleSheetInfo as *const ServoStyleSheetInner)
        }
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
    fn contents(&self, _: &SharedRwLockReadGuard) -> &StylesheetContents {
        debug_assert!(!self.inner().mContents.mRawPtr.is_null());
        unsafe {
            let contents =
                (&**StylesheetContents::as_arc(&&*self.inner().mContents.mRawPtr)) as *const _;
            &*contents
        }
    }

    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        use gecko_bindings::structs::ServoMediaList;
        use std::mem;

        unsafe {
            let servo_media_list =
                self.raw()._base.mMedia.mRawPtr as *const ServoMediaList;
            if servo_media_list.is_null() {
                return None;
            }
            let raw_list = &*(*servo_media_list).mRawList.mRawPtr;
            let list = Locked::<MediaList>::as_arc(mem::transmute(&raw_list));
            Some(list.read_with(guard))
        }
    }

    // All the stylesheets Servo knows about are enabled, because that state is
    // handled externally by Gecko.
    fn enabled(&self) -> bool {
        true
    }
}

#[derive(Default)]
/// Helper struct for counting traversals
pub struct TraversalCount {
    /// Total number of events
    pub total_count: AtomicUsize,
    /// Number of events which were parallel
    pub parallel_count: AtomicUsize
}

impl TraversalCount {
    fn record(&self, parallel: bool, count: u32) {
        self.total_count.fetch_add(count as usize, Ordering::Relaxed);
        if parallel {
            self.parallel_count.fetch_add(count as usize, Ordering::Relaxed);
        }
    }

    fn get(&self) -> (u32, u32) {
        (self.total_count.load(Ordering::Relaxed) as u32,
         self.parallel_count.load(Ordering::Relaxed) as u32)
    }
}

/// The container for data that a Servo-backed Gecko document needs to style
/// itself.
pub struct PerDocumentStyleDataImpl {
    /// Rule processor.
    pub stylist: Stylist,
    /// Counter for traversals that could have been parallel, for telemetry
    pub traversal_count: TraversalCount,
    /// Counter for traversals, weighted by elements traversed,
    pub traversal_count_traversed: TraversalCount,
    /// Counter for traversals, weighted by elements styled,
    pub traversal_count_styled: TraversalCount,
}

/// The data itself is an `AtomicRefCell`, which guarantees the proper semantics
/// and unexpected races while trying to mutate it.
pub struct PerDocumentStyleData(AtomicRefCell<PerDocumentStyleDataImpl>);

impl PerDocumentStyleData {
    /// Create a dummy `PerDocumentStyleData`.
    pub fn new(pres_context: RawGeckoPresContextOwned) -> Self {
        let device = Device::new(pres_context);

        // FIXME(emilio, tlin): How is this supposed to work with XBL? This is
        // right now not always honored, see bug 1405543...
        //
        // Should we just force XBL Stylists to be NoQuirks?
        let quirks_mode = unsafe {
            (*device.pres_context().mDocument.raw::<nsIDocument>()).mCompatMode
        };

        PerDocumentStyleData(AtomicRefCell::new(PerDocumentStyleDataImpl {
            stylist: Stylist::new(device, quirks_mode.into()),
            traversal_count: Default::default(),
            traversal_count_traversed: Default::default(),
            traversal_count_styled: Default::default(),
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

impl Drop for PerDocumentStyleDataImpl {
    fn drop(&mut self) {
        if !structs::GECKO_IS_NIGHTLY {
            return
        }
        let (total, parallel)  = self.traversal_count.get();
        let (total_t, parallel_t)  = self.traversal_count_traversed.get();
        let (total_s, parallel_s)  = self.traversal_count_styled.get();

        unsafe { bindings::Gecko_RecordTraversalStatistics(total, parallel,
                                                           total_t, parallel_t,
                                                           total_s, parallel_s) }
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
        self.stylist.flush(
            &StylesheetGuards::same(guard),
            document_element,
            snapshots,
        )
    }

    /// Returns whether private browsing is enabled.
    fn is_private_browsing_enabled(&self) -> bool {
        let doc =
            self.stylist.device().pres_context().mDocument.raw::<nsIDocument>();
        unsafe { bindings::Gecko_IsPrivateBrowsingEnabled(doc) }
    }

    /// Returns whether the document is being used as an image.
    fn is_being_used_as_an_image(&self) -> bool {
        let doc =
            self.stylist.device().pres_context().mDocument.raw::<nsIDocument>();

        unsafe { (*doc).mIsBeingUsedAsImage() }
    }

    /// Get the default computed values for this document.
    pub fn default_computed_values(&self) -> &Arc<ComputedValues> {
        self.stylist.device().default_computed_values_arc()
    }

    /// Returns whether visited links are enabled.
    fn visited_links_enabled(&self) -> bool {
        unsafe { structs::StylePrefs_sVisitedLinksEnabled }
    }

    /// Returns whether visited styles are enabled.
    pub fn visited_styles_enabled(&self) -> bool {
        if !self.visited_links_enabled() {
            return false;
        }

        if self.is_private_browsing_enabled() {
            return false;
        }

        if self.is_being_used_as_an_image() {
            return false;
        }

        true
    }

    /// Measure heap usage.
    pub fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        self.stylist.add_size_of(ops, sizes);
    }

    /// Record that a traversal happened for later collection as telemetry
    pub fn record_traversal(&self, was_parallel: bool, stats: Option<TraversalStatistics>) {
        self.traversal_count.record(was_parallel, 1);
        if let Some(stats) = stats {
            self.traversal_count_traversed.record(was_parallel, stats.elements_traversed);
            self.traversal_count_styled.record(was_parallel, stats.elements_styled);
        }
    }
}

unsafe impl HasFFI for PerDocumentStyleData {
    type FFIType = RawServoStyleSet;
}
unsafe impl HasSimpleFFI for PerDocumentStyleData {}
unsafe impl HasBoxFFI for PerDocumentStyleData {}
