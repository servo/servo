/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use time::OffsetDateTime;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::str::{DOMString, FromInputValueString, ToInputValueString};
use crate::dom::htmlinputelement::text_input_widget::TextInputWidget;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DateInputType {
    text_input_widget: DomRefCell<TextInputWidget>,
}

impl SpecificInputType for DateInputType {
    fn sanitize_value(&self, _input: &HTMLInputElement, value: &mut DOMString) {
        if !value.str().is_valid_date_string() {
            value.clear();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#date-state-(type=date):concept-input-value-string-number>
    fn convert_string_to_number(&self, input: &str) -> Option<f64> {
        // > The algorithm to convert a string to a number, given a string input, is as
        // > follows: If parsing a date from input results in an error, then return an
        // > error; otherwise, return the number of milliseconds elapsed from midnight
        // > UTC on the morning of 1970-01-01 (the time represented by the value
        // > "1970-01-01T00:00:00.0Z") to midnight UTC on the morning of the parsed
        // > date, ignoring leap seconds.
        input
            .parse_date_string()
            .map(|date_time| (date_time - OffsetDateTime::UNIX_EPOCH).whole_milliseconds() as f64)
    }

    /// <https://html.spec.whatwg.org/multipage/#date-state-(type=date):concept-input-value-string-number>
    fn convert_number_to_string(&self, input: f64) -> Option<DOMString> {
        OffsetDateTime::from_unix_timestamp_nanos((input * 1e6) as i128)
            .ok()
            .map(|value| value.to_date_string().into())
    }

    /// <https://html.spec.whatwg.org/multipage/#date-state-(type=date):concept-input-value-string-date>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in GetValueAsDate
    fn convert_string_to_naive_datetime(&self, value: DOMString) -> Option<OffsetDateTime> {
        value.str().parse_date_string()
    }

    /// <https://html.spec.whatwg.org/multipage/#date-state-(type=date):concept-input-value-date-string>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in SetValueAsDate
    fn convert_datetime_to_dom_string(&self, value: OffsetDateTime) -> DOMString {
        value.to_date_string().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#date-state-(type=date):suffering-from-bad-input>
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        !value.str().is_valid_date_string()
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
