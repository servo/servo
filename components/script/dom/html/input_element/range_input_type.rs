/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use js::context::JSContext;
use script_bindings::domstring::parse_floating_point_number;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::str::DOMString;
use crate::dom::htmlinputelement::text_input_widget::TextInputWidget;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct RangeInputType {
    text_input_widget: DomRefCell<TextInputWidget>,
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
        self.text_input_widget
            .borrow()
            .update_shadow_tree(cx, input)
    }

    fn update_placeholder_contents(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.text_input_widget
            .borrow()
            .update_placeholder_contents(cx, input)
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
