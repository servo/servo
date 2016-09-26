/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element an snapshot common logic.

use gecko_bindings::structs::nsIAtom;
use std::{ptr, slice};
use string_cache::Atom;

pub type ClassOrClassList<T> = unsafe extern fn (T, *mut *mut nsIAtom, *mut *mut *mut nsIAtom) -> u32;

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
