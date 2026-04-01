/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Ref;

use cssparser::{Parser, ParserInput};
use embedder_traits::{EmbedderControlRequest, RgbColor};
use html5ever::{local_name, ns};
use js::context::JSContext;
use markup5ever::QualName;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use selectors::context::QuirksMode;
use style::color::{AbsoluteColor, ColorFlags, ColorSpace};
use style::parser::ParserContext;
use style::selector_parser::PseudoElement;
use style::stylesheets::{CssRuleType, Origin};
use style::values::specified::Color;
use style_traits::{ParsingMode, ToCss};
use url::Url;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::{DOMString, FromInputValueString};
use crate::dom::document_embedder_controls::ControlElement;
use crate::dom::element::{AttributeMutation, CustomElementCreationMode, Element, ElementCreator};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::node::{Node, NodeTraits, UnbindContext};

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ColorInputType {
    shadow_tree: DomRefCell<Option<ColorInputShadowTree>>,
}

impl ColorInputType {
    pub(crate) fn handle_color_picker_response(
        &self,
        input: &HTMLInputElement,
        response: Option<RgbColor>,
        can_gc: CanGc,
    ) {
        let Some(selected_color) = response else {
            return;
        };

        let formatted_color = format!(
            "#{:0>2x}{:0>2x}{:0>2x}",
            selected_color.red, selected_color.green, selected_color.blue
        );
        let _ = input.SetValue(formatted_color.into(), can_gc);
    }

    /// Get the shadow tree for this [`HTMLInputElement`], if it is created and valid, otherwise
    /// recreate the shadow tree and return it.
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
    ) -> Ref<'_, ColorInputShadowTree> {
        {
            if let Ok(shadow_tree) = Ref::filter_map(self.shadow_tree.borrow(), |shadow_tree| {
                shadow_tree.as_ref()
            }) {
                return shadow_tree;
            }
        }

        let element = input.upcast::<Element>();
        let shadow_root = element
            .shadow_root()
            .unwrap_or_else(|| element.attach_ua_shadow_root(cx, true));
        let shadow_root = shadow_root.upcast();
        *self.shadow_tree.borrow_mut() = Some(ColorInputShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, input)
    }

    /// <https://html.spec.whatwg.org/multipage/#update-a-color-well-control-color>
    pub(crate) fn update_a_color_well_control_color(
        input: &HTMLInputElement,
        element_value: &mut DOMString,
    ) {
        // Step 1. Assert: element is an input element whose type attribute is in the Color state.
        // Step 2. Let value be the result of running these steps:
        // Step 2.1 If element's dirty value flag is true, then return the result of getting an attribute
        // by namespace and local name given null, "value", and element.
        // FIXME: If we do this then things break
        // Step 2.2. Return element's value.
        let value = element_value.to_owned();

        // Step 3. Let color be the result of parsing value.
        // Step 4. If color is failure, then set color to opaque black.
        let color = parse_color_value(
            &value.str(),
            input.owner_document().url().as_url().to_owned(),
        );

        // Step 5. Set element's value to the result of serializing a color well control color
        // given element and color.
        Self::serialize_a_color_well_control_color(input, color, element_value);
    }

    /// <https://html.spec.whatwg.org/multipage/#serialize-a-color-well-control-color>
    fn serialize_a_color_well_control_color(
        input: &HTMLInputElement,
        mut color: AbsoluteColor,
        destination: &mut DOMString,
    ) {
        // Step 1. Assert: element is an input element whose type attribute is in the Color state.

        // Step 2. Let htmlCompatible be false.
        let mut html_compatible = false;

        // Step 3. If element's alpha attribute is not specified, then set color's alpha component to be fully opaque.
        let has_alpha = input.Alpha();
        if !has_alpha {
            color.alpha = 1.0;
        }

        // Step 4. If element's colorspace attribute is in the Limited sRGB state:
        let colorspace_attribute = Self::colorspace(input);
        if colorspace_attribute == ColorSpace::Srgb {
            // Step 4.1 Set color to color converted to the 'srgb' color space.
            color = color.to_color_space(ColorSpace::Srgb);

            // Step 4.2 Round each of color's components so they are in the range 0 to 255, inclusive.
            // Components are to be rounded towards +∞.
            color.components.0 = color.components.0.clamp(0.0, 1.0);
            color.components.1 = color.components.1.clamp(0.0, 1.0);
            color.components.2 = color.components.2.clamp(0.0, 1.0);

            // Step 4.3 If element's alpha attribute is not specified, then set htmlCompatible to true.
            if !has_alpha {
                html_compatible = true;
            }
            // Step 4.4 Otherwise, set color to color converted using the 'color()' function.
            // NOTE: Unsetting the legacy bit forces `color()`
            else {
                color.flags &= !ColorFlags::IS_LEGACY_SRGB;
            }
        }
        // Step 5. Otherwise:
        else {
            // Step 5.1 Assert: element's colorspace attribute is in the Display P3 state.
            debug_assert_eq!(colorspace_attribute, ColorSpace::DisplayP3);

            // Step 5.2 Set color to color converted to the 'display-p3' color space.
            color = color.to_color_space(ColorSpace::DisplayP3);
        }

        // Step 6. Return the result of serializing color. If htmlCompatible is true,
        // then do so with HTML-compatible serialization requested.
        *destination = if html_compatible {
            color = color.to_color_space(ColorSpace::Srgb);
            format!(
                "#{:0>2x}{:0>2x}{:0>2x}",
                (color.components.0 * 255.0).round() as usize,
                (color.components.1 * 255.0).round() as usize,
                (color.components.2 * 255.0).round() as usize
            )
        } else {
            color.to_css_string()
        }
        .into();
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-input-colorspace>
    fn colorspace(input: &HTMLInputElement) -> ColorSpace {
        let colorspace = input
            .upcast::<Element>()
            .get_string_attribute(&local_name!("colorspace"));
        if colorspace.str() == "display-p3" {
            ColorSpace::DisplayP3
        } else {
            ColorSpace::Srgb
        }
    }
}

