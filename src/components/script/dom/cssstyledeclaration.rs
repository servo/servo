/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::utils::{Reflectable, Reflector};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct CSSStyleDeclaration {
    pub reflector_: Reflector,
}

pub trait CSSStyleDeclarationMethods {
    fn CssText(&self) -> DOMString;
    fn SetCssText(&mut self, cssText: DOMString) -> ErrorResult;
    fn Length(&self) -> u32;
    fn Item(&self, _index: u32) -> DOMString;
    fn GetPropertyValue(&self, property: DOMString) -> DOMString;
    fn GetPropertyPriority(&self, property: DOMString) -> DOMString;
    fn SetProperty(&mut self, property: DOMString, value: DOMString,
                   priority: DOMString) -> ErrorResult;
    fn SetPropertyValue(&mut self, property: DOMString, value: DOMString) -> ErrorResult;
    fn SetPropertyPriority(&mut self, property: DOMString, priority: DOMString) -> ErrorResult;
    fn RemoveProperty(&mut self, property: DOMString) -> Fallible<DOMString>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> DOMString;
}

impl CSSStyleDeclarationMethods for CSSStyleDeclaration {
    fn CssText(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCssText(&mut self, _cssText: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Length(&self) -> u32 {
        0
    }

    fn Item(&self, _index: u32) -> DOMString {
        "".to_owned()
    }

    fn GetPropertyValue(&self, _property: DOMString) -> DOMString {
        "".to_owned()
    }

    fn GetPropertyPriority(&self, _property: DOMString) -> DOMString {
        "".to_owned()
    }

    fn SetProperty(&mut self, _property: DOMString, _value: DOMString,
                   _priority: DOMString) -> ErrorResult {
        Ok(())
    }

    fn SetPropertyValue(&mut self, _property: DOMString, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn SetPropertyPriority(&mut self, _property: DOMString, _priority: DOMString) -> ErrorResult {
        Ok(())
    }

    fn RemoveProperty(&mut self, _property: DOMString) -> Fallible<DOMString> {
        Ok("".to_owned())
    }

    fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> DOMString {
        "".to_owned()
    }
}

impl Reflectable for CSSStyleDeclaration {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

