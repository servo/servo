/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use gecko_bindings::bindings;
use gecko_bindings::structs::ServoElementSnapshot;
use gecko_bindings::structs::ServoElementSnapshotFlags as Flags;
use selectors::parser::AttrSelector;
use snapshot_helpers;
use string_cache::Atom;
use style::element_state::ElementState;
use style::restyle_hints::ElementSnapshot;
use wrapper::AttrSelectorHelpers;

// NB: This is sound, in some sense, because during computation of restyle hints
// the snapshot is kept alive by the modified elements table.
#[derive(Debug)]
pub struct GeckoElementSnapshot(*mut ServoElementSnapshot);

impl GeckoElementSnapshot {
    #[inline]
    pub unsafe fn from_raw(raw: *mut ServoElementSnapshot) -> Self {
        GeckoElementSnapshot(raw)
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
    type AttrString = Atom;

    fn match_attr_has(&self, attr: &AttrSelector) -> bool {
        unsafe {
            bindings::Gecko_SnapshotHasAttr(self.0,
                                            attr.ns_or_null(),
                                            attr.select_name(self.is_html_element_in_html_document()))
        }
    }

    fn match_attr_equals(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrEquals(self.0,
                                               attr.ns_or_null(),
                                               attr.select_name(self.is_html_element_in_html_document()),
                                               value.as_ptr(),
                                               /* ignoreCase = */ false)
        }
    }

    fn match_attr_equals_ignore_ascii_case(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrEquals(self.0,
                                               attr.ns_or_null(),
                                               attr.select_name(self.is_html_element_in_html_document()),
                                               value.as_ptr(),
                                               /* ignoreCase = */ true)
        }
    }
    fn match_attr_includes(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrIncludes(self.0,
                                                 attr.ns_or_null(),
                                                 attr.select_name(self.is_html_element_in_html_document()),
                                                 value.as_ptr())
        }
    }
    fn match_attr_dash(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrDashEquals(self.0,
                                                   attr.ns_or_null(),
                                                   attr.select_name(self.is_html_element_in_html_document()),
                                                   value.as_ptr())
        }
    }
    fn match_attr_prefix(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasPrefix(self.0,
                                                  attr.ns_or_null(),
                                                  attr.select_name(self.is_html_element_in_html_document()),
                                                  value.as_ptr())
        }
    }
    fn match_attr_substring(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasSubstring(self.0,
                                                     attr.ns_or_null(),
                                                     attr.select_name(self.is_html_element_in_html_document()),
                                                     value.as_ptr())
        }
    }
    fn match_attr_suffix(&self, attr: &AttrSelector, value: &Self::AttrString) -> bool {
        unsafe {
            bindings::Gecko_SnapshotAttrHasSuffix(self.0,
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
            bindings::Gecko_SnapshotAtomAttrValue(self.0,
                                                  atom!("id").as_ptr())
        };

        if ptr.is_null() {
            None
        } else {
            Some(Atom::from(ptr))
        }
    }

    // TODO: share logic with Element::{has_class, each_class}?
    fn has_class(&self, name: &Atom) -> bool {
        snapshot_helpers::has_class(self.0,
                                    name,
                                    bindings::Gecko_SnapshotClassOrClassList)
    }

    fn each_class<F>(&self, callback: F)
        where F: FnMut(&Atom)
    {
        snapshot_helpers::each_class(self.0,
                                     callback,
                                     bindings::Gecko_SnapshotClassOrClassList)
    }
}
