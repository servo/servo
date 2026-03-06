/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::{BaseCommand, DefaultSingleLineContainerName};
use crate::dom::selection::Selection;

impl From<DefaultSingleLineContainerName> for DOMString {
    fn from(default_single_line_container_name: DefaultSingleLineContainerName) -> Self {
        match default_single_line_container_name {
            DefaultSingleLineContainerName::Div => DOMString::from("div"),
            DefaultSingleLineContainerName::Paragraph => DOMString::from("p"),
        }
    }
}

pub(crate) struct DefaultParagraphSeparatorCommand {}

impl BaseCommand for DefaultParagraphSeparatorCommand {
    /// <https://w3c.github.io/editing/docs/execCommand/#the-defaultparagraphseparator-command>
    fn execute(
        &self,
        _cx: &mut js::context::JSContext,
        document: &Document,
        _selection: &Selection,
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

    /// <https://w3c.github.io/editing/docs/execCommand/#the-defaultparagraphseparator-command>
    fn current_value(&self, document: &Document) -> Option<DOMString> {
        // > Return the context object's default single-line container name.
        Some(document.default_single_line_container_name().into())
    }
}
