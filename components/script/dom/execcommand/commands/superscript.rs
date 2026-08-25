/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-superscript-command>
pub(crate) fn execute_superscript_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    // Step 1. Call queryCommandState("superscript"), and let state be the result.
    let state = CommandName::Superscript.current_state(cx, document);
    // Step 2. Set the selection's value to null.
    selection.set_the_selection_value(cx, None, CommandName::Superscript, document);
    // Step 3. If state is false, set the selection's value to "superscript".
    if state.is_none_or(|state| !state) {
        selection.set_the_selection_value(
            cx,
            Some("superscript".into()),
            CommandName::Superscript,
            document,
        );
    }
    // Step 4. Return true.
    true
}
