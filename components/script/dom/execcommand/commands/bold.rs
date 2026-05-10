/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::execcommand::execcommands::DocumentExecCommandSupport;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-bold-command>
pub(crate) fn execute_bold_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    // > If queryCommandState("bold") returns true, set the selection's value to "normal".
    // > Otherwise set the selection's value to "bold". Either way, return true.
    let value = if document.command_state_for_command(cx, "bold".into()) {
        Some("normal".into())
    } else {
        Some("bold".into())
    };
    selection.set_the_selection_value(cx, value, CommandName::Bold, document);

    true
}
