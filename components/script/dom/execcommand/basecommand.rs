/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::inheritance::Castable;
use style::properties::PropertyDeclarationId;
use style::properties::generated::LonghandId;
use style::values::specified::text::TextDecorationLine;
use style_traits::ToCss;

use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::commands::defaultparagraphseparator::execute_default_paragraph_separator_command;
use crate::dom::execcommand::commands::delete::execute_delete_command;
use crate::dom::execcommand::commands::fontsize::{
    execute_fontsize_command, font_size_loosely_equivalent, value_for_fontsize_command,
};
use crate::dom::execcommand::commands::stylewithcss::execute_style_with_css_command;
use crate::dom::execcommand::commands::underline::execute_underline_command;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlfontelement::HTMLFontElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::selection::Selection;
use crate::script_runtime::CanGc;

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

/// <https://w3c.github.io/editing/docs/execCommand/#relevant-css-property>
#[derive(Clone, Copy, Eq, PartialEq)]
#[expect(unused)] // TODO(25005): implement all commands
pub(crate) enum CssPropertyName {
    BackgroundColor,
    FontSize,
    FontWeight,
    FontStyle,
    TextDecorationLine,
}

impl CssPropertyName {
    fn resolved_value_for_node(&self, element: &Element) -> Option<DOMString> {
        let style = element.style()?;

        Some(
            match self {
                CssPropertyName::BackgroundColor => style.clone_background_color().to_css_string(),
                CssPropertyName::FontSize => {
                    // Font size is special, in that it can't use the resolved styles to compute
                    // values. That's because it is influenced by other factors as well, and it
                    // should also take into account size attributes of font elements.
                    //
                    // Therefore, we do a manual traversal up the chain to mimic what style
                    // resolution would have done. This also allows us to later check for
                    // loose equivalence for font elements, since we would return the size as an
                    // integer, without a size indicator (e.g. `px`).
                    //
                    // However, if no such relevant declaration exists, then we should fallback
                    // to pixels after all. For the effective command value, this essentially means
                    // we will overwrite it. For the value of the "fontsize" command, we would then
                    // need to convert it using [`legacy_font_size_for`].
                    return element
                        .upcast::<Node>()
                        .inclusive_ancestors(ShadowIncluding::No)
                        .find_map(|ancestor| {
                            if let Some(ancestor_font) = ancestor.downcast::<HTMLFontElement>() {
                                Some(ancestor_font.Size())
                            } else {
                                self.value_set_for_style(ancestor.downcast::<Element>()?)
                            }
                        })
                        .or_else(|| {
                            let pixels = style.get_font().font_size.computed_size().px();
                            Some(format!("{}px", pixels).into())
                        });
                },
                CssPropertyName::FontWeight => style.clone_font_weight().to_css_string(),
                CssPropertyName::FontStyle => style.clone_font_style().to_css_string(),
                CssPropertyName::TextDecorationLine => {
                    let text_decoration_line = style.get_text().text_decoration_line;
                    if text_decoration_line == TextDecorationLine::NONE {
                        return None;
                    }
                    text_decoration_line.to_css_string()
                },
            }
            .into(),
        )
    }

    /// Retrieves a respective css longhand value from the style declarations of an
    /// element. Note that this is different than the computed values, since this is
    /// only relevant when the author specified rules on the specific element.
    pub(crate) fn value_set_for_style(&self, element: &Element) -> Option<DOMString> {
        let style_attribute = element.style_attribute().borrow();
        let declarations = style_attribute.as_ref()?;
        let document = element.owner_document();
        let shared_lock = document.style_shared_lock();
        let read_lock = shared_lock.read();
        let style = declarations.read_with(&read_lock);

        let longhand_id = match self {
            CssPropertyName::BackgroundColor => LonghandId::BackgroundColor,
            CssPropertyName::FontSize => LonghandId::FontSize,
            CssPropertyName::FontWeight => LonghandId::FontWeight,
            CssPropertyName::FontStyle => LonghandId::FontStyle,
            CssPropertyName::TextDecorationLine => LonghandId::TextDecorationLine,
        };
        style
            .get(PropertyDeclarationId::Longhand(longhand_id))
            .and_then(|value| {
                let mut dest = String::new();
                value.0.to_css(&mut dest).ok()?;
                Some(dest.into())
            })
    }

    fn property_name(&self) -> DOMString {
        match self {
            CssPropertyName::BackgroundColor => "background-color",
            CssPropertyName::FontSize => "font-size",
            CssPropertyName::FontWeight => "font-weight",
            CssPropertyName::FontStyle => "font-style",
            CssPropertyName::TextDecorationLine => "text-decoration-line",
        }
        .into()
    }

    pub(crate) fn set_for_element(
        &self,
        cx: &mut JSContext,
        element: &HTMLElement,
        new_value: DOMString,
    ) {
        let style = element.Style(CanGc::from_cx(cx));

        let _ = style.SetProperty(cx, self.property_name(), new_value, "".into());
    }

    pub(crate) fn remove_from_element(&self, cx: &mut JSContext, element: &HTMLElement) {
        let _ = element
            .Style(CanGc::from_cx(cx))
            .RemoveProperty(cx, self.property_name());
    }

