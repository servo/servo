/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::{
    BoolOrOptionalString, CommandName, RecordedStateOfCommand,
};
use crate::dom::execcommand::commands::fontsize::legacy_font_size_for;
use crate::dom::html::htmllielement::HTMLLIElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::Node;
use crate::dom::range::Range;
use crate::dom::selection::Selection;
use crate::dom::text::Text;

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

    /// The definition of "effectively contained" contains the recursion of
    /// ancestors of a single fully selected text node. That is to say, that
    /// if the selection is a fully selected text node <div>[foobar]</div>,
    /// then the div would also be considered effectively contained. As such,
    /// we can't use the common ancestor container, since that would be the
    /// text node only.
    ///
    /// Instead, we traverse all the way up to the editing host, which we know
    /// is sufficient to know to include all contained nodes. That way, we also
    /// would traverse ancestors such as the parent div.
    fn ancestor_for_effectively_contained(&self) -> DomRoot<Node> {
        let ancestor_container = self.CommonAncestorContainer();
        ancestor_container
            .editing_host_of()
            .unwrap_or(ancestor_container)
    }

    pub(crate) fn first_formattable_contained_node(&self) -> Option<DomRoot<Node>> {
        if self.collapsed() {
            return None;
        }

        self.ancestor_for_effectively_contained()
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

        // Make sure to keep track of the tree nodes before, since `callback` might modify
        // the underyling tree and then the iterator would prematurely stop.
        let children = self
            .ancestor_for_effectively_contained()
            .traverse_preorder(ShadowIncluding::No)
            .collect::<Vec<DomRoot<Node>>>();

        for child in children {
            if self.is_effectively_contained_node(&child) {
                callback(&child);
            }
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-extend>
    pub(crate) fn block_extend(&self, cx: &mut JSContext, document: &Document) -> DomRoot<Range> {
        // Step 1. Let start node, start offset, end node,
        // and end offset be the start and end nodes and offsets of range.
        let mut start_node = self.start_container();
        let mut start_offset = self.start_offset();
        let mut end_node = self.end_container();
        let mut end_offset = self.end_offset();
        // Step 2. If some inclusive ancestor of start node is an li,
        // set start offset to the index of the last such li in tree order, and set start node to that li's parent.
        if let Some(li_ancestor) = start_node
            .inclusive_ancestors(ShadowIncluding::No)
            .find(|ancestor| ancestor.is::<HTMLLIElement>())
        {
            start_offset = li_ancestor.index();
            start_node = li_ancestor
                .GetParentNode()
                .expect("Must always have a parent");
        }
        // Step 3. If (start node, start offset) is not a block start point, repeat the following steps:
        if !start_node.is_block_start_point(start_offset as usize) {
            loop {
                // Step 3.1. If start offset is zero, set it to start node's index, then set start node to its parent.
                if start_offset == 0 {
                    start_offset = start_node.index();
                    start_node = start_node
                        .GetParentNode()
                        .expect("Must always have a parent");
                } else {
                    // Step 3.2. Otherwise, subtract one from start offset.
                    start_offset -= 1;
                }
                // Step 3.3. If (start node, start offset) is a block boundary point, break from this loop.
                if start_node.is_block_boundary_point(start_offset) {
                    break;
                }
            }
        }
        // Step 4. While start offset is zero and start node's parent is not null,
        // set start offset to start node's index, then set start node to its parent.
        while start_offset == 0 &&
            let Some(parent) = start_node.GetParentNode()
        {
            start_offset = start_node.index();
            start_node = parent;
        }
        // Step 5. If some inclusive ancestor of end node is an li,
        // set end offset to one plus the index of the last such li in tree order,
        // and set end node to that li's parent.
        if let Some(li_ancestor) = end_node
            .inclusive_ancestors(ShadowIncluding::No)
            .find(|ancestor| ancestor.is::<HTMLLIElement>())
        {
            end_offset = 1 + li_ancestor.index();
            end_node = li_ancestor
                .GetParentNode()
                .expect("Must always have a parent");
        }
        // Step 6. If (end node, end offset) is not a block end point, repeat the following steps:
        if !end_node.is_block_end_point(end_offset) {
            loop {
                // Step 6.1. If end offset is end node's length, set it to one plus end node's index, then set end node to its parent.
                if end_offset == end_node.len() {
                    end_offset = 1 + end_node.index();
                    end_node = end_node.GetParentNode().expect("Must always have a parent");
                } else {
                    // Step 6.2. Otherwise, add one to end offset.
                    end_offset += 1;
                }
                // Step 6.3. If (end node, end offset) is a block boundary point, break from this loop.
                if end_node.is_block_boundary_point(end_offset) {
                    break;
                }
            }
        }
        // Step 7. While end offset is end node's length and end node's parent is not null,
        // set end offset to one plus end node's index, then set end node to its parent.
        while end_offset == end_node.len() &&
            let Some(parent) = end_node.GetParentNode()
        {
            end_offset = 1 + end_node.index();
            end_node = parent;
        }
        // Step 8. Let new range be a new range whose start and end nodes and offsets are start node,
        // start offset, end node, and end offset.
        let new_range = document.CreateRange(cx);
        new_range.set_start(&start_node, start_offset);
        new_range.set_end(&end_node, end_offset);
        // Step 9. Return new range.
        new_range
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#record-current-states-and-values>
    pub(crate) fn record_current_states_and_values(
        &self,
        cx: &mut JSContext,
    ) -> Vec<RecordedStateOfCommand> {
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
        let document = node.owner_doc();
        vec![
            // Step 4. Add ("createLink", node's effective command value for "createLink") to overrides.
            RecordedStateOfCommand::for_command_node(CommandName::CreateLink, &node),
            // Step 5. For each command in the list
            // "bold", "italic", "strikethrough", "subscript", "superscript", "underline", in order:
            // if node's effective command value for command is one of its inline command activated values,
            // add (command, true) to overrides, and otherwise add (command, false) to overrides.
            RecordedStateOfCommand::for_command_node_with_inline_activated_values(
                CommandName::Bold,
                &node,
            ),
            RecordedStateOfCommand::for_command_node_with_inline_activated_values(
                CommandName::Italic,
                &node,
            ),
            RecordedStateOfCommand::for_command_node_with_inline_activated_values(
                CommandName::Strikethrough,
                &node,
            ),
            RecordedStateOfCommand::for_command_node_with_inline_activated_values(
                CommandName::Subscript,
                &node,
            ),
            RecordedStateOfCommand::for_command_node_with_inline_activated_values(
                CommandName::Superscript,
                &node,
            ),
            RecordedStateOfCommand::for_command_node_with_inline_activated_values(
                CommandName::Underline,
                &node,
            ),
            // Step 6. For each command in the list "fontName", "foreColor", "hiliteColor", in order:
            // add (command, command's value) to overrides.
            RecordedStateOfCommand::for_command_node_with_value(
                cx,
                CommandName::FontName,
                &document,
            ),
            RecordedStateOfCommand::for_command_node_with_value(
                cx,
                CommandName::ForeColor,
                &document,
            ),
            RecordedStateOfCommand::for_command_node_with_value(
                cx,
                CommandName::HiliteColor,
                &document,
            ),
            // Step 7. Add ("fontSize", node's effective command value for "fontSize") to overrides.
            RecordedStateOfCommand::for_command_node(CommandName::FontSize, &node),
        ]
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#restore-states-and-values>
    pub(crate) fn restore_states_and_values(
        &self,
        cx: &mut JSContext,
        selection: &Selection,
        context_object: &Document,
        overrides: Vec<RecordedStateOfCommand>,
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
