/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Clone, Debug, MallocSizeOf)]
pub struct FontContextHandle {
    ctx: ()
}

impl FontContextHandle {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    pub fn new() -> FontContextHandle {
        FontContextHandle { ctx: () }
    }
}
