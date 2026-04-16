/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::abstractrange::bp_position;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::SelectionBinding::SelectionMethods;
use crate::dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use crate::dom::bindings::root::{DomRoot, DomSlice};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::execcommand::contenteditable::node::{
    NodeOrString, is_allowed_child, move_preserving_ranges, record_the_values, restore_the_values,
    split_the_parent,
};
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmltablecellelement::HTMLTableCellElement;
use crate::dom::html::htmltablerowelement::HTMLTableRowElement;
use crate::dom::html::htmltablesectionelement::HTMLTableSectionElement;
use crate::dom::node::Node;
use crate::dom::selection::Selection;
use crate::dom::text::Text;
use crate::script_runtime::CanGc;

#[derive(Default, PartialEq)]
pub(crate) enum SelectionDeletionBlockMerging {
    #[default]
    Merge,
    Skip,
}

#[derive(Default, PartialEq)]
pub(crate) enum SelectionDeletionStripWrappers {
    #[default]
    Strip,
}

#[derive(Default, PartialEq)]
pub(crate) enum SelectionDeleteDirection {
    #[default]
    Forward,
    Backward,
}

trait EquivalentPoint {
    fn previous_equivalent_point(&self) -> Option<(DomRoot<Node>, u32)>;
    fn next_equivalent_point(&self) -> Option<(DomRoot<Node>, u32)>;
    fn first_equivalent_point(self) -> (DomRoot<Node>, u32);
    fn last_equivalent_point(self) -> (DomRoot<Node>, u32);
}

