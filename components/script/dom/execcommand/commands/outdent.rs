/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use html5ever::local_name;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomSlice;

use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::commands::indent::indent;
use crate::dom::execcommand::contenteditable::node::{
    record_the_values, restore_the_values, split_the_parent,
};
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmllielement::HTMLLIElement;
use crate::dom::html::htmlolistelement::HTMLOListElement;
use crate::dom::html::htmlulistelement::HTMLUListElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::Node;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#outdent>
pub(crate) fn outdent(cx: &mut JSContext, context_object: &Document, node: &DomRoot<Node>) {
    // Step 1. If node is not editable, abort these steps.
    if !node.is_editable() {
        return;
    }

    // Step 2. If node is a simple indentation element, remove node, preserving its descendants.
    //         Then abort these steps.
    if node
        .downcast::<Element>()
        .is_some_and(|element| element.is_simple_indentation_element())
    {
        node.remove_preserving_its_descendants(cx);
        return;
    }

    // Step 3. If node is an indentation element:
    if let Some(node_as_element) = node.downcast::<Element>() &&
        node_as_element.is_indentation_element()
    {
        // Step 3.1. Unset the dir attribute of node, if any.
        node_as_element.remove_attribute_by_name(cx, &local_name!("dir"));

        // Step 3.2. Unset the margin, padding, and border CSS properties of node.
        if let Some(node_as_html_element) = node_as_element.downcast::<HTMLElement>() {
            let _ = node_as_html_element
                .Style(cx)
                .RemoveProperty(cx, "margin".into());
            let _ = node_as_html_element
                .Style(cx)
                .RemoveProperty(cx, "padding".into());
            let _ = node_as_html_element
                .Style(cx)
                .RemoveProperty(cx, "border".into());
        }

        // Step 3.3. Set the tag name of node to "div".
        node_as_element.set_the_tag_name(cx, "div");

        // Step 3.4. Abort these steps.
        return;
    }

    // Step 4. Let current ancestor be node's parent.
    let mut current_ancestor = node.GetParentNode();

    // Step 5. Let ancestor list be a list of nodes, initially empty.
    let mut ancestor_list: Vec<DomRoot<Node>> = vec![];

    // Step 6. While current ancestor is an editable Element that is neither a simple indentation
    //         element nor an ol nor a ul, append current ancestor to ancestor list and then set
    //         current ancestor to its parent.
    while let Some(ancestor) = current_ancestor.clone() &&
        ancestor.is_editable() &&
        !ancestor
            .downcast::<Element>()
            .is_some_and(|element| element.is_simple_indentation_element()) &&
        !ancestor.is::<HTMLOListElement>() &&
        !ancestor.is::<HTMLUListElement>()
    {
        current_ancestor = ancestor.GetParentNode();
        ancestor_list.push(ancestor);
    }

    // Step 7. If current ancestor is not an editable simple indentation element:
    if !current_ancestor.as_ref().is_some_and(|ancestor| {
        ancestor.is_editable() &&
            ancestor
                .downcast::<Element>()
                .is_some_and(|element| element.is_simple_indentation_element())
    }) {
        // Step 7.1. Let current ancestor be node's parent.
        current_ancestor = node.GetParentNode();

        // Step 7.2. Let ancestor list be the empty list.
        ancestor_list.clear();

        // Step 7.3. While current ancestor is an editable Element that is neither an indentation
        //           element nor an ol nor a ul, append current ancestor to ancestor list and then
        //           set current ancestor to its parent.
        while let Some(ancestor) = current_ancestor.clone() &&
            ancestor.is_editable() &&
            !ancestor
                .downcast::<Element>()
                .is_some_and(|element| element.is_indentation_element()) &&
            !ancestor.is::<HTMLOListElement>() &&
            !ancestor.is::<HTMLUListElement>()
        {
            current_ancestor = ancestor.GetParentNode();
            ancestor_list.push(ancestor);
        }
    }

    // Step 8. If node is an ol or ul and current ancestor is not an editable indentation element:
    if let Some(node_as_element) = node.downcast::<Element>() &&
        (node.is::<HTMLOListElement>() || node.is::<HTMLUListElement>()) &&
        !current_ancestor.as_ref().is_some_and(|ancestor| {
            ancestor.is_editable() &&
                ancestor
                    .downcast::<Element>()
                    .is_some_and(|element| element.is_indentation_element())
        })
    {
        // Step 8.1. Unset the reversed, start, and type attributes of node, if any are set.
        node_as_element.remove_attribute_by_name(cx, &local_name!("reversed"));
        node_as_element.remove_attribute_by_name(cx, &local_name!("start"));
        node_as_element.remove_attribute_by_name(cx, &local_name!("type"));

        // Step 8.2. Let children be the children of node.
        let children = node.children();

        // Step 8.3. If node has attributes, and its parent is not an ol or ul, set the tag name of node to "div".
        if !node_as_element.attrs().borrow().is_empty() &&
            !node.GetParentElement().is_some_and(|parent| {
                parent.is::<HTMLOListElement>() || parent.is::<HTMLUListElement>()
            })
        {
            node_as_element.set_the_tag_name(cx, "div");
        }
        // Step 8.4. Otherwise:
        else {
            // Step 8.4.1. Record the values of node's children, and let values be the result.
            let values = record_the_values(node.children().collect());

            // Step 8.4.2. Remove node, preserving its descendants.
            node.remove_preserving_its_descendants(cx);

            // Step 8.4.3. Restore the values from values.
            restore_the_values(cx, values);
        }

        // Step 8.5. Fix disallowed ancestors of each member of children.
        for child in children {
            child.fix_disallowed_ancestors(cx, context_object);
        }

        // Step 8.6. Abort these steps.
        return;
    }

    // Step 9. If current ancestor is not an editable indentation element, abort these steps.
    if !current_ancestor.as_ref().is_some_and(|ancestor| {
        ancestor.is_editable() &&
            ancestor
                .downcast::<Element>()
                .is_some_and(|element| element.is_indentation_element())
    }) {
        return;
    }

    // Step 10. Append current ancestor to ancestor list.
    ancestor_list.push(
        current_ancestor
            .clone()
            .expect("Should have an ancestor here."),
    );

    // Step 11. Let original ancestor be current ancestor.
    let original_ancestor = current_ancestor;

    // Step 12. While ancestor list is not empty:
    while !ancestor_list.is_empty() {
        // Step 12.1. Let current ancestor be the last member of ancestor list.
        // Step 12.2. Remove the last member from ancestor list.
        let current_ancestor_ref = ancestor_list.pop();

        // Step 12.3. Let target be the child of current ancestor that is equal to either node or
        //            the last member of ancestor list.
        let target = current_ancestor_ref
            .expect("Must always have an ancestor here.")
            .children()
            .find(|child| child == node || Some(child) == ancestor_list.last())
            .expect("Must be able to find a target here.");

        // Step 12.4. If target is an inline node that is not a br, and its nextSibling is a br,
        //            remove target's nextSibling from its parent.
        if target.is_inline_node() &&
            let Some(next_br_sibling) = target.GetNextSibling() &&
            next_br_sibling.is::<HTMLBRElement>()
        {
            next_br_sibling.remove_self(cx);
        }

        // Step 12.5. Let preceding siblings be the precedings siblings of target, and let
        //            following siblings be the followings siblings of target.
        let preceding_siblings = target.preceding_siblings().collect();
        let following_siblings = target.following_siblings().collect();

        // Step 12.6. Indent preceding siblings.
        indent(cx, context_object, preceding_siblings);

        // Step 12.7. Indent following siblings.
        indent(cx, context_object, following_siblings);
    }

    // Step 13. Outdent original ancestor.
    outdent(
        cx,
        context_object,
        &original_ancestor.expect("Should have an ancestor here."),
    );
}

