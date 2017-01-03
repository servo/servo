/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A gecko snapshot, that stores the element attributes and state before they
//! change in order to properly calculate restyle hints.

use element_state::ElementState;
use gecko::snapshot_helpers;
use gecko::wrapper::{AttrSelectorHelpers, GeckoElement};
use gecko_bindings::bindings;
use gecko_bindings::structs::ServoElementSnapshot;
use gecko_bindings::structs::ServoElementSnapshotFlags as Flags;
use restyle_hints::ElementSnapshot;
use selector_parser::SelectorImpl;
use selectors::parser::AttrSelector;
use std::ptr;
use string_cache::Atom;

/// A snapshot of a Gecko element.
///
/// This is really a Gecko type (see `ServoElementSnapshot.h` in Gecko) we wrap
/// here.
#[derive(Debug)]
pub struct GeckoElementSnapshot(bindings::ServoElementSnapshotOwned);

// FIXME(bholley): Add support for *OwnedConst type, and then we get Sync
// automatically.
unsafe impl Sync for GeckoElementSnapshot {}

impl Drop for GeckoElementSnapshot {
    fn drop(&mut self) {
        unsafe {
            bindings::Gecko_DropElementSnapshot(ptr::read(&self.0 as *const _));
        }
    }
}

impl GeckoElementSnapshot {
    /// Create a new snapshot of the given element.
    pub fn new<'le>(el: GeckoElement<'le>) -> Self {
        unsafe { GeckoElementSnapshot(bindings::Gecko_CreateElementSnapshot(el.0)) }
    }

    /// Get a mutable reference to the snapshot.
    pub fn borrow_mut_raw(&mut self) -> bindings::ServoElementSnapshotBorrowedMut {
        &mut *self.0
    }

    /// Get the pointer to the actual snapshot.
    pub fn ptr(&self) -> *const ServoElementSnapshot {
        &*self.0
    }

    #[inline]
    fn is_html_element_in_html_document(&self) -> bool {
        unsafe { (*self.0).mIsHTMLElementInHTMLDocument }
    }

    #[inline]
    fn has_any(&self, flags: Flags) -> bool {
        unsafe { ((*self.0).mContains as u8 & flags as u8) != 0 }
    }
}

impl ::selectors::MatchAttr for GeckoElementSnapshot {
    type Impl = SelectorImpl;

    fn match_attr_has(&self, attr: &AttrSelector<SelectorImpl>) -> bool {
        unsafe {
            bindings::Gecko_SnapshotHasAttr(self.ptr(),
                                            attr.ns_or_null(),
                                            attr.select_name(self.is_html_element_in_html_document()))
        }
    }

    fn match_attr_equals(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrEquals(self.ptr(),
                                               attr.ns_or_null(),
                                               attr.select_name(self.is_html_element_in_html_document()),
                                               value.as_ptr(),
                                               /* ignoreCase = */ false)
        }
    }

    fn match_attr_equals_ignore_ascii_case(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrEquals(self.ptr(),
                                               attr.ns_or_null(),
                                               attr.select_name(self.is_html_element_in_html_document()),
                                               value.as_ptr(),
                                               /* ignoreCase = */ true)
        }
    }
    fn match_attr_includes(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrIncludes(self.ptr(),
                                                 attr.ns_or_null(),
                                                 attr.select_name(self.is_html_element_in_html_document()),
                                                 value.as_ptr())
        }
    }
    fn match_attr_dash(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrDashEquals(self.ptr(),
                                                   attr.ns_or_null(),
                                                   attr.select_name(self.is_html_element_in_html_document()),
                                                   value.as_ptr())
        }
    }
    fn match_attr_prefix(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasPrefix(self.ptr(),
                                                  attr.ns_or_null(),
                                                  attr.select_name(self.is_html_element_in_html_document()),
                                                  value.as_ptr())
        }
    }
    fn match_attr_substring(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasSubstring(self.ptr(),
                                                     attr.ns_or_null(),
                                                     attr.select_name(self.is_html_element_in_html_document()),
                                                     value.as_ptr())
        }
    }
    fn match_attr_suffix(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasSuffix(self.ptr(),
                                                  attr.ns_or_null(),
                                                  attr.select_name(self.is_html_element_in_html_document()),
                                                  value.as_ptr())
        }
    }
}

impl ElementSnapshot for GeckoElementSnapshot {
    fn state(&self) -> Option<ElementState> {
        if self.has_any(Flags::State) {
            Some(ElementState::from_bits_truncate(unsafe { (*self.0).mState as u16 }))
        } else {
            None
        }
    }

    #[inline]
    fn has_attrs(&self) -> bool {
        self.has_any(Flags::Attributes)
    }

    fn id_attr(&self) -> Option<Atom> {
        let ptr = unsafe {
            bindings::Gecko_SnapshotAtomAttrValue(self.ptr(),
                                                  atom!("id").as_ptr())
        };

        if ptr.is_null() {
            None
        } else {
            Some(Atom::from(ptr))
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        snapshot_helpers::has_class(self.ptr(),
                                    name,
                                    bindings::Gecko_SnapshotClassOrClassList)
    }

    fn each_class<F>(&self, callback: F)
        where F: FnMut(&Atom)
    {
        snapshot_helpers::each_class(self.ptr(),
                                     callback,
                                     bindings::Gecko_SnapshotClassOrClassList)
    }
}
