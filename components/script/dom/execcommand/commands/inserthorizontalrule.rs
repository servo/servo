/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::codegen::GenericBindings::RangeBinding::RangeMethods;
use script_bindings::codegen::GenericBindings::SelectionBinding::SelectionMethods;
use script_bindings::inheritance::Castable;

use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::execcommand::contenteditable::selection::SelectionDeletionBlockMerging;
use crate::dom::selection::Selection;
use crate::dom::text::Text;

/// <https://w3c.github.io/editing/docs/execCommand/#the-inserthorizontalrule-command>
pub(crate) fn execute_insert_horizontal_rule_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    // Step 1. Let start node, start offset, end node, and end offset be the active range's start and end nodes and offsets.
    let mut active_range = selection
        .active_range()
        .expect("Must always have an active range");
    let mut start_node = active_range.start_container();
    let mut start_offset = active_range.start_offset();
    let mut end_node = active_range.end_container();
    let mut end_offset = active_range.end_offset();

    // Step 2. While start offset is 0 and start node's parent is not null, set start offset to start node's index,
    //         then set start node to its parent.
    while let Some(parent) = start_node.GetParentNode() &&
        start_offset == 0
    {
        start_offset = start_node.index();
        start_node = parent;
    }

    // Step 3. While end offset is end node's length, and end node's parent is not null, set end offset to one plus end
    //         node's index, then set end node to its parent.
    while let Some(parent) = end_node.GetParentNode() &&
        end_offset == end_node.len()
    {
        end_offset = 1 + end_node.index();
        end_node = parent;
    }

    // Step 4. Call collapse(start node, start offset) on the context object's selection.
    if selection
        .Collapse(cx, Some(&start_node), start_offset)
        .is_err()
    {
        unreachable!("Should always be able to collapse the selection.");
    }

    // Step 5. Call extend(end node, end offset) on the context object's selection.
    if selection.Extend(cx, &end_node, end_offset).is_err() {
        unreachable!("Should always be able to extend the selection.");
    }

    // Step 6. Delete the selection, with block merging false.
    selection.delete_the_selection(
        cx,
        document,
        SelectionDeletionBlockMerging::Skip,
        Default::default(),
        Default::default(),
    );

    active_range = selection
        .active_range()
        .expect("Must always have an active range");

    // Step 7. If the active range's start node is neither editable nor an editing host, return true.
    if !active_range.start_container().is_editable_or_editing_host() {
        return true;
    }

    // Step 8. If the active range's start node is a Text node and its start offset is zero, call collapse() on the
    //         context object's selection, with first argument the active range's start node's parent and second
    //         argument the active range's start node's index.
    if active_range.start_container().is::<Text>() && active_range.start_offset() == 0 {
        if selection
            .Collapse(
                cx,
                active_range.start_container().GetParentNode().as_deref(),
                active_range.start_container().index(),
            )
            .is_err()
        {
            unreachable!("Should always be able to collapse the selection.");
        }
        active_range = selection
            .active_range()
            .expect("Must always have an active range");
    }

    // Step 9. If the active range's start node is a Text node and its start offset is the length of its start node,
    //         call collapse() on the context object's selection, with first argument the active range's start node's
    //         parent, and the second argument one plus the active range's start node's index.
    if active_range.start_container().is::<Text>() &&
        active_range.start_offset() == active_range.start_container().len()
    {
        if selection
            .Collapse(
                cx,
                active_range.start_container().GetParentNode().as_deref(),
                1 + active_range.start_container().index(),
            )
            .is_err()
        {
            unreachable!("Should always be able to collapse the selection.");
        }
        active_range = selection
            .active_range()
            .expect("Must always have an active range");
    }

    // Step 10. Let hr be the result of calling createElement("hr") on the context object.
    let hr = document.create_element(cx, "hr");

    // Step 11. Run insertNode(hr) on the active range.
    let hr_node = DomRoot::upcast(hr);
    if active_range.InsertNode(cx, &hr_node).is_err() {
        unreachable!("The image should always be insertable.");
    }

    // Step 12. Fix disallowed ancestors of hr.
    hr_node.fix_disallowed_ancestors(cx, document);

    // Step 13. Run collapse() on the context object's selection, with first argument hr's parent and the second argument equal to one plus hr's index.
    if selection
        .Collapse(cx, hr_node.GetParentNode().as_deref(), 1 + hr_node.index())
        .is_err()
    {
        unreachable!("Should always be able to collapse the selection.");
    }

    // Step 14.
    true
}
