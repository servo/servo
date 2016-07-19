/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use gecko_bindings::bindings;
use gecko_bindings::structs::ServoElementSnapshot;
use gecko_bindings::structs::ServoElementSnapshot_Flags_InternalTypeEnum as Flags;
use gecko_bindings::structs::nsIAtom;
use selectors::parser::AttrSelector;
use std::{ptr, slice};
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

    fn is_html_element_in_html_document(&self) -> bool {
        unsafe { (*self.0).mIsHTMLElementInHTMLDocument }
    }

    fn has_any(&self, flags: Flags) -> bool {
        unsafe { (*self.0).mContains.mInternal & flags as u8 != 0 }
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
                                               /* ignoreCase = */ false)
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

    fn has_attrs(&self) -> bool {
        self.has_any(Flags::Attributes)
    }

    fn id_attr(&self) -> Option<Atom> {
        let value = unsafe {
            bindings::Gecko_SnapshotAtomAttrValue(self.0, atom!("id").as_ptr())
        };

        if value.is_null() {
            None
        } else {
            Some(Atom::from(value))
        }
    }

    // TODO: share logic with Element::{has_class, each_class}?
    fn has_class(&self, name: &Atom) -> bool {
        unsafe {
            let mut class: *mut nsIAtom = ptr::null_mut();
            let mut list: *mut *mut nsIAtom = ptr::null_mut();
            let length = bindings::Gecko_SnapshotClassOrClassList(self.0,
                                                                  &mut class,
                                                                  &mut list);
            match length {
                0 => false,
                1 => name.as_ptr() == class,
                n => {
                    let classes = slice::from_raw_parts(list, n as usize);
                    classes.iter().any(|ptr| name.as_ptr() == *ptr)
                }
            }
        }
    }

    fn each_class<F>(&self, mut callback: F)
        where F: FnMut(&Atom)
    {
        unsafe {
            let mut class: *mut nsIAtom = ptr::null_mut();
            let mut list: *mut *mut nsIAtom = ptr::null_mut();
            let length = bindings::Gecko_SnapshotClassOrClassList(self.0,
                                                                  &mut class,
                                                                  &mut list);
            match length {
                0 => {}
                1 => Atom::with(class, &mut callback),
                n => {
                    let classes = slice::from_raw_parts(list, n as usize);
                    for c in classes {
                        Atom::with(*c, &mut callback)
                    }
                }
            }
        }
    }
}
