/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::NavigatorBinding;
use dom::bindings::js::{JSRef, Temporary};
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

    pub fn new(window: &JSRef<Window>) -> Temporary<Navigator> {
        reflect_dom_object(~Navigator::new_inherited(),
                           window,
                           NavigatorBinding::Wrap)
    }
}

pub trait NavigatorMethods {
    fn DoNotTrack(&self) -> DOMString;
    fn Vendor(&self) -> DOMString;
    fn VendorSub(&self) -> DOMString;
    fn Product(&self) -> DOMString;
    fn ProductSub(&self) -> DOMString;
    fn CookieEnabled(&self) -> bool;
    fn GetBuildID(&self) -> Fallible<DOMString>;
    fn JavaEnabled(&self) -> Fallible<bool>;
    fn TaintEnabled(&self) -> bool;
    fn AppName(&self) -> DOMString;
    fn GetAppCodeName(&self) -> Fallible<DOMString>;
    fn GetAppVersion(&self) -> Fallible<DOMString>;
    fn GetPlatform(&self) -> Fallible<DOMString>;
    fn GetUserAgent(&self) -> Fallible<DOMString>;
    fn GetLanguage(&self) -> Option<DOMString>;
    fn OnLine(&self) -> bool;
}

impl<'a> NavigatorMethods for JSRef<'a, Navigator> {
    fn DoNotTrack(&self) -> DOMString {
        ~"unspecified"
    }

    fn Vendor(&self) -> DOMString {
        "".to_owned() // Like Gecko
    }

    fn VendorSub(&self) -> DOMString {
        "".to_owned() // Like Gecko
    }

    fn Product(&self) -> DOMString {
        ~"Gecko"
    }

    fn ProductSub(&self) -> DOMString {
        "".to_owned()
    }

    fn CookieEnabled(&self) -> bool {
        false
    }

    fn GetBuildID(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn JavaEnabled(&self) -> Fallible<bool> {
        Ok(false)
    }

    fn TaintEnabled(&self) -> bool {
        false
    }

    fn AppName(&self) -> DOMString {
        ~"Netscape" // Like Gecko/Webkit
    }

    fn GetAppCodeName(&self) -> Fallible<DOMString> {
        Ok(~"Mozilla") // Like Gecko/Webkit
    }

    fn GetAppVersion(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn GetPlatform(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn GetUserAgent(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn GetLanguage(&self) -> Option<DOMString> {
        None
    }

    fn OnLine(&self) -> bool {
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
