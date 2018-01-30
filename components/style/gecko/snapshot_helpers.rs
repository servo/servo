/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element an snapshot common logic.

use CaseSensitivityExt;
use gecko_bindings::structs::nsAtom;
use gecko_string_cache::WeakAtom;
use selectors::attr::CaseSensitivity;
use std::{ptr, slice};
use string_cache::Atom;

/// A function that, given an element of type `T`, allows you to get a single
/// class or a class list.
pub type ClassOrClassList<T> = unsafe extern fn (T, *mut *mut nsAtom, *mut *mut *mut nsAtom) -> u32;

/// Given an item `T`, a class name, and a getter function, return whether that
/// element has the class that `name` represents.
pub fn has_class<T>(
    item: T,
    name: &Atom,
    case_sensitivity: CaseSensitivity,
    getter: ClassOrClassList<T>,
) -> bool {
    unsafe {
        let mut class: *mut nsAtom = ptr::null_mut();
        let mut list: *mut *mut nsAtom = ptr::null_mut();
        let length = getter(item, &mut class, &mut list);
        match length {
            0 => false,
            1 => case_sensitivity.eq_atom(name, WeakAtom::new(class)),
            n => {
                let classes = slice::from_raw_parts(list, n as usize);
                match case_sensitivity {
                    CaseSensitivity::CaseSensitive => {
                        classes.iter().any(|ptr| &**name == WeakAtom::new(*ptr))
                    }
                    CaseSensitivity::AsciiCaseInsensitive => {
                        classes.iter().any(|ptr| name.eq_ignore_ascii_case(WeakAtom::new(*ptr)))
                    }
                }
            }
        }
    }
}


/// Given an item, a callback, and a getter, execute `callback` for each class
/// this `item` has.
pub fn each_class<F, T>(
    item: T,
    mut callback: F,
    getter: ClassOrClassList<T>,
)
where
    F: FnMut(&Atom)
{
    unsafe {
        let mut class: *mut nsAtom = ptr::null_mut();
        let mut list: *mut *mut nsAtom = ptr::null_mut();
        let length = getter(item, &mut class, &mut list);
        match length {
            0 => {}
            1 => Atom::with(class, callback),
            n => {
                let classes = slice::from_raw_parts(list, n as usize);
                for c in classes {
                    Atom::with(*c, &mut callback)
                }
            }
        }
    }
}
