/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::utils::{DOMString, Fallible};
use dom::bindings::codegen::NavigatorBinding;
use script_task::{page_from_context};

use js::jsapi::{JSContext, JSObject};

pub struct Navigator {
    reflector_: Reflector
}

impl Navigator {
    pub fn new() -> @mut Navigator {
        @mut Navigator {
            reflector_: Reflector::new()
        }
    }

    pub fn DoNotTrack(&self) -> DOMString {
        Some(~"unspecified")
    }

    pub fn Vendor(&self) -> DOMString {
        Some(~"") // Like Gecko
    }

    pub fn VendorSub(&self) -> DOMString {
        Some(~"") // Like Gecko
    }

    pub fn Product(&self) -> DOMString {
        Some(~"Gecko") // This is supposed to be constant, see webidl.
    }

    pub fn ProductSub(&self) -> DOMString {
        None
    }

    pub fn CookieEnabled(&self) -> bool {
        false
    }

    pub fn GetBuildID(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn JavaEnabled(&self) -> Fallible<bool> {
        Ok(false)
    }

    pub fn TaintEnabled(&self) -> bool {
        false
    }

    pub fn AppName(&self) -> DOMString {
        Some(~"Netscape") // Like Gecko/Webkit
    }

    pub fn GetAppCodeName(&self) -> Fallible<DOMString> {
        Ok(Some(~"Mozilla")) // Like Gecko/Webkit
    }

    pub fn GetAppVersion(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn GetPlatform(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn GetUserAgent(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn GetLanguage(&self) -> DOMString {
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

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        NavigatorBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
