/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::SelectionBinding::SelectionMethods;
use crate::dom::document::Document;
use crate::dom::execcommand::contenteditable::{
    NodeExecCommandSupport, SelectionDeleteDirection, SelectionExecCommandSupport, split_the_parent,
};
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlhrelement::HTMLHRElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmllielement::HTMLLIElement;
use crate::dom::html::htmltableelement::HTMLTableElement;
use crate::dom::selection::Selection;
use crate::dom::text::Text;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/editing/docs/execCommand/#the-delete-command>
pub(crate) fn execute_delete_command(
    cx: &mut js::context::JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    let active_range = selection
        .active_range()
        .expect("Must always have an active range");
    // Step 1. If the active range is not collapsed, delete the selection and return true.
    if !active_range.collapsed() {
        selection.delete_the_selection(
            cx,
            document,
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
                    sibling.remove_self(cx);
                    continue;
                }
            }
        }
        // Step 4.2. Otherwise, if node has a child with index offset − 1 and that child is an editable invisible node,
        // remove that child from node, then subtract one from offset.
        if offset > 0 {
            if let Some(child) = node.children().nth(offset as usize - 1) {
                if child.is_editable() && child.is_invisible() {
                    child.remove_self(cx);
                    offset -= 1;
                    continue;
                }
            }
        }
        // Step 4.3. Otherwise, if offset is zero and node is an inline node, or if node is an invisible node,
        // set offset to the index of node, then set node to its parent.
        if (offset == 0 && node.is_inline_node()) || node.is_invisible() {
            offset = node.index();
            node = node.GetParentNode().expect("Must always have a parent");
            continue;
        }
        if offset > 0 {
            if let Some(child) = node.children().nth(offset as usize - 1) {
                // Step 4.4. Otherwise, if node has a child with index offset − 1 and that child is an editable a,
                // remove that child from node, preserving its descendants. Then return true.
                if child.is_editable() && child.is::<HTMLAnchorElement>() {
                    child.remove_preserving_its_descendants(cx);
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
            document,
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
    //
    // TODO: Handle dt or dd
    if offset == 0 &&
        node.is::<HTMLLIElement>() &&
        node.GetParentNode()
            .and_then(|parent| parent.children().next())
            .is_some_and(|first| first == node)
    {
        // Step 7.1. Let items be a list of all lis that are ancestors of node.
        // TODO
        // Step 7.2. Normalize sublists of each item in items.
        // TODO
        // Step 7.3. Record the values of the one-node list consisting of node, and let values be the result.
        // TODO
        // Step 7.4. Split the parent of the one-node list consisting of node.
        split_the_parent(cx, &[&node]);
        // Step 7.5. Restore the values from values.
        // TODO
        // Step 7.6. If node is a dd or dt, and it is not an allowed child of
        // any of its ancestors in the same editing host,
        // set the tag name of node to the default single-line container name
        // and let node be the result.
        // TODO
        // Step 7.7. Fix disallowed ancestors of node.
        // TODO
        // Step 7.8. Return true.
        return true;
    }

    // Step 8. Let start node equal node and let start offset equal offset.
    let mut start_node = node.clone();
    let mut start_offset = offset;

    // Step 9. Repeat the following steps:
    loop {
        // Step 9.1. If start offset is zero,
        // set start offset to the index of start node and then set start node to its parent.
        if start_offset == 0 {
            start_offset = start_node.index();
            start_node = start_node
                .GetParentNode()
                .expect("Must always have a parent");
            continue;
        }
        // Step 9.2. Otherwise, if start node has an editable invisible child with index start offset minus one,
        // remove it from start node and subtract one from start offset.
        assert!(
            start_offset > 0,
            "Must always have a start_offset greater than one"
        );
        if let Some(child) = start_node.children().nth(start_offset as usize - 1) {
            if child.is_editable() && child.is_invisible() {
                child.remove_self(cx);
                start_offset -= 1;
                continue;
            }
        }
        // Step 9.3. Otherwise, break from this loop.
        break;
    }

    // Step 10. If offset is zero, and node has an editable inclusive ancestor in the same editing host that's an indentation element:
    // TODO

    // Step 11. If the child of start node with index start offset is a table, return true.
    if start_node
        .children()
        .nth(start_offset as usize)
        .is_some_and(|child| child.is::<HTMLTableElement>())
    {
        return true;
    }

    // Step 12. If start node has a child with index start offset − 1, and that child is a table:
    // TODO

    // Step 13. If offset is zero; and either the child of start node with index start offset
    // minus one is an hr, or the child is a br whose previousSibling is either a br or not an inline node:
    if offset == 0 &&
        (start_offset > 0 &&
            start_node
                .children()
                .nth(start_offset as usize - 1)
                .is_some_and(|child| {
                    child.is::<HTMLHRElement>() ||
                        (child.is::<HTMLBRElement>() &&
                            child.GetPreviousSibling().is_some_and(|previous| {
                                previous.is::<HTMLBRElement>() || !previous.is_inline_node()
                            }))
                }))
    {
        // Step 13.1. Call collapse(start node, start offset − 1) on the context object's selection.
        if selection
            .Collapse(Some(&start_node), start_offset - 1, CanGc::from_cx(cx))
            .is_err()
        {
            unreachable!("Must not fail to collapse");
        }
        // Step 13.2. Call extend(start node, start offset) on the context object's selection.
        if selection
            .Extend(&start_node, start_offset, CanGc::from_cx(cx))
            .is_err()
        {
            unreachable!("Must not fail to extend");
        }
        // Step 13.3. Delete the selection.
        selection.delete_the_selection(
            cx,
            document,
            Default::default(),
            Default::default(),
            Default::default(),
        );
        // Step 13.4. Call collapse(node, offset) on the selection.
        if selection
            .Collapse(Some(&node), offset, CanGc::from_cx(cx))
            .is_err()
        {
            unreachable!("Must not fail to collapse");
        }
        // Step 13.5. Return true.
        return true;
    }

    // Step 14. If the child of start node with index start offset is an li or dt or dd, and
    // that child's firstChild is an inline node, and start offset is not zero:
    // TODO

    // Step 15. If start node's child with index start offset is an li or dt or dd, and
    // that child's previousSibling is also an li or dt or dd:
    // TODO

    // Step 16. While start node has a child with index start offset minus one:
    loop {
        if start_offset == 0 {
            break;
        }
        let Some(child) = start_node.children().nth(start_offset as usize - 1) else {
            break;
        };
        // Step 16.1. If start node's child with index start offset minus one
        // is editable and invisible, remove it from start node, then subtract one from start offset.
        if child.is_editable() && child.is_invisible() {
            child.remove_self(cx);
            start_offset -= 1;
        } else {
            // Step 16.2. Otherwise, set start node to its child with index start offset minus one,
            // then set start offset to the length of start node.
            start_node = child;
            start_offset = start_node.len();
        }
    }

    // Step 17. Call collapse(start node, start offset) on the context object's selection.
    if selection
        .Collapse(Some(&start_node), start_offset, CanGc::from_cx(cx))
        .is_err()
    {
        unreachable!("Must not fail to collapse");
    }

    // Step 18. Call extend(node, offset) on the context object's selection.
    if selection.Extend(&node, offset, CanGc::from_cx(cx)).is_err() {
        unreachable!("Must not fail to extend");
    }

    // Step 19. Delete the selection, with direction "backward".
    selection.delete_the_selection(
        cx,
        document,
        Default::default(),
        Default::default(),
        SelectionDeleteDirection::Backward,
    );

    // Step 20. Return true.
    true
}
