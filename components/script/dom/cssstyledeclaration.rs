/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::js::JSRef;
use servo_util::str::DOMString;

#[dom_struct]
pub struct CSSStyleDeclaration {
    reflector_: Reflector,
}

impl<'a> CSSStyleDeclarationMethods for JSRef<'a, CSSStyleDeclaration> {
    fn CssText(self) -> DOMString {
        "".to_string()
    }

    fn SetCssText(self, _cssText: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Length(self) -> u32 {
        0
    }

    fn Item(self, _index: u32) -> DOMString {
        "".to_string()
    }

    fn GetPropertyValue(self, _property: DOMString) -> DOMString {
        "".to_string()
    }

    fn GetPropertyPriority(self, _property: DOMString) -> DOMString {
        "".to_string()
    }

    fn SetProperty(self, _property: DOMString, _value: DOMString,
                   _priority: DOMString) -> ErrorResult {
        Ok(())
    }

    fn SetPropertyValue(self, _property: DOMString, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn SetPropertyPriority(self, _property: DOMString, _priority: DOMString) -> ErrorResult {
        Ok(())
    }

    fn RemoveProperty(self, _property: DOMString) -> Fallible<DOMString> {
        Ok("".to_string())
    }

    fn IndexedGetter(self, _index: u32, _found: &mut bool) -> DOMString {
        "".to_string()
    }
}

impl Reflectable for CSSStyleDeclaration {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
