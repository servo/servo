/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::NavigatorBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct Navigator {
    pub reflector_: Reflector //XXXjdm cycle: window->navigator->window
}

impl Navigator {
    pub fn new_inherited() -> Navigator {
        Navigator {
            reflector_: Reflector::new()
        }
    }

    pub fn new(window: &JS<Window>) -> JS<Navigator> {
        reflect_dom_object(~Navigator::new_inherited(),
                           window,
                           NavigatorBinding::Wrap)
    }

    pub fn DoNotTrack(&self) -> DOMString {
        ~"unspecified"
    }

    pub fn Vendor(&self) -> DOMString {
        ~"" // Like Gecko
    }

    pub fn VendorSub(&self) -> DOMString {
        ~"" // Like Gecko
    }

    pub fn Product(&self) -> DOMString {
        ~"Gecko"
    }

    pub fn ProductSub(&self) -> DOMString {
        ~""
    }

    pub fn CookieEnabled(&self) -> bool {
        false
    }

    pub fn GetBuildID(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn JavaEnabled(&self) -> Fallible<bool> {
        Ok(false)
    }

    pub fn TaintEnabled(&self) -> bool {
        false
    }

    pub fn AppName(&self) -> DOMString {
        ~"Netscape" // Like Gecko/Webkit
    }

    pub fn GetAppCodeName(&self) -> Fallible<DOMString> {
        Ok(~"Mozilla") // Like Gecko/Webkit
    }

    pub fn GetAppVersion(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn GetPlatform(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn GetUserAgent(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn GetLanguage(&self) -> Option<DOMString> {
        None
    }

    pub fn OnLine(&self) -> bool {
        true
    }
}

impl Reflectable for Navigator {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
