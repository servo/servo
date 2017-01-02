/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element an snapshot common logic.

use gecko_bindings::structs::nsIAtom;
use std::{ptr, slice};
use string_cache::Atom;

/// A function that, given an element of type `T`, allows you to get a single
/// class or a class list.
pub type ClassOrClassList<T> = unsafe extern fn (T, *mut *mut nsIAtom, *mut *mut *mut nsIAtom) -> u32;

/// Given an item `T`, a class name, and a getter function, return whether that
/// element has the class that `name` represents.
pub fn has_class<T>(item: T,
                    name: &Atom,
                    getter: ClassOrClassList<T>) -> bool
{
    unsafe {
        let mut class: *mut nsIAtom = ptr::null_mut();
        let mut list: *mut *mut nsIAtom = ptr::null_mut();
        let length = getter(item, &mut class, &mut list);
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


/// Given an item, a callback, and a getter, execute `callback` for each class
/// this `item` has.
pub fn each_class<F, T>(item: T,
                        mut callback: F,
                        getter: ClassOrClassList<T>)
    where F: FnMut(&Atom)
{
    unsafe {
        let mut class: *mut nsIAtom = ptr::null_mut();
        let mut list: *mut *mut nsIAtom = ptr::null_mut();
        let length = getter(item, &mut class, &mut list);
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
