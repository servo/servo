/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use eutil::fptr_is_null;
use libc::{size_t, c_int, c_ushort,c_void};
use libc::types::os::arch::c95::wchar_t;
use mem::{new0,newarray0,delete,deletearray};
use std::mem;
use std::ptr;
use types::{cef_string_utf16_t, cef_string_utf8_t, cef_string_wide_t};
use types::{cef_string_userfree_utf16_t, cef_string_userfree_utf8_t, cef_string_userfree_wide_t};

//cef_string

#[no_mangle]
extern "C" fn string_wide_dtor(str: *mut wchar_t) {
  deletearray(str as *mut c_void)
}

#[no_mangle]
extern "C" fn string_utf8_dtor(str: *mut u8) {
  deletearray(str as *mut c_void)
}

#[no_mangle]
extern "C" fn string_utf16_dtor(str: *mut c_ushort) {
  deletearray(str as *mut c_void)
}

#[no_mangle]
pub extern "C" fn cef_string_userfree_wide_free(cs: *mut cef_string_userfree_wide_t) {
   cef_string_wide_clear(cs);
   delete(cs as *mut c_void)
}

#[no_mangle]
pub extern "C" fn cef_string_userfree_utf8_free(cs: *mut cef_string_userfree_utf8_t) {
   cef_string_utf8_clear(cs);
   delete(cs as *mut c_void)
}

#[no_mangle]
pub extern "C" fn cef_string_userfree_utf16_free(cs: *mut cef_string_userfree_utf16_t) {
   cef_string_utf16_clear(cs);
   delete(cs as *mut c_void)
}

#[no_mangle]
pub extern "C" fn cef_string_utf8_clear(cs: *mut cef_string_utf8_t) {
    unsafe {
        if !fptr_is_null(mem::transmute((*cs).dtor)) {
            let dtor = (*cs).dtor;
            dtor((*cs).str);
        }
        (*cs).length = 0;
        (*cs).str = 0 as *mut u8;
        (*cs).dtor = mem::transmute(0 as *const u8);
    }
}

#[no_mangle]
pub extern "C" fn cef_string_userfree_utf8_alloc() -> *mut cef_string_utf8_t {
    #![inline(never)]
    new0::<cef_string_utf8_t>(1)
}

#[no_mangle]
pub extern "C" fn cef_string_utf8_set(src: *const u8, src_len: size_t, output: *mut cef_string_utf8_t, copy: c_int) -> c_int {
    cef_string_utf8_clear(output);
    unsafe {
       if copy != 0 {
           if !src.is_null() && src_len > 0 {
             (*output).str = newarray0::<u8>(src_len + 1);
             if (*output).str.is_null() {
                 return 0;
             }

             ptr::copy_memory((*output).str, src, src_len as uint);
             (*output).length = src_len;
             (*output).dtor = string_utf8_dtor;
           }
       } else {
         (*output).str = mem::transmute(src);
         (*output).length = src_len;
         (*output).dtor = mem::transmute(0 as *const u8);
       }
    }
    return 1;
}

#[no_mangle]
pub extern "C" fn cef_string_utf16_clear(cs: *mut cef_string_utf16_t) {
    unsafe {
        if !fptr_is_null(mem::transmute((*cs).dtor)) {
            let dtor = (*cs).dtor;
            dtor((*cs).str);
        }
        (*cs).length = 0;
        (*cs).str = 0 as *mut c_ushort;
        (*cs).dtor = mem::transmute(0 as *const u8);
    }
}

#[no_mangle]
pub extern "C" fn cef_string_userfree_utf16_alloc() -> *mut cef_string_utf16_t {
    #![inline(never)]
    new0::<cef_string_utf16_t>(1)
}

#[no_mangle]
pub extern "C" fn cef_string_utf16_set(src: *const c_ushort, src_len: size_t, output: *mut cef_string_utf16_t, copy: c_int) -> c_int {
    cef_string_utf16_clear(output);
    unsafe {
       if copy != 0 {
           if !src.is_null() && src_len > 0 {
             (*output).str = newarray0::<c_ushort>(src_len + 1);
             if (*output).str.is_null() {
                 return 0;
             }

             ptr::copy_memory((*output).str, src, src_len as uint);
             (*output).length = src_len;
             (*output).dtor = string_utf16_dtor;
           }
       } else {
         (*output).str = mem::transmute(src);
         (*output).length = src_len;
         (*output).dtor = mem::transmute(0 as *const u8);
       }
    }
    return 1;
}

#[no_mangle]
pub extern "C" fn cef_string_wide_clear(cs: *mut cef_string_wide_t) {
    unsafe {
        if !fptr_is_null(mem::transmute((*cs).dtor)) {
            let dtor = (*cs).dtor;
            dtor((*cs).str);
        }
        (*cs).length = 0;
        (*cs).str = 0 as *mut wchar_t;
        (*cs).dtor = mem::transmute(0 as *const u8);
    }
}

#[no_mangle]
pub extern "C" fn cef_string_userfree_wide_alloc() -> *mut cef_string_wide_t {
    #![inline(never)]
    new0::<cef_string_wide_t>(1)
}

#[no_mangle]
pub extern "C" fn cef_string_wide_set(src: *const wchar_t, src_len: size_t, output: *mut cef_string_wide_t, copy: c_int) -> c_int {
    cef_string_wide_clear(output);
    unsafe {
       if copy != 0 {
           if !src.is_null() && src_len > 0 {
             (*output).str = newarray0::<wchar_t>(src_len + 1);
             if (*output).str.is_null() {
                 return 0;
             }

             ptr::copy_memory((*output).str, src, src_len as uint);
             (*output).length = src_len;
             (*output).dtor = string_wide_dtor;
           }
       } else {
         (*output).str = mem::transmute(src);
         (*output).length = src_len;
         (*output).dtor = mem::transmute(0 as *const u8);
       }
    }
    return 1;
}
