/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::Reflector;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRPose {
    reflector_: Reflector,
}

impl XRPose {
    pub fn new_inherited() -> XRPose {
        XRPose {
            reflector_: Reflector::new(),
        }
    }
}
