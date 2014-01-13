/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMImplementationBinding;
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::window::Window;

pub struct DOMImplementation {
    owner: @mut Window,
    reflector_: Reflector
}

impl DOMImplementation {
    pub fn new_inherited(owner: @mut Window) -> DOMImplementation {
        DOMImplementation {
            owner: owner,
            reflector_: Reflector::new()
        }
    }

    pub fn new(owner: @mut Window) -> @mut DOMImplementation {
        reflect_dom_object(@mut DOMImplementation::new_inherited(owner), owner,
                           DOMImplementationBinding::Wrap)
    }
}

impl Reflectable for DOMImplementation {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
