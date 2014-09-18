/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NavigatorBinding;
use dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use dom::bindings::global::Window;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct Navigator {
    pub reflector_: Reflector //XXXjdm cycle: window->navigator->window
}

impl Navigator {
    pub fn new_inherited() -> Navigator {
        Navigator {
            reflector_: Reflector::new()
        }
    }

    pub fn new(window: JSRef<Window>) -> Temporary<Navigator> {
        reflect_dom_object(box Navigator::new_inherited(),
                           &Window(window),
                           NavigatorBinding::Wrap)
    }
}

impl<'a> NavigatorMethods for JSRef<'a, Navigator> {
    fn Product(&self) -> DOMString {
        "Gecko".to_string()
    }

    fn TaintEnabled(&self) -> bool {
        false
    }

    fn AppName(&self) -> DOMString {
        "Netscape".to_string() // Like Gecko/Webkit
    }

    fn AppCodeName(&self) -> DOMString {
        "Mozilla".to_string()
    }

    fn Platform(&self) -> DOMString {
        "".to_string()
    }
}

impl Reflectable for Navigator {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
