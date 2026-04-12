/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use js::context::JSContext;
use script_bindings::inheritance::Castable;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::SelectionBinding::SelectionMethods;
use crate::dom::bindings::inheritance::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::root::DomRoot;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::{CommandName, CssPropertyName};
use crate::dom::execcommand::contenteditable::node::move_preserving_ranges;
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlfontelement::HTMLFontElement;
use crate::dom::node::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::text::Text;
use crate::script_runtime::CanGc;

impl HTMLElement {
    pub(crate) fn local_name(&self) -> &str {
        self.upcast::<Element>().local_name()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#clear-the-value>
    pub(crate) fn clear_the_value(&self, cx: &mut JSContext, command: &CommandName) {
        // Step 1. Let command be the current command.
        //
        // Passed in as argument

        let node = self.upcast::<Node>();
        let element = self.upcast::<Element>();

        // Step 2. If element is not editable, return the empty list.
        if !node.is_editable() {
            return;
        }
        // Step 3. If element's specified command value for command is null,
        // return the empty list.
        if element.specified_command_value(command).is_none() {
            return;
        }
        // Step 4. If element is a simple modifiable element:
        if element.is_simple_modifiable_element() {
            // Step 4.1. Let children be the children of element.
            // Step 4.2. For each child in children, insert child into element's parent immediately before element, preserving ranges.
            let element_parent = node.GetParentNode().expect("Must always have a parent");
            for child in node.children() {
                move_preserving_ranges(cx, &child, |cx| {
                    element_parent.InsertBefore(cx, &child, Some(node))
                });
            }
            // Step 4.3. Remove element from its parent.
            node.remove_self(cx);
            // Step 4.4. Return children.
            return;
        }
        match command {
            // Step 5. If command is "strikethrough", and element has a style attribute
            // that sets "text-decoration" to some value containing "line-through",
            // delete "line-through" from the value.
            CommandName::Strikethrough => {
                let property = CssPropertyName::TextDecorationLine;
                if property.value_for_element(cx, self) == "line-through" {
                    // TODO: Only remove line-through
                    property.remove_from_element(cx, self);
                }
            },
            // Step 6. If command is "underline", and element has a style attribute that
            // sets "text-decoration" to some value containing "underline", delete "underline" from the value.
            CommandName::Underline => {
                let property = CssPropertyName::TextDecorationLine;
                if property.value_for_element(cx, self) == "underline" {
                    // TODO: Only remove underline
                    property.remove_from_element(cx, self);
                }
            },
            _ => {},
        }
        // Step 7. If the relevant CSS property for command is not null,
        // unset that property of element.
        if let Some(property) = command.relevant_css_property() {
            property.remove_from_element(cx, self);
        }
        // Step 8. If element is a font element:
        if self.is::<HTMLFontElement>() {
            match command {
                // Step 8.1. If command is "foreColor", unset element's color attribute, if set.
                CommandName::ForeColor => {
                    element.remove_attribute_by_name(&local_name!("color"), CanGc::from_cx(cx));
                },
                // Step 8.2. If command is "fontName", unset element's face attribute, if set.
                CommandName::FontName => {
                    element.remove_attribute_by_name(&local_name!("face"), CanGc::from_cx(cx));
                },
                // Step 8.3. If command is "fontSize", unset element's size attribute, if set.
                CommandName::FontSize => {
                    element.remove_attribute_by_name(&local_name!("size"), CanGc::from_cx(cx));
                },
                _ => {},
            }
        }
        // Step 9. If element is an a element and command is "createLink" or "unlink",
        // unset the href property of element.
        if self.is::<HTMLAnchorElement>() &&
            matches!(command, CommandName::CreateLink | CommandName::Unlink)
        {
            element.remove_attribute_by_name(&local_name!("href"), CanGc::from_cx(cx));
        }
        // Step 10. If element's specified command value for command is null,
        // return the empty list.
        if element.specified_command_value(command).is_none() {
            // TODO
        }
        // Step 11. Set the tag name of element to "span",
        // and return the one-node list consisting of the result.
        // TODO
    }

    /// There is no specification for this implementation. Instead, it is
    /// reverse-engineered based on the WPT test
    /// /selection/contenteditable/initial-selection-on-focus.tentative.html
    pub(crate) fn handle_focus_state_for_contenteditable(&self, can_gc: CanGc) {
        if !self.is_editing_host() {
            return;
        }
        let document = self.owner_document();
        let Some(selection) = document.GetSelection(can_gc) else {
            return;
        };
        let range = self
            .upcast::<Element>()
            .ensure_contenteditable_selection_range(&document, can_gc);
        // If the current range is already associated with this contenteditable
        // element, then we shouldn't do anything. This is important when focus
        // is lost and regained, but selection was changed beforehand. In that
        // case, we should maintain the selection as it were, by not creating
        // a new range.
        if selection
            .active_range()
            .is_some_and(|active| active == range)
        {
            return;
        }
        let node = self.upcast::<Node>();
        let mut selected_node = DomRoot::from_ref(node);
        let mut previous_eligible_node = DomRoot::from_ref(node);
        let mut previous_node = DomRoot::from_ref(node);
        let mut selected_offset = 0;
        for child in node.traverse_preorder(ShadowIncluding::Yes) {
            if let Some(text) = child.downcast::<Text>() {
                // Note that to consider it whitespace, it needs to take more
                // into account than simply "it has a non-whitespace" character.
                // Therefore, we need to first check if it is not a whitespace
                // node and only then can we find what the relevant character is.
                if !text.is_whitespace_node() {
                    // A node with "white-space: pre" set must select its first
                    // character, regardless if that's a whitespace character or not.
                    let is_pre_formatted_text_node = child
                        .GetParentElement()
                        .and_then(|parent| parent.style())
                        .is_some_and(|style| {
                            style.get_inherited_text().white_space_collapse ==
                                WhiteSpaceCollapse::Preserve
                        });
                    if !is_pre_formatted_text_node {
                        // If it isn't pre-formatted, then we should instead select the
                        // first non-whitespace character.
                        selected_offset = text
                            .data()
                            .find(|c: char| !c.is_whitespace())
                            .unwrap_or_default() as u32;
                    }
                    selected_node = child;
                    break;
                }
            }
            // For <input>, <textarea>, <hr> and <br> elements, we should select the previous
            // node, regardless if it was a block node or not
            if matches!(
                child.type_id(),
                NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLInputElement,
                )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLTextAreaElement,
                )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLHRElement,
                )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLBRElement,
                ))
            ) {
                selected_node = previous_node;
                break;
            }
            // When we encounter a non-contenteditable element, we should select the previous
            // eligible node
            if child
                .downcast::<HTMLElement>()
                .is_some_and(|el| el.ContentEditable().str() == "false")
            {
                selected_node = previous_eligible_node;
                break;
            }
            // We can only select block nodes as eligible nodes for the case of non-conenteditable
            // nodes
            if child.is_block_node() {
                previous_eligible_node = child.clone();
            }
            previous_node = child;
        }
        range.set_start(&selected_node, selected_offset);
        range.set_end(&selected_node, selected_offset);
        selection.AddRange(&range);
    }
}