impl SpecificInputType for ColorInputType {
    fn sanitize_value(&self, input: &HTMLInputElement, value: &mut DOMString) {
        // > The value sanitization algorithm is as follows:
        // > Run update a color well control color for the element.
        Self::update_a_color_well_control_color(input, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#color-state-(type=color):suffering-from-bad-input>
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        !value.str().is_valid_simple_color_string()
    }

    /// <https://html.spec.whatwg.org/multipage/#color-state-(type=color):input-activation-behavior>
    fn activation_behavior(
        &self,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        _can_gc: CanGc,
    ) {
        input.show_the_picker_if_applicable();
    }

    fn show_the_picker_if_applicable(&self, input: &HTMLInputElement) {
        let document = input.owner_document();
        let current_value = input.Value();
        let current_color = parse_color_value(
            &current_value.str(),
            input.owner_document().url().as_url().to_owned(),
        )
        .to_color_space(ColorSpace::Srgb);
        let current_color = RgbColor {
            red: (current_color.components.0 * 255.0).round() as u8,
            green: (current_color.components.1 * 255.0).round() as u8,
            blue: (current_color.components.2 * 255.0).round() as u8,
        };
        document.embedder_controls().show_embedder_control(
            ControlElement::ColorInput(DomRoot::from_ref(input)),
            EmbedderControlRequest::ColorPicker(current_color),
            None,
        );
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.get_or_create_shadow_tree(cx, input).update(cx, input)
    }

    fn attribute_mutated(
        &self,
        _cx: &mut JSContext,
        input: &HTMLInputElement,
        attr: &Attr,
        _mutation: AttributeMutation,
    ) {
        match *attr.local_name() {
            local_name!("alpha") | local_name!("colorspace") => {
                // https://html.spec.whatwg.org/multipage/#attr-input-colorspace
                // > Whenever the element's alpha or colorspace attributes are changed,
                // the user agent must run update a color well control color given the element.
                let mut textinput = input.textinput_mut();
                let mut value = textinput.get_content();
                Self::update_a_color_well_control_color(input, &mut value);
                textinput.set_content(value);
            },
            _ => {},
        }
    }

    fn unbind_from_tree(
        &self,
        input: &HTMLInputElement,
        _form_owner: Option<DomRoot<HTMLFormElement>>,
        _context: &UnbindContext,
        _can_gc: CanGc,
    ) {
        input
            .owner_document()
            .embedder_controls()
            .hide_embedder_control(input.upcast());
    }
}

fn parse_color_value(value: &str, url: Url) -> AbsoluteColor {
    // TODO: Use a dummy url here, like gecko
    // https://searchfox.org/firefox-main/rev/3eaf7e2acf8186eb7aa579561eaa1312cb89132b/servo/ports/geckolib/glue.rs#8931
    let urlextradata = url.into();
    let context = ParserContext::new(
        Origin::Author,
        &urlextradata,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        Default::default(),
        None,
        None,
    );
    let mut input = ParserInput::new(value);
    let mut input = Parser::new(&mut input);
    Color::parse_and_compute(&context, &mut input, None)
        .map(|computed_color| computed_color.resolve_to_absolute(&AbsoluteColor::BLACK))
        .unwrap_or(AbsoluteColor::BLACK)
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Contains references to the elements in the shadow tree for `<input type=color>`.
///
/// The shadow tree consists of a single div with the currently selected color as
/// the background.
pub(crate) struct ColorInputShadowTree {
    color_value: Dom<Element>,
}

impl ColorInputShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        let color_value = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        Node::replace_all(cx, Some(color_value.upcast()), shadow_root.upcast());
        color_value
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::ColorSwatch);

        Self {
            color_value: color_value.as_traced(),
        }
    }

    pub(crate) fn update(&self, cx: &mut JSContext, input_element: &HTMLInputElement) {
        let value = input_element.Value();
        let style = format!("background-color: {value}");
        self.color_value.set_string_attribute(
            &local_name!("style"),
            style.into(),
            CanGc::from_cx(cx),
        );
    }
}
