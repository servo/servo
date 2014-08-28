/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use types::{cef_post_data_element_t, cef_post_data_t, cef_request_t};

#[no_mangle]
pub extern "C" fn cef_request_create() -> *mut cef_request_t {
    0 as *mut cef_request_t
}

#[no_mangle]
pub extern "C" fn cef_post_data_create() -> *mut cef_post_data_t {
    0 as *mut cef_post_data_t
}

#[no_mangle]
pub extern "C" fn cef_post_data_element_create() -> *mut cef_post_data_element_t {
    0 as *mut cef_post_data_element_t
}
