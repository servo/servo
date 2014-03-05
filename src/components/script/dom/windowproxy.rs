/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::WindowProxyBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;

#[deriving(Encodable)]
pub struct WindowProxy {
    reflector_: Reflector
}

impl WindowProxy {
    pub fn new(owner: &JS<Window>) -> JS<WindowProxy> {
        let proxy = ~WindowProxy {
            reflector_: Reflector::new()
        };
        reflect_dom_object(proxy, owner, WindowProxyBinding::Wrap)
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
