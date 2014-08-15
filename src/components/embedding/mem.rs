/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::{c_void, size_t};
use std::mem;
use std::ptr::set_memory;

extern "C" {
    fn tc_new(size: size_t) -> *mut c_void;
    fn tc_delete(mem: *mut c_void);
    fn tc_newarray(size: size_t) -> *mut c_void;
    fn tc_deletearray(mem: *mut c_void);
}

#[allow(experimental)]
pub fn newarray0<T>(nmem: size_t) -> *mut T {
    let mem = newarray::<T>(nmem) as *mut T;
    unsafe {
        set_memory(mem, 0 as u8, nmem as uint);
    }
    mem
}

pub fn newarray<T>(nmem: size_t) -> *mut T {
    unsafe {
        tc_newarray(nmem * mem::size_of::<T>() as size_t) as *mut T
    }
}

#[allow(experimental)]
pub fn new0<T>(nmem: size_t) -> *mut T {
    let mem = new(nmem * mem::size_of::<T>() as size_t) as *mut T;
    unsafe {
        set_memory(mem, 0 as u8, nmem as uint);
    }
    mem
}

pub fn new(size: size_t) -> *mut c_void {
    unsafe {
        tc_new(size)
    }
}

pub fn delete(mem: *mut c_void) {
    unsafe {
        tc_delete(mem)
    }
}

pub fn deletearray(mem: *mut c_void) {
    unsafe {
        tc_deletearray(mem)
    }
}
