/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::execcommand::commands::fontsize::legacy_font_size_for;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::range::Range;
use crate::dom::selection::Selection;
use crate::dom::text::Text;

enum BoolOrOptionalString {
    Bool(bool),
    OptionalString(Option<DOMString>),
}

impl From<Option<DOMString>> for BoolOrOptionalString {
    fn from(optional_string: Option<DOMString>) -> Self {
        Self::OptionalString(optional_string)
    }
}

impl From<bool> for BoolOrOptionalString {
    fn from(bool_: bool) -> Self {
        Self::Bool(bool_)
    }
}

pub(crate) struct RecordedStateOfNode {
    command: CommandName,
    value: BoolOrOptionalString,
}

impl RecordedStateOfNode {
    fn for_command_node(command: CommandName, node: &Node) -> Self {
        let value = node.effective_command_value(&command).into();
        Self { command, value }
    }

    fn for_command_node_with_inline_activated_values(command: CommandName, node: &Node) -> Self {
        let effective_command_value = node.effective_command_value(&command);
        let value = effective_command_value
            .is_some_and(|effective_command_value| {
                command
                    .inline_command_activated_values()
                    .contains(&effective_command_value.str().as_ref())
            })
            .into();
        Self { command, value }
    }
}

impl Range {
    /// <https://w3c.github.io/editing/docs/execCommand/#effectively-contained>
    fn is_effectively_contained_node(&self, node: &Node) -> bool {
        // > A node node is effectively contained in a range range if range is not collapsed,
        if self.collapsed() {
            return false;
        }
        // > and at least one of the following holds:
        // > node is range's start node, it is a Text node, and its length is different from range's start offset.
        let start_container = self.start_container();
        if *start_container == *node && node.is::<Text>() && node.len() != self.start_offset() {
            return true;
        }
        // > node is range's end node, it is a Text node, and range's end offset is not 0.
        let end_container = self.end_container();
        if *end_container == *node && node.is::<Text>() && self.end_offset() != 0 {
            return true;
        }
        // > node is contained in range.
        if self.contains(node) {
            return true;
        }
        // > node has at least one child; and all its children are effectively contained in range;
        node.children_count() > 0 && node.children().all(|child| self.is_effectively_contained_node(&child))
        // > and either range's start node is not a descendant of node or is not a Text node or range's start offset is zero;
        && (!node.is_ancestor_of(&start_container) || !start_container.is::<Text>() || self.start_offset() == 0)
        // > and either range's end node is not a descendant of node or is not a Text node or range's end offset is its end node's length.
        && (!node.is_ancestor_of(&end_container) || !end_container.is::<Text>() || self.end_offset() == end_container.len())
    }

    pub(crate) fn first_formattable_contained_node(&self) -> Option<DomRoot<Node>> {
        if self.collapsed() {
            return None;
        }

        self.CommonAncestorContainer()
            .traverse_preorder(ShadowIncluding::No)
            .find(|child| child.is_formattable() && self.is_effectively_contained_node(child))
    }

