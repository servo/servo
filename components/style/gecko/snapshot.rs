/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A gecko snapshot, that stores the element attributes and state before they
//! change in order to properly calculate restyle hints.

use crate::dom::TElement;
use crate::element_state::ElementState;
use crate::gecko::snapshot_helpers;
use crate::gecko::wrapper::{GeckoElement, NamespaceConstraintHelpers};
use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::ServoElementSnapshot;
use crate::gecko_bindings::structs::ServoElementSnapshotFlags as Flags;
use crate::gecko_bindings::structs::ServoElementSnapshotTable;
use crate::invalidation::element::element_wrapper::ElementSnapshot;
use crate::string_cache::{Atom, Namespace};
use crate::WeakAtom;
use selectors::attr::{AttrSelectorOperation, AttrSelectorOperator};
use selectors::attr::{CaseSensitivity, NamespaceConstraint};

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
            let element = ::std::mem::transmute::<&E, &GeckoElement>(element);
            bindings::Gecko_GetElementSnapshot(self, element.0).as_ref()
        }
    }
}

impl GeckoElementSnapshot {
    #[inline]
    fn has_any(&self, flags: Flags) -> bool {
        (self.mContains as u8 & flags as u8) != 0
    }

    /// Returns true if the snapshot has stored state for pseudo-classes
    /// that depend on things other than `ElementState`.
    #[inline]
    pub fn has_other_pseudo_class_state(&self) -> bool {
        self.has_any(Flags::OtherPseudoClassState)
    }

    /// Returns true if the snapshot recorded an id change.
    #[inline]
    pub fn id_changed(&self) -> bool {
        self.mIdAttributeChanged()
    }

    /// Returns true if the snapshot recorded a class attribute change.
    #[inline]
    pub fn class_changed(&self) -> bool {
        self.mClassAttributeChanged()
    }

    /// Returns true if the snapshot recorded an attribute change which isn't a
    /// class or id change.
    #[inline]
    pub fn other_attr_changed(&self) -> bool {
        self.mOtherAttributeChanged()
    }

    /// selectors::Element::attr_matches
    pub fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Namespace>,
        local_name: &Atom,
        operation: &AttrSelectorOperation<&Atom>,
    ) -> bool {
        unsafe {
            match *operation {
                AttrSelectorOperation::Exists => {
                    bindings::Gecko_SnapshotHasAttr(self, ns.atom_or_null(), local_name.as_ptr())
                },
                AttrSelectorOperation::WithValue {
                    operator,
                    case_sensitivity,
                    expected_value,
                } => {
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
                            ignore_case,
                        ),
                        AttrSelectorOperator::Includes => bindings::Gecko_SnapshotAttrIncludes(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::DashMatch => bindings::Gecko_SnapshotAttrDashEquals(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::Prefix => bindings::Gecko_SnapshotAttrHasPrefix(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::Suffix => bindings::Gecko_SnapshotAttrHasSuffix(
                            self,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::Substring => {
                            bindings::Gecko_SnapshotAttrHasSubstring(
                                self,
                                ns.atom_or_null(),
                                local_name.as_ptr(),
                                expected_value.as_ptr(),
                                ignore_case,
                            )
                        },
                    }
                },
            }
        }
    }
}

impl ElementSnapshot for GeckoElementSnapshot {
    fn debug_list_attributes(&self) -> String {
        use nsstring::nsCString;
        let mut string = nsCString::new();
        unsafe {
            bindings::Gecko_Snapshot_DebugListAttributes(self, &mut string);
        }
        String::from_utf8_lossy(&*string).into_owned()
    }

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
    fn id_attr(&self) -> Option<&WeakAtom> {
        if !self.has_any(Flags::Id) {
            return None;
        }

        snapshot_helpers::get_id(&*self.mAttrs)
    }

    #[inline]
    fn is_part(&self, name: &Atom) -> bool {
        let attr = match snapshot_helpers::find_attr(&*self.mAttrs, &atom!("part")) {
            Some(attr) => attr,
            None => return false,
        };

        snapshot_helpers::has_class_or_part(name, CaseSensitivity::CaseSensitive, attr)
    }

    #[inline]
    fn has_class(&self, name: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        if !self.has_any(Flags::MaybeClass) {
            return false;
        }

        snapshot_helpers::has_class_or_part(name, case_sensitivity, &self.mClass)
    }

    #[inline]
    fn each_class<F>(&self, callback: F)
    where
        F: FnMut(&Atom),
    {
        if !self.has_any(Flags::MaybeClass) {
            return;
        }

        snapshot_helpers::each_class(&self.mClass, callback)
    }

    #[inline]
    fn lang_attr(&self) -> Option<Atom> {
        let ptr = unsafe { bindings::Gecko_SnapshotLangValue(self) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Atom::from_addrefed(ptr) })
        }
    }
}