impl EquivalentPoint for (DomRoot<Node>, u32) {
    /// <https://w3c.github.io/editing/docs/execCommand/#previous-equivalent-point>
    fn previous_equivalent_point(&self) -> Option<(DomRoot<Node>, u32)> {
        let (node, offset) = self;
        // Step 1. If node's length is zero, return null.
        let len = node.len();
        if len == 0 {
            return None;
        }
        // Step 2. If offset is 0, and node's parent is not null, and node is an inline node,
        // return (node's parent, node's index).
        if *offset == 0 && node.is_inline_node() {
            if let Some(parent) = node.GetParentNode() {
                return Some((parent, node.index()));
            }
        }
        // Step 3. If node has a child with index offset − 1, and that child's length is not zero,
        // and that child is an inline node, return (that child, that child's length).
        if *offset > 0 {
            if let Some(child) = node.children().nth(*offset as usize - 1) {
                if !child.is_empty() && child.is_inline_node() {
                    let len = child.len();
                    return Some((child, len));
                }
            }
        }

        // Step 4. Return null.
        None
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#next-equivalent-point>
    fn next_equivalent_point(&self) -> Option<(DomRoot<Node>, u32)> {
        let (node, offset) = self;
        // Step 1. If node's length is zero, return null.
        let len = node.len();
        if len == 0 {
            return None;
        }

        // Step 2.
        //
        // This step does not exist in the spec

        // Step 3. If offset is node's length, and node's parent is not null, and node is an inline node,
        // return (node's parent, 1 + node's index).
        if *offset == len && node.is_inline_node() {
            if let Some(parent) = node.GetParentNode() {
                return Some((parent, node.index() + 1));
            }
        }

        // Step 4.
        //
        // This step does not exist in the spec

        // Step 5. If node has a child with index offset, and that child's length is not zero,
        // and that child is an inline node, return (that child, 0).
        if let Some(child) = node.children().nth(*offset as usize) {
            if !child.is_empty() && child.is_inline_node() {
                return Some((child, 0));
            }
        }

        // Step 6.
        //
        // This step does not exist in the spec

        // Step 7. Return null.
        None
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#first-equivalent-point>
    fn first_equivalent_point(self) -> (DomRoot<Node>, u32) {
        let mut previous_equivalent_point = self;
        // Step 1. While (node, offset)'s previous equivalent point is not null, set (node, offset) to its previous equivalent point.
        loop {
            if let Some(next) = previous_equivalent_point.previous_equivalent_point() {
                previous_equivalent_point = next;
            } else {
                // Step 2. Return (node, offset).
                return previous_equivalent_point;
            }
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#last-equivalent-point>
    fn last_equivalent_point(self) -> (DomRoot<Node>, u32) {
        let mut next_equivalent_point = self;
        // Step 1. While (node, offset)'s next equivalent point is not null, set (node, offset) to its next equivalent point.
        loop {
            if let Some(next) = next_equivalent_point.next_equivalent_point() {
                next_equivalent_point = next;
            } else {
                // Step 2. Return (node, offset).
                return next_equivalent_point;
            }
        }
    }
}

impl Selection {
    /// <https://w3c.github.io/editing/docs/execCommand/#delete-the-selection>
    pub(crate) fn delete_the_selection(
        &self,
        cx: &mut JSContext,
        context_object: &Document,
        block_merging: SelectionDeletionBlockMerging,
        strip_wrappers: SelectionDeletionStripWrappers,
        direction: SelectionDeleteDirection,
    ) {
        // Step 1. If the active range is null, abort these steps and do nothing.
        let Some(active_range) = self.active_range() else {
            return;
        };

        // Step 2. Canonicalize whitespace at the active range's start.
        active_range
            .start_container()
            .canonicalize_whitespace(active_range.start_offset(), true);

        // Step 3. Canonicalize whitespace at the active range's end.
        active_range
            .end_container()
            .canonicalize_whitespace(active_range.end_offset(), true);

        // Step 4. Let (start node, start offset) be the last equivalent point for the active range's start.
        let (mut start_node, mut start_offset) =
            (active_range.start_container(), active_range.start_offset()).last_equivalent_point();

        // Step 5. Let (end node, end offset) be the first equivalent point for the active range's end.
        let (mut end_node, mut end_offset) =
            (active_range.end_container(), active_range.end_offset()).first_equivalent_point();

        // Step 6. If (end node, end offset) is not after (start node, start offset):
        if bp_position(&end_node, end_offset, &start_node, start_offset) != Some(Ordering::Greater)
        {
            // Step 6.1. If direction is "forward", call collapseToStart() on the context object's selection.
            if direction == SelectionDeleteDirection::Forward {
                if self.CollapseToStart(CanGc::from_cx(cx)).is_err() {
                    unreachable!("Should be able to collapse to start");
                }
            } else {
                // Step 6.2. Otherwise, call collapseToEnd() on the context object's selection.
                if self.CollapseToEnd(CanGc::from_cx(cx)).is_err() {
                    unreachable!("Should be able to collapse to end");
                }
            }
            // Step 6.3. Abort these steps.
            return;
        }

        // Step 7. If start node is a Text node and start offset is 0, set start offset to the index of start node,
        // then set start node to its parent.
        if start_node.is::<Text>() && start_offset == 0 {
            start_offset = start_node.index();
            start_node = start_node
                .GetParentNode()
                .expect("Must always have a parent");
        }

        // Step 8. If end node is a Text node and end offset is its length, set end offset to one plus the index of end node,
        // then set end node to its parent.
        if end_node.is::<Text>() && end_offset == end_node.len() {
            end_offset = end_node.index() + 1;
            end_node = end_node.GetParentNode().expect("Must always have a parent");
        }

        // Step 9. Call collapse(start node, start offset) on the context object's selection.
        if self
            .Collapse(Some(&start_node), start_offset, CanGc::from_cx(cx))
            .is_err()
        {
            unreachable!("Must always be able to collapse");
        }

        // Step 10. Call extend(end node, end offset) on the context object's selection.
        if self
            .Extend(&end_node, end_offset, CanGc::from_cx(cx))
            .is_err()
        {
            unreachable!("Must always be able to extend");
        }

        // Step 11.
        //
        // This step does not exist in the spec

        // Step 12. Let start block be the active range's start node.
        let Some(active_range) = self.active_range() else {
            return;
        };
        let mut start_block = active_range.start_container();

        // Step 13. While start block's parent is in the same editing host and start block is an inline node,
        // set start block to its parent.
        loop {
            if start_block.is_inline_node() {
                if let Some(parent) = start_block.GetParentNode() {
                    if parent.same_editing_host(&start_node) {
                        start_block = parent;
                        continue;
                    }
                }
            }
            break;
        }

        // Step 14. If start block is neither a block node nor an editing host,
        // or "span" is not an allowed child of start block,
        // or start block is a td or th, set start block to null.
        let start_block = if (!start_block.is_block_node() && !start_block.is_editing_host()) ||
            !is_allowed_child(
                NodeOrString::String("span".to_owned()),
                NodeOrString::Node(start_block.clone()),
            ) ||
            start_block.is::<HTMLTableCellElement>()
        {
            None
        } else {
            Some(start_block)
        };

        // Step 15. Let end block be the active range's end node.
        let mut end_block = active_range.end_container();

        // Step 16. While end block's parent is in the same editing host and end block is an inline node, set end block to its parent.
        loop {
            if end_block.is_inline_node() {
                if let Some(parent) = end_block.GetParentNode() {
                    if parent.same_editing_host(&end_block) {
                        end_block = parent;
                        continue;
                    }
                }
            }
            break;
        }

        // Step 17. If end block is neither a block node nor an editing host, or "span" is not an allowed child of end block,
        // or end block is a td or th, set end block to null.
        let end_block = if (!end_block.is_block_node() && !end_block.is_editing_host()) ||
            !is_allowed_child(
                NodeOrString::String("span".to_owned()),
                NodeOrString::Node(end_block.clone()),
            ) ||
            end_block.is::<HTMLTableCellElement>()
        {
            None
        } else {
            Some(end_block)
        };

        // Step 18.
        //
        // This step does not exist in the spec

        // Step 19. Record current states and values, and let overrides be the result.
        let overrides = active_range.record_current_states_and_values();

        // Step 20.
        //
        // This step does not exist in the spec

        // Step 21. If start node and end node are the same, and start node is an editable Text node:
        if start_node == end_node && start_node.is_editable() {
            if let Some(start_text) = start_node.downcast::<Text>() {
                // Step 21.1. Call deleteData(start offset, end offset − start offset) on start node.
                if start_text
                    .upcast::<CharacterData>()
                    .DeleteData(start_offset, end_offset - start_offset)
                    .is_err()
                {
                    unreachable!("Must always be able to delete");
                }
                // Step 21.2. Canonicalize whitespace at (start node, start offset), with fix collapsed space false.
                start_node.canonicalize_whitespace(start_offset, false);
                // Step 21.3. If direction is "forward", call collapseToStart() on the context object's selection.
                if direction == SelectionDeleteDirection::Forward {
                    if self.CollapseToStart(CanGc::from_cx(cx)).is_err() {
                        unreachable!("Should be able to collapse to start");
                    }
                } else {
                    // Step 21.4. Otherwise, call collapseToEnd() on the context object's selection.
                    if self.CollapseToEnd(CanGc::from_cx(cx)).is_err() {
                        unreachable!("Should be able to collapse to end");
                    }
                }
                // Step 21.5. Restore states and values from overrides.
                active_range.restore_states_and_values(cx, self, context_object, overrides);

                // Step 21.6. Abort these steps.
                return;
            }
        }

        // Step 22. If start node is an editable Text node, call deleteData() on it, with start offset as
        // the first argument and (length of start node − start offset) as the second argument.
        if start_node.is_editable() {
            if let Some(start_text) = start_node.downcast::<Text>() {
                if start_text
                    .upcast::<CharacterData>()
                    .DeleteData(start_offset, start_node.len() - start_offset)
                    .is_err()
                {
                    unreachable!("Must always be able to delete");
                }
            }
        }

        // Step 23. Let node list be a list of nodes, initially empty.
        rooted_vec!(let mut node_list);

        // Step 24. For each node contained in the active range, append node to node list if the
        // last member of node list (if any) is not an ancestor of node; node is editable;
        // and node is not a thead, tbody, tfoot, tr, th, or td.
        let Ok(contained_children) = active_range.contained_children() else {
            unreachable!("Must always have contained children");
        };
        for node in contained_children.contained_children {
            // This type is only used to tell the compiler how to handle the type of `node_list.last()`.
            // It is not allowed to add a `& DomRoot<Node>` annotation, as test-tidy disallows that.
            // However, if we omit the type, the compiler doesn't know what it is, since we also
            // aren't allowed to add a type annotation to `node_list` itself, as that is handled
            // by the `rooted_vec` macro. Lastly, we also can't make it `&Node`, since then the compiler
            // thinks that the contents of the `RootedVec` is `Node`, whereas it is should be
            // `RootedVec<DomRoot<Node>>`. The type alias here doesn't upset test-tidy,
            // while also providing the necessary information to the compiler to work.
            type DomRootNode = DomRoot<Node>;
            if node.is_editable() &&
                !(node.is::<HTMLTableSectionElement>() ||
                    node.is::<HTMLTableRowElement>() ||
                    node.is::<HTMLTableCellElement>()) &&
                node_list
                    .last()
                    .is_none_or(|last: &DomRootNode| !last.is_ancestor_of(&node))
            {
                node_list.push(node);
            }
        }

        // Step 25. For each node in node list:
        for node in node_list.iter() {
            // Step 25.1. Let parent be the parent of node.
            let parent = node.GetParentNode().expect("Must always have a parent");
            // Step 25.2. Remove node from parent.
            assert!(node.has_parent());
            node.remove_self(cx);
            // Step 25.3. If the block node of parent has no visible children, and parent is editable or an editing host,
            // call createElement("br") on the context object and append the result as the last child of parent.
            if parent
                .block_node_of()
                .is_some_and(|block_node| block_node.children().all(|child| child.is_invisible())) &&
                parent.is_editable_or_editing_host()
            {
                let br = context_object.create_element(cx, "br");
                if parent.AppendChild(cx, br.upcast()).is_err() {
                    unreachable!("Must always be able to append");
                }
            }
            // Step 25.4. If strip wrappers is true or parent is not an inclusive ancestor of start node,
            // while parent is an editable inline node with length 0, let grandparent be the parent of parent,
            // then remove parent from grandparent, then set parent to grandparent.
            if strip_wrappers == SelectionDeletionStripWrappers::Strip ||
                !parent.is_inclusive_ancestor_of(&start_node)
            {
                let mut parent = parent;
                loop {
                    if parent.is_editable() && parent.is_inline_node() && parent.is_empty() {
                        let grand_parent =
                            parent.GetParentNode().expect("Must always have a parent");
                        assert!(parent.has_parent());
                        parent.remove_self(cx);
                        parent = grand_parent;
                        continue;
                    }
                    break;
                }
            }
        }

        // Step 26. If end node is an editable Text node, call deleteData(0, end offset) on it.
        if end_node.is_editable() {
            if let Some(end_text) = end_node.downcast::<Text>() {
                if end_text
                    .upcast::<CharacterData>()
                    .DeleteData(0, end_offset)
                    .is_err()
                {
                    unreachable!("Must always be able to delete");
                }
            }
        }

        // Step 27. Canonicalize whitespace at the active range's start, with fix collapsed space false.
        active_range
            .start_container()
            .canonicalize_whitespace(active_range.start_offset(), false);

        // Step 28. Canonicalize whitespace at the active range's end, with fix collapsed space false.
        active_range
            .end_container()
            .canonicalize_whitespace(active_range.end_offset(), false);

        // Step 29.
        //
        // This step does not exist in the spec

        // Step 30. If block merging is false, or start block or end block is null, or start block is not
        // in the same editing host as end block, or start block and end block are the same:
        if block_merging == SelectionDeletionBlockMerging::Skip ||
            start_block.as_ref().zip(end_block.as_ref()).is_none_or(
                |(start_block, end_block)| {
                    start_block == end_block || !start_block.same_editing_host(end_block)
                },
            )
        {
            // Step 30.1. If direction is "forward", call collapseToStart() on the context object's selection.
            if direction == SelectionDeleteDirection::Forward {
                if self.CollapseToStart(CanGc::from_cx(cx)).is_err() {
                    unreachable!("Should be able to collapse to start");
                }
            } else {
                // Step 30.2. Otherwise, call collapseToEnd() on the context object's selection.
                if self.CollapseToEnd(CanGc::from_cx(cx)).is_err() {
                    unreachable!("Should be able to collapse to end");
                }
            }
            // Step 30.3. Restore states and values from overrides.
            active_range.restore_states_and_values(cx, self, context_object, overrides);

            // Step 30.4. Abort these steps.
            return;
        }
        let start_block = start_block.expect("Already checked for None in previous statement");
        let end_block = end_block.expect("Already checked for None in previous statement");

        // Step 31. If start block has one child, which is a collapsed block prop, remove its child from it.
        if start_block.children_count() == 1 {
            let Some(child) = start_block.children().nth(0) else {
                unreachable!("Must always have a single child");
            };
            if child.is_collapsed_block_prop() {
                assert!(child.has_parent());
                child.remove_self(cx);
            }
        }

        // Step 32. If start block is an ancestor of end block:
        let values = if start_block.is_ancestor_of(&end_block) {
            // Step 32.1. Let reference node be end block.
            let mut reference_node = end_block.clone();
            // Step 32.2. While reference node is not a child of start block, set reference node to its parent.
            loop {
                if start_block.children().all(|child| child != reference_node) {
                    reference_node = reference_node
                        .GetParentNode()
                        .expect("Must always have a parent, at least start_block");
                    continue;
                }
                break;
            }
            // Step 32.3. Call collapse() on the context object's selection,
            // with first argument start block and second argument the index of reference node.
            if self
                .Collapse(
                    Some(&start_block),
                    reference_node.index(),
                    CanGc::from_cx(cx),
                )
                .is_err()
            {
                unreachable!("Must always be able to collapse");
            }
            // Step 32.4. If end block has no children:
            if end_block.children_count() == 0 {
                let mut end_block = end_block;
                // Step 32.4.1. While end block is editable and is the only child of its parent and is not a child of start block,
                // let parent equal end block, then remove end block from parent, then set end block to parent.
                loop {
                    if end_block.is_editable() &&
                        start_block.children().all(|child| child != end_block)
                    {
                        if let Some(parent) = end_block.GetParentNode() {
                            if parent.children_count() == 1 {
                                assert!(end_block.has_parent());
                                end_block.remove_self(cx);
                                end_block = parent;
                                continue;
                            }
                        }
                    }
                    break;
                }
                // Step 32.4.2. If end block is editable and is not an inline node,
                // and its previousSibling and nextSibling are both inline nodes,
                // call createElement("br") on the context object and insert it into end block's parent immediately after end block.
                if end_block.is_editable() &&
                    !end_block.is_inline_node() &&
                    end_block
                        .GetPreviousSibling()
                        .is_some_and(|previous| previous.is_inline_node())
                {
                    if let Some(next_of_end_block) = end_block.GetNextSibling() {
                        if next_of_end_block.is_inline_node() {
                            let br = context_object.create_element(cx, "br");
                            let parent = end_block
                                .GetParentNode()
                                .expect("Must always have a parent");
                            if parent
                                .InsertBefore(cx, br.upcast(), Some(&next_of_end_block))
                                .is_err()
                            {
                                unreachable!("Must always be able to insert into parent");
                            }
                        }
                    }
                }
                // Step 32.4.3. If end block is editable, remove it from its parent.
                if end_block.is_editable() {
                    assert!(end_block.has_parent());
                    end_block.remove_self(cx);
                }
                // Step 32.4.4. Restore states and values from overrides.
                active_range.restore_states_and_values(cx, self, context_object, overrides);

                // Step 32.4.5. Abort these steps.
                return;
            }
            let first_child = end_block
                .children()
                .nth(0)
                .expect("Already checked at least 1 child in previous statement");
            // Step 32.5. If end block's firstChild is not an inline node,
            // restore states and values from record, then abort these steps.
            if !first_child.is_inline_node() {
                active_range.restore_states_and_values(cx, self, context_object, overrides);
                return;
            }
            // Step 32.6. Let children be a list of nodes, initially empty.
            rooted_vec!(let mut children);
            // Step 32.7. Append the first child of end block to children.
            children.push(first_child.as_traced());
            // Step 32.8. While children's last member is not a br,
            // and children's last member's nextSibling is an inline node,
            // append children's last member's nextSibling to children.
            loop {
                let Some(last) = children.last() else {
                    break;
                };
                if last.is::<HTMLBRElement>() {
                    break;
                }
                let Some(next) = last.GetNextSibling() else {
                    break;
                };
                if next.is_inline_node() {
                    children.push(next.as_traced());
                    continue;
                }
                break;
            }
            // Step 32.9. Record the values of children, and let values be the result.
            let values = record_the_values(children.iter().map(|dom| dom.as_rooted()).collect());

            // Step 32.10. While children's first member's parent is not start block,
            // split the parent of children.
            loop {
                if children
                    .first()
                    .and_then(|child| child.GetParentNode())
                    .is_some_and(|parent_of_child| parent_of_child != start_block)
                {
                    split_the_parent(cx, children.r());
                    continue;
                }
                break;
            }
            // Step 32.11. If children's first member's previousSibling is an editable br,
            // remove that br from its parent.
            if let Some(first) = children.first() {
                if let Some(previous_of_first) = first.GetPreviousSibling() {
                    if previous_of_first.is_editable() && previous_of_first.is::<HTMLBRElement>() {
                        assert!(previous_of_first.has_parent());
                        previous_of_first.remove_self(cx);
                    }
                }
            }

            values
        // Step 33. Otherwise, if start block is a descendant of end block:
        } else if end_block.is_ancestor_of(&start_block) {
            // Step 33.1. Call collapse() on the context object's selection,
            // with first argument start block and second argument start block's length.
            if self
                .Collapse(Some(&start_block), start_block.len(), CanGc::from_cx(cx))
                .is_err()
            {
                unreachable!("Must always be able to collapse");
            }
            // Step 33.2. Let reference node be start block.
            let mut reference_node = start_block.clone();
            // Step 33.3. While reference node is not a child of end block, set reference node to its parent.
            loop {
                if end_block.children().all(|child| child != reference_node) {
                    if let Some(parent) = reference_node.GetParentNode() {
                        reference_node = parent;
                        continue;
                    }
                }
                break;
            }
            // Step 33.4. If reference node's nextSibling is an inline node and start block's lastChild is a br,
            // remove start block's lastChild from it.
            if reference_node
                .GetNextSibling()
                .is_some_and(|next| next.is_inline_node())
            {
                if let Some(last) = start_block.children().last() {
                    if last.is::<HTMLBRElement>() {
                        assert!(last.has_parent());
                        last.remove_self(cx);
                    }
                }
            }
            // Step 33.5. Let nodes to move be a list of nodes, initially empty.
            rooted_vec!(let mut nodes_to_move);
            // Step 33.6. If reference node's nextSibling is neither null nor a block node,
            // append it to nodes to move.
            if let Some(next) = reference_node.GetNextSibling() {
                if !next.is_block_node() {
                    nodes_to_move.push(next);
                }
            }
            // Step 33.7. While nodes to move is nonempty and its last member isn't a br
            // and its last member's nextSibling is neither null nor a block node,
            // append its last member's nextSibling to nodes to move.
            loop {
                if let Some(last) = nodes_to_move.last() {
                    if !last.is::<HTMLBRElement>() {
                        if let Some(next_of_last) = last.GetNextSibling() {
                            if !next_of_last.is_block_node() {
                                nodes_to_move.push(next_of_last);
                                continue;
                            }
                        }
                    }
                }
                break;
            }
            // Step 33.8. Record the values of nodes to move, and let values be the result.
            let values = record_the_values(nodes_to_move.iter().cloned().collect());

            // Step 33.9. For each node in nodes to move,
            // append node as the last child of start block, preserving ranges.
            for node in nodes_to_move.iter() {
                move_preserving_ranges(cx, node, |cx| start_block.AppendChild(cx, node));
            }

            values
        // Step 34. Otherwise:
        } else {
            // Step 34.1. Call collapse() on the context object's selection,
            // with first argument start block and second argument start block's length.
            if self
                .Collapse(Some(&start_block), start_block.len(), CanGc::from_cx(cx))
                .is_err()
            {
                unreachable!("Must always be able to collapse");
            }
            // Step 34.2. If end block's firstChild is an inline node and start block's lastChild is a br,
            // remove start block's lastChild from it.
            if end_block
                .children()
                .nth(0)
                .is_some_and(|next| next.is_inline_node())
            {
                if let Some(last) = start_block.children().last() {
                    if last.is::<HTMLBRElement>() {
                        assert!(last.has_parent());
                        last.remove_self(cx);
                    }
                }
            }
            // Step 34.3. Record the values of end block's children, and let values be the result.
            let values = record_the_values(end_block.children().collect());

            // Step 34.4. While end block has children,
            // append the first child of end block to start block, preserving ranges.
            loop {
                if let Some(first_child) = end_block.children().nth(0) {
                    move_preserving_ranges(cx, &first_child, |cx| {
                        start_block.AppendChild(cx, &first_child)
                    });
                    continue;
                }
                break;
            }
            // Step 34.5. While end block has no children,
            // let parent be the parent of end block, then remove end block from parent,
            // then set end block to parent.
            let mut end_block = end_block;
            loop {
                if end_block.children_count() == 0 {
                    if let Some(parent) = end_block.GetParentNode() {
                        assert!(end_block.has_parent());
                        end_block.remove_self(cx);
                        end_block = parent;
                        continue;
                    }
                }
                break;
            }

            values
        };

        // Step 35.
        //
        // This step does not exist in the spec

        // Step 36. Let ancestor be start block.
        // TODO

        // Step 37. While ancestor has an inclusive ancestor ol in the same editing host whose nextSibling is
        // also an ol in the same editing host, or an inclusive ancestor ul in the same editing host whose nextSibling
        // is also a ul in the same editing host:
        // TODO

        // Step 38. Restore the values from values.
        restore_the_values(cx, values);

        // Step 39. If start block has no children, call createElement("br") on the context object and
        // append the result as the last child of start block.
        if start_block.children_count() == 0 {
            let br = context_object.create_element(cx, "br");
            if start_block.AppendChild(cx, br.upcast()).is_err() {
                unreachable!("Must always be able to append");
            }
        }

        // Step 40. Remove extraneous line breaks at the end of start block.
        start_block.remove_extraneous_line_breaks_at_the_end_of(cx);

        // Step 41. Restore states and values from overrides.
        active_range.restore_states_and_values(cx, self, context_object, overrides);
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#set-the-selection%27s-value>
    pub(crate) fn set_the_selection_value(
        &self,
        cx: &mut JSContext,
        new_value: Option<DOMString>,
        command: CommandName,
        context_object: &Document,
    ) {
        let active_range = self
            .active_range()
            .expect("Must always have an active range");

        // Step 1. Let command be the current command.
        //
        // Passed as argument

        // Step 2. If there is no formattable node effectively contained in the active range:
        if active_range.first_formattable_contained_node().is_none() {
            // Step 2.1. If command has inline command activated values, set the state override to true if new value is among them and false if it's not.
            let inline_command_activated_values = command.inline_command_activated_values();
            if !inline_command_activated_values.is_empty() {
                context_object.set_state_override(
                    command,
                    Some(new_value.as_ref().is_some_and(|new_value| {
                        inline_command_activated_values.contains(&new_value.str().as_ref())
                    })),
                );
            }
            // Step 2.2. If command is "subscript", unset the state override for "superscript".
            if command == CommandName::Subscript {
                context_object.set_state_override(CommandName::Superscript, None);
            }
            // Step 2.3. If command is "superscript", unset the state override for "subscript".
            if command == CommandName::Superscript {
                context_object.set_state_override(CommandName::Subscript, None);
            }
            // Step 2.4. If new value is null, unset the value override (if any).
            // Step 2.5. Otherwise, if command is "createLink" or it has a value specified, set the value override to new value.
            context_object.set_value_override(command, new_value);
            // Step 2.6. Abort these steps.
            return;
        }
        // Step 3. If the active range's start node is an editable Text node,
        // and its start offset is neither zero nor its start node's length,
        // call splitText() on the active range's start node,
        // with argument equal to the active range's start offset.
        // Then set the active range's start node to the result, and its start offset to zero.
        let start_node = active_range.start_container();
        let start_offset = active_range.start_offset();
        if start_node.is_editable() && start_offset != 0 && start_offset != start_node.len() {
            if let Some(start_text) = start_node.downcast::<Text>() {
                let Ok(start_text) = start_text.SplitText(cx, start_offset) else {
                    unreachable!("Must always be able to split");
                };
                active_range.set_start(start_text.upcast(), 0);
            }
        }
        // Step 4. If the active range's end node is an editable Text node,
        // and its end offset is neither zero nor its end node's length,
        // call splitText() on the active range's end node,
        // with argument equal to the active range's end offset.
        let end_node = active_range.end_container();
        let end_offset = active_range.end_offset();
        if end_node.is_editable() && end_offset != 0 && end_offset != end_node.len() {
            if let Some(end_text) = end_node.downcast::<Text>() {
                if end_text.SplitText(cx, end_offset).is_err() {
                    unreachable!("Must always be able to split");
                };
            }
        }
        // Step 5. Let element list be all editable Elements effectively contained in the active range.
        // Step 6. For each element in element list, clear the value of element.
        active_range.for_each_effectively_contained_child(|child| {
            if child.is_editable() {
                if let Some(element_child) = child.downcast::<HTMLElement>() {
                    element_child.clear_the_value(cx, &command);
                }
            }
        });
        // Step 7. Let node list be all editable nodes effectively contained in the active range.
        // Step 8. For each node in node list:
        active_range.for_each_effectively_contained_child(|child| {
            if child.is_editable() {
                // Step 8.1. Push down values on node.
                child.push_down_values(cx, &command, new_value.clone());
                // Step 8.2. If node is an allowed child of "span", force the value of node.
                if is_allowed_child(
                    NodeOrString::Node(DomRoot::from_ref(child)),
                    NodeOrString::String("span".to_owned()),
                ) {
                    child.force_the_value(cx, &command, new_value.as_ref());
                }
            }
        });
    }
}
