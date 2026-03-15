/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;

use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::commands::defaultparagraphseparator::execute_default_paragraph_separator_command;
use crate::dom::execcommand::commands::delete::execute_delete_command;
use crate::dom::execcommand::commands::fontsize::execute_fontsize_command;
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum CssPropertyName {
    FontSize,
    FontWeight,
    FontStyle,
}

impl CssPropertyName {
    fn value_set_for_style(&self, _style: ServoArc<ComputedValues>) -> Option<DOMString> {
        match self {
            CssPropertyName::FontSize => {
                // TODO: How to retrieve the font-size if it is set?
                None
            },
            _ => None,
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

    /// <https://w3c.github.io/editing/docs/execCommand/#equivalent-values>
    pub(crate) fn are_equivalent_values(
        &self,
        first: Option<&DOMString>,
        second: Option<&DOMString>,
    ) -> bool {
        match (first, second) {
            // > Two quantities are equivalent values for a command if either both are null,
            (None, None) => true,
            (Some(first_str), Some(second_str)) => {
                // > or both are strings and the command defines equivalent values and they match the definition.
                match self {
                    CommandName::Bold => {
                        // https://w3c.github.io/editing/docs/execCommand/#the-bold-command
                        // > Either the two strings are equal, or one is "bold" and the other is "700",
                        // > or one is "normal" and the other is "400".
                        first_str == second_str ||
                            matches!(
                                (first_str.str().as_ref(), second_str.str().as_ref()),
                                ("bold", "700") |
                                    ("700", "bold") |
                                    ("normal", "400") |
                                    ("400", "normal")
                            )
                    },
                    // > or both are strings and they're equal and the command does not define any equivalent values,
                    _ => first_str == second_str,
                }
            },
            _ => false,
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#loosely-equivalent-values>
    pub(crate) fn are_loosely_equivalent_values(
        &self,
        first: Option<&DOMString>,
        second: Option<&DOMString>,
    ) -> bool {
        // > Two quantities are loosely equivalent values for a command if either they are equivalent values for the command,
        self.are_equivalent_values(first, second)
        // > or if the command is the fontSize command;
        // > one of the quantities is one of "x-small", "small", "medium", "large", "x-large", "xx-large", or "xxx-large";
        // > and the other quantity is the resolved value of "font-size" on a font element whose size attribute
        // > has the corresponding value set ("1" through "7" respectively).
        // TODO
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#relevant-css-property>
    fn relevant_css_property(&self) -> Option<CssPropertyName> {
        // > This is defined for certain inline formatting commands, and is used in algorithms specific to those commands.
        // > It is an implementation detail, and is not exposed to authors.
        Some(match self {
            CommandName::FontSize => CssPropertyName::FontSize,
            CommandName::Bold => CssPropertyName::FontWeight,
            CommandName::Italic => CssPropertyName::FontStyle,
            // > If a command does not have a relevant CSS property specified, it defaults to null.
            _ => return None,
        })
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#specified-command-value>
    pub(crate) fn specified_command_value(&self, element: &Element) -> Option<DOMString> {
        match self {
            // Step 1. If command is "backColor" or "hiliteColor" and the Element's display property does not have resolved value "inline", return null.
            CommandName::BackColor | CommandName::HiliteColor => {
                // TODO
            },
            // Step 2. If command is "createLink" or "unlink":
            CommandName::CreateLink | CommandName::Unlink => {
                // TODO
            },
            // Step 3. If command is "subscript" or "superscript":
            CommandName::Subscript | CommandName::Superscript => {
                // TODO
            },
            CommandName::Strikethrough => {
                // Step 4. If command is "strikethrough", and element has a style attribute set, and that attribute sets "text-decoration":
                // TODO
                // Step 5. If command is "strikethrough" and element is an s or strike element, return "line-through".
                // TODO
            },
            CommandName::Underline => {
                // Step 6. If command is "underline", and element has a style attribute set, and that attribute sets "text-decoration":
                // TODO
                // Step 7. If command is "underline" and element is a u element, return "underline".
                // TODO
            },
            _ => {},
        };
        // Step 8. Let property be the relevant CSS property for command.
        // Step 9. If property is null, return null.
        let property = self.relevant_css_property()?;
        // Step 10. If element has a style attribute set, and that attribute has the effect of setting property,
        // return the value that it sets property to.
        if let Some(value) = element
            .style()
            .and_then(|style| property.value_set_for_style(style))
        {
            return Some(value);
        }
        // Step 11. If element is a font element that has an attribute whose effect is to create a presentational hint for property,
        // return the value that the hint sets property to. (For a size of 7, this will be the non-CSS value "xxx-large".)
        // TODO

        // Step 12. If element is in the following list, and property is equal to the CSS property name listed for it,
        // return the string listed for it.
        let element_name = element.local_name();
        match property {
            CssPropertyName::FontWeight
                if element_name == &local_name!("b") || element_name == &local_name!("strong") =>
            {
                Some("bold".into())
            },
            CssPropertyName::FontStyle
                if element_name == &local_name!("i") || element_name == &local_name!("em") =>
            {
                Some("italic".into())
            },
            // Step 13. Return null.
            _ => None,
        }
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
            CommandName::FontSize => execute_fontsize_command(cx, document, selection, value),
            CommandName::StyleWithCss => execute_style_with_css_command(document, value),
            _ => false,
        }
    }
}
