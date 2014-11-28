/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::{c_int};
use std::mem;
use string::{cef_string_userfree_utf16_alloc,cef_string_userfree_utf16_free,cef_string_utf16_set};
use types::{cef_string_list_t,cef_string_t};


fn string_list_to_vec(lt: *mut cef_string_list_t) -> *mut Vec<*mut cef_string_t> {
    lt as *mut Vec<*mut cef_string_t>
}

//cef_string_list

#[no_mangle]
pub extern "C" fn cef_string_list_alloc() -> *mut cef_string_list_t {
    unsafe {
         let lt: Box<Vec<*mut cef_string_t>> = box vec!();
         mem::transmute(lt)
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_size(lt: *mut cef_string_list_t) -> c_int {
    unsafe {
        if lt.is_null() { return 0; }
        let v = string_list_to_vec(lt);
        (*v).len() as c_int
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_append(lt: *mut cef_string_list_t, value: *const cef_string_t) {
    unsafe {
        if lt.is_null() { return; }
        let v = string_list_to_vec(lt);
        let cs = cef_string_userfree_utf16_alloc();
        cef_string_utf16_set(mem::transmute((*value).str), (*value).length, cs, 1);
        (*v).push(cs);
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_value(lt: *mut cef_string_list_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || lt.is_null() { return 0; }
        let v = string_list_to_vec(lt);
        if index as uint > (*v).len() - 1 { return 0; }
        let cs = (*v)[index as uint];
        cef_string_utf16_set(mem::transmute((*cs).str), (*cs).length, value, 1)
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_clear(lt: *mut cef_string_list_t) {
    unsafe {
        if lt.is_null() { return; }
        let v = string_list_to_vec(lt);
        if (*v).len() == 0 { return; }
        let mut cs;
        while (*v).len() != 0 {
            cs = (*v).pop();
            cef_string_userfree_utf16_free(cs.unwrap());
        }
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_free(lt: *mut cef_string_list_t) {
    unsafe {
        if lt.is_null() { return; }
        let v: Box<Vec<*mut cef_string_t>> = mem::transmute(lt);
        cef_string_list_clear(lt);
        drop(v);
    }
}

#[no_mangle]
pub extern "C" fn cef_string_list_copy(lt: *mut cef_string_list_t) -> *mut cef_string_list_t {
    unsafe {
        if lt.is_null() { return 0 as *mut cef_string_list_t; }
        let v = string_list_to_vec(lt);
        let lt2 = cef_string_list_alloc();
        for cs in (*v).iter() {
            cef_string_list_append(lt2, mem::transmute((*cs)));
        }
        lt2
    }
}
