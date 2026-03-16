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
pub(crate) struct NumberInputType {
    text_input_widget: DomRefCell<TextInputWidget>,
}

impl SpecificInputType for NumberInputType {
    fn sanitize_value(&self, _input: &HTMLInputElement, value: &mut DOMString) {
        if !value.is_valid_floating_point_number_string() {
            value.clear();
        }
        // Spec says that user agent "may" round the value
        // when it's suffering a step mismatch, but WPT tests
        // want it unrounded, and this matches other browser
        // behavior (typing an unrounded number into an
        // integer field box and pressing enter generally keeps
        // the number intact but makes the input box :invalid)
    }

    /// <https://html.spec.whatwg.org/multipage/#number-state-(type=number):concept-input-value-string-number>
    fn convert_string_to_number(&self, input: &str) -> Option<f64> {
        parse_floating_point_number(input)
    }

    /// <https://html.spec.whatwg.org/multipage/#number-state-(type=number):concept-input-value-string-number>
    fn convert_number_to_string(&self, input: f64) -> Option<DOMString> {
        let mut value = DOMString::from(input.to_string());
        value.set_best_representation_of_the_floating_point_number();
        Some(value)
    }

    /// <https://html.spec.whatwg.org/multipage/#number-state-(type=number):suffering-from-bad-input>
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
