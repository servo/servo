/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::slice_to_str;
use libc::c_int;
use std::collections::BTreeMap;
use string::{cef_string_userfree_utf16_alloc, cef_string_userfree_utf16_free};
use string::{cef_string_utf16_set};
use types::{cef_string_multimap_t,cef_string_t};

//cef_string_multimap

#[no_mangle]
pub extern "C" fn cef_string_multimap_alloc() -> *mut cef_string_multimap_t {
    Box::into_raw(box BTreeMap::new())
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_size(smm: *mut cef_string_multimap_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        // t1 : collections::btree::map::Values<'_, collections::string::String, collections::vec::Vec<*mut types::cef_string_utf16>>`
        let t1 = (*smm).values();
        // t2 : collections::btree::map::BTreeMap<collections::string::String, collections::vec::Vec<*mut types::cef_string_utf16>>
        let t2 : usize = t1.map(|val| (*val).len()).sum();
        t2 as c_int
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_find_count(smm: *mut cef_string_multimap_t, key: *const cef_string_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        slice_to_str((*key).str as *const u8, (*key).length as usize, |result| {
            match (*smm).get(result) {
                Some(s) =>  s.len() as c_int,
                None => 0
            }
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_append(smm: *mut cef_string_multimap_t, key: *const cef_string_t, value: *const cef_string_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        slice_to_str((*key).str as *const u8, (*key).length as usize, |result| {
            let csv = cef_string_userfree_utf16_alloc();
            cef_string_utf16_set((*value).str as *const u16, (*value).length, csv, 1);
            match (*smm).get_mut(result) {
                Some(vc) => (*vc).push(csv),
                None => { (*smm).insert(result.to_owned(), vec!(csv)); }
            }
            1
        })
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_enumerate(smm: *mut cef_string_multimap_t, key: *const cef_string_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if smm.is_null() { return 0; }
        slice_to_str((*key).str as *const u8, (*key).length as usize, |result| {
            match (*smm).get(result) {
                Some(s) => {
                    if (*s).len() <= index as usize {
                        return 0;
                    }
                    let cs = (*s)[index as usize];
                    cef_string_utf16_set((*cs).str as *const u16, (*cs).length, value, 1)
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
        let mut rem = index as usize;

        for (key, val) in (*smm).iter() {
            if rem < (*val).len() {
                return cef_string_utf16_set((*key).as_bytes().as_ptr() as *const u16,
                                            (*key).len() as u64,
                                            value,
                                            1);
            } else {
                rem -= (*val).len();
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_value(smm: *mut cef_string_multimap_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || smm.is_null() { return 0; }
        let mut rem = index as usize;

        for val in (*smm).values() {
            if rem < (*val).len() {
                let cs = (*val)[rem as usize];
                return cef_string_utf16_set((*cs).str as *const u16, (*cs).length, value, 1);
            } else {
                rem -= (*val).len();
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_clear(smm: *mut cef_string_multimap_t) {
    unsafe {
        if smm.is_null() { return; }
        if (*smm).is_empty() { return; }
        for (_, val) in (*smm).iter_mut() {
            while let Some(cs) = (*val).pop() {
                cef_string_userfree_utf16_free(cs);
            }
        }
        (*smm).clear();
    }
}

#[no_mangle]
pub extern "C" fn cef_string_multimap_free(smm: *mut cef_string_multimap_t) {
    unsafe {
        if smm.is_null() { return; }
        cef_string_multimap_clear(smm);
        drop(Box::from_raw(smm));
    }
}
