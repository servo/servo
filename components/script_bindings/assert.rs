/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::thread_state;

pub fn assert_in_layout() {
    debug_assert!(thread_state::get().is_layout());
}

pub fn assert_in_script() {
    debug_assert!(thread_state::get().is_script());
}
