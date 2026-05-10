/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use style::color::AbsoluteColor;

use crate::dom::bindings::str::{DOMString, FromInputValueString};
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-forecolor-command>
pub(crate) fn execute_forecolor_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
    value: DOMString,
) -> bool {
    // Step 1. If value is not a valid CSS color, prepend "#" to it.
    let value = if !value.str().is_valid_simple_color_string() {
        ("#".to_owned() + &*value.str()).into()
    } else {
        value
    };
    // Step 2. If value is still not a valid CSS color, or if it is currentColor, return false.
    //
    // TODO: figure out what to do with currentColor
    if !value.str().is_valid_simple_color_string() {
        return false;
    }
    // Step 3. Set the selection's value to value.
    selection.set_the_selection_value(cx, Some(value), CommandName::ForeColor, document);
    // Step 4. Return true.
    true
}

pub(crate) fn serialize_to_simple_color(absolute_color: AbsoluteColor) -> DOMString {
    let r = absolute_color
        .c0()
        .map(|v| (v * 255.0).round() as u8)
        .unwrap_or_default();
    let g = absolute_color
        .c1()
        .map(|v| (v * 255.0).round() as u8)
        .unwrap_or_default();
    let b = absolute_color
        .c2()
        .map(|v| (v * 255.0).round() as u8)
        .unwrap_or_default();

    format!("#{r:02x}{g:02x}{b:02x}").into()
}
