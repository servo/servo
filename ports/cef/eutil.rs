/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::c_int;
use std::slice;
use std::str;

pub fn slice_to_str(s: *const u8, l: uint, f: |&str| -> c_int) -> c_int {
    unsafe {
        slice::raw::buf_as_slice(s, l, |result| {
             str::from_utf8(result).map(|s| f(s)).unwrap_or(0)
        })
    }
}