    pub(crate) fn for_each_effectively_contained_child<Callback: FnMut(&Node)>(
        &self,
        mut callback: Callback,
    ) {
        if self.collapsed() {
            return;
        }

        for child in self
            .CommonAncestorContainer()
            .traverse_preorder(ShadowIncluding::No)
        {
            if self.is_effectively_contained_node(&child) {
                callback(&child);
            }
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#record-current-states-and-values>
    pub(crate) fn record_current_states_and_values(&self) -> Vec<RecordedStateOfNode> {
        // Step 1. Let overrides be a list of (string, string or boolean) ordered pairs, initially empty.
        //
        // We return the vec in one go for the relevant values

        // Step 2. Let node be the first formattable node effectively contained in the active range,
        // or null if there is none.
        let Some(node) = self.first_formattable_contained_node() else {
            // Step 3. If node is null, return overrides.
            return vec![];
        };
        // Step 8. Return overrides.
        vec![
            // Step 4. Add ("createLink", node's effective command value for "createLink") to overrides.
            RecordedStateOfNode::for_command_node(CommandName::CreateLink, &node),
            // Step 5. For each command in the list
            // "bold", "italic", "strikethrough", "subscript", "superscript", "underline", in order:
            // if node's effective command value for command is one of its inline command activated values,
            // add (command, true) to overrides, and otherwise add (command, false) to overrides.
            RecordedStateOfNode::for_command_node_with_inline_activated_values(
                CommandName::Bold,
                &node,
            ),
            RecordedStateOfNode::for_command_node_with_inline_activated_values(
                CommandName::Italic,
                &node,
            ),
            RecordedStateOfNode::for_command_node_with_inline_activated_values(
                CommandName::Strikethrough,
                &node,
            ),
            RecordedStateOfNode::for_command_node_with_inline_activated_values(
                CommandName::Subscript,
                &node,
            ),
            RecordedStateOfNode::for_command_node_with_inline_activated_values(
                CommandName::Superscript,
                &node,
            ),
            RecordedStateOfNode::for_command_node_with_inline_activated_values(
                CommandName::Underline,
                &node,
            ),
            // Step 6. For each command in the list "fontName", "foreColor", "hiliteColor", in order:
            // add (command, command's value) to overrides.
            // TODO

            // Step 7. Add ("fontSize", node's effective command value for "fontSize") to overrides.
            RecordedStateOfNode::for_command_node(CommandName::FontSize, &node),
        ]
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#restore-states-and-values>
    pub(crate) fn restore_states_and_values(
        &self,
        cx: &mut JSContext,
        selection: &Selection,
        context_object: &Document,
        overrides: Vec<RecordedStateOfNode>,
    ) {
        // Step 1. Let node be the first formattable node effectively contained in the active range,
        // or null if there is none.
        let mut first_formattable_contained_node = self.first_formattable_contained_node();
        for override_state in overrides {
            // Step 2. If node is not null, then for each (command, override) pair in overrides, in order:
            if let Some(ref node) = first_formattable_contained_node {
                match override_state.value {
                    // Step 2.1. If override is a boolean, and queryCommandState(command)
                    // returns something different from override, take the action for command,
                    // with value equal to the empty string.
                    BoolOrOptionalString::Bool(bool_)
                        if override_state
                            .command
                            .current_state(cx, context_object)
                            .is_some_and(|value| value != bool_) =>
                    {
                        override_state
                            .command
                            .execute(cx, context_object, selection, "".into());
                    },
                    BoolOrOptionalString::OptionalString(optional_string) => {
                        match override_state.command {
                            // Step 2.3. Otherwise, if override is a string; and command is "createLink";
                            // and either there is a value override for "createLink" that is not equal to override,
                            // or there is no value override for "createLink" and node's effective command value
                            // for "createLink" is not equal to override: take the action for "createLink", with value equal to override.
                            CommandName::CreateLink => {
                                let value_override =
                                    context_object.value_override(&CommandName::CreateLink);
                                if value_override != optional_string {
                                    CommandName::CreateLink.execute(
                                        cx,
                                        context_object,
                                        selection,
                                        optional_string.unwrap_or_default(),
                                    );
                                }
                            },
                            // Step 2.4. Otherwise, if override is a string; and command is "fontSize";
                            // and either there is a value override for "fontSize" that is not equal to override,
                            // or there is no value override for "fontSize" and node's effective command value for "fontSize"
                            // is not loosely equivalent to override:
                            CommandName::FontSize => {
                                let value_override =
                                    context_object.value_override(&CommandName::FontSize);
                                if value_override != optional_string ||
                                    (value_override.is_none() &&
                                        !CommandName::FontSize.are_loosely_equivalent_values(
                                            node.effective_command_value(&CommandName::FontSize)
                                                .as_ref(),
                                            optional_string.as_ref(),
                                        ))
                                {
                                    // Step 2.5. Convert override to an integer number of pixels,
                                    // and set override to the legacy font size for the result.
                                    let pixels = optional_string
                                        .and_then(|value| value.parse::<i32>().ok())
                                        .map(|value| {
                                            legacy_font_size_for(value as f32, context_object)
                                        })
                                        .unwrap_or("7".into());
                                    // Step 2.6. Take the action for "fontSize", with value equal to override.
                                    CommandName::FontSize.execute(
                                        cx,
                                        context_object,
                                        selection,
                                        pixels,
                                    );
                                }
                            },
                            // Step 2.2. Otherwise, if override is a string, and command is neither "createLink" nor "fontSize",
                            // and queryCommandValue(command) returns something not equivalent to override,
                            // take the action for command, with value equal to override.
                            command
                                if command.current_value(cx, context_object) != optional_string =>
                            {
                                command.execute(
                                    cx,
                                    context_object,
                                    selection,
                                    optional_string.unwrap_or_default(),
                                );
                            },
                            // Step 2.5. Otherwise, continue this loop from the beginning.
                            _ => {
                                continue;
                            },
                        }
                    },
                    // Step 2.5. Otherwise, continue this loop from the beginning.
                    _ => {
                        continue;
                    },
                }
                // Step 2.6. Set node to the first formattable node effectively contained in the active range, if there is one.
                first_formattable_contained_node = self.first_formattable_contained_node();
            } else {
                // Step 3. Otherwise, for each (command, override) pair in overrides, in order:
                // Step 3.1. If override is a boolean, set the state override for command to override.
                match override_state.value {
                    BoolOrOptionalString::Bool(bool_) => {
                        context_object.set_state_override(override_state.command, Some(bool_))
                    },
                    // Step 3.2. If override is a string, set the value override for command to override.
                    BoolOrOptionalString::OptionalString(optional_string) => {
                        context_object.set_value_override(override_state.command, optional_string)
                    },
                }
            }
        }
    }
}
