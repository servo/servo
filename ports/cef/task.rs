/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::c_int;
use types::cef_thread_id_t;

//FIXME: this should check the current servo task I guess?
#[no_mangle]
pub extern "C" fn cef_currently_on(_tid: cef_thread_id_t) -> c_int {
    1
}
