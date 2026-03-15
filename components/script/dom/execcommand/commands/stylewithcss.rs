/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;

/// <https://w3c.github.io/editing/docs/execCommand/#the-stylewithcss-command>
pub(crate) fn execute_style_with_css_command(document: &Document, value: DOMString) -> bool {
    // > If value is an ASCII case-insensitive match for the string "false", set the CSS styling flag to false.
    // > Otherwise, set the CSS styling flag to true. Either way, return true.
    let value = value.to_ascii_lowercase().as_str() != "false";

    document.set_css_styling_flag(value);

    true
}
