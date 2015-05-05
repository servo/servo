/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{cef_binary_value_t, cef_dictionary_value_t, cef_list_value_t, cef_value_t};

use libc;

cef_stub_static_method_impls! {
    fn cef_binary_value_create(_data: *const (), _size: libc::size_t) -> *mut cef_binary_value_t
    fn cef_dictionary_value_create() -> *mut cef_dictionary_value_t
    fn cef_list_value_create() -> *mut cef_list_value_t
    fn cef_value_create() -> *mut cef_value_t
}

