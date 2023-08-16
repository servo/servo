/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Element an snapshot common logic.

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::{self, nsAtom};
use crate::string_cache::WeakAtom;
use crate::values::AtomIdent;
use crate::Atom;
use crate::CaseSensitivityExt;
use selectors::attr::CaseSensitivity;

/// A function that, given an element of type `T`, allows you to get a single
/// class or a class list.
enum Class<'a> {
    None,
    One(*const nsAtom),
    More(&'a [structs::RefPtr<nsAtom>]),
}

#[inline(always)]
fn base_type(attr: &structs::nsAttrValue) -> structs::nsAttrValue_ValueBaseType {
    (attr.mBits & structs::NS_ATTRVALUE_BASETYPE_MASK) as structs::nsAttrValue_ValueBaseType
}

#[inline(always)]
unsafe fn ptr<T>(attr: &structs::nsAttrValue) -> *const T {
    (attr.mBits & !structs::NS_ATTRVALUE_BASETYPE_MASK) as *const T
}

#[inline(always)]
unsafe fn get_class_or_part_from_attr(attr: &structs::nsAttrValue) -> Class {
    debug_assert!(bindings::Gecko_AssertClassAttrValueIsSane(attr));
    let base_type = base_type(attr);
    if base_type == structs::nsAttrValue_ValueBaseType_eAtomBase {
        return Class::One(ptr::<nsAtom>(attr));
    }
    if base_type == structs::nsAttrValue_ValueBaseType_eOtherBase {
        let container = ptr::<structs::MiscContainer>(attr);
        debug_assert_eq!(
            (*container).mType,
            structs::nsAttrValue_ValueType_eAtomArray
        );
        // NOTE: Bindgen doesn't deal with AutoTArray, so cast it below.
        let array: *mut u8 = *(*container)
            .__bindgen_anon_1
            .mValue
            .as_ref()
            .__bindgen_anon_1
            .mAtomArray
            .as_ref();
        let array = array as *const structs::nsTArray<structs::RefPtr<nsAtom>>;
        return Class::More(&**array);
    }
    debug_assert_eq!(base_type, structs::nsAttrValue_ValueBaseType_eStringBase);
    Class::None
}

#[inline(always)]
unsafe fn get_id_from_attr(attr: &structs::nsAttrValue) -> &WeakAtom {
    debug_assert_eq!(
        base_type(attr),
        structs::nsAttrValue_ValueBaseType_eAtomBase
    );
    WeakAtom::new(ptr::<nsAtom>(attr))
}

/// Find an attribute value with a given name and no namespace.
#[inline(always)]
pub fn find_attr<'a>(
    attrs: &'a [structs::AttrArray_InternalAttr],
    name: &Atom,
) -> Option<&'a structs::nsAttrValue> {
    attrs
        .iter()
        .find(|attr| attr.mName.mBits == name.as_ptr() as usize)
        .map(|attr| &attr.mValue)
}

/// Finds the id attribute from a list of attributes.
#[inline(always)]
pub fn get_id(attrs: &[structs::AttrArray_InternalAttr]) -> Option<&WeakAtom> {
    Some(unsafe { get_id_from_attr(find_attr(attrs, &atom!("id"))?) })
}

#[inline(always)]
pub(super) fn each_exported_part(
    attrs: &[structs::AttrArray_InternalAttr],
    name: &AtomIdent,
    mut callback: impl FnMut(&AtomIdent),
) {
    let attr = match find_attr(attrs, &atom!("exportparts")) {
        Some(attr) => attr,
        None => return,
    };
    let mut length = 0;
    let atoms = unsafe { bindings::Gecko_Element_ExportedParts(attr, name.as_ptr(), &mut length) };
    if atoms.is_null() {
        return;
    }

    unsafe {
        for atom in std::slice::from_raw_parts(atoms, length) {
            AtomIdent::with(*atom, &mut callback)
        }
    }
}

#[inline(always)]
pub(super) fn imported_part(
    attrs: &[structs::AttrArray_InternalAttr],
    name: &AtomIdent,
) -> Option<AtomIdent> {
    let attr = find_attr(attrs, &atom!("exportparts"))?;
    let atom = unsafe { bindings::Gecko_Element_ImportedPart(attr, name.as_ptr()) };
    if atom.is_null() {
        return None;
    }
    Some(AtomIdent(unsafe { Atom::from_raw(atom) }))
}

/// Given a class or part name, a case sensitivity, and an array of attributes,
/// returns whether the attribute has that name.
#[inline(always)]
pub fn has_class_or_part(
    name: &AtomIdent,
    case_sensitivity: CaseSensitivity,
    attr: &structs::nsAttrValue,
) -> bool {
    match unsafe { get_class_or_part_from_attr(attr) } {
        Class::None => false,
        Class::One(atom) => unsafe { case_sensitivity.eq_atom(name, WeakAtom::new(atom)) },
        Class::More(atoms) => match case_sensitivity {
            CaseSensitivity::CaseSensitive => {
                let name_ptr = name.as_ptr();
                atoms.iter().any(|atom| atom.mRawPtr == name_ptr)
            },
            CaseSensitivity::AsciiCaseInsensitive => unsafe {
                atoms
                    .iter()
                    .any(|atom| WeakAtom::new(atom.mRawPtr).eq_ignore_ascii_case(name))
            },
        },
    }
}

/// Given an item, a callback, and a getter, execute `callback` for each class
/// or part name this `item` has.
#[inline(always)]
pub fn each_class_or_part<F>(attr: &structs::nsAttrValue, mut callback: F)
where
    F: FnMut(&AtomIdent),
{
    unsafe {
        match get_class_or_part_from_attr(attr) {
            Class::None => {},
            Class::One(atom) => AtomIdent::with(atom, callback),
            Class::More(atoms) => {
                for atom in atoms {
                    AtomIdent::with(atom.mRawPtr, &mut callback)
                }
            },
        }
    }
}
