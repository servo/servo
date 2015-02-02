/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::cef_process_message_t;
use types::cef_string_t;

cef_stub_static_method_impls! {
    fn cef_process_message_create(name: *const cef_string_t) -> *mut cef_process_message_t
}

