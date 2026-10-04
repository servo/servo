/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::ElementBinding::ScrollLogicalPosition;
use script_bindings::codegen::GenericBindings::WindowBinding::ScrollBehavior;
use script_bindings::str::DOMString;
use style::attr::AttrValue;
use style::parser::ParserContext;
use style::properties::{PropertyDeclaration, longhands};
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style::values::specified::{AllowQuirks, LengthPercentage};
use style_traits::ParsingMode;
use stylo_dom::ElementState;

use crate::dom::bindings::codegen::Bindings::HTMLOrSVGOrMathMLElementBinding::FocusOptions;
use crate::dom::bindings::codegen::Bindings::MathMLElementBinding::MathMLElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::css::cssstyledeclaration::CSSStyleDeclaration;
use crate::dom::document::Document;
use crate::dom::document::focus::FocusableArea;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::node::focus::FocusTrigger;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::window::scrolling_box::{ScrollAxisState, ScrollRequirement};

#[dom_struct]
pub(crate) struct MathMLElement {
    element: Element,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
}

/// <https://w3c.github.io/mathml-core/#dom-mathmlelement>
impl MathMLElement {
    fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> MathMLElement {
        MathMLElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub(crate) fn new_inherited_with_state(
        state: ElementState,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> MathMLElement {
        MathMLElement {
            element: Element::new_inherited_with_state(
                state,
                tag_name,
                ns!(mathml),
                prefix,
                document,
            ),
            style_decl: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<MathMLElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(MathMLElement::new_inherited(tag_name, prefix, document)),
            document,
            proto,
        )
    }

    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}

impl VirtualMethods for MathMLElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.as_element() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);

        let element = self.as_element();
        if let (&local_name!("nonce"), mutation) = (attr.local_name(), mutation) {
            match mutation {
                AttributeMutation::Set(..) => {
                    let nonce = &**attr.value();
                    element.update_nonce_internal_slot(nonce.to_owned(), cx.no_gc());
                },
                AttributeMutation::Removed => {
                    element.update_nonce_internal_slot(String::new(), cx.no_gc());
                },
            }
        }
    }

    fn attribute_affects_presentational_hints(&self, attr: AttrRef<'_>) -> bool {
        matches!(
            attr.local_name(),
            &local_name!("mathcolor") |
                &local_name!("mathbackground") |
                &local_name!("mathsize") |
                &local_name!("dir")
        ) || self
            .super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("mathsize") => {
                let value_str = value.str();
                let doc = self.owner_document();
                let url = doc.url().into_url().into();

                let context = ParserContext::new(
                    Origin::Author,
                    &url,
                    None,
                    ParsingMode::ALLOW_UNITLESS_LENGTH,
                    doc.quirks_mode(),
                    Default::default(),
                    None,
                    None,
                    Default::default(),
                );

                let mut parser = Parser::new(&value_str);
                let val = LengthPercentage::parse_quirky(&context, &mut parser, AllowQuirks::No);

                AttrValue::LengthPercentage(value_str.to_string(), val.ok())
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}

impl<'dom> LayoutDom<'dom, MathMLElement> {
    /// Synthesizes presentational hints for MathML attributes that map directly to CSS properties.
    /// <https://w3c.github.io/mathml-core/#css-styling>
    pub(crate) fn synthesize_presentational_hints(
        self,
        document: LayoutDom<'dom, Document>,
        push: &mut impl FnMut(PropertyDeclaration),
    ) {
        let url_data = UrlExtraData(document.url_for_layout().get_arc());
        let parsing_mode =
            ParsingMode::ALLOW_UNITLESS_LENGTH | ParsingMode::ALLOW_ALL_NUMERIC_VALUES;

        let parser_context = ParserContext::new(
            Origin::Author,
            &url_data,
            Some(CssRuleType::Style),
            parsing_mode,
            document.quirks_mode(),
            Default::default(),
            None,
            None,
            Default::default(),
        );

        // Standard MathML Core presentational hints
        self.parse_mathml_attribute(
            &parser_context,
            "mathcolor",
            longhands::color::parse_declared,
            push,
        );
        self.parse_mathml_attribute(
            &parser_context,
            "mathbackground",
            longhands::background_color::parse_declared,
            push,
        );
        self.parse_mathml_attribute(
            &parser_context,
            "mathsize",
            longhands::font_size::parse_declared,
            push,
        );
        self.parse_mathml_attribute(
            &parser_context,
            "dir",
            longhands::direction::parse_declared,
            push,
        );
    }

    fn parse_mathml_attribute<F>(
        self,
        parser_context: &ParserContext,
        attr_name: &str,
        parse: F,
        push: &mut impl FnMut(PropertyDeclaration),
    ) where
        F: for<'i, 't> FnOnce(
            &ParserContext,
            &mut cssparser::Parser<'i>,
        ) -> Result<PropertyDeclaration, style_traits::ParseError>,
    {
        let element = self.upcast::<Element>();
        if let Some(value) = element.get_attr_val_for_layout(&ns!(), &LocalName::from(attr_name)) {
            let mut parser = cssparser::Parser::new(value);
            if let Ok(property) =
                parser.parse_entirely(|parse_input| parse(parser_context, parse_input))
            {
                push(property);
            }
        }
    }
}

impl MathMLElementMethods<crate::DomTypeHolder> for MathMLElement {
    global_event_handlers!();

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn Nonce(&self) -> DOMString {
        self.as_element().nonce_value().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn SetNonce(&self, cx: &mut JSContext, value: DOMString) {
        self.as_element()
            .update_nonce_internal_slot(String::from(value), cx.no_gc());
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fe-autofocus>
    fn Autofocus(&self) -> bool {
        self.element.has_attribute(&local_name!("autofocus"))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fe-autofocus>
    fn SetAutofocus(&self, cx: &mut JSContext, autofocus: bool) {
        self.element
            .set_bool_attribute(cx, &local_name!("autofocus"), autofocus);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-focus>
    fn Focus(&self, cx: &mut JSContext, options: &FocusOptions) {
        if !self
            .upcast::<Node>()
            .run_the_focusing_steps(cx, None, FocusTrigger::Other)
        {
            return;
        }

        if !options.preventScroll {
            let scroll_axis = ScrollAxisState {
                position: ScrollLogicalPosition::Center,
                requirement: ScrollRequirement::IfNotVisible,
            };
            self.upcast::<Element>().scroll_into_view_with_options(
                cx,
                ScrollBehavior::Smooth,
                scroll_axis,
                scroll_axis,
                None,
                None,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-blur>
    fn Blur(&self, cx: &mut JSContext) {
        if !self.as_element().focus_state() {
            return;
        }
        self.owner_document()
            .focus_handler()
            .focus(cx, &FocusableArea::Viewport);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tabindex>
    fn TabIndex(&self) -> i32 {
        self.element.tab_index()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tabindex>
    fn SetTabIndex(&self, cx: &mut JSContext, tab_index: i32) {
        self.element
            .set_attribute(cx, &local_name!("tabindex"), tab_index.into());
    }
}
