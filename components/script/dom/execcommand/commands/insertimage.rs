/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::codegen::GenericBindings::SelectionBinding::SelectionMethods;
use script_bindings::inheritance::Castable;
use style::attr::AttrValue;

use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::contenteditable::selection::SelectionDeletionStripWrappers;
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-insertimage-command>
pub(crate) fn execute_insert_image_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
    value: DOMString,
) -> bool {
    // Step 1. If value is the empty string, return false.
    if value.is_empty() {
        return false;
    }

    // Step 2. Delete the selection, with strip wrappers false.
    selection.delete_the_selection(
        cx,
        document,
        Default::default(),
        SelectionDeletionStripWrappers::NoStrip,
        Default::default(),
    );

    // Step 3. Let range be the active range.
    let range = selection
        .active_range()
        .expect("Must always have an active range");

    // Step 4. If the active range's start node is neither editable nor an editing host, return true.
    let start_node = range.start_container();
    if !start_node.is_editable_or_editing_host() {
        return true;
    }

    // Step 5. If range's start node is a block node whose sole child is a br, and its start offset is 0, remove its start node's child from it.
    if start_node.is_block_node() && start_node.children_count() == 1 {
        let sole_child = start_node
            .children()
            .next()
            .expect("range's start node has one child.");
        if sole_child.is::<HTMLBRElement>() {
            sole_child.remove_self(cx);
        }
    }

    // Step 6. Let img be the result of calling createElement("img") on the context object.
    let img = document.create_element(cx, "img");

    // Step 7. Run setAttribute("src", value) on img.
    img.set_attribute(
        cx,
        &html5ever::local_name!("src"),
        AttrValue::String(value.str().to_owned()),
    );

    // Step 8. Run insertNode(img) on range.
    let img_node = DomRoot::upcast(img);
    if range.InsertNode(cx, &img_node).is_err() {
        unreachable!("The image should always be insertable.");
    }

    // Step 9. Let selection be the result of calling getSelection() on the context object.
    // Step 10. Run collapse() on selection, with first argument equal to the parent of img and the second argument equal to one plus the index of img.
    if selection
        .Collapse(
            cx,
            img_node.GetParentNode().as_deref(),
            1 + img_node.index(),
        )
        .is_err()
    {
        unreachable!("The selection should always be collapsible.");
    }

    // Step 11. Return true.
    true
}
