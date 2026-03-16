/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cmp::Ordering;

use js::context::JSContext;
use time::{Month, OffsetDateTime};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::str::{DOMString, FromInputValueString, ToInputValueString};
use crate::dom::htmlinputelement::text_input_widget::TextInputWidget;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct MonthInputType {
    text_input_widget: DomRefCell<TextInputWidget>,
}

impl SpecificInputType for MonthInputType {
    fn sanitize_value(&self, _input: &HTMLInputElement, value: &mut DOMString) {
        if !value.str().is_valid_month_string() {
            value.clear();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#month-state-(type=month):concept-input-value-string-number>
    fn convert_string_to_number(&self, input: &str) -> Option<f64> {
        // > The algorithm to convert a string to a number, given a string input, is as
        // > follows: If parsing a month from input results in an error, then return an
        // > error; otherwise, return the number of months between January 1970 and the
        // > parsed month.
        //
        // This one returns number of months, not milliseconds (specification requires
        // this, presumably because number of milliseconds is not consistent across
        // months) the - 1.0 is because january is 1, not 0
        input.parse_month_string().map(|date_time| {
            ((date_time.year() - 1970) * 12) as f64 + (date_time.month() as u8 - 1) as f64
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#month-state-(type=month):concept-input-value-string-number>
    fn convert_number_to_string(&self, input: f64) -> Option<DOMString> {
        // > The algorithm to convert a number to a string, given a number input,
        // > is as follows: Return a valid month string that represents the month
        // > that has input months between it and January 1970.
        let date = OffsetDateTime::UNIX_EPOCH;
        let years = (input / 12.) as i32;
        let year = date.year() + years;

        let months = input as i32 - (years * 12);
        let months = match months.cmp(&0) {
            Ordering::Less => (12 - months) as u8,
            Ordering::Equal | Ordering::Greater => months as u8,
        } + 1;

        let date = date
            .replace_year(year)
            .ok()?
            .replace_month(Month::try_from(months).ok()?)
            .ok()?;
        Some(date.to_month_string().into())
    }

    /// <https://html.spec.whatwg.org/multipage/#month-state-(type=month):concept-input-value-string-date>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in GetValueAsDate
    fn convert_string_to_naive_datetime(&self, value: DOMString) -> Option<OffsetDateTime> {
        value.str().parse_month_string()
    }

    /// <https://html.spec.whatwg.org/multipage/#month-state-(type=month):concept-input-value-date-string>
    /// This does the safe Rust part of conversion; the unsafe JS Date part
    /// is in SetValueAsDate
    fn convert_datetime_to_dom_string(&self, value: OffsetDateTime) -> DOMString {
        value.to_month_string().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#month-state-(type=month):suffering-from-bad-input>
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        !value.str().is_valid_month_string()
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
