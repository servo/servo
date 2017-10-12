/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::slice_to_str;
use libc::{c_int};
use std::collections::BTreeMap;
use string::{cef_string_userfree_utf16_alloc, cef_string_userfree_utf16_free};
use string::{cef_string_utf16_set};
use types::{cef_string_map_t, cef_string_t};

//cef_string_map

#[no_mangle]
pub extern "C" fn cef_string_map_alloc() -> *mut cef_string_map_t {
    Box::into_raw(Box::new(BTreeMap::new()))
}

#[no_mangle]
pub extern "C" fn cef_string_map_size(sm: *mut cef_string_map_t) -> c_int {
    unsafe {
        if sm.is_null() { return 0; }
        (*sm).len() as c_int
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_append(sm: *mut cef_string_map_t, key: *const cef_string_t, value: *const cef_string_t) -> c_int {
    unsafe {
        if sm.is_null() { return 0; }
        slice_to_str((*key).str as *const u8, (*key).length as usize, |result| {
            let csv = cef_string_userfree_utf16_alloc();
            cef_string_utf16_set((*value).str as *const u16, (*value).length, csv, 1);
            (*sm).insert(result.to_owned(), csv);
            1
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_find(sm: *mut cef_string_map_t, key: *const cef_string_t, value: *mut cef_string_t) -> c_int {
    unsafe {
        if sm.is_null() { return 0; }
        slice_to_str((*key).str as *const u8, (*key).length as usize, |result| {
            match (*sm).get(result) {
                Some(s) => {
                    cef_string_utf16_set((**s).str as *const u16, (**s).length, value, 1)
                }
                None => 0
            }
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_key(sm: *mut cef_string_map_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || sm.is_null() { return 0; }
        if index as usize > (*sm).len() - 1 { return 0; }

        match (*sm).keys().nth(index as usize) {
            Some(k) => {
                cef_string_utf16_set(k.as_bytes().as_ptr() as *const u16,
                                     k.len(),
                                     value,
                                     1)
            },
            None => 0,
        }
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_value(sm: *mut cef_string_map_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || sm.is_null() { return 0; }
        if index as usize > (*sm).len() - 1 { return 0; }

        match (*sm).values().nth(index as usize) {
            Some(val) => {
                cef_string_utf16_set((**val).str as *const u16, (**val).length, value, 1);
                1
            },
            None => 0,
        }
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_clear(sm: *mut cef_string_map_t) {
    unsafe {
        if sm.is_null() { return; }
        for val in (*sm).values() {
            cef_string_userfree_utf16_free(*val);
        }
        (*sm).clear();
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_free(sm: *mut cef_string_map_t) {
    unsafe {
        if sm.is_null() { return; }
        cef_string_map_clear(sm);
        drop(Box::from_raw(sm));
    }
}
