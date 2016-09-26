/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::eventtarget::EventTarget;

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
}

impl GlobalScope {
    pub fn new_inherited() -> GlobalScope {
        GlobalScope {
            eventtarget: EventTarget::new_inherited(),
        }
    }
}
