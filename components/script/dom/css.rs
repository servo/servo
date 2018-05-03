/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput, serialize_identifier};
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::Reflector;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::window::Window;
use dom::worklet::Worklet;
use dom_struct::dom_struct;
use style::context::QuirksMode;
use style::parser::ParserContext;
use style::stylesheets::CssRuleType;
use style::stylesheets::supports_rule::{Declaration, parse_condition_or_declaration};
use style_traits::ParsingMode;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct CSS<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    _p: PhantomData<TH>,
}

impl<TH: TypeHolderTrait> CSS<TH> {
    /// <http://dev.w3.org/csswg/cssom/#serialize-an-identifier>
    pub fn Escape(_: &Window<TH>, ident: DOMString) -> Fallible<DOMString> {
        let mut escaped = String::new();
        serialize_identifier(&ident, &mut escaped).unwrap();
        Ok(DOMString::from(escaped))
    }

    /// <https://drafts.csswg.org/css-conditional/#dom-css-supports>
    pub fn Supports(win: &Window<TH>, property: DOMString, value: DOMString) -> bool {
        let mut decl = String::new();
        serialize_identifier(&property, &mut decl).unwrap();
        decl.push_str(": ");
        decl.push_str(&value);
        let decl = Declaration(decl);
        let url = win.Document().url();
        let context = ParserContext::new_for_cssom(
            &url,
            Some(CssRuleType::Style),
            ParsingMode::DEFAULT,
            QuirksMode::NoQuirks,
            None,
        );
        decl.eval(&context)
    }

    /// <https://drafts.csswg.org/css-conditional/#dom-css-supports>
    pub fn Supports_(win: &Window<TH>, condition: DOMString) -> bool {
        let mut input = ParserInput::new(&condition);
        let mut input = Parser::new(&mut input);
        let cond = parse_condition_or_declaration(&mut input);
        if let Ok(cond) = cond {
            let url = win.Document().url();
            let context = ParserContext::new_for_cssom(
                &url,
                Some(CssRuleType::Style),
                ParsingMode::DEFAULT,
                QuirksMode::NoQuirks,
                None,
            );
            cond.eval(&context)
        } else {
            false
        }
    }

    /// <https://drafts.css-houdini.org/css-paint-api-1/#paint-worklet>
    pub fn PaintWorklet(win: &Window<TH>) -> DomRoot<Worklet<TH>> {
        win.paint_worklet()
    }
}
