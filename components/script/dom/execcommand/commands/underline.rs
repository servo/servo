/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::execcommand::execcommands::DocumentExecCommandSupport;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-underline-command>
pub(crate) fn execute_underline_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    // > If queryCommandState("underline") returns true, set the selection's value to null.
    // > Otherwise set the selection's value to "underline". Either way, return true.
    let value = Some("underline".into())
        .filter(|_| !document.command_state_for_command(cx, "underline".into()));
    selection.set_the_selection_value(cx, value, CommandName::Underline, document);

    true
}
