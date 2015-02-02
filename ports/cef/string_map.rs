/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::slice_to_str;
use libc::{c_int};
use std::collections::BTreeMap;
use std::mem;
use std::string::String;
use string::{cef_string_userfree_utf16_alloc, cef_string_userfree_utf16_free};
use string::{cef_string_utf16_set};
use types::{cef_string_map_t, cef_string_t};

fn string_map_to_treemap(sm: *mut cef_string_map_t) -> *mut BTreeMap<String, *mut cef_string_t> {
    sm as *mut BTreeMap<String, *mut cef_string_t>
}

//cef_string_map

#[no_mangle]
pub extern "C" fn cef_string_map_alloc() -> *mut cef_string_map_t {
    unsafe {
         let sm: Box<BTreeMap<String, *mut cef_string_t>> = box BTreeMap::new();
         mem::transmute(sm)
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_size(sm: *mut cef_string_map_t) -> c_int {
    unsafe {
        if sm.is_null() { return 0; }
        let v = string_map_to_treemap(sm);
        (*v).len() as c_int
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_append(sm: *mut cef_string_map_t, key: *const cef_string_t, value: *const cef_string_t) -> c_int {
    unsafe {
        if sm.is_null() { return 0; }
        let v = string_map_to_treemap(sm);
        slice_to_str((*key).str as *const u8, (*key).length as uint, |result| {
            let s = String::from_str(result);
            let csv = cef_string_userfree_utf16_alloc();
            cef_string_utf16_set((*value).str as *const u16, (*value).length, csv, 1);
            (*v).insert(s, csv);
            1
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_find(sm: *mut cef_string_map_t, key: *const cef_string_t, value: *mut cef_string_t) -> c_int {
    unsafe {
        if sm.is_null() { return 0; }
        let v = string_map_to_treemap(sm);
        slice_to_str((*key).str as *const u8, (*key).length as uint, |result| {
            match (*v).get(&String::from_str(result)) {
                Some(s) => {
                    cef_string_utf16_set((**s).str as *const u16, (**s).length, value, 1);
                    1
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
        let v = string_map_to_treemap(sm);
        if index as uint > (*v).len() - 1 { return 0; }

        for (i, k) in (*v).keys().enumerate() {
            if i == index as uint {
                cef_string_utf16_set(k.as_bytes().as_ptr() as *const u16,
                                     k.len() as u64,
                                     value,
                                     1);
                return 1;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn cef_string_map_value(sm: *mut cef_string_map_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || sm.is_null() { return 0; }
        let v = string_map_to_treemap(sm);
        if index as uint > (*v).len() - 1 { return 0; }

        for (i, val) in (*v).values().enumerate() {
            if i == index as uint {
                cef_string_utf16_set((**val).str as *const u16, (**val).length, value, 1);
                return 1;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn cef_string_map_clear(sm: *mut cef_string_map_t) {
    unsafe {
        if sm.is_null() { return; }
        let v = string_map_to_treemap(sm);
        for val in (*v).values() {
            cef_string_userfree_utf16_free(*val);
        }
        (*v).clear();
    }
}

#[no_mangle]
pub extern "C" fn cef_string_map_free(sm: *mut cef_string_map_t) {
    unsafe {
        if sm.is_null() { return; }
        let _v: Box<BTreeMap<String, *mut cef_string_t>> = mem::transmute(sm);
        cef_string_map_clear(sm);
    }
}
