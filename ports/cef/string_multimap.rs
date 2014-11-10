/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::slice_to_str;
use libc::{c_int};
use std::collections::TreeMap;
use std::mem;
use std::string::String;
use string::{cef_string_userfree_utf8_alloc,cef_string_userfree_utf8_free,cef_string_utf8_set};
use types::{cef_string_multimap_t,cef_string_t};

fn string_multimap_to_treemap(smm: *mut cef_string_multimap_t) -> *mut TreeMap<String, Vec<*mut cef_string_t>> {
    smm as *mut TreeMap<String, Vec<*mut cef_string_t>>
}

//cef_string_multimap

#[no_mangle]
pub extern "C" fn cef_string_multimap_alloc() -> *mut cef_string_multimap_t {
    unsafe {
         let smm: Box<TreeMap<String, Vec<*mut cef_string_t>>> = box TreeMap::new();
         mem::transmute(smm)
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_size(smm: *mut cef_string_multimap_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        let mut c: c_int = 0;
        let v = string_multimap_to_treemap(smm);
        for (_, val) in (*v).iter() {
            c = c + (*val).len() as c_int;
        }
        c
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_find_count(smm: *mut cef_string_multimap_t, key: *const cef_string_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        let v = string_multimap_to_treemap(smm);
        slice_to_str((*key).str as *const u8, (*key).length as uint, |result| {
            match (*v).find(&String::from_str(result)) {
                Some(s) => {
                    s.len() as c_int
                }
                None => 0
            }
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_append(smm: *mut cef_string_multimap_t, key: *const cef_string_t, value: *const cef_string_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        let v = string_multimap_to_treemap(smm);
        slice_to_str((*key).str as *const u8, (*key).length as uint, |result| {
            let s = String::from_str(result);
            let csv = cef_string_userfree_utf8_alloc();
            cef_string_utf8_set((*value).str as *const u8, (*value).length, csv, 1);
            match (*v).find_mut(&s) {
                Some(vc) => {
                    (*vc).push(csv);
                    1
                }
                None => {
                    let vc = vec!(csv);
                    (*v).insert(s, vc);
                    1
                }
            }
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_enumerate(smm: *mut cef_string_multimap_t, key: *const cef_string_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        let v = string_multimap_to_treemap(smm);
        slice_to_str((*key).str as *const u8, (*key).length as uint, |result| {
            match (*v).find(&String::from_str(result)) {
                Some(s) => {
                    if (*s).len() <= index as uint {
                        return 0;
                    }
                    let cs = (*s)[index as uint];
                    cef_string_utf8_set((*cs).str as *const u8, (*cs).length, value, 1)
                }
                None => 0
            }
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_key(smm: *mut cef_string_multimap_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || smm.is_null() { return 0; }
        let v = string_multimap_to_treemap(smm);
        let mut rem = index as uint;

        for (key, val) in (*v).iter() {
            if rem < (*val).len() {
                return cef_string_utf8_set((*key).as_bytes().as_ptr(), (*key).len() as u64, value, 1);
            } else {
                rem = rem - (*val).len();
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_value(smm: *mut cef_string_multimap_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || smm.is_null() { return 0; }
        let v = string_multimap_to_treemap(smm);
        let mut rem = index as uint;

        for (_, val) in (*v).values().enumerate() {
            if rem < (*val).len() {
                let cs = (*val)[rem as uint];
                return cef_string_utf8_set((*cs).str as *const u8, (*cs).length, value, 1);
            } else {
                rem = rem - (*val).len();
            }
        }
    }
    0
}
