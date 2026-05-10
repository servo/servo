/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-fontname-command>
pub(crate) fn execute_fontname_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
    value: DOMString,
) -> bool {
    // > Set the selection's value to value, then return true.
    selection.set_the_selection_value(cx, Some(value), CommandName::FontName, document);

    true
}
