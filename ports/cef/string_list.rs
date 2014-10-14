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
