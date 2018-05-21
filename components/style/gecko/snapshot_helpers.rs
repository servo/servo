/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element an snapshot common logic.

use gecko_bindings::structs::nsAtom;
use selectors::attr::CaseSensitivity;
use std::{ptr, slice};
use string_cache::Atom;

/// A function that, given an element of type `T`, allows you to get a single
/// class or a class list.
pub type ClassOrClassList<T> =
    unsafe extern "C" fn(T, *mut *mut nsAtom, *mut *mut *mut nsAtom) -> u32;

/// A function to return whether an element of type `T` has a given class.
///
/// The `bool` argument represents whether it should compare case-insensitively
/// or not.
pub type HasClass<T> = unsafe extern "C" fn(T, *mut nsAtom, bool) -> bool;

/// Given an item `T`, a class name, and a getter function, return whether that
/// element has the class that `name` represents.
#[inline(always)]
pub fn has_class<T>(
    item: T,
    name: &Atom,
    case_sensitivity: CaseSensitivity,
    getter: HasClass<T>,
) -> bool {
    let ignore_case = match case_sensitivity {
        CaseSensitivity::CaseSensitive => false,
        CaseSensitivity::AsciiCaseInsensitive => true,
    };

    unsafe { getter(item, name.as_ptr(), ignore_case) }
}

/// Given an item, a callback, and a getter, execute `callback` for each class
/// this `item` has.
pub fn each_class<F, T>(item: T, mut callback: F, getter: ClassOrClassList<T>)
where
    F: FnMut(&Atom),
{
    unsafe {
        let mut class: *mut nsAtom = ptr::null_mut();
        let mut list: *mut *mut nsAtom = ptr::null_mut();
        let length = getter(item, &mut class, &mut list);
        match length {
            0 => {},
            1 => Atom::with(class, callback),
            n => {
                let classes = slice::from_raw_parts(list, n as usize);
                for c in classes {
                    Atom::with(*c, &mut callback)
                }
            },
        }
    }
}