/// <https://w3c.github.io/editing/docs/execCommand/#the-outdent-command>
pub(crate) fn execute_outdent_command(
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
    rooted_vec!(let mut node_list);

    // Step 5. For each node node contained in new range,
    //         append node to node list if the last member of node list (if any)
    //         is not an ancestor of node; node is editable;
    //         and either node has no editable descendants, or is an ol or ul,
    //         or is an li whose parent is an ol or ul.
    for node in new_range.contained_nodes() {
        if !node_list
            .last()
            .is_some_and(|last: &Dom<Node>| last.is_ancestor_of(&node)) &&
            node.is_editable() &&
            (!node
                .traverse_preorder(ShadowIncluding::No)
                .any(|node| node.is_editable()) ||
                node.is::<HTMLOListElement>() ||
                node.is::<HTMLUListElement>() ||
                (node.is::<HTMLLIElement>() &&
                    node.GetParentElement().is_some_and(|parent| {
                        parent.is::<HTMLOListElement>() || parent.is::<HTMLUListElement>()
                    })))
        {
            node_list.push(node.as_traced());
        }
    }

    // Step 6. While node list is not empty:
    let mut node_list_iter = node_list.iter().peekable();
    while node_list_iter.peek().is_some() {
        // Step 6.1. While the first member of node list is an ol or ul or is not the child of an
        //           ol or ul, outdent it and remove it from node list.
        while node_list_iter.peek().is_some_and(|first| {
            (first.is::<HTMLOListElement>() || first.is::<HTMLUListElement>()) &&
                !first.GetParentElement().is_some_and(|parent| {
                    parent.is::<HTMLOListElement>() || parent.is::<HTMLUListElement>()
                })
        }) {
            // We just walk the iterator instead of continuously removing things from node list.
            outdent(
                cx,
                document,
                &node_list_iter
                    .next()
                    .expect("Should have a node here.")
                    .as_rooted(),
            );
        }

        // Step 6.2. If node list is empty, break from these substeps.
        if !node_list_iter.peek().is_some() {
            break;
        }

        // Step 6.3. Let sublist be a list of nodes, initially empty.
        rooted_vec!(let mut sublist);

        // Step 6.4. Remove the first member of node list and append it to sublist.
        sublist.push(
            node_list_iter
                .next()
                .expect("Must always have a next item")
                .clone(),
        );

        // Step 6.5. While the first member of node list is the nextSibling of the last member of
        //           sublist, and the first member of node list is not an ol or ul, remove the
        //           first member of node list and append it to sublist.
        while node_list_iter.peek().is_some_and(|node| {
            sublist
                .last()
                .expect("Must always have last element here.")
                .GetNextSibling()
                .is_some_and(|next_sibling| next_sibling == node.as_rooted()) &&
                !node.is::<HTMLOListElement>() &&
                !node.is::<HTMLUListElement>()
        }) {
            sublist.push(
                node_list_iter
                    .next()
                    .expect("Must always have a next item")
                    .clone(),
            );
        }

        // Step 6.6. Record the values of sublist, and let values be the result.
        let values = record_the_values(sublist.iter().map(|node| node.as_rooted()).collect());

        // Step 6.7. Split the parent of sublist.
        split_the_parent(cx, sublist.r());

        // Step 6.8. Fix disallowed ancestors of each member of sublist.
        for node in sublist.iter() {
            node.fix_disallowed_ancestors(cx, document);
        }

        // Step 6.9. Restore the values from values.
        restore_the_values(cx, values);
    }

    // Step 7. Return true.
    true
}
