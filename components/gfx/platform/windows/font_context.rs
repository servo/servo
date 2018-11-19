/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of::malloc_size_of_is_0;

#[derive(Clone, Debug)]
pub struct FontContextHandle;

impl FontContextHandle {
    // *shrug*
    pub fn new() -> FontContextHandle {
        FontContextHandle {}
    }
}

malloc_size_of_is_0!(FontContextHandle);
