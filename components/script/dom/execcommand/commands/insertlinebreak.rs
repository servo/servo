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
use crate::dom::element::Element;
use crate::dom::execcommand::contenteditable::node::{NodeOrString, is_allowed_child};
use crate::dom::execcommand::contenteditable::selection::SelectionDeletionStripWrappers;
use crate::dom::selection::Selection;
use crate::dom::text::Text;

/// <https://w3c.github.io/editing/docs/execCommand/#the-insertlinebreak-command>
pub(crate) fn execute_insert_line_break_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    // Step 1. Delete the selection, with strip wrappers false.
    selection.delete_the_selection(
        cx,
        document,
        Default::default(),
        SelectionDeletionStripWrappers::NoStrip,
        Default::default(),
    );

    // Step 2. If the active range's start node is neither editable nor an editing host, return
    //         true.
    let mut active_range = selection
        .active_range(cx)
        .expect("Must always have an active range.");
    if !active_range.start_container().is_editable_or_editing_host() {
        return true;
    }

    // Step 3. If the active range's start node is an Element, and "br" is not an allowed child of
    //         it, return true.
    if active_range.start_container().is::<Element>() &&
        !is_allowed_child(
            NodeOrString::String("br".to_owned()),
            NodeOrString::from_node(&active_range.start_container(), cx.no_gc()),
        )
    {
        return false;
    }

    // Step 4. If the active range's start node is not an Element, and "br" is not an allowed child
    //         of the active range's start node's parent, return true.
    if !active_range.start_container().is::<Element>() &&
        !is_allowed_child(
            NodeOrString::String("br".to_owned()),
            NodeOrString::from_node(
                &active_range
                    .start_container()
                    .GetParentNode()
                    .expect("Must always have a parent."),
                cx.no_gc(),
            ),
        )
    {
        return false;
    }

    // Step 5. If the active range's start node is a Text node and its start offset is zero, call
    //         collapse() on the context object's selection, with first argument equal to the
    //         active range's start node's parent and second argument equal to the active range's
    //         start node's index.
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
            .active_range(cx)
            .expect("Must always have an active range");
    }

    // Step 6. If the active range's start node is a Text node and its start offset is the length
    //         of its start node, call collapse() on the context object's selection, with first
    //         argument equal to the active range's start node's parent and second argument equal
    //         to one plus the active range's start node's index.
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
            .active_range(cx)
            .expect("Must always have an active range");
    }

    // Step 7. Let br be the result of calling createElement("br") on the context object.
    let br = document.create_element(cx, "br");

    // Step 8. Call insertNode(br) on the active range.
    let br_node = DomRoot::upcast(br);
    if active_range.InsertNode(cx, &br_node).is_err() {
        unreachable!("The node should always be insertable.");
    }

    // Step 9. Call collapse() on the context object's selection, with br's parent as the first
    //         argument and one plus br's index as the second argument.
    if selection
        .Collapse(cx, br_node.GetParentNode().as_deref(), 1 + br_node.index())
        .is_err()
    {
        unreachable!("Should always be able to collapse the selection.");
    }

    // Step 10. If br is a collapsed line break, call createElement("br") on the context object and
    //          let extra br be the result, then call insertNode(extra br) on the active range.
    // TODO: Implement this.

    // Step 11. Return true.
    true
}
