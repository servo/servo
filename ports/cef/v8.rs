/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{cef_v8accessor_t, cef_v8context_t, cef_v8handler_t, cef_v8stack_trace_t};
use interfaces::{cef_v8value_t};
use types::{cef_string_t, cef_time_t};

use libc::{self, c_double, c_int};

cef_stub_static_method_impls! {
    fn cef_v8context_get_current_context() -> *mut cef_v8context_t
    fn cef_v8context_get_entered_context() -> *mut cef_v8context_t
    fn cef_v8context_in_context() -> libc::c_int
    fn cef_v8value_create_undefined() -> *mut cef_v8value_t
    fn cef_v8value_create_null() -> *mut cef_v8value_t
    fn cef_v8value_create_bool(_value: c_int) -> *mut cef_v8value_t
    fn cef_v8value_create_int(_value: i32) -> *mut cef_v8value_t
    fn cef_v8value_create_uint(_value: u32) -> *mut cef_v8value_t
    fn cef_v8value_create_double(_value: c_double) -> *mut cef_v8value_t
    fn cef_v8value_create_date(_date: *const cef_time_t) -> *mut cef_v8value_t
    fn cef_v8value_create_string(_value: *const cef_string_t) -> *mut cef_v8value_t
    fn cef_v8value_create_object(_accessor: *mut cef_v8accessor_t) -> *mut cef_v8value_t
    fn cef_v8value_create_array(_length: libc::c_int) -> *mut cef_v8value_t
    fn cef_v8value_create_function(_name: *const cef_string_t, _handler: *mut cef_v8handler_t)
                                   -> *mut cef_v8value_t
    fn cef_v8stack_trace_get_current(_frame_limit: libc::c_int) -> *mut cef_v8stack_trace_t
}

