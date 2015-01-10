/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::size_t;

#[allow(raw_pointer_deriving)]
#[deriving(Copy)]
pub struct StructuredCloneData {
    pub data: *mut u64,
    pub nbytes: size_t,
}
