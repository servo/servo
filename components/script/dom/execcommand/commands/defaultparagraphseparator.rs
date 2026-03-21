/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::DefaultSingleLineContainerName;

/// <https://w3c.github.io/editing/docs/execCommand/#the-defaultparagraphseparator-command>
pub(crate) fn execute_default_paragraph_separator_command(
    document: &Document,
    value: DOMString,
) -> bool {
    // > Let value be converted to ASCII lowercase. If value is then equal to "p" or "div",
    // > set the context object's default single-line container name to value, then return true. Otherwise, return false.
    let value = match value.to_ascii_lowercase().as_str() {
        "div" => DefaultSingleLineContainerName::Div,
        "p" => DefaultSingleLineContainerName::Paragraph,
        _ => return false,
    };

    document.set_default_single_line_container_name(value);

    true
}
