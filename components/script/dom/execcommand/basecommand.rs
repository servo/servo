/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::inheritance::Castable;
use style::attr::parse_legacy_color;
use style::color::ColorFlags;
use style::properties::PropertyDeclarationId;
use style::properties::generated::{LonghandId, ShorthandId};
use style::values::specified::text::TextDecorationLine;
use style_traits::ToCss;

use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFontElementBinding::HTMLFontElementMethods;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::commands::backcolor::execute_backcolor_command;
use crate::dom::execcommand::commands::bold::execute_bold_command;
use crate::dom::execcommand::commands::defaultparagraphseparator::execute_default_paragraph_separator_command;
use crate::dom::execcommand::commands::delete::execute_delete_command;
use crate::dom::execcommand::commands::fontname::execute_fontname_command;
use crate::dom::execcommand::commands::fontsize::{
    execute_fontsize_command, font_size_loosely_equivalent, value_for_fontsize_command,
};
use crate::dom::execcommand::commands::forecolor::execute_forecolor_command;
use crate::dom::execcommand::commands::hilitecolor::execute_hilitecolor_command;
use crate::dom::execcommand::commands::italic::execute_italic_command;
use crate::dom::execcommand::commands::strikethrough::execute_strikethrough_command;
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
pub(crate) enum CssPropertyName {
    BackgroundColor,
    Color,
    FontFamily,
    FontSize,
    FontWeight,
    FontStyle,
    TextDecoration,
    TextDecorationLine,
}

