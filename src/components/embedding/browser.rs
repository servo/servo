/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use libc::{calloc, size_t,c_int};
use std::mem;
use types::{cef_browser_settings_t, cef_browser_t, cef_client_t, cef_request_context_t, cef_string_t, cef_window_info_t};

#[no_mangle]
pub extern "C" fn cef_browser_host_create_browser(_windowInfo: *cef_window_info_t,
                                                  _client: *mut cef_client_t,
                                                  _url: *cef_string_t,
                                                  _settings: *cef_browser_settings_t,
                                                  _request_context: *mut cef_request_context_t)
                                                  -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn cef_browser_host_create_browser_sync(_windowInfo: *cef_window_info_t,
                                                       _client: *mut cef_client_t,
                                                       _url: *cef_string_t,
                                                       _settings: *cef_browser_settings_t,
                                                       _request_context: *mut cef_request_context_t)
                                                       -> *mut cef_browser_t {
    unsafe {
        let browser = calloc(1, mem::size_of::<cef_browser_t>() as size_t) as *mut cef_browser_t;
        browser
    }
}
