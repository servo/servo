/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use types::cef_base_t;

use libc::{self, c_int, c_void, size_t};
use std::mem;
use std::slice;
use std::str;

/// Allows you to downcast a CEF interface to a CEF class instance.
///
/// FIXME(pcwalton): This is currently unsafe. I think the right way to make this safe is to (a)
/// forbid more than one Rust implementation of a given interface (easy to do by manufacturing an
/// impl that will conflict if there is more than one) and then (b) add a dynamic check to make
/// sure the `release` for the object is equal to `servo_release`.
pub trait Downcast<Class> {
    fn downcast(&self) -> &Class;
}

pub fn slice_to_str<F>(s: *const u8, l: usize, f: F) -> c_int where F: FnOnce(&str) -> c_int {
    unsafe {
        let s = slice::from_raw_parts(s, l);
        str::from_utf8(s).map(f).unwrap_or(0)
    }
}

/// Creates a new raw CEF object of the given type and sets up its reference counting machinery.
/// All fields are initialized to zero. It is the caller's responsibility to ensure that the given
/// type is a CEF type with `cef_base_t` as its first member.
pub unsafe fn create_cef_object<Base,Extra>(size: size_t) -> *mut Base {
    let object = libc::calloc(1, (mem::size_of::<Base>() + mem::size_of::<Extra>())) as
        *mut cef_base_t;
    (*object).size = size;
    (*object).add_ref = Some(servo_add_ref as extern "C" fn(*mut cef_base_t) -> c_int);
    (*object).release = Some(servo_release as extern "C" fn(*mut cef_base_t) -> c_int);
    *ref_count(object) = 1;
    object as *mut Base
}

/// Returns a pointer to the Servo-specific reference count for the given object. This only works
/// on objects that Servo created!
unsafe fn ref_count(object: *mut cef_base_t) -> *mut usize {
    // The reference count should be the first field of the extra data.
    (object as *mut u8).offset((*object).size as isize) as *mut usize
}

/// Increments the reference count on a CEF object. This only works on objects that Servo created!
extern "C" fn servo_add_ref(object: *mut cef_base_t) -> c_int {
    unsafe {
        let count = ref_count(object);
        *count += 1;
        *count as c_int
    }
}

/// Decrements the reference count on a CEF object. If zero, frees it. This only works on objects
/// that Servo created!
extern "C" fn servo_release(object: *mut cef_base_t) -> c_int {
    unsafe {
        let count = ref_count(object);
        *count -= 1;
        let new_count = *count;
        if new_count == 0 {
            servo_free(object);
        }
        (new_count == 0) as c_int
    }
}

unsafe fn servo_free(object: *mut cef_base_t) {
    libc::free(object as *mut c_void);
}

pub unsafe fn add_ref(c_object: *mut cef_base_t) {
    ((*c_object).add_ref.unwrap())(c_object);
}

#[no_mangle]
pub extern "C" fn servo_test() -> c_int {
    1
}
