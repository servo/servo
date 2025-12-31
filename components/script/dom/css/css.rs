/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput, serialize_identifier};
use dom_struct::dom_struct;
use layout_api::{PropertyRegistration, RegisterPropertyError};
use script_bindings::codegen::GenericBindings::CSSBinding::PropertyDefinition;
use style::stylesheets::supports_rule::{Declaration, parse_condition_or_declaration};
use style::stylesheets::{CssRuleType, UrlExtraData};
use style_traits::ParsingMode;

use crate::css::parser_context_for_anonymous_content;
use crate::dom::bindings::codegen::Bindings::CSSBinding::CSSMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::dom::worklet::Worklet;

#[dom_struct]
#[allow(clippy::upper_case_acronyms)]
pub(crate) struct CSS {
    reflector_: Reflector,
}

impl CSSMethods<crate::DomTypeHolder> for CSS {
    /// <https://drafts.csswg.org/cssom/#the-css.escape()-method>
    fn Escape(_: &Window, ident: DOMString) -> Fallible<DOMString> {
        let mut escaped = String::new();
        serialize_identifier(&ident.str(), &mut escaped).unwrap();
        Ok(DOMString::from(escaped))
    }

    /// <https://drafts.csswg.org/css-conditional/#dom-css-supports>
    fn Supports(win: &Window, property: DOMString, value: DOMString) -> bool {
        let mut decl = String::new();
        serialize_identifier(&property.str(), &mut decl).unwrap();
        decl.push_str(": ");
        decl.push_str(&value.str());
        let decl = Declaration(decl);
        let url_data = UrlExtraData(win.Document().url().get_arc());
        let context = parser_context_for_anonymous_content(
            CssRuleType::Style,
            ParsingMode::DEFAULT,
            &url_data,
        );
        decl.eval(&context)
    }

    /// <https://drafts.csswg.org/css-conditional/#dom-css-supports>
    fn Supports_(win: &Window, condition: DOMString) -> bool {
        let condition = condition.str();
        let mut input = ParserInput::new(&condition);
        let mut input = Parser::new(&mut input);
        let cond = match parse_condition_or_declaration(&mut input) {
            Ok(c) => c,
            Err(..) => return false,
        };

        let url_data = UrlExtraData(win.Document().url().get_arc());
        let context = parser_context_for_anonymous_content(
            CssRuleType::Style,
            ParsingMode::DEFAULT,
            &url_data,
        );
        cond.eval(&context)
    }

    /// <https://drafts.css-houdini.org/css-paint-api-1/#paint-worklet>
    fn PaintWorklet(win: &Window) -> DomRoot<Worklet> {
        win.paint_worklet()
    }

    /// <https://drafts.css-houdini.org/css-properties-values-api/#the-registerproperty-function>
    fn RegisterProperty(window: &Window, property_definition: &PropertyDefinition) -> Fallible<()> {
        let property_registration = PropertyRegistration {
            name: property_definition.name.str().to_owned(),
            inherits: property_definition.inherits,
            url_data: UrlExtraData(window.get_url().get_arc()),
            initial_value: property_definition
                .initialValue
                .as_ref()
                .map(|value| value.str().to_owned()),
            syntax: property_definition.syntax.str().to_owned(),
        };

        window
            .layout_mut()
            .register_custom_property(property_registration)
            .map_err(|error| match error {
                RegisterPropertyError::InvalidName |
                RegisterPropertyError::InvalidSyntax |
                RegisterPropertyError::InvalidInitialValue |
                RegisterPropertyError::NoInitialValue |
                RegisterPropertyError::InitialValueNotComputationallyIndependent => {
                    Error::Syntax(None)
                },
                RegisterPropertyError::AlreadyRegistered => Error::InvalidModification(None),
            })?;

        Ok(())
    }
}
