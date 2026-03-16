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
pub(crate) struct DatetimeLocalInputType {
    text_input_widget: DomRefCell<TextInputWidget>,
}

impl SpecificInputType for DatetimeLocalInputType {
    fn sanitize_value(&self, _input: &HTMLInputElement, value: &mut DOMString) {
        let time = value
            .str()
            .parse_local_date_time_string()
            .map(|date_time| date_time.to_local_date_time_string());
        match time {
            Some(normalized_string) => *value = normalized_string.into(),
            None => value.clear(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type=datetime-local):concept-input-value-string-number>
    fn convert_string_to_number(&self, input: &str) -> Option<f64> {
        // > The algorithm to convert a string to a number, given a string input, is as
        // > follows: If parsing a date and time from input results in an error, then
        // > return an error; otherwise, return the number of milliseconds elapsed from
        // > midnight on the morning of 1970-01-01 (the time represented by the value
        // > "1970-01-01T00:00:00.0") to the parsed local date and time, ignoring leap
        // > seconds.
        input
            .parse_local_date_time_string()
            .map(|date_time| (date_time - OffsetDateTime::UNIX_EPOCH).whole_milliseconds() as f64)
    }

    /// <https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type=datetime-local):concept-input-value-string-number>
    fn convert_number_to_string(&self, input: f64) -> Option<DOMString> {
        OffsetDateTime::from_unix_timestamp_nanos((input * 1e6) as i128)
            .ok()
            .map(|value| value.to_local_date_time_string().into())
    }

    /// <https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type=datetime-local):concept-input-value-string-date>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in GetValueAsDate
    fn convert_string_to_naive_datetime(&self, value: DOMString) -> Option<OffsetDateTime> {
        value.str().parse_local_date_time_string()
    }

    /// <https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type=datetime-local):concept-input-value-date-string>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in SetValueAsDate
    fn convert_datetime_to_dom_string(&self, value: OffsetDateTime) -> DOMString {
        value.to_local_date_time_string().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type=datetime-local):suffering-from-bad-input>
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        !value.str().is_valid_local_date_time_string()
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
