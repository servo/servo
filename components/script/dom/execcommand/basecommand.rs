/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::selection::Selection;

pub(crate) trait BaseCommand {
    /// <https://w3c.github.io/editing/docs/execCommand/#indeterminate>
    fn is_indeterminate(&self) -> bool {
        false
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#state>
    fn current_state(&self, _document: &Document) -> Option<bool> {
        None
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#value>
    fn current_value(&self, _document: &Document) -> Option<DOMString> {
        None
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#action>
    fn execute(
        &self,
        cx: &mut js::context::JSContext,
        document: &Document,
        selection: &Selection,
        value: DOMString,
    ) -> bool;
}

#[derive(Default, Clone, Copy, MallocSizeOf)]
pub(crate) enum DefaultSingleLineContainerName {
    #[default]
    Div,
    Paragraph,
}
