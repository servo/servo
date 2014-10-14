/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::fptr_is_null;
use libc::{c_int};
use std::mem;
use string::{cef_string_userfree_utf8_alloc,cef_string_userfree_utf8_free,cef_string_utf8_set};
use types::{cef_string_list_t,cef_string_t};

//cef_string_list

#[no_mangle]
extern "C" fn cef_string_list_alloc() -> *mut cef_string_list_t {
    unsafe {
         let lt: Box<Vec<*mut cef_string_t>> = box vec!();
         mem::transmute(lt)
    }
}

#[no_mangle]
extern "C" fn cef_string_list_size(lt: *const cef_string_list_t) -> c_int {
    unsafe {
        if fptr_is_null(mem::transmute(lt)) { return 0; }
        let v: Box<Vec<*mut cef_string_t>> = mem::transmute(lt);
        v.len() as c_int
    }
}

#[no_mangle]
extern "C" fn cef_string_list_append(lt: *mut cef_string_list_t, value: *const cef_string_t) {
    unsafe {
        if fptr_is_null(mem::transmute(lt)) { return; }
        let mut v: Box<Vec<*mut cef_string_t>> = mem::transmute(lt);
        let cs = cef_string_userfree_utf8_alloc();
        cef_string_utf8_set(mem::transmute((*value).str), (*value).length, cs, 1);
        v.push(cs);
    }
}

#[no_mangle]
extern "C" fn cef_string_list_value(lt: *mut cef_string_list_t, index: c_int, value: *mut cef_string_t) -> c_int {
    unsafe {
        if index < 0 || fptr_is_null(mem::transmute(lt)) { return 0; }
        let v: Box<Vec<*mut cef_string_t>> = mem::transmute(lt);
        if index as uint > v.len() - 1 { return 0; }
        let cs = v.get(index as uint);
        cef_string_utf8_set(mem::transmute((**cs).str), (**cs).length, value, 1)
    }
}

#[no_mangle]
extern "C" fn cef_string_list_clear(lt: *mut cef_string_list_t) {
    unsafe {
        if fptr_is_null(mem::transmute(lt)) { return; }
        let mut v: Box<Vec<*mut cef_string_t>> = mem::transmute(lt);
        if v.len() == 0 { return; }
        let mut cs;
        while v.len() != 0 {
            cs = v.pop();
            cef_string_userfree_utf8_free(cs.unwrap());
        }
    }
}

#[no_mangle]
extern "C" fn cef_string_list_free(lt: *mut cef_string_list_t) {
    unsafe {
        if fptr_is_null(mem::transmute(lt)) { return; }
        let mut v: Box<Vec<*mut cef_string_t>> = mem::transmute(lt);
        cef_string_list_clear(lt);
        drop(v);
    }
}

#[no_mangle]
extern "C" fn cef_string_list_copy(lt: *mut cef_string_list_t) -> *mut cef_string_list_t {
    unsafe {
        if fptr_is_null(mem::transmute(lt)) { return 0 as *mut cef_string_list_t; }
        let v: Box<Vec<*mut cef_string_t>> = mem::transmute(lt);
        let lt2 = cef_string_list_alloc();
        for cs in v.iter() {
            cef_string_list_append(lt2, mem::transmute((*cs)));
        }
        lt2
    }
}
