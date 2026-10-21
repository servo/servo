/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use time::{OffsetDateTime, Time};

use crate::dom::bindings::str::{DOMString, FromInputValueString, ToInputValueString};
use crate::dom::html::form_controls::htmlinputelement::HTMLInputElement;
use crate::dom::html::form_controls::input_type::SpecificInputType;
use crate::dom::html::form_controls::input_type::text_input_widget::TextInputWidget;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TimeInputType {
    text_input_widget: DomRefCell<TextInputWidget>,
}

impl SpecificInputType for TimeInputType {
    fn text_input_widget(&self) -> Option<&DomRefCell<TextInputWidget>> {
        Some(&self.text_input_widget)
    }

    fn sanitize_value(&self, _input: &HTMLInputElement, value: &mut DOMString) {
        if !value.str().is_valid_time_string() {
            value.clear();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#time-state-(type=time):concept-input-value-string-number>
    fn convert_string_to_number(&self, input: &str) -> Option<f64> {
        // > The algorithm to convert a string to a number, given a string input, is as
        // > follows: If parsing a time from input results in an error, then return an
        // > error; otherwise, return the number of milliseconds elapsed from midnight to
        // > the parsed time on a day with no time changes.
        input
            .parse_time_string()
            .map(|date_time| (date_time.time() - Time::MIDNIGHT).whole_milliseconds() as f64)
    }

    /// <https://html.spec.whatwg.org/multipage/#time-state-(type=time):concept-input-value-string-number>
    fn convert_number_to_string(&self, input: f64) -> Option<DOMString> {
        if input.is_nan() || input.is_infinite() {
            return None;
        }
        let day_ms = 86_400_000.0;
        let ms = input.rem_euclid(day_ms);
        let total_ms = ms.round() as u64;
        let total_seconds = total_ms / 1000;
        let millis = total_ms % 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let formatted = if millis > 0 {
            format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
        } else if seconds > 0 {
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        } else {
            format!("{hours:02}:{minutes:02}")
        };
        Some(formatted.into())
    }

    /// <https://html.spec.whatwg.org/multipage/#time-state-(type=time):concept-input-value-string-date>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in GetValueAsDate
    fn convert_string_to_naive_datetime(&self, value: DOMString) -> Option<OffsetDateTime> {
        value.str().parse_time_string()
    }

    /// <https://html.spec.whatwg.org/multipage/#time-state-(type=time):concept-input-value-date-string>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in SetValueAsDate
    fn convert_datetime_to_dom_string(&self, value: OffsetDateTime) -> DOMString {
        value.to_time_string().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#time-state-(type=time):suffering-from-bad-input>
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        !value.str().is_valid_time_string()
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
