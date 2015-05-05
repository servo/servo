/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{cef_request_context_handler_t, cef_request_context_t, cef_request_context_settings_t};

cef_stub_static_method_impls! {
    fn cef_request_context_get_global_context() -> *mut cef_request_context_t
    fn cef_request_context_create_context(_settings: *const cef_request_context_settings_t,
                                          _handler: *mut cef_request_context_handler_t)
                                          -> *mut cef_request_context_t
    fn cef_request_context_create_context_shared(_other: *mut cef_request_context_t,
                                                 _handler: *mut cef_request_context_handler_t)
                                          -> *mut cef_request_context_t
}

