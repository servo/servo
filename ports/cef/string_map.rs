/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::fptr_is_null;
use libc::{c_int};
use std::collections::TreeMap;
use std::mem;
use std::slice;
use std::str;
use std::string::String;
use string::{cef_string_userfree_utf8_alloc,cef_string_userfree_utf8_free,cef_string_utf8_set};
use types::{cef_string_map_t,cef_string_t};

fn string_map_to_treemap(sm: *mut cef_string_map_t) -> *mut TreeMap<String, *mut cef_string_t> {
    sm as *mut TreeMap<String, *mut cef_string_t>
}

//cef_string_map

#[no_mangle]
pub extern "C" fn cef_string_map_alloc() -> *mut cef_string_map_t {
    unsafe {
         let sm: Box<TreeMap<String, *mut cef_string_t>> = box TreeMap::new();
         mem::transmute(sm)
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_size(sm: *mut cef_string_map_t) -> c_int {
    unsafe {
        if fptr_is_null(mem::transmute(sm)) { return 0; }
        let v = string_map_to_treemap(sm);
        (*v).len() as c_int
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_append(sm: *mut cef_string_map_t, key: *const cef_string_t, value: *const cef_string_t) -> c_int {
    unsafe {
        if fptr_is_null(mem::transmute(sm)) { return 0; }
        let v = string_map_to_treemap(sm);
        slice::raw::buf_as_slice(mem::transmute((*key).str), (*key).length as uint, |result| {
            match str::from_utf8(result) {
                Some(k) => {
                    let s = String::from_str(k);
                    let csv = cef_string_userfree_utf8_alloc();
                    cef_string_utf8_set(mem::transmute((*value).str), (*value).length, csv, 1);
                    (*v).insert(s, csv);
                    1
                },
                None => 0
            }
        })
    }
}
