/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
use style::values::generics::NonNegative;
use style::values::specified;
use style_traits::ParsingMode;
use stylo_dom::ElementState;

use crate::dom::bindings::codegen::Bindings::HTMLOrSVGElementBinding::FocusOptions;
use crate::dom::bindings::codegen::Bindings::SVGElementBinding::SVGElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::css::cssstyledeclaration::{
    CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner,
};
use crate::dom::document::Document;
use crate::dom::document::focus::FocusableArea;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::scrolling_box::{ScrollAxisState, ScrollRequirement};
use crate::dom::svg::svgsvgelement::SVGSVGElement;

#[dom_struct]
pub(crate) struct SVGElement {
    element: Element,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
}

impl SVGElement {
    fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGElement {
        SVGElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub(crate) fn new_inherited_with_state(
        state: ElementState,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGElement {
        SVGElement {
            element: Element::new_inherited_with_state(state, tag_name, ns!(svg), prefix, document),
            style_decl: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<SVGElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(SVGElement::new_inherited(tag_name, prefix, document)),
            document,
            proto,
        )
    }

    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}

impl VirtualMethods for SVGElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.as_element() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
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
                    element.update_nonce_internal_slot(nonce.to_owned());
                },
                AttributeMutation::Removed => {
                    element.update_nonce_internal_slot(String::new());
                },
            }
        }
    }

    fn attribute_affects_presentational_hints(&self, attr: AttrRef<'_>) -> bool {
        matches!(
            attr.local_name(),
            &local_name!("fill") |
                &local_name!("fill-opacity") |
                &local_name!("fill-rule") |
                &local_name!("stroke") |
                &local_name!("stroke-width") |
                &local_name!("stroke-linecap") |
                &local_name!("stroke-linejoin") |
                &local_name!("stroke-dasharray") |
                &local_name!("stroke-dashoffset") |
                &local_name!("stroke-miterlimit") |
                &local_name!("stroke-opacity")
        ) || self
            .super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }
}

impl SVGElementMethods<crate::DomTypeHolder> for SVGElement {
    /// <https://html.spec.whatwg.org/multipage/#the-style-attribute>
    fn Style(&self, cx: &mut JSContext) -> DomRoot<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let global = self.owner_window();
            CSSStyleDeclaration::new(
                cx,
                &global,
                CSSStyleOwner::Element(Dom::from_ref(self.upcast())),
                None,
                CSSModificationAccess::ReadWrite,
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!();

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn Nonce(&self) -> DOMString {
        self.as_element().nonce_value().into()
    }

    /// <https://svgwg.org/svg2-draft/types.html#__svg__SVGElement__ownerSVGElement>
    fn GetOwnerSVGElement(&self) -> Option<DomRoot<SVGSVGElement>> {
        let mut ancestor = self.upcast::<Node>().parent_in_flat_tree();
        while let Some(node) = ancestor {
            let element = DomRoot::downcast::<Element>(node.clone())?;
            if element.namespace() != &ns!(svg) ||
                element.local_name() == &local_name!("foreignObject")
            {
                return None;
            }
            if let Some(svg) = DomRoot::downcast::<SVGSVGElement>(node.clone()) {
                return Some(svg);
            }
            ancestor = node.parent_in_flat_tree();
        }
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn SetNonce(&self, _cx: &mut JSContext, value: DOMString) {
        self.as_element()
            .update_nonce_internal_slot(String::from(value))
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
    fn Focus(&self, cx: &mut js::context::JSContext, options: &FocusOptions) {
        // 1. If the allow focus steps given this's node document return false, then return.
        // TODO: Implement this.

        // 2. Run the focusing steps for this.
        if !self.upcast::<Node>().run_the_focusing_steps(cx, None) {
            // The specification seems to imply we should scroll into view even if this element
            // is not a focusable area. No browser does this, so we return early in that case.
            // See https://github.com/whatwg/html/issues/12231.
            return;
        }

        // > 3. If options["focusVisible"] is true, or does not exist but in an
        // >    implementation-defined  way the user agent determines it would be best to do so,
        // >    then indicate focus. TODO: Implement this.
        // TODO: Implement this.

        // > 4. If options["preventScroll"] is false, then scroll a target into view given this,
        // >    "auto", "center", and "center".
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
    fn Blur(&self, cx: &mut js::context::JSContext) {
        // TODO: Run the unfocusing steps. Focus the top-level document, not
        //       the current document.
        if !self.as_element().focus_state() {
            return;
        }
        // <https://html.spec.whatwg.org/multipage/#unfocusing-steps>
        self.owner_document()
            .focus_handler()
            .focus(cx, FocusableArea::Viewport);
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

impl<'dom> LayoutDom<'dom, SVGElement> {
    pub(crate) fn synthesize_presentational_hints(
        self,
        document: LayoutDom<'dom, Document>,
        push: &mut impl FnMut(PropertyDeclaration),
    ) {
        let element = self.upcast::<Element>();

        if element.is::<SVGSVGElement>() {
            if let Some(width) = element
                .get_attr_for_layout(&ns!(), &local_name!("width"))
                .and_then(AttrValue::as_length_percentage)
            {
                push(PropertyDeclaration::Width(
                    specified::Size::LengthPercentage(NonNegative(width.clone())),
                ));
            }
            if let Some(height) = element
                .get_attr_for_layout(&ns!(), &local_name!("height"))
                .and_then(AttrValue::as_length_percentage)
            {
                push(PropertyDeclaration::Height(
                    specified::Size::LengthPercentage(NonNegative(height.clone())),
                ));
            }
        }
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

        self.parse_svg_attribute(
            &parser_context,
            "fill",
            longhands::fill::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "fill-opacity",
            longhands::fill_opacity::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "fill-rule",
            longhands::fill_rule::parse_declared,
            push,
        );

        self.parse_svg_attribute(
            &parser_context,
            "stroke",
            longhands::stroke::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "stroke-width",
            longhands::stroke_width::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "stroke-linecap",
            longhands::stroke_linecap::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "stroke-linejoin",
            longhands::stroke_linejoin::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "stroke-dasharray",
            longhands::stroke_dasharray::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "stroke-dashoffset",
            longhands::stroke_dashoffset::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "stroke-miterlimit",
            longhands::stroke_miterlimit::parse_declared,
            push,
        );
        self.parse_svg_attribute(
            &parser_context,
            "stroke-opacity",
            longhands::stroke_opacity::parse_declared,
            push,
        );
    }

    fn parse_svg_attribute<F>(
        self,
        parser_context: &ParserContext,
        attr_name: &str,
        parse: F,
        push: &mut impl FnMut(PropertyDeclaration),
    ) where
        F: for<'i, 't> FnOnce(
            &ParserContext,
            &mut cssparser::Parser<'i, 't>,
        ) -> Result<PropertyDeclaration, style_traits::ParseError<'i>>,
    {
        let element = self.upcast::<Element>();
        if let Some(value) = element.get_attr_val_for_layout(&ns!(), &LocalName::from(attr_name)) {
            let mut input = cssparser::ParserInput::new(value);
            let mut parser = cssparser::Parser::new(&mut input);
            if let Ok(property) =
                parser.parse_entirely(|parse_input| parse(parser_context, parse_input))
            {
                push(property);
            }
        }
    }
}
