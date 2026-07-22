/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::execcommand::contenteditable::node::{move_preserving_ranges, split_the_parent};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::selection::Selection;
use crate::dom::text::Text;

/// <https://w3c.github.io/editing/docs/execCommand/#removeformat-candidate>
fn is_remove_format_candidate(element: &Element) -> bool {
    // > A removeFormat candidate is an editable HTML element with local name
    // > "abbr", "acronym", "b", "bdi", "bdo", "big", "blink", "cite", "code",
    // > "dfn", "em", "font", "i", "ins", "kbd", "mark", "nobr", "q", "s",
    // > "samp", "small", "span", "strike", "strong", "sub", "sup", "tt", "u", or "var".
    matches!(
        *element.local_name(),
        local_name!("abbr") |
            local_name!("acronym") |
            local_name!("b") |
            local_name!("bdi") |
            local_name!("bdo") |
            local_name!("big") |
            local_name!("blink") |
            local_name!("cite") |
            local_name!("code") |
            local_name!("dfn") |
            local_name!("em") |
            local_name!("font") |
            local_name!("i") |
            local_name!("ins") |
            local_name!("kbd") |
            local_name!("mark") |
            local_name!("nobr") |
            local_name!("q") |
            local_name!("s") |
            local_name!("samp") |
            local_name!("small") |
            local_name!("span") |
            local_name!("strike") |
            local_name!("strong") |
            local_name!("sub") |
            local_name!("sup") |
            local_name!("tt") |
            local_name!("u") |
            local_name!("var")
    )
}

/// <https://w3c.github.io/editing/docs/execCommand/#the-removeformat-command>
pub(crate) fn execute_removeformat_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
) -> bool {
    // Step 1. Let elements to remove be a list of every removeFormat candidate effectively contained in the active range.
    let active_range = selection
        .active_range()
        .expect("Must always have an active range");
    let mut elements = vec![];
    active_range.for_each_effectively_contained_child(|node| {
        if let Some(html_element) = node.downcast::<HTMLElement>() &&
            is_remove_format_candidate(html_element.upcast())
        {
            elements.push(DomRoot::from_ref(node));
        }
    });
    // Step 2. For each element in elements to remove:
    for element in elements {
        // Step 2.1. While element has children,
        // insert the first child of element into the parent of element immediately before element, preserving ranges.
        let parent_element = element.GetParentNode().expect("Must always have a parent");
        for child in element.children() {
            move_preserving_ranges(cx, &child, |cx| {
                parent_element.InsertBefore(cx, &child, Some(&element))
            });
        }
        // Step 2.2. Remove element from its parent.
        element.remove_self(cx);
    }
    // Step 3. If the active range's start node is an editable Text node,
    // and its start offset is neither zero nor its start node's length,
    // call splitText() on the active range's start node, with argument equal
    // to the active range's start offset. Then set the active range's start node
    // to the result, and its start offset to zero.
    let start_node = active_range.start_container();
    let start_offset = active_range.start_offset();
    if start_node.is_editable() &&
        start_offset != 0 &&
        start_offset != start_node.len() &&
        let Some(start_text) = start_node.downcast::<Text>()
    {
        let Ok(start_text) = start_text.SplitText(cx, start_offset) else {
            unreachable!("Must always be able to split");
        };
        active_range.set_start(start_text.upcast(), 0);
    }
    // Step 4. If the active range's end node is an editable Text node,
    // and its end offset is neither zero nor its end node's length,
    // call splitText() on the active range's end node, with argument
    // equal to the active range's end offset.
    let end_node = active_range.end_container();
    let end_offset = active_range.end_offset();
    if end_node.is_editable() &&
        end_offset != 0 &&
        end_offset != end_node.len() &&
        let Some(end_text) = end_node.downcast::<Text>() &&
        end_text.SplitText(cx, end_offset).is_err()
    {
        unreachable!("Must always be able to split");
    };
    // Step 5. Let node list consist of all editable nodes effectively contained in the active range.
    let mut node_list = vec![];
    active_range.for_each_effectively_contained_child(|node| {
        if node.is_editable() {
            node_list.push(DomRoot::from_ref(node));
        }
    });
    // Step 6. For each node in node list, while node's parent is a removeFormat
    // candidate in the same editing host as node,
    // split the parent of the one-node list consisting of node.
    for node in node_list {
        while let Some(parent) = node.GetParentElement() {
            if !is_remove_format_candidate(&parent) {
                break;
            }
            if !node.same_editing_host(parent.upcast()) {
                break;
            }
            split_the_parent(cx, &[&node]);
        }
    }
    // Step 7. For each of the entries in the following list,
    // in the given order, set the selection's value to null, with command as given.
    selection.set_the_selection_value(cx, None, CommandName::Subscript, document);
    selection.set_the_selection_value(cx, None, CommandName::Bold, document);
    selection.set_the_selection_value(cx, None, CommandName::FontName, document);
    selection.set_the_selection_value(cx, None, CommandName::FontSize, document);
    selection.set_the_selection_value(cx, None, CommandName::ForeColor, document);
    selection.set_the_selection_value(cx, None, CommandName::HiliteColor, document);
    selection.set_the_selection_value(cx, None, CommandName::Italic, document);
    selection.set_the_selection_value(cx, None, CommandName::Strikethrough, document);
    selection.set_the_selection_value(cx, None, CommandName::Underline, document);
    // Step 8. Return true.
    true
}
