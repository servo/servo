/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed to style a Gecko document.

use Atom;
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use dom::TElement;
use fnv::FnvHashMap;
use gecko::rules::{CounterStyleRule, FontFaceRule};
use gecko_bindings::bindings::{self, RawServoStyleSet};
use gecko_bindings::structs::{ServoStyleSheet, StyleSheetInfo, ServoStyleSheetInner};
use gecko_bindings::structs::RawGeckoPresContextOwned;
use gecko_bindings::structs::nsIDocument;
use gecko_bindings::sugar::ownership::{HasArcFFI, HasBoxFFI, HasFFI, HasSimpleFFI};
use invalidation::media_queries::{MediaListKey, ToMediaListKey};
use media_queries::{Device, MediaList};
use properties::ComputedValuesInner;
use shared_lock::{Locked, StylesheetGuards, SharedRwLockReadGuard};
use stylearc::Arc;
use stylesheet_set::StylesheetSet;
use stylesheets::{Origin, StylesheetContents, StylesheetInDocument};
use stylist::{ExtraStyleData, Stylist};

/// Little wrapper to a Gecko style sheet.
#[derive(PartialEq, Eq, Debug)]
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

/// The container for data that a Servo-backed Gecko document needs to style
/// itself.
pub struct PerDocumentStyleDataImpl {
    /// Rule processor.
    pub stylist: Stylist,

    /// List of stylesheets, mirrored from Gecko.
    pub stylesheets: StylesheetSet<GeckoStyleSheet>,

    /// List of effective font face rules.
    pub font_faces: Vec<(Arc<Locked<FontFaceRule>>, Origin)>,

    /// Map for effective counter style rules.
    pub counter_styles: FnvHashMap<Atom, Arc<Locked<CounterStyleRule>>>,
}

/// The data itself is an `AtomicRefCell`, which guarantees the proper semantics
/// and unexpected races while trying to mutate it.
pub struct PerDocumentStyleData(AtomicRefCell<PerDocumentStyleDataImpl>);

impl PerDocumentStyleData {
    /// Create a dummy `PerDocumentStyleData`.
    pub fn new(pres_context: RawGeckoPresContextOwned) -> Self {
        let device = Device::new(pres_context);
        let quirks_mode = unsafe {
            (*device.pres_context().mDocument.raw::<nsIDocument>()).mCompatMode
        };

        PerDocumentStyleData(AtomicRefCell::new(PerDocumentStyleDataImpl {
            stylist: Stylist::new(device, quirks_mode.into()),
            stylesheets: StylesheetSet::new(),
            font_faces: vec![],
            counter_styles: FnvHashMap::default(),
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
    pub fn flush_stylesheets<E>(&mut self,
                                guard: &SharedRwLockReadGuard,
                                document_element: Option<E>)
        where E: TElement,
    {
        if !self.stylesheets.has_changed() {
            return;
        }

        let mut extra_data = ExtraStyleData {
            font_faces: &mut self.font_faces,
            counter_styles: &mut self.counter_styles,
        };

        let author_style_disabled = self.stylesheets.author_style_disabled();
        self.stylist.clear();
        let iter = self.stylesheets.flush(document_element);
        self.stylist.rebuild(
            iter,
            &StylesheetGuards::same(guard),
            /* ua_sheets = */ None,
            /* stylesheets_changed = */ true,
            author_style_disabled,
            &mut extra_data
        );
    }

    /// Get the default computed values for this document.
    pub fn default_computed_values(&self) -> &ComputedValuesInner {
        self.stylist.device().default_computed_values()
    }

    /// Clear the stylist.  This will be a no-op if the stylist is
    /// already cleared; the stylist handles that.
    pub fn clear_stylist(&mut self) {
        self.stylist.clear();
    }
}

unsafe impl HasFFI for PerDocumentStyleData {
    type FFIType = RawServoStyleSet;
}
unsafe impl HasSimpleFFI for PerDocumentStyleData {}
unsafe impl HasBoxFFI for PerDocumentStyleData {}
