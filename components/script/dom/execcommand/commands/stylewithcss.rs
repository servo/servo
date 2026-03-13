/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::BaseCommand;
use crate::dom::selection::Selection;

pub(crate) struct StyleWithCssCommand {}

impl BaseCommand for StyleWithCssCommand {
    /// <https://w3c.github.io/editing/docs/execCommand/#the-stylewithcss-command>
    fn execute(
        &self,
        _cx: &mut js::context::JSContext,
        document: &Document,
        _selection: &Selection,
        value: DOMString,
    ) -> bool {
        // > If value is an ASCII case-insensitive match for the string "false", set the CSS styling flag to false.
        // > Otherwise, set the CSS styling flag to true. Either way, return true.
        let value = value.to_ascii_lowercase().as_str() != "false";

        document.set_css_styling_flag(value);

        true
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#the-stylewithcss-command>
    fn current_state(&self, document: &Document) -> Option<bool> {
        Some(document.css_styling_flag())
    }
}
