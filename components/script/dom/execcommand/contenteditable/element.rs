/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use script_bindings::inheritance::Castable;
use style::attr::AttrValue;
use style::properties::{LonghandId, PropertyDeclaration, PropertyDeclarationId};
use style::values::specified::TextDecorationLine;
use style::values::specified::box_::DisplayOutside;

use crate::dom::bindings::inheritance::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::{CommandName, CssPropertyName};
use crate::dom::execcommand::commands::fontsize::font_size_to_css_font;
use crate::dom::html::htmlfontelement::HTMLFontElement;
use crate::dom::node::node::{Node, NodeTraits};

impl Element {
    pub(crate) fn resolved_display_value(&self) -> Option<DisplayOutside> {
        self.style().map(|style| style.get_box().display.outside())
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#specified-command-value>
    pub(crate) fn specified_command_value(&self, command: &CommandName) -> Option<DOMString> {
        match command {
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
                if let Some(value) = CssPropertyName::TextDecorationLine.value_set_for_style(self) {
                    // Step 6.1. If element's style attribute sets "text-decoration" to a value containing "underline", return "underline".
                    // Step 6.2. Return null.
                    return Some("underline".into()).filter(|_| value.contains("underline"));
                }
                // Step 7. If command is "underline" and element is a u element, return "underline".
                if *self.local_name() == local_name!("u") {
                    return Some("underline".into());
                }
            },
            _ => {},
        };
        // Step 8. Let property be the relevant CSS property for command.
        // Step 9. If property is null, return null.
        let property = command.relevant_css_property()?;
        // Step 10. If element has a style attribute set, and that attribute has the effect of setting property,
        // return the value that it sets property to.
        if let Some(value) = property.value_set_for_style(self) {
            return Some(value);
        }
        // Step 11. If element is a font element that has an attribute whose effect is to create a presentational hint for property,
        // return the value that the hint sets property to. (For a size of 7, this will be the non-CSS value "xxx-large".)
        if self.is::<HTMLFontElement>() {
            if let Some(font_size) = self.get_attribute(&local_name!("size")) {
                if let AttrValue::UInt(_, value) = *font_size.value() {
                    return Some(font_size_to_css_font(&value).into());
                }
            }
        }

        // Step 12. If element is in the following list, and property is equal to the CSS property name listed for it,
        // return the string listed for it.
        let element_name = self.local_name();
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

    /// <https://w3c.github.io/editing/docs/execCommand/#simple-modifiable-element>
    pub(crate) fn is_simple_modifiable_element(&self) -> bool {
        let attrs = self.attrs();
        let attr_count = attrs.len();
        let type_id = self.upcast::<Node>().type_id();

        if matches!(
            type_id,
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLFontElement,
            )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSpanElement,
            ))
        ) || matches!(
            *self.local_name(),
            local_name!("b") |
                local_name!("em") |
                local_name!("i") |
                local_name!("s") |
                local_name!("strike") |
                local_name!("strong") |
                local_name!("sub") |
                local_name!("sup") |
                local_name!("u")
        ) {
            // > It is an a, b, em, font, i, s, span, strike, strong, sub, sup, or u element with no attributes.
            if attr_count == 0 {
                return true;
            }

            // > It is an a, b, em, font, i, s, span, strike, strong, sub, sup, or u element
            // > with exactly one attribute, which is style,
            // > which sets no CSS properties (including invalid or unrecognized properties).
            if attr_count == 1 &&
                self.attrs().first().expect("Size is 1").local_name() == &local_name!("style")
            {
                let style_attribute = self.style_attribute().borrow();
                if style_attribute.as_ref().is_some_and(|declarations| {
                    let document = self.owner_document();
                    let shared_lock = document.style_shared_lock();
                    let read_lock = shared_lock.read();
                    let style = declarations.read_with(&read_lock);

                    style.is_empty()
                }) {
                    return true;
                }
            }
        }

        if attr_count != 1 {
            return false;
        }

        let only_attribute = attrs.first().expect("Size is 1").local_name();

        // > It is an a element with exactly one attribute, which is href.
        if matches!(
            type_id,
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            ))
        ) {
            return only_attribute == &local_name!("href");
        }

        // > It is a font element with exactly one attribute, which is either color, face, or size.
        if matches!(
            type_id,
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLFontElement,
            ))
        ) {
            return only_attribute == &local_name!("color") ||
                only_attribute == &local_name!("face") ||
                only_attribute == &local_name!("size");
        }

        if only_attribute != &local_name!("style") {
            return false;
        }
        let style_attribute = self.style_attribute().borrow();
        let Some(declarations) = style_attribute.as_ref() else {
            return false;
        };
        let document = self.owner_document();
        let shared_lock = document.style_shared_lock();
        let read_lock = shared_lock.read();
        let style = declarations.read_with(&read_lock);

        if style.len() != 1 {
            return false;
        }

        // > It is a b or strong element with exactly one attribute, which is style,
        // > and the style attribute sets exactly one CSS property
        // > (including invalid or unrecognized properties), which is "font-weight".
        if matches!(*self.local_name(), local_name!("b") | local_name!("strong")) {
            return style.contains(PropertyDeclarationId::Longhand(LonghandId::FontWeight));
        }

        // > It is an i or em element with exactly one attribute, which is style,
        // > and the style attribute sets exactly one CSS property (including invalid or unrecognized properties),
        // > which is "font-style".
        if matches!(*self.local_name(), local_name!("i") | local_name!("em")) {
            return style.contains(PropertyDeclarationId::Longhand(LonghandId::FontStyle));
        }

        let a_font_or_span = matches!(
            type_id,
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLFontElement,
            )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSpanElement,
            ))
        );
        let s_strike_or_u = matches!(
            *self.local_name(),
            local_name!("s") | local_name!("strike") | local_name!("u")
        );
        if a_font_or_span || s_strike_or_u {
            if let Some((text_decoration, _)) = style.get(PropertyDeclarationId::Longhand(
                LonghandId::TextDecorationLine,
            )) {
                // > It is an a, font, s, span, strike, or u element with exactly one attribute,
                // > which is style, and the style attribute sets exactly one CSS property
                // > (including invalid or unrecognized properties), which is "text-decoration",
                // > which is set to "line-through" or "underline" or "overline" or "none".
                return matches!(
                    text_decoration,
                    PropertyDeclaration::TextDecorationLine(
                        TextDecorationLine::LINE_THROUGH |
                            TextDecorationLine::UNDERLINE |
                            TextDecorationLine::OVERLINE |
                            TextDecorationLine::NONE
                    )
                );
            } else {
                // > It is an a, font, or span element with exactly one attribute, which is style,
                // > and the style attribute sets exactly one CSS property (including invalid or unrecognized properties),
                // > and that property is not "text-decoration".
                return a_font_or_span;
            }
        }

        false
    }
}
