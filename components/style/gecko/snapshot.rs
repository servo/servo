/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A gecko snapshot, that stores the element attributes and state before they
//! change in order to properly calculate restyle hints.

use dom::TElement;
use element_state::ElementState;
use gecko::snapshot_helpers;
use gecko::wrapper::{NamespaceConstraintHelpers, GeckoElement};
use gecko_bindings::bindings;
use gecko_bindings::structs::ServoElementSnapshot;
use gecko_bindings::structs::ServoElementSnapshotFlags as Flags;
use gecko_bindings::structs::ServoElementSnapshotTable;
use restyle_hints::ElementSnapshot;
use selectors::attr::{AttrSelectorOperation, AttrSelectorOperator, CaseSensitivity, NamespaceConstraint};
use string_cache::{Atom, Namespace};

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
    fn has_any(&self, flags: Flags) -> bool {
        (self.mContains as u8 & flags as u8) != 0
    }

    fn as_ptr(&self) -> *const Self {
        self
    }

    /// Returns true if the snapshot has stored state for pseudo-classes
    /// that depend on things other than `ElementState`.
    #[inline]
    pub fn has_other_pseudo_class_state(&self) -> bool {
        self.has_any(Flags::OtherPseudoClassState)
    }

    /// selectors::Element::attr_matches
    pub fn attr_matches(&self,
                        ns: &NamespaceConstraint<&Namespace>,
                        local_name: &Atom,
                        operation: &AttrSelectorOperation<&Atom>)
                        -> bool {
        unsafe {
            match *operation {
                AttrSelectorOperation::Exists => {
                    bindings:: Gecko_SnapshotHasAttr(self,
                                                     ns.atom_or_null(),
                                                     local_name.as_ptr())
                }
                AttrSelectorOperation::WithValue { operator, case_sensitivity, expected_value } => {
                    let ignore_case = match case_sensitivity {
                        CaseSensitivity::CaseSensitive => false,
                        CaseSensitivity::AsciiCaseInsensitive => true,
                    };
                    // FIXME: case sensitivity for operators other than Equal
                    match operator {
                        AttrSelectorOperator::Equal => bindings::Gecko_SnapshotAttrEquals(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case
                        ),
                        AttrSelectorOperator::Includes => bindings::Gecko_SnapshotAttrIncludes(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                        ),
                        AttrSelectorOperator::DashMatch => bindings::Gecko_SnapshotAttrDashEquals(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                        ),
                        AttrSelectorOperator::Prefix => bindings::Gecko_SnapshotAttrHasPrefix(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                        ),
                        AttrSelectorOperator::Suffix => bindings::Gecko_SnapshotAttrHasSuffix(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                        ),
                        AttrSelectorOperator::Substring => bindings::Gecko_SnapshotAttrHasSubstring(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                        ),
                    }
                }
            }
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

    #[inline]
    fn id_attr(&self) -> Option<Atom> {
        if !self.has_any(Flags::Id) {
            return None
        }

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

    #[inline]
    fn has_class(&self, name: &Atom) -> bool {
        if !self.has_any(Flags::MaybeClass) {
            return false;
        }

        snapshot_helpers::has_class(self.as_ptr(),
                                    name,
                                    bindings::Gecko_SnapshotClassOrClassList)
    }

    #[inline]
    fn each_class<F>(&self, callback: F)
        where F: FnMut(&Atom)
    {
        if !self.has_any(Flags::MaybeClass) {
            return;
        }

        snapshot_helpers::each_class(self.as_ptr(),
                                     callback,
                                     bindings::Gecko_SnapshotClassOrClassList)
    }
}
