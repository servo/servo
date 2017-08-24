/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput, serialize_identifier};
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::PropertyDescriptorDictBinding::PropertyDescriptorDict;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::node::{Node, NodeDamage};
use dom::window::Window;
use dom_struct::dom_struct;
use style::context::QuirksMode;
use style::parser::ParserContext;
use style::properties_and_values::{self, PropertyRegistrationResult};
use style::stylesheets::CssRuleType;
use style::stylesheets::supports_rule::{Declaration, parse_condition_or_declaration};
use style_traits::PARSING_MODE_DEFAULT;

fn handle_property_registration_result(
    win: &Window,
    result: PropertyRegistrationResult
) -> Result<(), Error> {
    match result {
        PropertyRegistrationResult::Ok => {
            // Should lead to the RestyleHint::restyle_subtree() being inserted
            // eventually in handle_reflow due to checks in Stylist.
            if let Some(element) = win.Document().GetDocumentElement() {
                element.upcast::<Node>().dirty(NodeDamage::NodeStyleDamaged);
            }
            Ok(())
        },
        PropertyRegistrationResult::SyntaxError => Err(Error::Syntax),
        PropertyRegistrationResult::InvalidModificationError => Err(Error::InvalidModification),
        PropertyRegistrationResult::NotFoundError => Err(Error::NotFound),
    }
}

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
        let mut decl = String::new();
        serialize_identifier(&property, &mut decl).unwrap();
        decl.push_str(": ");
        decl.push_str(&value);
        let decl = Declaration(decl);
        let url = win.Document().url();
        let context = ParserContext::new_for_cssom(
            &url,
            win.css_error_reporter(),
            Some(CssRuleType::Style),
            PARSING_MODE_DEFAULT,
            QuirksMode::NoQuirks
        );
        decl.eval(&context)
    }

    /// https://drafts.csswg.org/css-conditional/#dom-css-supports
    pub fn Supports_(win: &Window, condition: DOMString) -> bool {
        let mut input = ParserInput::new(&condition);
        let mut input = Parser::new(&mut input);
        let cond = parse_condition_or_declaration(&mut input);
        if let Ok(cond) = cond {
            let url = win.Document().url();
            let context = ParserContext::new_for_cssom(
                &url,
                win.css_error_reporter(),
                Some(CssRuleType::Style),
                PARSING_MODE_DEFAULT,
                QuirksMode::NoQuirks
            );
            cond.eval(&context)
        } else {
            false
        }
    }

    /// https://drafts.css-houdini.org/css-properties-values-api/#dom-css-registerproperty
    pub fn RegisterProperty(win: &Window, options: &PropertyDescriptorDict) -> Result<(), Error> {
        let document = win.Document();
        let registered_property_set = document.registered_property_set();
        let mut guard = document.style_shared_lock().write();
        let registered_property_set = registered_property_set.write_with(&mut guard);
        handle_property_registration_result(
            win,
            properties_and_values::register_property(
                registered_property_set,
                &ParserContext::new_for_cssom(
                    &win.Document().url(),
                    win.css_error_reporter(),
                    /* rule_type */ None,
                    PARSING_MODE_DEFAULT,
                    win.Document().quirks_mode(),
                ),
                &*options.name,
                &options.syntax,
                options.inherits,
                options.initialValue.as_ref().map(|x| &**x),
            )
        )
    }

    /// https://drafts.css-houdini.org/css-properties-values-api/#dom-css-unregisterproperty
    pub fn UnregisterProperty(win: &Window, name: DOMString) -> Result<(), Error> {
        let document = win.Document();
        let registered_property_set = document.registered_property_set();
        let mut guard = document.style_shared_lock().write();
        let registered_property_set = registered_property_set.write_with(&mut guard);
        handle_property_registration_result(
            win,
            properties_and_values::unregister_property(registered_property_set, &*name)
        )
    }
}