    pub(crate) fn value_for_element(&self, cx: &mut JSContext, element: &HTMLElement) -> DOMString {
        element
            .Style(CanGc::from_cx(cx))
            .GetPropertyValue(self.property_name())
    }
}

#[derive(Clone, Copy, Eq, Hash, MallocSizeOf, PartialEq)]
#[expect(unused)] // TODO(25005): implement all commands
pub(crate) enum CommandName {
    BackColor,
    Bold,
    Copy,
    CreateLink,
    Cut,
    DefaultParagraphSeparator,
    Delete,
    FontName,
    FontSize,
    ForeColor,
    FormatBlock,
    ForwardDelete,
    HiliteColor,
    Indent,
    InsertHorizontalRule,
    InsertHtml,
    InsertLineBreak,
    InsertOrderedList,
    InsertParagraph,
    InsertText,
    InsertUnorderedList,
    Italic,
    JustifyCenter,
    JustifyFull,
    JustifyLeft,
    JustifyRight,
    Outdent,
    Paste,
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
    pub(crate) fn current_state(&self, cx: &mut JSContext, document: &Document) -> Option<bool> {
        Some(match self {
            CommandName::StyleWithCss => {
                // https://w3c.github.io/editing/docs/execCommand/#the-stylewithcss-command
                // > True if the CSS styling flag is true, otherwise false.
                document.css_styling_flag()
            },
            CommandName::FontSize => {
                // Font size does not have a state defined for its command
                false
            },
            _ => {
                // https://w3c.github.io/editing/docs/execCommand/#state
                // > The state of a command is true if it is already in effect,
                // > in some sense specific to the command.
                let selection = document.GetSelection(CanGc::from_cx(cx))?;
                let active_range = selection.active_range()?;
                active_range
                    .first_formattable_contained_node()
                    .unwrap_or_else(|| active_range.start_container())
                    .effective_command_value(self)
                    .is_some()
            },
        })
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#value>
    pub(crate) fn current_value(
        &self,
        cx: &mut JSContext,
        document: &Document,
    ) -> Option<DOMString> {
        Some(match self {
            CommandName::DefaultParagraphSeparator => {
                // https://w3c.github.io/editing/docs/execCommand/#the-defaultparagraphseparator-command
                // > Return the context object's default single-line container name.
                document.default_single_line_container_name().into()
            },
            CommandName::FontSize => value_for_fontsize_command(cx, document)?,
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
        if self.are_equivalent_values(first, second) {
            return true;
        }
        // > or if the command is the fontSize command;
        // > one of the quantities is one of "x-small", "small", "medium", "large", "x-large", "xx-large", or "xxx-large";
        // > and the other quantity is the resolved value of "font-size" on a font element whose size attribute
        // > has the corresponding value set ("1" through "7" respectively).
        if let (CommandName::FontSize, Some(first), Some(second)) = (self, first, second) {
            font_size_loosely_equivalent(first, second)
        } else {
            false
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#relevant-css-property>
    pub(crate) fn relevant_css_property(&self) -> Option<CssPropertyName> {
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

    pub(crate) fn resolved_value_for_node(&self, element: &Element) -> Option<DOMString> {
        let property = self.relevant_css_property()?;
        property.resolved_value_for_node(element)
    }

    pub(crate) fn is_enabled_in_plaintext_only_state(&self) -> bool {
        matches!(
            self,
            CommandName::Copy |
                CommandName::Cut |
                CommandName::DefaultParagraphSeparator |
                CommandName::FormatBlock |
                CommandName::ForwardDelete |
                CommandName::InsertHtml |
                CommandName::InsertLineBreak |
                CommandName::InsertParagraph |
                CommandName::InsertText |
                CommandName::Paste |
                CommandName::Redo |
                CommandName::StyleWithCss |
                CommandName::Undo |
                CommandName::Usecss |
                CommandName::Delete
        )
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#action>
    pub(crate) fn execute(
        &self,
        cx: &mut JSContext,
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
            CommandName::Underline => execute_underline_command(cx, document, selection),
            _ => false,
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#inline-command-activated-values>
    pub(crate) fn inline_command_activated_values(&self) -> Vec<&str> {
        match self {
            // https://w3c.github.io/editing/docs/execCommand/#the-bold-command
            CommandName::Bold => vec!["bold", "600", "700", "800", "900"],
            // https://w3c.github.io/editing/docs/execCommand/#the-italic-command
            CommandName::Italic => vec!["italic", "oblique"],
            // https://w3c.github.io/editing/docs/execCommand/#the-strikethrough-command
            CommandName::Strikethrough => vec!["line-through"],
            // https://w3c.github.io/editing/docs/execCommand/#the-subscript-command
            CommandName::Subscript => vec!["subscript"],
            // https://w3c.github.io/editing/docs/execCommand/#the-superscript-command
            CommandName::Superscript => vec!["superscript"],
            // https://w3c.github.io/editing/docs/execCommand/#the-underline-command
            CommandName::Underline => vec!["underline"],
            _ => vec![],
        }
    }
}
