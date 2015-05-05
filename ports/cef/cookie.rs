/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{cef_completion_callback_t, cef_cookie_manager_t};
use types::cef_string_t;

use libc::c_int;

cef_stub_static_method_impls! {
    fn cef_cookie_manager_get_global_manager(callback: *mut cef_completion_callback_t) -> *mut cef_cookie_manager_t
    fn cef_cookie_manager_create_manager(path: *const cef_string_t,
                                         persist_session_cookies: c_int,
                                         callback: *mut cef_completion_callback_t)
                                         -> *mut cef_cookie_manager_t
}

