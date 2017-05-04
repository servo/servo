/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, serialize_identifier};
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::window::Window;
use dom_struct::dom_struct;
use style::context::QuirksMode;
use style::parser::{LengthParsingMode, ParserContext};
use style::stylesheets::CssRuleType;
use style::supports::{Declaration, parse_condition_or_declaration};

#[dom_struct]
pub struct CSS {
    reflector_: Reflector,
}

impl CSS {
    /// http://dev.w3.org/csswg/cssom/#serialize-an-identifier
    pub fn Escape(_: &Window, ident: DOMString) -> Fallible<DOMString> {
        let mut escaped = String::new();
        serialize_identifier(&ident, &mut escaped).unwrap();
        Ok(DOMString::from(escaped))
    }

    /// https://drafts.csswg.org/css-conditional/#dom-css-supports
    pub fn Supports(win: &Window, property: DOMString, value: DOMString) -> bool {
        let decl = Declaration { prop: property.into(), val: value.into() };
        let url = win.Document().url();
        let context = ParserContext::new_for_cssom(&url, win.css_error_reporter(), Some(CssRuleType::Supports),
                                                   LengthParsingMode::Default,
                                                   QuirksMode::NoQuirks);
        decl.eval(&context)
    }

    /// https://drafts.csswg.org/css-conditional/#dom-css-supports
    pub fn Supports_(win: &Window, condition: DOMString) -> bool {
        let mut input = Parser::new(&condition);
        let cond = parse_condition_or_declaration(&mut input);
        if let Ok(cond) = cond {
            let url = win.Document().url();
            let context = ParserContext::new_for_cssom(&url, win.css_error_reporter(), Some(CssRuleType::Supports),
                                                       LengthParsingMode::Default,
                                                       QuirksMode::NoQuirks);
            cond.eval(&context)
        } else {
            false
        }
    }
}
