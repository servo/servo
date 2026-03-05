/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::SelectionBinding::SelectionMethods;
use crate::dom::bindings::str::DOMString;
use crate::dom::execcommand::basecommand::BaseCommand;
use crate::dom::execcommand::contenteditable::{
    NodeExecCommandSupport, SelectionExecCommandSupport,
};
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlhrelement::HTMLHRElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::selection::Selection;
use crate::dom::text::Text;
use crate::script_runtime::CanGc;

pub(crate) struct DeleteCommand {}

impl BaseCommand for DeleteCommand {
    /// <https://w3c.github.io/editing/docs/execCommand/#the-delete-command>
    fn execute(
        &self,
        cx: &mut js::context::JSContext,
        selection: &Selection,
        _value: DOMString,
    ) -> bool {
        let active_range = selection
            .active_range()
            .expect("Must always have an active range");
        // Step 1. If the active range is not collapsed, delete the selection and return true.
        if !active_range.collapsed() {
            selection.delete_the_selection(
                cx,
                Default::default(),
                Default::default(),
                Default::default(),
            );
            return true;
        }
        // Step 2. Canonicalize whitespace at the active range's start.
        active_range
            .start_container()
            .canonicalize_whitespace(active_range.start_offset(), true);

        // Step 3. Let node and offset be the active range's start node and offset.
        let mut node = active_range.start_container();
        let mut offset = active_range.start_offset();

        // Step 4. Repeat the following steps:
        loop {
            // Step 4.1. If offset is zero and node's previousSibling is an editable invisible node,
            // remove node's previousSibling from its parent.
            if offset == 0 {
                if let Some(sibling) = node.GetPreviousSibling() {
                    if sibling.is_editable() && sibling.is_invisible() {
                        if let Some(parent) = node.GetParentNode() {
                            if parent.RemoveChild(&sibling, CanGc::from_cx(cx)).is_err() {
                                unreachable!("Must always have a parent to be able to remove from");
                            }
                            continue;
                        }
                    }
                }
            }
            // Step 4.2. Otherwise, if node has a child with index offset − 1 and that child is an editable invisible node,
            // remove that child from node, then subtract one from offset.
            if offset > 0 {
                if let Some(child) = node.children().nth(offset as usize - 1) {
                    if child.is_editable() && child.is_invisible() {
                        if node.RemoveChild(&child, CanGc::from_cx(cx)).is_err() {
                            unreachable!("Must always have a parent to be able to remove from");
                        }
                        offset -= 1;
                        continue;
                    }
                }
            }
            // Step 4.3. Otherwise, if offset is zero and node is an inline node, or if node is an invisible node,
            // set offset to the index of node, then set node to its parent.
            if (offset == 0 && node.is_inline_node()) || node.is_invisible() {
                offset = node.index();
                node = match node.GetParentNode() {
                    None => break,
                    Some(node) => node,
                };
                continue;
            }
            if offset > 0 {
                if let Some(child) = node.children().nth(offset as usize - 1) {
                    // Step 4.4. Otherwise, if node has a child with index offset − 1 and that child is an editable a,
                    // remove that child from node, preserving its descendants. Then return true.
                    if child.is_editable() && child.is::<HTMLAnchorElement>() {
                        child.remove_preserving_its_descendants(cx, &node);
                        return true;
                    }
                    // Step 4.5. Otherwise, if node has a child with index offset − 1 and that child is not a block node or a br or an img,
                    // set node to that child, then set offset to the length of node.
                    if !(child.is_block_node() ||
                        child.is::<HTMLBRElement>() ||
                        child.is::<HTMLImageElement>())
                    {
                        node = child;
                        offset = node.len();
                        continue;
                    }
                }
            }
            // Step 4.6. Otherwise, break from this loop.
            break;
        }

        // Step 5. If node is a Text node and offset is not zero, or if node is
        // a block node that has a child with index offset − 1 and that child is a br or hr or img:
        if (node.is::<Text>() && offset != 0) ||
            (offset > 0 &&
                node.is_block_node() &&
                node.children()
                    .nth(offset as usize - 1)
                    .is_some_and(|child| {
                        child.is::<HTMLBRElement>() ||
                            child.is::<HTMLHRElement>() ||
                            child.is::<HTMLImageElement>()
                    }))
        {
            // Step 5.1. Call collapse(node, offset) on the context object's selection.
            if selection
                .Collapse(Some(&node), offset, CanGc::from_cx(cx))
                .is_err()
            {
                unreachable!("Must not fail to collapse");
            }
            // Step 5.2. Call extend(node, offset − 1) on the context object's selection.
            if selection
                .Extend(&node, offset - 1, CanGc::from_cx(cx))
                .is_err()
            {
                unreachable!("Must not fail to extend");
            }
            // Step 5.3. Delete the selection.
            selection.delete_the_selection(
                cx,
                Default::default(),
                Default::default(),
                Default::default(),
            );
            // Step 5.4. Return true.
            return true;
        }

        // Step 6. If node is an inline node, return true.
        if node.is_inline_node() {
            return true;
        }

        // Step 7. If node is an li or dt or dd and is the first child of its parent, and offset is zero:
        // TODO

        // Step 8. Let start node equal node and let start offset equal offset.
        // TODO

        // Step 9. Repeat the following steps:
        // TODO

        // Step 10. If offset is zero, and node has an editable inclusive ancestor in the same editing host that's an indentation element:
        // TODO

        // Step 11. If the child of start node with index start offset is a table, return true.
        // TODO

        // Step 12. If start node has a child with index start offset − 1, and that child is a table:
        // TODO

        // Step 13. If offset is zero; and either the child of start node with index start offset
        // minus one is an hr, or the child is a br whose previousSibling is either a br or not an inline node:
        // TODO

        // Step 14. If the child of start node with index start offset is an li or dt or dd, and
        // that child's firstChild is an inline node, and start offset is not zero:
        // TODO

        // Step 15. If start node's child with index start offset is an li or dt or dd, and
        // that child's previousSibling is also an li or dt or dd:
        // TODO

        // Step 16. While start node has a child with index start offset minus one:
        // TODO

        // Step 17. Call collapse(start node, start offset) on the context object's selection.
        // TODO

        // Step 18. Call extend(node, offset) on the context object's selection.
        // TODO

        // Step 19. Delete the selection, with direction "backward".
        // TODO

        // Step 20. Return true.
        true
    }
}