impl CssPropertyName {
    pub(crate) fn resolved_value_for_node(&self, element: &Element) -> Option<DOMString> {
        let style = element.style()?;

        Some(
            match self {
                CssPropertyName::BackgroundColor => {
                    let background_color = style.clone_background_color();
                    if let Some(absolute_color) = background_color.as_absolute() {
                        // Used as an early-exit when figuring out on which element to resolve
                        // the style in `effective_command_value`
                        if absolute_color.is_transparent() {
                            return None;
                        }
                        // Requires legacy SRGB syntax, which is what all tests expect.
                        // E.g. it should use `rgba()` instead of `rgb()`, even if the alpha
                        // is zero.
                        let mut absolute_color = *absolute_color;
                        absolute_color.flags.insert(ColorFlags::IS_LEGACY_SRGB);
                        return Some(absolute_color.to_css_string().into());
                    }
                    background_color.to_css_string()
                },
                CssPropertyName::Color => {
                    // Detached font elements (e.g. does created with `document.createElement`
                    // and not yet present in DOM) dont have a computed style for `color`.
                    // Since we create detached parent elements and compute "effective command
                    // value" for these elements, we need to special case this. Otherwise, we
                    // would add both a `color` attribute and `color` style declaration
                    // to a parent font element.
                    if let Some(ancestor_font) = element.downcast::<HTMLFontElement>() {
                        let color = ancestor_font.Color();
                        if !color.is_empty() {
                            return Some(color);
                        }
                    }
                    style.clone_color().to_css_string()
                },
                CssPropertyName::FontFamily => {
                    // Detached font elements (e.g. does created with `document.createElement`
                    // and not yet present in DOM) dont have a computed style for `fontFamily`.
                    // Since we create detached parent elements and compute "effective command
                    // value" for these elements, we need to special case this. Otherwise, we
                    // would add both a `face` attribute and `font-family` style declaration
                    // to a parent font element.
                    if let Some(ancestor_font) = element.downcast::<HTMLFontElement>() {
                        let face = ancestor_font.Face();
                        if !face.is_empty() {
                            return Some(face);
                        }
                    }
                    style.clone_font_family().to_css_string()
                },
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
                CssPropertyName::TextDecoration => unreachable!("Should use longhands instead"),
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
            CssPropertyName::Color => LonghandId::Color,
            CssPropertyName::FontFamily => LonghandId::FontFamily,
            CssPropertyName::FontSize => LonghandId::FontSize,
            CssPropertyName::FontWeight => LonghandId::FontWeight,
            CssPropertyName::FontStyle => LonghandId::FontStyle,
            CssPropertyName::TextDecoration => {
                let mut dest = String::new();
                style
                    .shorthand_to_css(ShorthandId::TextDecoration, &mut dest)
                    .ok()?;
                return Some(dest.into());
            },
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
            CssPropertyName::Color => "color",
            CssPropertyName::FontFamily => "font-family",
            CssPropertyName::FontSize => "font-size",
            CssPropertyName::FontWeight => "font-weight",
            CssPropertyName::FontStyle => "font-style",
            CssPropertyName::TextDecoration => "text-decoration",
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
    pub(crate) fn is_indeterminate(&self, cx: &mut JSContext, document: &Document) -> bool {
        if !self.is_standard_inline_value_command() {
            return false;
        }
        // https://w3c.github.io/editing/docs/execCommand/#standard-inline-value-command
        // > it is indeterminate if among formattable nodes that are effectively contained in the active range,
        // > there are two that have distinct effective command values.
        let Some(selection) = document.GetSelection(cx) else {
            return false;
        };
        let Some(active_range) = selection.active_range() else {
            return false;
        };
        let mut at_least_two_different_effective_values = false;
        let mut previous_effective_value: Option<DOMString> = None;
        active_range.for_each_effectively_contained_child(|node| {
            if at_least_two_different_effective_values || !node.is_formattable() {
                return;
            }
            if let Some(effective_command_value) = node.effective_command_value(self) {
                if let Some(previous_effective_value) = &previous_effective_value {
                    if &effective_command_value != previous_effective_value {
                        at_least_two_different_effective_values = true;
                    }
                } else {
                    previous_effective_value = Some(effective_command_value);
                }
            }
        });
        at_least_two_different_effective_values
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#state>
    pub(crate) fn current_state(&self, cx: &mut JSContext, document: &Document) -> Option<bool> {
        Some(match self {
            CommandName::StyleWithCss => {
                // https://w3c.github.io/editing/docs/execCommand/#the-stylewithcss-command
                // > True if the CSS styling flag is true, otherwise false.
                document.css_styling_flag()
            },
            _ => {
                // https://w3c.github.io/editing/docs/execCommand/#inline-formatting-command-definitions
                // > If a command has inline command activated values defined, its state is true if either
                // > no formattable node is effectively contained in the active range,
                // > and the active range's start node's effective command value is one of the given values;
                // > or if there is at least one formattable node effectively contained in the active range,
                // > and all of them have an effective command value equal to one of the given values.
                let inline_command_activated_values = self.inline_command_activated_values();
                if inline_command_activated_values.is_empty() {
                    return None;
                }
                let selection = document.GetSelection(cx)?;
                let active_range = selection.active_range()?;
                let mut at_least_one_child_is_formattable = false;
                let mut all_children_have_matching_command_values = true;
                active_range.for_each_effectively_contained_child(|node| {
                    if !node.is_formattable() {
                        return;
                    }
                    at_least_one_child_is_formattable = true;
                    all_children_have_matching_command_values &= node
                        .effective_command_value(self)
                        .is_some_and(|effective_value| {
                            inline_command_activated_values.contains(&&*effective_value.str())
                        });
                });
                if at_least_one_child_is_formattable {
                    all_children_have_matching_command_values
                } else {
                    active_range
                        .start_container()
                        .effective_command_value(self)
                        .is_some_and(|effective_value| {
                            inline_command_activated_values.contains(&&*effective_value.str())
                        })
                }
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
            _ if self.is_standard_inline_value_command() => {
                // https://w3c.github.io/editing/docs/execCommand/#standard-inline-value-command
                // > Its value is the effective command value of the first formattable node that
                // > is effectively contained in the active range; or if there is no such node,
                // > the effective command value of the active range's start node;
                // > or if that is null, the empty string.
                let selection = document.GetSelection(cx)?;
                let active_range = selection.active_range()?;

                active_range
                    .first_formattable_contained_node()
                    .unwrap_or_else(|| active_range.start_container())
                    .effective_command_value(self)
                    .unwrap_or_default()
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
                    CommandName::BackColor | CommandName::ForeColor | CommandName::HiliteColor => {
                        // https://w3c.github.io/editing/docs/execCommand/#the-backcolor-command
                        // https://w3c.github.io/editing/docs/execCommand/#the-forecolor-command
                        // https://w3c.github.io/editing/docs/execCommand/#the-hilitecolor-command
                        // > Either both strings are valid CSS colors and have the same red, green, blue, and alpha components,
                        // > or neither string is a valid CSS color.
                        match (
                            parse_legacy_color(&first_str.str()),
                            parse_legacy_color(&second_str.str()),
                        ) {
                            (Ok(first_legacy_color), Ok(second_legacy_color)) => {
                                first_legacy_color == second_legacy_color
                            },
                            (Err(_), Err(_)) => true,
                            _ => false,
                        }
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
            CommandName::BackColor => CssPropertyName::BackgroundColor,
            CommandName::Bold => CssPropertyName::FontWeight,
            CommandName::FontName => CssPropertyName::FontFamily,
            CommandName::FontSize => CssPropertyName::FontSize,
            CommandName::ForeColor => CssPropertyName::Color,
            CommandName::HiliteColor => CssPropertyName::BackgroundColor,
            CommandName::Italic => CssPropertyName::FontStyle,
            // > If a command does not have a relevant CSS property specified, it defaults to null.
            _ => return None,
        })
    }

    pub(crate) fn resolved_value_for_node(&self, element: &Element) -> Option<DOMString> {
        let property = self.relevant_css_property()?;
        property.resolved_value_for_node(element)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#standard-inline-value-command>
    pub(crate) fn is_standard_inline_value_command(&self) -> bool {
        matches!(
            self,
            CommandName::BackColor |
                CommandName::FontName |
                CommandName::ForeColor |
                CommandName::HiliteColor
        )
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
            CommandName::BackColor => execute_backcolor_command(cx, document, selection, value),
            CommandName::Bold => execute_bold_command(cx, document, selection),
            CommandName::DefaultParagraphSeparator => {
                execute_default_paragraph_separator_command(document, value)
            },
            CommandName::Delete => execute_delete_command(cx, document, selection),
            CommandName::FontName => execute_fontname_command(cx, document, selection, value),
            CommandName::FontSize => execute_fontsize_command(cx, document, selection, value),
            CommandName::ForeColor => execute_forecolor_command(cx, document, selection, value),
            CommandName::HiliteColor => execute_hilitecolor_command(cx, document, selection, value),
            CommandName::Italic => execute_italic_command(cx, document, selection),
            CommandName::Strikethrough => execute_strikethrough_command(cx, document, selection),
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
