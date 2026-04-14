/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use js::context::JSContext;
use servo_config::pref;
use style::attr::parse_integer;
use style::values::computed::CSSPixelLength;
use style::values::specified::FontSize;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::selection::Selection;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/editing/docs/execCommand/#legacy-font-size-for>
pub(crate) fn legacy_font_size_for(pixel_size: f32, document: &Document) -> DOMString {
    let quirks_mode = document.quirks_mode();
    let base_size = CSSPixelLength::from(Au::from_f32_px(pref!(fonts_default_size) as f32));
    // Step 1. Let returned size be 1.
    let mut returned_size = 1;
    // Step 2. While returned size is less than 7:
    while returned_size < 7 {
        // Step 2.1. Let lower bound be the resolved value of "font-size" in pixels
        // of a font element whose size attribute is set to returned size.
        let FontSize::Keyword(lower_keyword) = FontSize::from_html_size(returned_size) else {
            unreachable!("Always computed as keyword");
        };
        let lower_bound = lower_keyword
            .kw
            .to_length_without_context(quirks_mode, base_size);
        // Step 2.2. Let upper bound be the resolved value of "font-size" in pixels
        // of a font element whose size attribute is set to one plus returned size.
        let FontSize::Keyword(upper_keyword) = FontSize::from_html_size(returned_size + 1) else {
            unreachable!("Always computed as keyword");
        };
        let upper_bound = upper_keyword
            .kw
            .to_length_without_context(quirks_mode, base_size);
        // Step 2.3. Let average be the average of upper bound and lower bound.
        let average = (lower_bound.0.px() + upper_bound.0.px()) / 2.0;
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
    returned_size.to_string().into()
}

enum ParsingMode {
    RelativePlus,
    RelativeMinus,
    Absolute,
}

/// <https://w3c.github.io/editing/docs/execCommand/#the-fontsize-command>
pub(crate) fn execute_fontsize_command(
    cx: &mut JSContext,
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
        ParsingMode::RelativeMinus => (-number) + 3,
        ParsingMode::Absolute => number,
    };
    // Step 9. If number is less than one, let number equal 1.
    // Step 10. If number is greater than seven, let number equal 7.
    let number = number.clamp(1, 7) as u32;
    // Step 11. Set value to the string here corresponding to number:
    let value = font_size_to_css_font(&number);
    // Step 12. Set the selection's value to value.
    selection.set_the_selection_value(cx, Some(value.into()), CommandName::FontSize, document);
    // Step 13. Return true.
    true
}

/// <https://w3c.github.io/editing/docs/execCommand/#the-fontsize-command>
pub(crate) fn value_for_fontsize_command(
    cx: &mut JSContext,
    document: &Document,
) -> Option<DOMString> {
    // Step 1. If the active range is null, return the empty string.
    let selection = document.GetSelection(CanGc::from_cx(cx))?;
    let active_range = selection.active_range()?;
    // Step 2. Let pixel size be the effective command value of the first formattable
    // node that is effectively contained in the active range, or if there is no such node,
    // the effective command value of the active range's start node,
    // in either case interpreted as a number of pixels.
    let command_value = active_range
        .first_formattable_contained_node()
        .unwrap_or_else(|| active_range.start_container())
        .effective_command_value(&CommandName::FontSize)?;
    // Step 3. Return the legacy font size for pixel size.
    maybe_normalize_pixels(&command_value, document)
}

/// Only in the case we have resolved to actual pixels, we need to
/// do its conversion. In other cases, we already have the relevant
/// font size or corresponding css value. This avoids expensive
/// conversions of pixels to other values.
pub(crate) fn maybe_normalize_pixels(
    command_value: &DOMString,
    document: &Document,
) -> Option<DOMString> {
    if command_value.ends_with_str("px") {
        command_value.str()[0..command_value.len() - 2]
            .parse::<f32>()
            .ok()
            .map(|value| legacy_font_size_for(value, document))
    } else {
        Some(css_font_to_font_size(&command_value.str()).into())
    }
}

fn css_font_to_font_size(str_: &str) -> &str {
    match str_ {
        "x-small" => "1",
        "small" => "2",
        "medium" => "3",
        "large" => "4",
        "x-large" => "5",
        "xx-large" => "6",
        "xxx-large" => "7",
        _ => str_,
    }
}

pub(crate) fn font_size_to_css_font(value: &u32) -> &str {
    match value {
        1 => "x-small",
        2 => "small",
        3 => "medium",
        4 => "large",
        5 => "x-large",
        6 => "xx-large",
        7 => "xxx-large",
        _ => unreachable!(),
    }
}

/// Handles fontsize command part of
/// <https://w3c.github.io/editing/docs/execCommand/#loosely-equivalent-values>
pub(crate) fn font_size_loosely_equivalent(first: &DOMString, second: &DOMString) -> bool {
    // > one of the quantities is one of "x-small", "small", "medium", "large", "x-large", "xx-large", or "xxx-large";
    // > and the other quantity is the resolved value of "font-size" on a font element whose size attribute
    // > has the corresponding value set ("1" through "7" respectively).
    css_font_to_font_size(&first.str()) == second || first == css_font_to_font_size(&second.str())
}
