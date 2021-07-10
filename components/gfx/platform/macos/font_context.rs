/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

#[derive(Clone, Debug)]
pub struct FontContextHandle {
    ctx: (),
}

impl FontContextHandle {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    pub fn new() -> FontContextHandle {
        FontContextHandle { ctx: () }
    }
}

impl MallocSizeOf for FontContextHandle {
    fn size_of(&self, _: &mut MallocSizeOfOps) -> usize {
        0
    }
}
