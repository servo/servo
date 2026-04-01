/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Ref;

use html5ever::{local_name, ns};
use js::context::JSContext;
use markup5ever::QualName;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::domstring::parse_floating_point_number;
use script_bindings::root::Dom;
use script_bindings::script_runtime::CanGc;
use style::selector_parser::PseudoElement;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::node::{Node, NodeTraits};

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct RangeInputType {
    shadow_tree: DomRefCell<Option<RangeInputShadowTree>>,
}

impl RangeInputType {
    /// Get the shadow tree for this [`HTMLInputElement`], if it is created and valid, otherwise
    /// recreate the shadow tree and return it.
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
    ) -> Ref<'_, RangeInputShadowTree> {
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
        *self.shadow_tree.borrow_mut() = Some(RangeInputShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, input)
    }
}

impl SpecificInputType for RangeInputType {
    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):value-sanitization-algorithm>
    fn sanitize_value(&self, input: &HTMLInputElement, value: &mut DOMString) {
        if !value.is_valid_floating_point_number_string() {
            *value = DOMString::from(input.default_range_value().to_string());
        }
        if let Ok(fval) = &value.parse::<f64>() {
            let mut fval = *fval;
            // comparing max first, because if they contradict
            // the spec wants min to be the one that applies
            if let Some(max) = input.maximum() {
                if fval > max {
                    fval = max;
                }
            }
            if let Some(min) = input.minimum() {
                if fval < min {
                    fval = min;
                }
            }
            // https://html.spec.whatwg.org/multipage/#range-state-(type=range):suffering-from-a-step-mismatch
            // Spec does not describe this in a way that lends itself to
            // reproducible handling of floating-point rounding;
            // Servo may fail a WPT test because .1 * 6 == 6.000000000000001
            if let Some(allowed_value_step) = input.allowed_value_step() {
                let step_base = input.step_base();
                let steps_from_base = (fval - step_base) / allowed_value_step;
                if steps_from_base.fract() != 0.0 {
                    // not an integer number of steps, there's a mismatch
                    // round the number of steps...
                    let int_steps = round_halves_positive(steps_from_base);
                    // and snap the value to that rounded value...
                    fval = int_steps * allowed_value_step + step_base;

                    // but if after snapping we're now outside min..max
                    // we have to adjust! (adjusting to min last because
                    // that "wins" over max in the spec)
                    if let Some(stepped_maximum) = input.stepped_maximum() {
                        if fval > stepped_maximum {
                            fval = stepped_maximum;
                        }
                    }
                    if let Some(stepped_minimum) = input.stepped_minimum() {
                        if fval < stepped_minimum {
                            fval = stepped_minimum;
                        }
                    }
                }
            }
            *value = DOMString::from(fval.to_string());
        };
    }

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):concept-input-value-string-number>
    fn convert_string_to_number(&self, input: &str) -> Option<f64> {
        parse_floating_point_number(input)
    }

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):concept-input-value-string-number>
    fn convert_number_to_string(&self, input: f64) -> Option<DOMString> {
        let mut value = DOMString::from(input.to_string());
        value.set_best_representation_of_the_floating_point_number();
        Some(value)
    }

    /// <https://html.spec.whatwg.org/multipage/#range-state-(type=range):suffering-from-bad-input>
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        !value.is_valid_floating_point_number_string()
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.get_or_create_shadow_tree(cx, input).update(cx, input)
    }
}

fn round_halves_positive(n: f64) -> f64 {
    // WHATWG specs about input steps say to round to the nearest step,
    // rounding halves always to positive infinity.
    // This differs from Rust's .round() in the case of -X.5.
    if n.fract() == -0.5 {
        n.ceil()
    } else {
        n.round()
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Contains references to the elements in the shadow tree for `<input type=range>`.
///
/// The shadow tree consists of three div in the following structure:
/// <input type=range>
/// ├─ ::slider-track
/// │ └─ ::slider-fill
/// └─ ::slider-thumb
pub(crate) struct RangeInputShadowTree {
    slider_fill: Dom<Element>,
    slider_thumb: Dom<Element>,
    slider_track: Dom<Element>,
}

impl RangeInputShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        Node::replace_all(cx, None, shadow_root.upcast::<Node>());

        let slider_fill = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        let slider_thumb = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        let slider_track = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        shadow_root
            .upcast::<Node>()
            .AppendChild(cx, slider_track.upcast::<Node>())
            .unwrap();
        slider_track
            .upcast::<Node>()
            .AppendChild(cx, slider_fill.upcast::<Node>())
            .unwrap();
        shadow_root
            .upcast::<Node>()
            .AppendChild(cx, slider_thumb.upcast::<Node>())
            .unwrap();

        slider_fill
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::SliderFill);
        slider_thumb
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::SliderThumb);
        slider_track
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::SliderTrack);

        Self {
            slider_fill: slider_fill.as_traced(),
            slider_thumb: slider_thumb.as_traced(),
            slider_track: slider_track.as_traced(),
        }
    }

    pub(crate) fn update(&self, cx: &mut JSContext, input_element: &HTMLInputElement) {
        let value = input_element.Value();
        let min = input_element
            .minimum()
            .expect("This value should be available for range input.");
        let max = input_element
            .maximum()
            .expect("This value should be available for range input.");
        let value_num = input_element
            .convert_string_to_number(&value.str())
            .unwrap_or(input_element.default_range_value());

        let percent = if min > max || (max - min).abs() < f64::EPSILON {
            0.0
        } else {
            let clamped_value = value_num.clamp(min, max);
            (clamped_value - min) / (max - min) * 100.0
        };

        self.slider_thumb.set_string_attribute(
            &local_name!("style"),
            format!("inset-inline-start: {percent}% !important;").into(),
            CanGc::from_cx(cx),
        );
        self.slider_fill.set_string_attribute(
            &local_name!("style"),
            format!("width: {percent}% !important;").into(),
            CanGc::from_cx(cx),
        );
    }
}
