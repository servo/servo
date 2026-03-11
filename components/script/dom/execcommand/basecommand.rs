/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::commands::defaultparagraphseparator::execute_default_paragraph_separator_command;
use crate::dom::execcommand::commands::delete::execute_delete_command;
use crate::dom::execcommand::commands::stylewithcss::execute_style_with_css_command;
use crate::dom::selection::Selection;

#[derive(Default, Clone, Copy, MallocSizeOf)]
pub(crate) enum DefaultSingleLineContainerName {
    #[default]
    Div,
    Paragraph,
}

impl From<DefaultSingleLineContainerName> for DOMString {
    fn from(default_single_line_container_name: DefaultSingleLineContainerName) -> Self {
        match default_single_line_container_name {
            DefaultSingleLineContainerName::Div => DOMString::from("div"),
            DefaultSingleLineContainerName::Paragraph => DOMString::from("p"),
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, MallocSizeOf, PartialEq)]
#[expect(unused)] // TODO(25005): implement all commands
pub(crate) enum CommandName {
    BackColor,
    Bold,
    CreateLink,
    DefaultParagraphSeparator,
    Delete,
    FontSize,
    HiliteColor,
    Italic,
    Redo,
    SelectAll,
    Strikethrough,
    StyleWithCss,
    Subscript,
    Superscript,
    Underline,
    Undo,
    Unlink,
    Usecss,
}

impl CommandName {
    /// <https://w3c.github.io/editing/docs/execCommand/#indeterminate>
    pub(crate) fn is_indeterminate(&self) -> bool {
        false
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#state>
    pub(crate) fn current_state(&self, document: &Document) -> Option<bool> {
        Some(match self {
            CommandName::StyleWithCss => {
                // https://w3c.github.io/editing/docs/execCommand/#the-stylewithcss-command
                // > True if the CSS styling flag is true, otherwise false.
                document.css_styling_flag()
            },
            _ => return None,
        })
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#value>
    pub(crate) fn current_value(&self, document: &Document) -> Option<DOMString> {
        Some(match self {
            CommandName::DefaultParagraphSeparator => {
                // https://w3c.github.io/editing/docs/execCommand/#the-defaultparagraphseparator-command
                // > Return the context object's default single-line container name.
                document.default_single_line_container_name().into()
            },
            _ => return None,
        })
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#action>
    pub(crate) fn execute(
        &self,
        cx: &mut js::context::JSContext,
        document: &Document,
        selection: &Selection,
        value: DOMString,
    ) -> bool {
        match self {
            CommandName::DefaultParagraphSeparator => {
                execute_default_paragraph_separator_command(document, value)
            },
            CommandName::Delete => execute_delete_command(cx, document, selection),
            CommandName::StyleWithCss => execute_style_with_css_command(document, value),
            _ => false,
        }
    }
}
