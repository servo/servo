/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::Node;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::comment::Comment;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::contenteditable::node::{
    NodeOrString, is_allowed_child, node_matches_local_name, split_the_parent, wrap_node_list,
};
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::selection::Selection;
use crate::dom::text::Text;

/// <https://w3c.github.io/editing/docs/execCommand/#the-insertparagraph-command>
pub(crate) fn execute_insert_paragraph_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    // Step 1. Delete the selection.
    selection.delete_the_selection(
        cx,
        document,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    // Step 3. Let node and offset be the active range's start node and offset.
    let active_range = selection
        .active_range()
        .expect("Must always have an active range");
    let mut node = active_range.start_container();
    let mut offset = active_range.start_offset();
    // Step 2. If the active range's start node is neither editable
    // nor an editing host, return true.
    if !node.is_editable_or_editing_host() {
        return true;
    }
    // Step 4. If node is a Text node, and offset is neither 0 nor the length of node,
    // call splitText(offset) on node.
    if offset != 0
        && offset != node.len()
        && let Some(text_node) = node.downcast::<Text>()
        && text_node.SplitText(cx, offset).is_err()
    {
        unreachable!("Must always be able to split");
    }
    // Step 5. If node is a Text node and offset is its length,
    // set offset to one plus the index of node, then set node to its parent.
    if node.is::<Text>() && offset == node.len() {
        offset = 1 + node.index();
        node = node.GetParentNode().expect("Must always have a parent");
    }
    // Step 6. If node is a Text or Comment node, set offset to the index of node,
    // then set node to its parent.
    if node.is::<Text>() || node.is::<Comment>() {
        offset = node.index();
        node = node.GetParentNode().expect("Must always have a parent");
    }
    // Step 7. Call collapse(node, offset) on the context object's selection.
    selection.collapse_current_range(&node, offset);
    // Step 8. Let container equal node.
    let mut container = node.clone();
    // Step 9. While container is not a single-line container,
    // and container's parent is editable and in the same editing host as node,
    // set container to its parent.
    while !container.is_single_line_container()
        && let Some(parent) = container.GetParentNode()
        && parent.is_editable()
        && parent.same_editing_host(&node)
    {
        container = parent;
    }
    // Step 10. If container is an editable single-line container in the same editing host as node,
    // and its local name is "p" or "div":
    if container.is_editable()
        && container.is_single_line_container()
        && container.same_editing_host(&node)
        && node_matches_local_name!(container, local_name!("p") | local_name!("div"))
    {
        // Step 10.1. Let outer container equal container.
        let mut outer_container = container.clone();
        // Step 10.2. While outer container is not a dd or dt or li,
        // and outer container's parent is editable, set outer container to its parent.
        while !node_matches_local_name!(
            outer_container,
            local_name!("dd") | local_name!("dt") | local_name!("li")
        ) && let Some(parent) = outer_container.GetParentNode()
            && parent.is_editable()
        {
            outer_container = parent;
        }
        // Step 10.3. If outer container is a dd or dt or li, set container to outer container.
        if node_matches_local_name!(
            outer_container,
            local_name!("dd") | local_name!("dt") | local_name!("li")
        ) {
            container = outer_container;
        }
    }
    // Step 11. If container is not editable or not in the same editing host as node or is not a single-line container:
    if !container.is_editable()
        || !container.same_editing_host(&node)
        || !container.is_single_line_container()
    {
        // Step 11.1. Let tag be the default single-line container name.
        let tag = document.default_single_line_container_name();
        // Step 11.2. Block-extend the active range, and let new range be the result.
        let new_range = active_range.block_extend(cx, document);
        // Step 11.4. Append to node list the first node in tree order that is contained in new range and is an allowed child of "p", if any.
        let mut node_list = if let Some(eligible_node) = new_range
            .contained_children()
            .ok()
            .and_then(|contained_children| {
                contained_children
                    .contained_children
                    .into_iter()
                    .find(|node| {
                        is_allowed_child(
                            NodeOrString::Node(node.clone()),
                            NodeOrString::String("p".to_owned()),
                        )
                    })
            }) {
            vec![eligible_node]
        } else {
            // Step 11.3. Let node list be a list of nodes, initially empty.
            // Step 11.5. If node list is empty:
            // Step 11.5.1. If tag is not an allowed child of the active range's start node, return true.
            if !is_allowed_child(
                NodeOrString::String(tag.str().to_owned()),
                NodeOrString::Node(active_range.start_container()),
            ) {
                return true;
            }
            // Step 11.5.2. Set container to the result of calling createElement(tag) on the context object.
            let container = document.create_element(cx, tag.str());
            let container = container.upcast::<Node>();
            // Step 11.5.3. Call insertNode(container) on the active range.
            if active_range.InsertNode(cx, container).is_err() {
                unreachable!("Must always be able to insert");
            }
            // Step 11.5.4. Call createElement("br") on the context object,
            // and append the result as the last child of container.
            let br = document.create_element(cx, "br");
            if container.AppendChild(cx, br.upcast()).is_err() {
                unreachable!("Must always be able to append");
            }
            // Step 11.5.5. Call collapse(container, 0) on the context object's selection.
            selection.collapse_current_range(container, 0);
            // Step 11.5.6. Return true.
            return true;
        };
        // Step 11.6. While the nextSibling of the last member of node list is not null
        // and is an allowed child of "p", append it to node list.
        while let Some(next_of_last) = node_list
            .iter()
            .last()
            .and_then(|node| node.GetNextSibling())
            .filter(|next_of_last| {
                is_allowed_child(
                    NodeOrString::Node(DomRoot::from_ref(next_of_last)),
                    NodeOrString::String("p".to_owned()),
                )
            })
        {
            node_list.push(next_of_last);
        }
        // Step 11.7. Wrap node list, with sibling criteria returning false
        // and new parent instructions returning the result of calling createElement(tag) on the context object.
        // Set container to the result.
        container = wrap_node_list(
            cx,
            node_list,
            |_| false,
            |cx| Some(DomRoot::upcast(document.create_element(cx, tag.str()))),
        )
        .expect("Must always be able to wrap");
    }
    // Step 12. If container's local name is "address", "listing", or "pre":
    if node_matches_local_name!(
        container,
        local_name!("address") | local_name!("listing") | local_name!("pre")
    ) {
        // Step 12.1. Let br be the result of calling createElement("br") on the context object.
        let br = document.create_element(cx, "br");
        // Step 12.2. Call insertNode(br) on the active range.
        if active_range.InsertNode(cx, br.upcast()).is_err() {
            unreachable!("Must always be able to insert");
        }
        // Step 12.3. Call collapse(node, offset + 1) on the context object's selection.
        selection.collapse_current_range(&node, offset + 1);
        // Step 12.4. If br is the last descendant of container,
        // let br be the result of calling createElement("br") on the context object,
        // then call insertNode(br) on the active range.
        if container
            .children()
            .last()
            .is_some_and(|child| *child == *br.upcast())
        {
            let br = document.create_element(cx, "br");
            if active_range.InsertNode(cx, br.upcast()).is_err() {
                unreachable!("Must always be able to insert");
            }
        }
        // Step 12.5. Return true.
        return true;
    }
    // Step 13. If container's local name is "li", "dt", or "dd";
    // and either it has no children or it has a single child and that child is a br:
    if node_matches_local_name!(
        container,
        local_name!("li") | local_name!("dt") | local_name!("dd")
    ) && (container.children_count() == 0
        || (container.children_count() == 1
            && container
                .children()
                .next()
                .expect("has one child")
                .is::<HTMLBRElement>()))
    {
        // Step 13.1. Split the parent of the one-node list consisting of container.
        split_the_parent(cx, &[&container]);
        // Step 13.2. If container has no children,
        // call createElement("br") on the context object and append the result as the last child of container.
        if container.children_count() == 0 {
            let br = document.create_element(cx, "br");
            if container.AppendChild(cx, br.upcast()).is_err() {
                unreachable!("Must always be able to append");
            }
        }
        // Step 13.3. If container is a dd or dt,
        // and it is not an allowed child of any of its ancestors in the same editing host,
        // set the tag name of container to the default single-line container name and let container be the result.
        if node_matches_local_name!(container, local_name!("dd") | local_name!("dt"))
            && container.is_no_allowed_child_in_same_editing_host()
        {
            container = container
                .downcast::<Element>()
                .expect("Must always be an element")
                .set_the_tag_name(cx, document.default_single_line_container_name().str());
        }
        // Step 13.4. Fix disallowed ancestors of container.
        container.fix_disallowed_ancestors(cx, document);
        // Step 13.5. Return true.
        return true;
    }
    // Step 14. Let new line range be a new range whose start is the same as the active range's,
    // and whose end is (container, length of container).
    let new_line_range = document.CreateRange(cx);
    new_line_range.set_start(&active_range.start_container(), active_range.start_offset());
    new_line_range.set_end(&container, container.len());
    // Step 15. While new line range's start offset is zero and its start node
    // is not a prohibited paragraph child,
    // set its start to (parent of start node, index of start node).
    while new_line_range.start_offset() == 0
        && !new_line_range
            .start_container()
            .is_prohibited_paragraph_child()
    {
        let start = new_line_range.start_container();
        new_line_range.set_start(
            &start.GetParentNode().expect("Must always have a parent"),
            start.index(),
        );
    }
    // Step 16. While new line range's start offset is the length of its start node
    // and its start node is not a prohibited paragraph child,
    // set its start to (parent of start node, 1 + index of start node).
    while new_line_range.start_offset() == new_line_range.start_container().len()
        && !new_line_range
            .start_container()
            .is_prohibited_paragraph_child()
    {
        let start = new_line_range.start_container();
        new_line_range.set_start(
            &start.GetParentNode().expect("Must always have a parent"),
            1 + start.index(),
        );
    }
    // Step 17. Let end of line be true if new line range contains either nothing or a single br, and false otherwise.
    let end_of_line = new_line_range
        .contained_children()
        .is_ok_and(|contained_children| {
            let contained_children = contained_children.contained_children;
            contained_children.is_empty()
                || (contained_children.len() == 1 && contained_children[0].is::<HTMLBRElement>())
        });
    // Step 18. If the local name of container is "h1", "h2", "h3", "h4", "h5", or "h6",
    // and end of line is true, let new container name be the default single-line container name.
    let container_as_element = container
        .downcast::<Element>()
        .expect("Must always be an element");
    let container_name = container_as_element.local_name();
    let new_container_name = if end_of_line
        && matches!(
            *container_name,
            local_name!("h1")
                | local_name!("h2")
                | local_name!("h3")
                | local_name!("h4")
                | local_name!("h5")
                | local_name!("h6")
        ) {
        document
            .default_single_line_container_name()
            .str()
            .to_owned()
    } else
    // Step 19. Otherwise, if the local name of container is "dt" and end of line is true, let new container name be "dd".
    if end_of_line && container_name == &local_name!("dt") {
        "dd".to_owned()
    } else
    // Step 20. Otherwise, if the local name of container is "dd" and end of line is true, let new container name be "dt".
    if end_of_line && container_name == &local_name!("dd") {
        "dt".to_owned()
    } else {
        // Step 21. Otherwise, let new container name be the local name of container.
        container_name.to_string()
    };
    // Step 22. Let new container be the result of calling createElement(new container name) on the context object.
    let new_container = document.create_element(cx, &new_container_name);
    // Step 23. Copy all attributes of container to new container.
    container_as_element.copy_all_attributes_to_other_element(cx, &new_container);
    // Step 24. If new container has an id attribute, unset it.
    new_container.remove_attribute_by_name(cx, &local_name!("id"));
    // Step 25. Insert new container into the parent of container immediately after container.
    let new_container_node = DomRoot::upcast(new_container);
    if container
        .GetParentNode()
        .expect("Must always have a parent")
        .InsertBefore(
            cx,
            &new_container_node,
            container.GetNextSibling().as_deref(),
        )
        .is_err()
    {
        unreachable!("Must always be able to insert");
    }
    // Step 26. Let contained nodes be all nodes contained in new line range.
    let Ok(contained_nodes) = new_line_range.contained_children() else {
        unreachable!("Must always have contained children");
    };
    // Step 27. Let frag be the result of calling extractContents() on new line range.
    let Ok(frag) = new_line_range.ExtractContents(cx) else {
        unreachable!("Must always be able to extract");
    };
    let frag_as_node = frag.upcast::<Node>();
    // Step 28. Unset the id attribute (if any) of each Element descendant of frag
    // that is not in contained nodes.
    for descendant in frag_as_node.traverse_preorder(ShadowIncluding::No) {
        if !contained_nodes.contained_children.contains(&descendant)
            && let Some(descendant) = descendant.downcast::<Element>()
        {
            descendant.remove_attribute_by_name(cx, &local_name!("id"));
        }
    }
    // Step 29. Call appendChild(frag) on new container.
    if new_container_node.AppendChild(cx, frag_as_node).is_err() {
        unreachable!("Must always be able to append");
    }
    // Step 30. While container's lastChild is a prohibited paragraph child,
    // set container to its lastChild.
    loop {
        let Some(last_child) = container.children().last() else {
            break;
        };
        if !last_child.is_prohibited_paragraph_child() {
            break;
        }
        container = last_child;
    }
    // Step 31. While new container's lastChild is a prohibited paragraph child,
    // set new container to its lastChild.
    let mut new_container_node = new_container_node;
    loop {
        let Some(last_child) = new_container_node.children().last() else {
            break;
        };
        if !last_child.is_prohibited_paragraph_child() {
            break;
        }
        new_container_node = last_child;
    }
    // Step 32. If container has no visible children,
    // call createElement("br") on the context object,
    // and append the result as the last child of container.
    if container
        .children()
        .all(|child| child.is_invisible(cx.no_gc()))
    {
        let br = document.create_element(cx, "br");
        if container.AppendChild(cx, br.upcast()).is_err() {
            unreachable!("Must always be able to append");
        }
    }
    // Step 33. If new container has no visible children,
    // call createElement("br") on the context object,
    // and append the result as the last child of new container.
    if new_container_node
        .children()
        .all(|child| child.is_invisible(cx.no_gc()))
    {
        let br = document.create_element(cx, "br");
        if new_container_node.AppendChild(cx, br.upcast()).is_err() {
            unreachable!("Must always be able to append");
        }
    }
    // Step 34. Call collapse(new container, 0) on the context object's selection.
    selection.collapse_current_range(&new_container_node, 0);
    // Step 35. Return true.
    true
}
