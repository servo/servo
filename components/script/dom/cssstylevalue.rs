/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleValueBinding::CSSStyleValueMethods;
use dom::bindings::codegen::Bindings::CSSStyleValueBinding::Wrap;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflector;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct CSSStyleValue {
    reflector: Reflector,
    value: String,
}

impl CSSStyleValue {
    fn new_inherited(value: String) -> CSSStyleValue {
        CSSStyleValue {
            reflector: Reflector::new(),
            value: value,
        }
    }

    pub fn new(global: &GlobalScope, value: String) -> Root<CSSStyleValue> {
        reflect_dom_object(box CSSStyleValue::new_inherited(value), global, Wrap)
    }
}

impl CSSStyleValueMethods for CSSStyleValue {
    /// https://drafts.css-houdini.org/css-typed-om-1/#CSSStyleValue-stringification-behavior
    fn Stringifier(&self) -> DOMString {
        DOMString::from(&*self.value)
    }

    /// This attribute is no longer part of the `CSSStyleValue` interface,
    /// but is still used in some examples.
    /// https://github.com/GoogleChrome/houdini-samples/issues/16
    // check-tidy: no specs after this line
    fn CssText(&self) -> DOMString {
        self.Stringifier()
    }
}
