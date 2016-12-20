/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::{c_int};
use std::slice;
use string::cef_string_utf16_set;
use types::{cef_string_list_t,cef_string_t};

//cef_string_list

#[no_mangle]
pub extern "C" fn cef_string_list_alloc() -> *mut cef_string_list_t {
    Box::into_raw(box vec!())
}

#[no_mangle]
pub extern "C" fn cef_string_list_size(lt: *mut cef_string_list_t) -> c_int {
    unsafe {
        if lt.is_null() { return 0; }
        (*lt).len() as c_int
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_append(lt: *mut cef_string_list_t, value: *const cef_string_t) {
    unsafe {
        if lt.is_null() { return; }
        (*lt).push(String::from_utf16(slice::from_raw_parts((*value).str, (*value).length as usize)).unwrap());
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_value(lt: *mut cef_string_list_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || lt.is_null() { return 0; }
        if index as usize > (*lt).len() - 1 { return 0; }
        let ref string = (*lt)[index as usize];
        let utf16_chars: Vec<u16> = string.encode_utf16().collect();
        cef_string_utf16_set(utf16_chars.as_ptr(), utf16_chars.len(), value, 1)
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_clear(lt: *mut cef_string_list_t) {
    unsafe {
        if lt.is_null() { return; }
        (*lt).clear();
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_free(lt: *mut cef_string_list_t) {
    unsafe {
        if lt.is_null() { return; }
        cef_string_list_clear(lt);
        drop(Box::from_raw(lt));
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_copy(lt: *mut cef_string_list_t) -> *mut cef_string_list_t {
    unsafe {
        if lt.is_null() { return 0 as *mut cef_string_list_t; }
        let copy = (*lt).clone();
        Box::into_raw(box copy)
    }
}
