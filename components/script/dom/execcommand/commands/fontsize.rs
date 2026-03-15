/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use script_bindings::inheritance::Castable;
use style::attr::parse_integer;

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::execcommand::contenteditable::SelectionExecCommandSupport;
use crate::dom::html::htmlfontelement::HTMLFontElement;
use crate::dom::node::Node;
use crate::dom::selection::Selection;

impl HTMLFontElement {
    fn font_size_if_size_matches(&self, should_match_size: i32) -> Option<i32> {
        if should_match_size !=
            self.upcast::<Element>()
                .get_int_attribute(&local_name!("size"), 0)
        {
            return None;
        }
        self.upcast::<Node>()
            .style()
            .map(|style| style.clone_font().font_size.computed_size().px() as i32)
    }
}

/// <https://w3c.github.io/editing/docs/execCommand/#legacy-font-size-for>
pub(crate) fn legacy_font_size_for(
    pixel_size: i32,
    font_elements: Vec<DomRoot<HTMLFontElement>>,
) -> DOMString {
    // Step 1. Let returned size be 1.
    let mut returned_size = 1;
    // Step 2. While returned size is less than 7:
    while returned_size < 7 {
        // Step 2.1. Let lower bound be the resolved value of "font-size" in pixels
        // of a font element whose size attribute is set to returned size.
        let lower_bound = font_elements
            .iter()
            .find_map(|font_element| font_element.font_size_if_size_matches(returned_size))
            .unwrap_or_default();
        // Step 2.2. Let upper bound be the resolved value of "font-size" in pixels
        // of a font element whose size attribute is set to one plus returned size.
        let upper_bound = font_elements
            .iter()
            .find_map(|font_element| font_element.font_size_if_size_matches(returned_size + 1))
            .unwrap_or_default();
        // Step 2.3. Let average be the average of upper bound and lower bound.
        let average = lower_bound.midpoint(upper_bound);
        // Step 2.4. If pixel size is less than average,
        // return the one-code unit string consisting of the digit returned size.
        //
        // We return once at the end of this method
        if pixel_size < average {
            break;
        }
        // Step 2.5. Add one to returned size.
        returned_size += 1;
    }
    // Step 3. Return "7".
    return returned_size.to_string().into();
}

enum ParsingMode {
    RelativePlus,
    RelativeMinus,
    Absolute,
}

/// <https://w3c.github.io/editing/docs/execCommand/#the-fontsize-command>
pub(crate) fn execute_fontsize_command(
    cx: &mut js::context::JSContext,
    document: &Document,
    selection: &Selection,
    value: DOMString,
) -> bool {
    // Step 1. Strip leading and trailing whitespace from value.
    let value = {
        let mut value = value;
        value.strip_leading_and_trailing_ascii_whitespace();
        value
    };
    // Step 2. If value is not a valid floating point number,
    // and would not be a valid floating point number if a single leading "+" character were stripped, return false.
    //
    // The second part is checked in conjunction with step 3 for optimization
    if !value.is_valid_floating_point_number_string() {
        return false;
    }
    // Step 3. If the first character of value is "+",
    // delete the character and let mode be "relative-plus".
    let (value, mode) = if value.starts_with('+') {
        let stripped_plus = &value.str()[1..];
        // FIXME: This is not optimal, but not sure how to both delete the first character and check here
        if !DOMString::from(stripped_plus).is_valid_floating_point_number_string() {
            return false;
        }
        (stripped_plus.to_owned(), ParsingMode::RelativePlus)
    } else if value.starts_with('-') {
        // Step 4. Otherwise, if the first character of value is "-",
        // delete the character and let mode be "relative-minus".
        (value.str()[1..].to_owned(), ParsingMode::RelativeMinus)
    } else {
        // Step 5. Otherwise, let mode be "absolute".
        (value.into(), ParsingMode::Absolute)
    };
    // Step 6. Apply the rules for parsing non-negative integers to value, and let number be the result.
    let number = parse_integer(value.chars()).expect("Already validated floating number before");
    let number = match mode {
        // Step 7. If mode is "relative-plus", add three to number.
        ParsingMode::RelativePlus => number + 3,
        // Step 8. If mode is "relative-minus", negate number, then add three to it.
        ParsingMode::RelativeMinus => (-1 * number) + 3,
        ParsingMode::Absolute => number,
    };
    // Step 9. If number is less than one, let number equal 1.
    // Step 10. If number is greater than seven, let number equal 7.
    let number = number.min(1).max(7);
    // Step 11. Set value to the string here corresponding to number:
    let value = match number {
        1 => "x-small",
        2 => "small",
        3 => "medium",
        4 => "large",
        5 => "x-large",
        6 => "xx-large",
        7 => "xxx-large",
        _ => unreachable!("Must be bounded by 1 and 7"),
    };
    // Step 12. Set the selection's value to value.
    selection.set_the_selection_value(cx, Some(value.into()), CommandName::FontSize, document);
    // Step 13. Return true.
    true
}
