/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::CSSStyleValueBinding::CSSStyleValueMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSStyleValue {
    reflector: Reflector,
    value: String,
}

impl CSSStyleValue {
    fn new_inherited(value: String) -> CSSStyleValue {
        CSSStyleValue {
            reflector: Reflector::new(),
            value,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        value: String,
        can_gc: CanGc,
    ) -> DomRoot<CSSStyleValue> {
        reflect_dom_object(
            Box::new(CSSStyleValue::new_inherited(value)),
            global,
            can_gc,
        )
    }
}

impl CSSStyleValueMethods<crate::DomTypeHolder> for CSSStyleValue {
    /// <https://drafts.css-houdini.org/css-typed-om-1/#CSSStyleValue-stringification-behavior>
    fn Stringifier(&self) -> DOMString {
        DOMString::from(&*self.value)
    }
}

impl CSSStyleValue {
    /// Parse the value as a `url()`.
    /// TODO: This should really always be an absolute URL, but we currently
    /// return relative URLs for computed values, so we pass in a base.
    /// <https://github.com/servo/servo/issues/17625>
    pub(crate) fn get_url(&self, base_url: ServoUrl) -> Option<ServoUrl> {
        let mut input = ParserInput::new(&self.value);
        let mut parser = Parser::new(&mut input);
        parser
            .expect_url()
            .ok()
            .and_then(|string| base_url.join(&string).ok())
    }
}
