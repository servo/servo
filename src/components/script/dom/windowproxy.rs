/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector};

pub struct WindowProxy {
    reflector_: Reflector
}

impl WindowProxy {
    pub fn new() -> @mut WindowProxy {
        @mut WindowProxy {
            reflector_: Reflector::new()
        }
    }
}

impl Reflectable for WindowProxy {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
