/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSS2PropertiesBinding::CSS2PropertiesMethods;
use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::codegen::InheritTypes::CSSStyleDeclarationCast;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::js::JSRef;
use dom::cssstyledeclaration::CSSStyleDeclaration;
use servo_util::str::DOMString;

#[dom_struct]
pub struct CSS2Properties {
    declaration: CSSStyleDeclaration,
}

impl<'a> CSS2PropertiesMethods for JSRef<'a, CSS2Properties> {
    fn Color(self) -> DOMString {
        "".to_string()
    }

    fn SetColor(self, _value: DOMString) {
    }

    fn Background(self) -> DOMString {
        "".to_string()
    }

    fn SetBackground(self, _value: DOMString) {
    }

    fn Display(self) -> DOMString {
        "".to_string()
    }

    fn SetDisplay(self, _value: DOMString) {
    }

    fn Width(self) -> DOMString {
        "".to_string()
    }

    fn SetWidth(self, _value: DOMString) {
    }

    fn Height(self) -> DOMString {
        "".to_string()
    }

    fn SetHeight(self, _value: DOMString) {
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> DOMString {
        let decl: JSRef<CSSStyleDeclaration> = CSSStyleDeclarationCast::from_ref(self);
        decl.IndexedGetter(index, found)
    }
}

impl Reflectable for CSS2Properties {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.declaration.reflector()
    }
}
