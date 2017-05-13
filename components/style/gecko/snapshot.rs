/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A gecko snapshot, that stores the element attributes and state before they
//! change in order to properly calculate restyle hints.

use dom::TElement;
use element_state::ElementState;
use gecko::snapshot_helpers;
use gecko::wrapper::{AttrSelectorHelpers, GeckoElement};
use gecko_bindings::bindings;
use gecko_bindings::structs::ServoElementSnapshot;
use gecko_bindings::structs::ServoElementSnapshotFlags as Flags;
use gecko_bindings::structs::ServoElementSnapshotTable;
use restyle_hints::ElementSnapshot;
use selector_parser::SelectorImpl;
use selectors::parser::AttrSelector;
use string_cache::Atom;

/// A snapshot of a Gecko element.
pub type GeckoElementSnapshot = ServoElementSnapshot;

/// A map from elements to snapshots for Gecko's style back-end.
pub type SnapshotMap = ServoElementSnapshotTable;

impl SnapshotMap {
    /// Gets the snapshot for this element, if any.
    ///
    /// FIXME(emilio): The transmute() business we do here is kind of nasty, but
    /// it's a consequence of the map being a OpaqueNode -> Snapshot table in
    /// Servo and an Element -> Snapshot table in Gecko.
    ///
    /// We should be able to make this a more type-safe with type annotations by
    /// making SnapshotMap a trait and moving the implementations outside, but
    /// that's a pain because it implies parameterizing SharedStyleContext.
    pub fn get<E: TElement>(&self, element: &E) -> Option<&GeckoElementSnapshot> {
        debug_assert!(element.has_snapshot());

        unsafe {
            let element =
                unsafe { ::std::mem::transmute::<&E, &GeckoElement>(element) };

            bindings::Gecko_GetElementSnapshot(self, element.0).as_ref()
        }
    }
}

impl GeckoElementSnapshot {
    #[inline]
    fn is_html_element_in_html_document(&self) -> bool {
        self.mIsHTMLElementInHTMLDocument
    }

    #[inline]
    fn has_any(&self, flags: Flags) -> bool {
        (self.mContains as u8 & flags as u8) != 0
    }

    fn as_ptr(&self) -> *const Self {
        self
    }
}

impl ::selectors::MatchAttr for GeckoElementSnapshot {
    type Impl = SelectorImpl;

    fn match_attr_has(&self, attr: &AttrSelector<SelectorImpl>) -> bool {
        unsafe {
            bindings::Gecko_SnapshotHasAttr(self,
                                            attr.ns_or_null(),
                                            attr.select_name(self.is_html_element_in_html_document()))
        }
    }

    fn match_attr_equals(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrEquals(self,
                                               attr.ns_or_null(),
                                               attr.select_name(self.is_html_element_in_html_document()),
                                               value.as_ptr(),
                                               /* ignoreCase = */ false)
        }
    }

    fn match_attr_equals_ignore_ascii_case(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrEquals(self,
                                               attr.ns_or_null(),
                                               attr.select_name(self.is_html_element_in_html_document()),
                                               value.as_ptr(),
                                               /* ignoreCase = */ true)
        }
    }
    fn match_attr_includes(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrIncludes(self,
                                                 attr.ns_or_null(),
                                                 attr.select_name(self.is_html_element_in_html_document()),
                                                 value.as_ptr())
        }
    }
    fn match_attr_dash(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrDashEquals(self,
                                                   attr.ns_or_null(),
                                                   attr.select_name(self.is_html_element_in_html_document()),
                                                   value.as_ptr())
        }
    }
    fn match_attr_prefix(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasPrefix(self,
                                                  attr.ns_or_null(),
                                                  attr.select_name(self.is_html_element_in_html_document()),
                                                  value.as_ptr())
        }
    }
    fn match_attr_substring(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasSubstring(self,
                                                     attr.ns_or_null(),
                                                     attr.select_name(self.is_html_element_in_html_document()),
                                                     value.as_ptr())
        }
    }
    fn match_attr_suffix(&self, attr: &AttrSelector<SelectorImpl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasSuffix(self,
                                                  attr.ns_or_null(),
                                                  attr.select_name(self.is_html_element_in_html_document()),
                                                  value.as_ptr())
        }
    }
}

impl ElementSnapshot for GeckoElementSnapshot {
    fn state(&self) -> Option<ElementState> {
        if self.has_any(Flags::State) {
            Some(ElementState::from_bits_truncate(self.mState))
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
            bindings::Gecko_SnapshotAtomAttrValue(self,
                                                  atom!("id").as_ptr())
        };

        if ptr.is_null() {
            None
        } else {
            Some(Atom::from(ptr))
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        snapshot_helpers::has_class(self.as_ptr(),
                                    name,
                                    bindings::Gecko_SnapshotClassOrClassList)
    }

    fn each_class<F>(&self, callback: F)
        where F: FnMut(&Atom)
    {
        snapshot_helpers::each_class(self.as_ptr(),
                                     callback,
                                     bindings::Gecko_SnapshotClassOrClassList)
    }
}
