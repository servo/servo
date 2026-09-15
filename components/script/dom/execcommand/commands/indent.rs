/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use js::context::JSContext;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;

use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::contenteditable::node::{
    NodeOrString, is_allowed_child, wrap_node_list,
};
use crate::dom::html::htmllielement::HTMLLIElement;
use crate::dom::html::htmlolistelement::HTMLOListElement;
use crate::dom::html::htmlulistelement::HTMLUListElement;
use crate::dom::node::Node;
use crate::dom::selection::Selection;

// <https://w3c.github.io/editing/docs/execCommand/#indent>
pub(crate) fn indent(cx: &mut JSContext, context_object: &Document, node_list: Vec<DomRoot<Node>>) {
    // Step 1. If node list is empty, do nothing and abort these steps.
    if node_list.is_empty() {
        return;
    }

    // Step 2. Let first node be the first member of node list.
    let first_node = node_list
        .first()
        .expect("Must have a first node by now.")
        .clone();

    // Step 3. If first node's parent is an ol or ul:
    if let Some(parent) = first_node.GetParentElement() &&
        (parent.is::<HTMLOListElement>() || parent.is::<HTMLUListElement>())
    {
        // Step 3.1. Let tag be the local name of the parent of first node.
        let tag = parent.local_name();

        // Step 3.2. Wrap node list,
        //           with sibling criteria returning true for an HTML element with local name tag and false otherwise,
        //           and new parent instructions returning the result of calling createElement(tag) on the ownerDocument of first node.
        wrap_node_list(
            cx,
            node_list,
            |sibling| {
                sibling
                    .downcast::<Element>()
                    .is_some_and(|sibling| sibling.local_name() == tag)
            },
            |cx| {
                Some(DomRoot::upcast(
                    first_node.owner_doc().create_element(cx, tag),
                ))
            },
        );

        // Step 3.3. Abort these steps.
        return;
    }

    // Step 4. Wrap node list,
    //         with sibling criteria returning true for a simple indentation element and false otherwise,
    //         and new parent instructions returning the result of calling createElement("blockquote") on the ownerDocument of first node.
    //         Let new parent be the result.
    let new_parent = wrap_node_list(
        cx,
        node_list,
        |sibling| {
            sibling
                .downcast::<Element>()
                .is_some_and(|sibling| sibling.is_simple_indentation_element())
        },
        |cx| {
            Some(DomRoot::upcast(
                first_node.owner_doc().create_element(cx, "blockquote"),
            ))
        },
    );

    // Step 5. Fix disallowed ancestors of new parent.
    if let Some(new_parent) = new_parent {
        new_parent.fix_disallowed_ancestors(cx, context_object);
    }
}

/// <https://w3c.github.io/editing/docs/execCommand/#the-indent-command>
pub(crate) fn execute_indent_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    let mut active_range = selection
        .active_range(cx)
        .expect("Must always have an active range.");
    // Step 1. Let items be a list of all lis that are inclusive ancestors of the active range's start and/or end node.
    let items: HashSet<DomRoot<Node>> = active_range
        .start_container()
        .ancestors()
        .chain(active_range.end_container().ancestors())
        .filter(|ancestor| ancestor.is::<HTMLLIElement>())
        .collect();

    // Step 2. For each item in items, normalize sublists of item.
    for item in items {
        item.normalize_sublists(cx);
    }

    // Normalizing sublists probably messes up the range
    active_range = selection
        .active_range(cx)
        .expect("Must always have an active range.");

    // Step 3. Block-extend the active range, and let new range be the result.
    let new_range = active_range.block_extend(cx, document);

    // Step 4. Let node list be a list of nodes, initially empty.
    let mut node_list: Vec<DomRoot<Node>> = vec![];

    // Step 5. For each node node contained in new range,
    //         if node is editable and is an allowed child of "div" or "ol"
    //         and if the last member of node list (if any) is not an ancestor of node,
    //         append node to node list.
    for node in new_range.contained_nodes(cx.no_gc()) {
        if node.is_editable() &&
            (is_allowed_child(
                NodeOrString::Node(node.clone()),
                NodeOrString::String("div".to_owned()),
            ) || is_allowed_child(
                NodeOrString::Node(node.clone()),
                NodeOrString::String("ol".to_owned()),
            )) &&
            node_list
                .last()
                .is_none_or(|last| !last.is_ancestor_of(&node))
        {
            node_list.push(node.as_rooted());
        }
    }

    // Step 6. If the first visible member of node list is an li whose parent is an ol or ul:
    if let Some(first_visible_member) = node_list.iter().find(|node| node.is_visible(cx.no_gc())) &&
        first_visible_member.is::<HTMLLIElement>() &&
        let Some(parent) = first_visible_member.GetParentNode() &&
        (parent.is::<HTMLOListElement>() || parent.is::<HTMLUListElement>())
    {
        // Step 6.1. Let sibling be node list's first visible member's previousSibling.
        let mut sibling = first_visible_member.GetPreviousSibling();

        // Step 6.2. While sibling is invisible, set sibling to its previousSibling.
        while let Some(ref some_sibling) = sibling &&
            some_sibling.is_invisible(cx.no_gc())
        {
            sibling = some_sibling.GetPreviousSibling();
        }

        // Step 6.3. If sibling is an li, normalize sublists of sibling.
        if let Some(sibling) = sibling &&
            sibling.is::<HTMLLIElement>()
        {
            sibling.normalize_sublists(cx);
        }
    }

    // Step 7. While node list is not empty:
    let mut node_list_iter = node_list.iter().peekable();
    while node_list_iter.peek().is_some() {
        // Step 7.1. Let sublist be a list of nodes, initially empty.
        let mut sublist: Vec<DomRoot<Node>> = vec![];

        // Step 7.2. Remove the first member of node list and append it to sublist.
        sublist.push(
            node_list_iter
                .next()
                .expect("Must always have a next item")
                .clone(),
        );

        // Step 7.3. While the first member of node list is the nextSibling of the last member of
        //           sublist, remove the first member of node list and append it to sublist.
        while node_list_iter.peek().is_some_and(|node| {
            sublist
                .last()
                .expect("Must always have last element here.")
                .GetNextSibling()
                .is_some_and(|next_sibling| &&next_sibling == node)
        }) {
            sublist.push(
                node_list_iter
                    .next()
                    .expect("Must always have a next item")
                    .clone(),
            );
        }

        // Step 7.4. Indent sublist.
        indent(cx, document, sublist);
    }

    // Step 8. Return true.
    // Note: This isn't in the spec (yet), see https://github.com/w3c/editing/pull/547
    true
}
