/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;
use std::ops::Deref;

use html5ever::local_name;
use js::context::JSContext;
use script_bindings::inheritance::Castable;
use style::values::specified::box_::DisplayOutside;

use crate::dom::abstractrange::bp_position;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::NodeTypeId;
use crate::dom::bindings::root::{DomRoot, DomSlice};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::{CommandName, CssPropertyName};
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmllielement::HTMLLIElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::text::Text;
use crate::script_runtime::CanGc;

pub(crate) enum NodeOrString {
    String(String),
    Node(DomRoot<Node>),
}

impl NodeOrString {
    fn name(&self) -> &str {
        match self {
            NodeOrString::String(str_) => str_,
            NodeOrString::Node(node) => node
                .downcast::<Element>()
                .map(|element| element.local_name().as_ref())
                .unwrap_or_default(),
        }
    }

    fn as_node(&self) -> Option<DomRoot<Node>> {
        match self {
            NodeOrString::String(_) => None,
            NodeOrString::Node(node) => Some(node.clone()),
        }
    }
}

/// <https://w3c.github.io/editing/docs/execCommand/#prohibited-paragraph-child-name>
const PROHIBITED_PARAGRAPH_CHILD_NAMES: [&str; 47] = [
    "address",
    "article",
    "aside",
    "blockquote",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "hr",
    "li",
    "listing",
    "menu",
    "nav",
    "ol",
    "p",
    "plaintext",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
    "xmp",
];
/// <https://w3c.github.io/editing/docs/execCommand/#name-of-an-element-with-inline-contents>
const NAME_OF_AN_ELEMENT_WITH_INLINE_CONTENTS: [&str; 43] = [
    "a", "abbr", "b", "bdi", "bdo", "cite", "code", "dfn", "em", "h1", "h2", "h3", "h4", "h5",
    "h6", "i", "kbd", "mark", "p", "pre", "q", "rp", "rt", "ruby", "s", "samp", "small", "span",
    "strong", "sub", "sup", "u", "var", "acronym", "listing", "strike", "xmp", "big", "blink",
    "font", "marquee", "nobr", "tt",
];

/// <https://w3c.github.io/editing/docs/execCommand/#element-with-inline-contents>
fn is_element_with_inline_contents(element: &Node) -> bool {
    // > An element with inline contents is an HTML element whose local name is a name of an element with inline contents.
    let Some(html_element) = element.downcast::<HTMLElement>() else {
        return false;
    };
    NAME_OF_AN_ELEMENT_WITH_INLINE_CONTENTS.contains(&html_element.local_name())
}

/// <https://w3c.github.io/editing/docs/execCommand/#preserving-ranges>
pub(crate) fn move_preserving_ranges<Move>(cx: &mut JSContext, node: &Node, mut move_: Move)
where
    Move: FnMut(&mut JSContext) -> Fallible<DomRoot<Node>>,
{
    // Step 1. Let node be the moved node, old parent and old index be the old parent
    // (which may be null) and index, and new parent and new index be the new parent and index.
    let old_parent = node.GetParentNode();
    let old_index = node.index();

    if move_(cx).is_err() {
        unreachable!("Must always be able to move");
    }

    let Some(selection) = node.owner_document().GetSelection(CanGc::from_cx(cx)) else {
        return;
    };
    let Some(active_range) = selection.active_range() else {
        return;
    };

    let new_parent = node.GetParentNode().expect("Must always have a new parent");
    let new_index = node.index();

    let mut start_node = active_range.start_container();
    let mut start_offset = active_range.start_offset();
    let mut end_node = active_range.end_container();
    let mut end_offset = active_range.end_offset();

    // Step 2. If a boundary point's node is the same as or a descendant of node, leave it unchanged, so it moves to the new location.
    //
    // From the spec:
    // > This is actually implicit, but I state it anyway for completeness.

    // Step 3. If a boundary point's node is new parent and its offset is greater than new index, add one to its offset.
    if start_node == new_parent && start_offset > new_index {
        start_offset += 1;
    }
    if end_node == new_parent && end_offset > new_index {
        end_offset += 1;
    }

    if let Some(old_parent) = old_parent {
        // Step 4. If a boundary point's node is old parent and its offset is old index or old index + 1,
        // set its node to new parent and add new index − old index to its offset.
        if start_node == old_parent && (start_offset == old_index || start_offset == old_index + 1)
        {
            start_node = new_parent.clone();
            start_offset += new_index;
            start_offset -= old_index;
        }
        if end_node == old_parent && (end_offset == old_index || end_offset == old_index + 1) {
            end_node = new_parent;
            end_offset += new_index;
            end_offset -= old_index;
        }

        // Step 5. If a boundary point's node is old parent and its offset is greater than old index + 1,
        // subtract one from its offset.
        if start_node == old_parent && (start_offset > old_index + 1) {
            start_offset -= 1;
        }
        if end_node == old_parent && (end_offset > old_index + 1) {
            end_offset -= 1;
        }
    }

    active_range.set_start(&start_node, start_offset);
    active_range.set_end(&end_node, end_offset);
}

/// <https://w3c.github.io/editing/docs/execCommand/#allowed-child>
pub(crate) fn is_allowed_child(child: NodeOrString, parent: NodeOrString) -> bool {
    // Step 1. If parent is "colgroup", "table", "tbody", "tfoot", "thead", "tr",
    // or an HTML element with local name equal to one of those,
    // and child is a Text node whose data does not consist solely of space characters, return false.
    if matches!(
        parent.name(),
        "colgroup" | "table" | "tbody" | "tfoot" | "thead" | "tr"
    ) && child.as_node().is_some_and(|node| {
        // Note: cannot use `.and_then` here, since `downcast` would outlive its reference
        node.downcast::<Text>()
            .is_some_and(|text| !text.data().bytes().all(|byte| byte == b' '))
    }) {
        return false;
    }
    // Step 2. If parent is "script", "style", "plaintext", or "xmp",
    // or an HTML element with local name equal to one of those, and child is not a Text node, return false.
    if matches!(parent.name(), "script" | "style" | "plaintext" | "xmp") &&
        child.as_node().is_none_or(|node| !node.is::<Text>())
    {
        return false;
    }
    // Step 3. If child is a document, DocumentFragment, or DocumentType, return false.
    if let NodeOrString::Node(ref node) = child {
        if matches!(
            node.type_id(),
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) | NodeTypeId::DocumentType
        ) {
            return false;
        }
    }
    // Step 4. If child is an HTML element, set child to the local name of child.
    let child_name = match child {
        NodeOrString::String(str_) => str_,
        NodeOrString::Node(node) => match node.downcast::<HTMLElement>() {
            // Step 5. If child is not a string, return true.
            None => return true,
            Some(html_element) => html_element.local_name().to_owned(),
        },
    };
    let child = child_name.as_str();
    let parent_name = match parent {
        NodeOrString::String(str_) => str_,
        NodeOrString::Node(parent) => {
            // Step 6. If parent is an HTML element:
            if let Some(parent_element) = parent.downcast::<HTMLElement>() {
                // Step 6.1. If child is "a", and parent or some ancestor of parent is an a, return false.
                if child == "a" &&
                    parent
                        .inclusive_ancestors(ShadowIncluding::No)
                        .any(|node| node.is::<HTMLAnchorElement>())
                {
                    return false;
                }
                // Step 6.2. If child is a prohibited paragraph child name and parent or some ancestor of parent
                // is an element with inline contents, return false.
                if PROHIBITED_PARAGRAPH_CHILD_NAMES.contains(&child) &&
                    parent
                        .inclusive_ancestors(ShadowIncluding::No)
                        .any(|node| is_element_with_inline_contents(&node))
                {
                    return false;
                }
                // Step 6.3. If child is "h1", "h2", "h3", "h4", "h5", or "h6",
                // and parent or some ancestor of parent is an HTML element with local name
                // "h1", "h2", "h3", "h4", "h5", or "h6", return false.
                if matches!(child, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") &&
                    parent.inclusive_ancestors(ShadowIncluding::No).any(|node| {
                        node.downcast::<HTMLElement>().is_some_and(|html_element| {
                            matches!(
                                html_element.local_name(),
                                "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                            )
                        })
                    })
                {
                    return false;
                }
                // Step 6.4. Let parent be the local name of parent.
                parent_element.local_name().to_owned()
            } else {
                // Step 7. If parent is an Element or DocumentFragment, return true.
                // Step 8. If parent is not a string, return false.
                return matches!(
                    parent.type_id(),
                    NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(_)
                );
            }
        },
    };
    let parent = parent_name.as_str();
    // Step 9. If parent is on the left-hand side of an entry on the following list,
    // then return true if child is listed on the right-hand side of that entry, and false otherwise.
    match parent {
        "colgroup" => return child == "col",
        "table" => {
            return matches!(
                child,
                "caption" | "col" | "colgroup" | "tbody" | "td" | "tfoot" | "th" | "thead" | "tr"
            );
        },
        "tbody" | "tfoot" | "thead" => return matches!(child, "td" | "th" | "tr"),
        "tr" => return matches!(child, "td" | "th"),
        "dl" => return matches!(child, "dt" | "dd"),
        "dir" | "ol" | "ul" => return matches!(child, "dir" | "li" | "ol" | "ul"),
        "hgroup" => return matches!(child, "h1" | "h2" | "h3" | "h4" | "h5" | "h6"),
        _ => {},
    };
    // Step 10. If child is "body", "caption", "col", "colgroup", "frame", "frameset", "head",
    // "html", "tbody", "td", "tfoot", "th", "thead", or "tr", return false.
    if matches!(
        child,
        "body" |
            "caption" |
            "col" |
            "colgroup" |
            "frame" |
            "frameset" |
            "head" |
            "html" |
            "tbody" |
            "td" |
            "tfoot" |
            "th" |
            "thead" |
            "tr"
    ) {
        return false;
    }
    // Step 11. If child is "dd" or "dt" and parent is not "dl", return false.
    if matches!(child, "dd" | "dt") && parent != "dl" {
        return false;
    }
    // Step 12. If child is "li" and parent is not "ol" or "ul", return false.
    if child == "li" && !matches!(parent, "ol" | "ul") {
        return false;
    }
    // Step 13. If parent is on the left-hand side of an entry on the following list
    // and child is listed on the right-hand side of that entry, return false.
    if match parent {
        "a" => child == "a",
        "dd" | "dt" => matches!(child, "dd" | "dt"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            matches!(child, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
        },
        "li" => child == "li",
        "nobr" => child == "nobr",
        "td" | "th" => {
            matches!(
                child,
                "caption" | "col" | "colgroup" | "tbody" | "td" | "tfoot" | "th" | "thead" | "tr"
            )
        },
        _ if NAME_OF_AN_ELEMENT_WITH_INLINE_CONTENTS.contains(&parent) => {
            PROHIBITED_PARAGRAPH_CHILD_NAMES.contains(&child)
        },
        _ => false,
    } {
        return false;
    }
    // Step 14. Return true.
    true
}

/// <https://w3c.github.io/editing/docs/execCommand/#split-the-parent>
pub(crate) fn split_the_parent<'a>(cx: &mut JSContext, node_list: &'a [&'a Node]) {
    assert!(!node_list.is_empty());
    // Step 1. Let original parent be the parent of the first member of node list.
    let Some(original_parent) = node_list.first().and_then(|first| first.GetParentNode()) else {
        return;
    };
    let context_object = original_parent.owner_document();
    // Step 2. If original parent is not editable or its parent is null, do nothing and abort these steps.
    if !original_parent.is_editable() {
        return;
    }
    let Some(parent_of_original_parent) = original_parent.GetParentNode() else {
        return;
    };
    // Step 3. If the first child of original parent is in node list, remove extraneous line breaks before original parent.
    if original_parent
        .children()
        .next()
        .is_some_and(|first_child| node_list.contains(&first_child.deref()))
    {
        original_parent.remove_extraneous_line_breaks_before(cx);
    }
    // Step 4. If the first child of original parent is in node list, and original parent follows a line break,
    // set follows line break to true. Otherwise, set follows line break to false.
    let first_child_is_in_node_list = original_parent
        .children()
        .next()
        .is_some_and(|first_child| node_list.contains(&first_child.deref()));
    let follows_line_break = first_child_is_in_node_list && original_parent.follows_a_line_break();
    // Step 5. If the last child of original parent is in node list, and original parent precedes a line break,
    // set precedes line break to true. Otherwise, set precedes line break to false.
    let last_child_is_in_node_list = original_parent
        .children()
        .last()
        .is_some_and(|last_child| node_list.contains(&last_child.deref()));
    let precedes_line_break = last_child_is_in_node_list && original_parent.precedes_a_line_break();
    // Step 6. If the first child of original parent is not in node list, but its last child is:
    if !first_child_is_in_node_list && last_child_is_in_node_list {
        // Step 6.1. For each node in node list, in reverse order,
        // insert node into the parent of original parent immediately after original parent, preserving ranges.
        let next_of_original_parent = original_parent.GetNextSibling();
        for node in node_list.iter().rev() {
            move_preserving_ranges(cx, node, |cx| {
                parent_of_original_parent.InsertBefore(cx, node, next_of_original_parent.as_deref())
            });
        }
        // Step 6.2. If precedes line break is true, and the last member of node list does not precede a line break,
        // call createElement("br") on the context object and insert the result immediately after the last member of node list.
        if precedes_line_break {
            if let Some(last) = node_list.last() {
                if !last.precedes_a_line_break() {
                    let br = context_object.create_element(cx, "br");
                    if last
                        .GetParentNode()
                        .expect("Must always have a parent")
                        .InsertBefore(cx, br.upcast(), last.GetNextSibling().as_deref())
                        .is_err()
                    {
                        unreachable!("Must always be able to append");
                    }
                }
            }
        }
        // Step 6.3. Remove extraneous line breaks at the end of original parent.
        original_parent.remove_extraneous_line_breaks_at_the_end_of(cx);
        // Step 6.4. Abort these steps.
        return;
    }
    // Step 7. If the first child of original parent is not in node list:
    if first_child_is_in_node_list {
        // Step 7.1. Let cloned parent be the result of calling cloneNode(false) on original parent.
        let Ok(cloned_parent) = original_parent.CloneNode(cx, false) else {
            unreachable!("Must always be able to clone node");
        };
        // Step 7.2. If original parent has an id attribute, unset it.
        if let Some(element) = original_parent.downcast::<Element>() {
            element.remove_attribute_by_name(&local_name!("id"), CanGc::from_cx(cx));
        }
        // Step 7.3. Insert cloned parent into the parent of original parent immediately before original parent.
        if parent_of_original_parent
            .InsertBefore(cx, &cloned_parent, Some(&original_parent))
            .is_err()
        {
            unreachable!("Must always have a parent");
        }
        // Step 7.4. While the previousSibling of the first member of node list is not null,
        // append the first child of original parent as the last child of cloned parent, preserving ranges.
        loop {
            if node_list
                .first()
                .and_then(|first| first.GetPreviousSibling())
                .is_some()
            {
                if let Some(first_of_original) = original_parent.children().next() {
                    move_preserving_ranges(cx, &first_of_original, |cx| {
                        cloned_parent.AppendChild(cx, &first_of_original)
                    });
                    continue;
                }
            }
            break;
        }
    }
    // Step 8. For each node in node list, insert node into the parent of original parent immediately before original parent, preserving ranges.
    for node in node_list.iter() {
        move_preserving_ranges(cx, node, |cx| {
            parent_of_original_parent.InsertBefore(cx, node, Some(&original_parent))
        });
    }
    // Step 9. If follows line break is true, and the first member of node list does not follow a line break,
    // call createElement("br") on the context object and insert the result immediately before the first member of node list.
    if follows_line_break {
        if let Some(first) = node_list.first() {
            if !first.follows_a_line_break() {
                let br = context_object.create_element(cx, "br");
                if first
                    .GetParentNode()
                    .expect("Must always have a parent")
                    .InsertBefore(cx, br.upcast(), Some(first))
                    .is_err()
                {
                    unreachable!("Must always be able to insert");
                }
            }
        }
    }
    // Step 10. If the last member of node list is an inline node other than a br,
    // and the first child of original parent is a br, and original parent is not an inline node,
    // remove the first child of original parent from original parent.
    if node_list
        .last()
        .is_some_and(|last| last.is_inline_node() && !last.is::<HTMLBRElement>()) &&
        !original_parent.is_inline_node()
    {
        if let Some(first_of_original) = original_parent.children().next() {
            if first_of_original.is::<HTMLBRElement>() {
                assert!(first_of_original.has_parent());
                first_of_original.remove_self(cx);
            }
        }
    }
    // Step 11. If original parent has no children:
    if original_parent.children_count() == 0 {
        // Step 11.1. Remove original parent from its parent.
        assert!(original_parent.has_parent());
        original_parent.remove_self(cx);
        // Step 11.2. If precedes line break is true, and the last member of node list does not precede a line break,
        // call createElement("br") on the context object and insert the result immediately after the last member of node list.
        if precedes_line_break {
            if let Some(last) = node_list.last() {
                if !last.precedes_a_line_break() {
                    let br = context_object.create_element(cx, "br");
                    if last
                        .GetParentNode()
                        .expect("Must always have a parent")
                        .InsertBefore(cx, br.upcast(), last.GetNextSibling().as_deref())
                        .is_err()
                    {
                        unreachable!("Must always be able to insert");
                    }
                }
            }
        }
    } else {
        // Step 12. Otherwise, remove extraneous line breaks before original parent.
        original_parent.remove_extraneous_line_breaks_before(cx);
    }
    // Step 13. If node list's last member's nextSibling is null, but its parent is not null,
    // remove extraneous line breaks at the end of node list's last member's parent.
    if let Some(last) = node_list.last() {
        if last.GetNextSibling().is_none() {
            if let Some(parent_of_last) = last.GetParentNode() {
                parent_of_last.remove_extraneous_line_breaks_at_the_end_of(cx);
            }
        }
    }
}

/// <https://w3c.github.io/editing/docs/execCommand/#wrap>
fn wrap_node_list<SiblingCriteria, NewParentInstructions>(
    cx: &mut JSContext,
    node_list: Vec<DomRoot<Node>>,
    sibling_criteria: SiblingCriteria,
    new_parent_instructions: NewParentInstructions,
) -> Option<DomRoot<Node>>
where
    SiblingCriteria: Fn(&Node) -> bool,
    NewParentInstructions: Fn() -> Option<DomRoot<Node>>,
{
    // Step 1. If every member of node list is invisible,
    // and none is a br, return null and abort these steps.
    if node_list
        .iter()
        .all(|node| node.is_invisible() && !node.is::<HTMLBRElement>())
    {
        return None;
    }
    // Step 2. If node list's first member's parent is null, return null and abort these steps.
    node_list.first().and_then(|first| first.GetParentNode())?;
    // Step 3. If node list's last member is an inline node that's not a br,
    // and node list's last member's nextSibling is a br, append that br to node list.
    let mut node_list = node_list;
    if let Some(last) = node_list.last() {
        if last.is_inline_node() && !last.is::<HTMLBRElement>() {
            if let Some(next_of_last) = last.GetNextSibling() {
                if next_of_last.is::<HTMLBRElement>() {
                    node_list.push(next_of_last);
                }
            }
        }
    }
    // Step 4. While node list's first member's previousSibling is invisible, prepend it to node list.
    while let Some(previous_of_first) = node_list.first().and_then(|last| last.GetPreviousSibling())
    {
        if previous_of_first.is_invisible() {
            node_list.insert(0, previous_of_first);
            continue;
        }
        break;
    }
    // Step 5. While node list's last member's nextSibling is invisible, append it to node list.
    while let Some(next_of_last) = node_list.last().and_then(|last| last.GetNextSibling()) {
        if next_of_last.is_invisible() {
            node_list.push(next_of_last);
            continue;
        }
        break;
    }
    // Step 6. If the previousSibling of the first member of node list is editable
    // and running sibling criteria on it returns true,
    // let new parent be the previousSibling of the first member of node list.
    let new_parent = node_list
        .first()
        .and_then(|first| first.GetPreviousSibling())
        .filter(|previous_of_first| {
            previous_of_first.is_editable() && sibling_criteria(previous_of_first)
        });
    // Step 7. Otherwise, if the nextSibling of the last member of node list is editable
    // and running sibling criteria on it returns true,
    // let new parent be the nextSibling of the last member of node list.
    let new_parent = new_parent.or_else(|| {
        node_list
            .last()
            .and_then(|first| first.GetNextSibling())
            .filter(|next_of_last| next_of_last.is_editable() && sibling_criteria(next_of_last))
    });
    // Step 8. Otherwise, run new parent instructions, and let new parent be the result.
    // Step 9. If new parent is null, abort these steps and return null.
    let new_parent = new_parent.or_else(new_parent_instructions)?;
    // Step 11. Let original parent be the parent of the first member of node list.
    let first_in_node_list = node_list
        .first()
        .expect("Must always have at least one node");
    let original_parent = first_in_node_list
        .GetParentNode()
        .expect("First node must have a parent");
    // Step 10. If new parent's parent is null:
    if new_parent.GetParentNode().is_none() {
        // Step 10.1. Insert new parent into the parent of the first member
        // of node list immediately before the first member of node list.
        if original_parent
            .InsertBefore(cx, &new_parent, Some(first_in_node_list))
            .is_err()
        {
            unreachable!("Must always be able to insert");
        }
        // Step 10.2. If any range has a boundary point with node equal
        // to the parent of new parent and offset equal to the index of new parent,
        // add one to that boundary point's offset.
        if let Some(range) = first_in_node_list
            .owner_document()
            .GetSelection(CanGc::from_cx(cx))
            .and_then(|selection| selection.active_range())
        {
            let parent_of_new_parent = new_parent.GetParentNode().expect("Must have a parent");
            let start_container = range.start_container();
            let start_offset = range.start_offset();

            if start_container == parent_of_new_parent && start_offset == new_parent.index() {
                range.set_start(&start_container, start_offset + 1);
            }

            let end_container = range.end_container();
            let end_offset = range.end_offset();

            if end_container == parent_of_new_parent && end_offset == new_parent.index() {
                range.set_start(&end_container, end_offset + 1);
            }
        }
    }
    // Step 12. If new parent is before the first member of node list in tree order:
    if new_parent.is_before(first_in_node_list) {
        // Step 12.1. If new parent is not an inline node, but the last visible child of new parent
        // and the first visible member of node list are both inline nodes,
        // and the last child of new parent is not a br,
        // call createElement("br") on the ownerDocument of new parent
        // and append the result as the last child of new parent.
        // TODO
        // Step 12.2. For each node in node list, append node as the last child of new parent, preserving ranges.
        for node in node_list {
            move_preserving_ranges(cx, &node, |cx| new_parent.AppendChild(cx, &node));
        }
    } else {
        // Step 13. Otherwise:
        // Step 13.1. If new parent is not an inline node, but the first visible child of new parent
        // and the last visible member of node list are both inline nodes,
        // and the last member of node list is not a br,
        // call createElement("br") on the ownerDocument of new parent
        // and insert the result as the first child of new parent.
        // TODO
        // Step 13.2. For each node in node list, in reverse order,
        // insert node as the first child of new parent, preserving ranges.
        let mut before = new_parent.GetFirstChild();
        for node in node_list.iter().rev() {
            move_preserving_ranges(cx, node, |cx| {
                new_parent.InsertBefore(cx, node, before.as_deref())
            });
            before = Some(DomRoot::from_ref(node));
        }
    }
    // Step 14. If original parent is editable and has no children, remove it from its parent.
    if original_parent.is_editable() && original_parent.children_count() == 0 {
        original_parent.remove_self(cx);
    }
    // Step 15. If new parent's nextSibling is editable and running sibling criteria on it returns true:
    if let Some(next_of_new_parent) = new_parent.GetNextSibling() {
        if next_of_new_parent.is_editable() && sibling_criteria(&next_of_new_parent) {
            // Step 15.1. If new parent is not an inline node,
            // but new parent's last child and new parent's nextSibling's first child are both inline nodes,
            // and new parent's last child is not a br, call createElement("br") on the ownerDocument
            // of new parent and append the result as the last child of new parent.
            if !new_parent.is_inline_node() {
                if let Some(last_child_of_new_parent) = new_parent.children().last() {
                    if last_child_of_new_parent.is_inline_node() &&
                        !last_child_of_new_parent.is::<HTMLBRElement>() &&
                        next_of_new_parent
                            .children()
                            .next()
                            .is_some_and(|first| first.is_inline_node())
                    {
                        let new_br_element = new_parent.owner_document().create_element(cx, "br");
                        if new_parent.AppendChild(cx, new_br_element.upcast()).is_err() {
                            unreachable!("Must always be able to append");
                        }
                    }
                }
            }
            // Step 15.2. While new parent's nextSibling has children,
            // append its first child as the last child of new parent, preserving ranges.
            while let Some(first_of_next) = next_of_new_parent.children().next() {
                move_preserving_ranges(cx, &first_of_next, |cx| {
                    new_parent.AppendChild(cx, &first_of_next)
                });
            }
            // Step 15.3. Remove new parent's nextSibling from its parent.
            next_of_new_parent.remove_self(cx);
        }
    }
    // Step 16. Remove extraneous line breaks from new parent.
    new_parent.remove_extraneous_line_breaks_from(cx);
    // Step 17. Return new parent.
    Some(new_parent)
}

pub(crate) struct RecordedValueAndCommandOfNode {
    node: DomRoot<Node>,
    command: CommandName,
    specified_command_value: Option<DOMString>,
}

/// <https://w3c.github.io/editing/docs/execCommand/#record-the-values>
pub(crate) fn record_the_values(
    node_list: Vec<DomRoot<Node>>,
) -> Vec<RecordedValueAndCommandOfNode> {
    // Step 1. Let values be a list of (node, command, specified command value) triples, initially empty.
    let mut values = vec![];
    // Step 2. For each node in node list,
    // for each command in the list "subscript", "bold", "fontName", "fontSize", "foreColor",
    // "hiliteColor", "italic", "strikethrough", and "underline" in that order:
    for node in node_list {
        for command in vec![
            CommandName::Subscript,
            CommandName::Bold,
            CommandName::FontName,
            CommandName::FontSize,
            CommandName::ForeColor,
            CommandName::HiliteColor,
            CommandName::Italic,
            CommandName::Strikethrough,
            CommandName::Underline,
        ] {
            // Step 2.1. Let ancestor equal node.
            let mut ancestor =
                if let Some(node_element) = DomRoot::downcast::<Element>(node.clone()) {
                    Some(node_element)
                } else {
                    // Step 2.2. If ancestor is not an Element, set it to its parent.
                    node.GetParentElement()
                };
            // Step 2.3. While ancestor is an Element and its specified command value for command is null, set it to its parent.
            while let Some(ref ancestor_element) = ancestor {
                if ancestor_element.specified_command_value(&command).is_none() {
                    ancestor = ancestor_element.upcast::<Node>().GetParentElement();
                    continue;
                }
                break;
            }
            // Step 2.4. If ancestor is an Element,
            // add (node, command, ancestor's specified command value for command) to values.
            // Otherwise add (node, command, null) to values.
            let specified_command_value =
                ancestor.and_then(|ancestor| ancestor.specified_command_value(&command));
            values.push(RecordedValueAndCommandOfNode {
                node: node.clone(),
                command,
                specified_command_value,
            });
        }
    }
    // Step 3. Return values.
    values
}

/// <https://w3c.github.io/editing/docs/execCommand/#restore-the-values>
pub(crate) fn restore_the_values(cx: &mut JSContext, values: Vec<RecordedValueAndCommandOfNode>) {
    // Step 1. For each (node, command, value) triple in values:
    for triple in values {
        let RecordedValueAndCommandOfNode {
            node,
            command,
            specified_command_value,
        } = triple;
        // Step 1.1. Let ancestor equal node.
        let mut ancestor = if let Some(node_element) = DomRoot::downcast::<Element>(node.clone()) {
            Some(node_element)
        } else {
            // Step 1.2. If ancestor is not an Element, set it to its parent.
            node.GetParentElement()
        };
        // Step 1.3. While ancestor is an Element and its specified command value for command is null, set it to its parent.
        while let Some(ref ancestor_element) = ancestor {
            if ancestor_element.specified_command_value(&command).is_none() {
                ancestor = ancestor_element.upcast::<Node>().GetParentElement();
                continue;
            }
            break;
        }
        // Step 1.4. If value is null and ancestor is an Element,
        // push down values on node for command, with new value null.
        if specified_command_value.is_none() && ancestor.is_some() {
            node.push_down_values(cx, &command, None);
        } else {
            // Step 1.5. Otherwise, if ancestor is an Element and its specified command value for command is not equivalent to value,
            // or if ancestor is not an Element and value is not null, force the value of command to value on node.
            if match (ancestor, specified_command_value.as_ref()) {
                (Some(ancestor), value) => !command.are_equivalent_values(
                    ancestor.specified_command_value(&command).as_ref(),
                    value,
                ),
                (None, Some(_)) => true,
                _ => false,
            } {
                node.force_the_value(cx, &command, specified_command_value.as_ref());
            }
        }
    }
}

impl HTMLBRElement {
    /// <https://w3c.github.io/editing/docs/execCommand/#extraneous-line-break>
    fn is_extraneous_line_break(&self) -> bool {
        let node = self.upcast::<Node>();
        // > An extraneous line break is a br that has no visual effect, in that removing it from the DOM would not change layout,
        // except that a br that is the sole child of an li is not extraneous.
        if node
            .GetParentNode()
            .filter(|parent| parent.is::<HTMLLIElement>())
            .is_some_and(|li| li.children_count() == 1)
        {
            return false;
        }
        // TODO: Figure out what this actually makes it have no visual effect
        !node.is_block_node()
    }
}

impl Node {
    /// <https://w3c.github.io/editing/docs/execCommand/#push-down-values>
    pub(crate) fn push_down_values(
        &self,
        cx: &mut JSContext,
        command: &CommandName,
        new_value: Option<DOMString>,
    ) {
        // Step 1. Let command be the current command.
        //
        // Passed in as argument

        // Step 4. Let current ancestor be node's parent.
        let mut current_ancestor = self.GetParentElement();
        // Step 2. If node's parent is not an Element, abort this algorithm.
        if current_ancestor.is_none() {
            return;
        };
        // Step 3. If the effective command value of command is loosely equivalent to new value on node,
        // abort this algorithm.
        if command.are_loosely_equivalent_values(
            self.effective_command_value(command).as_ref(),
            new_value.as_ref(),
        ) {
            return;
        }
        // Step 5. Let ancestor list be a list of nodes, initially empty.
        rooted_vec!(let mut ancestor_list);
        // Step 6. While current ancestor is an editable Element and
        // the effective command value of command is not loosely equivalent to new value on it,
        // append current ancestor to ancestor list, then set current ancestor to its parent.
        while let Some(ancestor) = current_ancestor {
            let ancestor_node = ancestor.upcast::<Node>();
            if ancestor_node.is_editable() &&
                !command.are_loosely_equivalent_values(
                    ancestor_node.effective_command_value(command).as_ref(),
                    new_value.as_ref(),
                )
            {
                ancestor_list.push(ancestor.clone());
                current_ancestor = ancestor_node.GetParentElement();
                continue;
            }
            break;
        }
        let Some(last_ancestor) = ancestor_list.last() else {
            // Step 7. If ancestor list is empty, abort this algorithm.
            return;
        };
        // Step 8. Let propagated value be the specified command value of command on the last member of ancestor list.
        let mut propagated_value = last_ancestor.specified_command_value(command);
        // Step 9. If propagated value is null and is not equal to new value, abort this algorithm.
        if propagated_value.is_none() && new_value.is_some() {
            return;
        }
        // Step 10. If the effective command value of command is not loosely equivalent to new value on the parent
        // of the last member of ancestor list, and new value is not null, abort this algorithm.
        if new_value.is_some() &&
            !last_ancestor
                .upcast::<Node>()
                .GetParentNode()
                .is_some_and(|last_ancestor_parent| {
                    command.are_loosely_equivalent_values(
                        last_ancestor_parent
                            .effective_command_value(command)
                            .as_ref(),
                        new_value.as_ref(),
                    )
                })
        {
            return;
        }
        // Step 11. While ancestor list is not empty:
        let mut ancestor_list_iter = ancestor_list.iter().rev().peekable();
        while let Some(current_ancestor) = ancestor_list_iter.next() {
            let current_ancestor_node = current_ancestor.upcast::<Node>();
            // Step 11.1. Let current ancestor be the last member of ancestor list.
            // Step 11.2. Remove the last member from ancestor list.
            //
            // Both of these steps done by iterating and reversing the iterator

            // Step 11.3. If the specified command value of current ancestor for command is not null, set propagated value to that value.
            let command_value = current_ancestor.specified_command_value(command);
            let has_command_value = command_value.is_some();
            propagated_value = command_value.or(propagated_value);
            // Step 11.4. Let children be the children of current ancestor.
            let children = current_ancestor_node.children();
            // Step 11.5. If the specified command value of current ancestor for command is not null, clear the value of current ancestor.
            if has_command_value {
                if let Some(html_element) = current_ancestor.downcast::<HTMLElement>() {
                    html_element.clear_the_value(cx, command);
                }
            }
            // Step 11.6. For every child in children:
            for child in children {
                // Step 11.6.1. If child is node, continue with the next child.
                if *child == *self {
                    continue;
                }
                // Step 11.6.2. If child is an Element whose specified command value for command is neither null
                // nor equivalent to propagated value, continue with the next child.
                if let Some(child_element) = child.downcast::<Element>() {
                    let specified_value = child_element.specified_command_value(command);
                    if specified_value.is_some() &&
                        !command.are_equivalent_values(
                            specified_value.as_ref(),
                            propagated_value.as_ref(),
                        )
                    {
                        continue;
                    }
                }

                // Step 11.6.3. If child is the last member of ancestor list, continue with the next child.
                //
                // Since we had to remove the last member in step 11.2, if we now peek at the next possible
                // value, we essentially have the "last member after removal"
                if ancestor_list_iter
                    .peek()
                    .is_some_and(|ancestor| *ancestor.upcast::<Node>() == *child)
                {
                    continue;
                }
                // step 11.6.4. Force the value of child, with command as in this algorithm and new value equal to propagated value.
                child.force_the_value(cx, command, propagated_value.as_ref());
            }
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#force-the-value>
    pub(crate) fn force_the_value(
        &self,
        cx: &mut JSContext,
        command: &CommandName,
        new_value: Option<&DOMString>,
    ) {
        // Step 1. Let command be the current command.
        //
        // That's command

        // Step 2. If node's parent is null, abort this algorithm.
        if self.GetParentNode().is_none() {
            return;
        }
        // Step 3. If new value is null, abort this algorithm.
        let Some(new_value) = new_value else {
            return;
        };
        // Step 4. If node is an allowed child of "span":
        if is_allowed_child(
            NodeOrString::Node(DomRoot::from_ref(self)),
            NodeOrString::String("span".to_owned()),
        ) {
            // Step 4.1. Reorder modifiable descendants of node's previousSibling.
            // TODO
            // Step 4.2. Reorder modifiable descendants of node's nextSibling.
            // TODO
            // Step 4.3. Wrap the one-node list consisting of node,
            // with sibling criteria returning true for a simple modifiable element whose
            // specified command value is equivalent to new value and whose effective command value
            // is loosely equivalent to new value and false otherwise,
            // and with new parent instructions returning null.
            wrap_node_list(
                cx,
                vec![DomRoot::from_ref(self)],
                |sibling| {
                    sibling
                        .downcast::<Element>()
                        .is_some_and(|sibling_element| {
                            sibling_element.is_simple_modifiable_element() &&
                                command.are_equivalent_values(
                                    sibling_element.specified_command_value(command).as_ref(),
                                    Some(new_value),
                                ) &&
                                command.are_loosely_equivalent_values(
                                    sibling.effective_command_value(command).as_ref(),
                                    Some(new_value),
                                )
                        })
                },
                || None,
            );
        }
        // Step 5. If node is invisible, abort this algorithm.
        if self.is_invisible() {
            return;
        }
        // Step 6. If the effective command value of command is loosely equivalent to new value on node, abort this algorithm.
        if command.are_loosely_equivalent_values(
            self.effective_command_value(command).as_ref(),
            Some(new_value),
        ) {
            return;
        }
        // Step 7. If node is not an allowed child of "span":
        if !is_allowed_child(
            NodeOrString::Node(DomRoot::from_ref(self)),
            NodeOrString::String("span".to_owned()),
        ) {
            for child in self.children() {
                // Step 7.1. Let children be all children of node, omitting any that are Elements whose
                // specified command value for command is neither null nor equivalent to new value.
                if let Some(child_element) = child.downcast::<Element>() {
                    let specified_value = child_element.specified_command_value(command);
                    if specified_value.is_some() &&
                        !command.are_equivalent_values(specified_value.as_ref(), Some(new_value))
                    {
                        continue;
                    }
                }
                // Step 7.2. Force the value of each node in children,
                // with command and new value as in this invocation of the algorithm.
                child.force_the_value(cx, command, Some(new_value));
            }
            // Step 7.3. Abort this algorithm.
            return;
        }
        // Step 8. If the effective command value of command is loosely equivalent to new value on node, abort this algorithm.
        if command.are_loosely_equivalent_values(
            self.effective_command_value(command).as_ref(),
            Some(new_value),
        ) {
            return;
        }
        // Step 9. Let new parent be null.
        let mut new_parent = None;
        let document = self.owner_document();
        let css_styling_flag = document.css_styling_flag();
        // Step 10. If the CSS styling flag is false:
        if !css_styling_flag {
            match command {
                // Step 10.1. If command is "bold" and new value is "bold",
                // let new parent be the result of calling createElement("b") on the ownerDocument of node.
                CommandName::Bold => {
                    new_parent = Some(document.create_element(cx, "b"));
                },
                // Step 10.2. If command is "italic" and new value is "italic",
                // let new parent be the result of calling createElement("i") on the ownerDocument of node.
                CommandName::Italic => {
                    new_parent = Some(document.create_element(cx, "i"));
                },
                // Step 10.3. If command is "strikethrough" and new value is "line-through",
                // let new parent be the result of calling createElement("s") on the ownerDocument of node.
                CommandName::Strikethrough => {
                    new_parent = Some(document.create_element(cx, "s"));
                },
                // Step 10.4. If command is "underline" and new value is "underline",
                // let new parent be the result of calling createElement("u") on the ownerDocument of node.
                CommandName::Underline => {
                    new_parent = Some(document.create_element(cx, "u"));
                },
                // Step 10.5. If command is "foreColor", and new value is fully opaque with
                // red, green, and blue components in the range 0 to 255:
                CommandName::ForeColor => {
                    // TODO
                },
                // Step 10.6. If command is "fontName",
                // let new parent be the result of calling createElement("font") on the ownerDocument of node,
                // then set the face attribute of new parent to new value.
                CommandName::FontName => {
                    let new_font_element = document.create_element(cx, "font");
                    new_font_element.set_string_attribute(
                        &local_name!("face"),
                        new_value.clone(),
                        CanGc::from_cx(cx),
                    );
                    new_parent = Some(new_font_element);
                },
                _ => {},
            }
        }

        match command {
            // Step 11. If command is "createLink" or "unlink":
            // TODO
            // Step 12. If command is "fontSize"; and new value is one of
            // "x-small", "small", "medium", "large", "x-large", "xx-large", or "xxx-large";
            // and either the CSS styling flag is false, or new value is "xxx-large":
            // let new parent be the result of calling createElement("font") on the ownerDocument of node,
            // then set the size attribute of new parent to the number from the following table based on new value:
            CommandName::FontSize => {
                if !css_styling_flag || new_value == "xxx-large" {
                    let size = match &*new_value.str() {
                        "x-small" => 1,
                        "small" => 2,
                        "medium" => 3,
                        "large" => 4,
                        "x-large" => 5,
                        "xx-large" => 6,
                        "xxx-large" => 7,
                        _ => 0,
                    };

                    if size > 0 {
                        let new_font_element = document.create_element(cx, "font");
                        new_font_element.set_int_attribute(
                            &local_name!("size"),
                            size,
                            CanGc::from_cx(cx),
                        );
                        new_parent = Some(new_font_element);
                    }
                }
            },
            CommandName::Subscript | CommandName::Superscript => {
                // Step 13. If command is "subscript" or "superscript" and new value is "subscript",
                // let new parent be the result of calling createElement("sub") on the ownerDocument of node.
                if new_value == "subscript" {
                    new_parent = Some(document.create_element(cx, "sub"));
                }
                // Step 14. If command is "subscript" or "superscript" and new value is "superscript",
                // let new parent be the result of calling createElement("sup") on the ownerDocument of node.
                if new_value == "superscript" {
                    new_parent = Some(document.create_element(cx, "sup"));
                }
            },
            _ => {},
        }
        // Step 15. If new parent is null, let new parent be the result of calling createElement("span") on the ownerDocument of node.
        let new_parent = new_parent.unwrap_or_else(|| document.create_element(cx, "span"));
        let new_parent_html_element = new_parent
            .downcast::<HTMLElement>()
            .expect("Must always create a HTML element");
        // Step 16. Insert new parent in node's parent before node.
        if self
            .GetParentNode()
            .expect("Must always have a parent")
            .InsertBefore(cx, new_parent.upcast(), Some(self))
            .is_err()
        {
            unreachable!("Must always be able to insert");
        }
        // Step 17. If the effective command value of command for new parent is not loosely equivalent to new value,
        // and the relevant CSS property for command is not null,
        // set that CSS property of new parent to new value (if the new value would be valid).
        if !command.are_loosely_equivalent_values(
            new_parent
                .upcast::<Node>()
                .effective_command_value(command)
                .as_ref(),
            Some(new_value),
        ) {
            if let Some(css_property) = command.relevant_css_property() {
                css_property.set_for_element(cx, new_parent_html_element, new_value.clone());
            }
        }
        match command {
            // Step 18. If command is "strikethrough", and new value is "line-through",
            // and the effective command value of "strikethrough" for new parent is not "line-through",
            // set the "text-decoration" property of new parent to "line-through".
            CommandName::Strikethrough => {
                // TODO
            },
            // Step 19. If command is "underline", and new value is "underline",
            // and the effective command value of "underline" for new parent is not "underline",
            // set the "text-decoration" property of new parent to "underline".
            CommandName::Underline => {
                if new_value == "underline" &&
                    self.effective_command_value(&CommandName::Underline)
                        .is_some_and(|value| value == "underline")
                {
                    CssPropertyName::TextDecorationLine.set_for_element(
                        cx,
                        new_parent_html_element,
                        new_value.clone(),
                    );
                }
            },
            _ => {},
        }
        // Step 20. Append node to new parent as its last child, preserving ranges.
        let new_parent = new_parent.upcast::<Node>();
        move_preserving_ranges(cx, self, |cx| new_parent.AppendChild(cx, self));
        // Step 21. If node is an Element and the effective command value of command for node is not loosely equivalent to new value:
        if self.is::<Element>() &&
            !command.are_loosely_equivalent_values(
                self.effective_command_value(command).as_ref(),
                Some(new_value),
            )
        {
            // Step 21.1. Insert node into the parent of new parent before new parent, preserving ranges.
            let parent_of_new_parent = new_parent.GetParentNode().expect("Must have a parent");
            move_preserving_ranges(cx, self, |cx| {
                parent_of_new_parent.InsertBefore(cx, self, Some(new_parent))
            });
            // Step 21.2. Remove new parent from its parent.
            new_parent.remove_self(cx);
            // Step 21.3. Let children be all children of node,
            // omitting any that are Elements whose specified command value for command is neither null nor equivalent to new value.
            for child in self.children() {
                if child.downcast::<Element>().is_some_and(|child_element| {
                    let specified_command_value = child_element.specified_command_value(command);
                    specified_command_value.is_some() &&
                        !command.are_equivalent_values(
                            specified_command_value.as_ref(),
                            Some(new_value),
                        )
                }) {
                    continue;
                }
                // Step 21.4. Force the value of each node in children,
                // with command and new value as in this invocation of the algorithm.
                child.force_the_value(cx, command, Some(new_value));
            }
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#in-the-same-editing-host>
    pub(crate) fn same_editing_host(&self, other: &Node) -> bool {
        // > Two nodes are in the same editing host if the editing host of the first is non-null and the same as the editing host of the second.
        self.editing_host_of()
            .is_some_and(|editing_host| other.editing_host_of() == Some(editing_host))
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-node>
    pub(crate) fn is_block_node(&self) -> bool {
        // > A block node is either an Element whose "display" property does not have resolved value "inline" or "inline-block" or "inline-table" or "none",
        if self
            .downcast::<Element>()
            .and_then(Element::resolved_display_value)
            .is_some_and(|display| {
                display != DisplayOutside::Inline && display != DisplayOutside::None
            })
        {
            return true;
        }
        // > or a document, or a DocumentFragment.
        matches!(
            self.type_id(),
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_)
        )
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#inline-node>
    pub(crate) fn is_inline_node(&self) -> bool {
        // > An inline node is a node that is not a block node.
        !self.is_block_node()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-node-of>
    pub(crate) fn block_node_of(&self) -> Option<DomRoot<Node>> {
        let mut node = DomRoot::from_ref(self);

        loop {
            // Step 1. While node is an inline node, set node to its parent.
            if node.is_inline_node() {
                node = node.GetParentNode()?;
                continue;
            }
            // Step 2. Return node.
            return Some(node);
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#visible>
    pub(crate) fn is_visible(&self) -> bool {
        for parent in self.inclusive_ancestors(ShadowIncluding::No) {
            // > excluding any node with an inclusive ancestor Element whose "display" property has resolved value "none".
            if parent
                .downcast::<Element>()
                .and_then(Element::resolved_display_value)
                .is_some_and(|display| display == DisplayOutside::None)
            {
                return false;
            }
        }
        // > Something is visible if it is a node that either is a block node,
        if self.is_block_node() {
            return true;
        }
        // > or a Text node that is not a collapsed whitespace node,
        if self
            .downcast::<Text>()
            .is_some_and(|text| !text.is_collapsed_whitespace_node())
        {
            return true;
        }
        // > or an img, or a br that is not an extraneous line break, or any node with a visible descendant;
        if self.is::<HTMLImageElement>() {
            return true;
        }
        if self
            .downcast::<HTMLBRElement>()
            .is_some_and(|br| !br.is_extraneous_line_break())
        {
            return true;
        }
        for child in self.children() {
            if child.is_visible() {
                return true;
            }
        }
        false
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#invisible>
    pub(crate) fn is_invisible(&self) -> bool {
        // > Something is invisible if it is a node that is not visible.
        !self.is_visible()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#formattable-node>
    pub(crate) fn is_formattable(&self) -> bool {
        // > A formattable node is an editable visible node that is either a Text node, an img, or a br.
        self.is_editable() &&
            self.is_visible() &&
            (self.is::<Text>() || self.is::<HTMLImageElement>() || self.is::<HTMLBRElement>())
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-start-point>
    fn is_block_start_point(&self, offset: usize) -> bool {
        // > A boundary point (node, offset) is a block start point if either node's parent is null and offset is zero;
        if offset == 0 {
            return self.GetParentNode().is_none();
        }
        // > or node has a child with index offset − 1, and that child is either a visible block node or a visible br.
        self.children().nth(offset - 1).is_some_and(|child| {
            child.is_visible() && (child.is_block_node() || child.is::<HTMLBRElement>())
        })
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-end-point>
    fn is_block_end_point(&self, offset: u32) -> bool {
        // > A boundary point (node, offset) is a block end point if either node's parent is null and offset is node's length;
        if self.GetParentNode().is_none() && offset == self.len() {
            return true;
        }
        // > or node has a child with index offset, and that child is a visible block node.
        self.children()
            .nth(offset as usize)
            .is_some_and(|child| child.is_visible() && child.is_block_node())
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-boundary-point>
    fn is_block_boundary_point(&self, offset: u32) -> bool {
        // > A boundary point is a block boundary point if it is either a block start point or a block end point.
        self.is_block_start_point(offset as usize) || self.is_block_end_point(offset)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#collapsed-block-prop>
    pub(crate) fn is_collapsed_block_prop(&self) -> bool {
        // > A collapsed block prop is either a collapsed line break that is not an extraneous line break,

        // TODO: Check for collapsed line break
        if self
            .downcast::<HTMLBRElement>()
            .is_some_and(|br| !br.is_extraneous_line_break())
        {
            return true;
        }
        // > or an Element that is an inline node and whose children are all either invisible or collapsed block props
        if !self.is::<Element>() {
            return false;
        };
        if !self.is_inline_node() {
            return false;
        }
        let mut at_least_one_collapsed_block_prop = false;
        for child in self.children() {
            if child.is_collapsed_block_prop() {
                at_least_one_collapsed_block_prop = true;
                continue;
            }
            if child.is_invisible() {
                continue;
            }

            return false;
        }
        // > and that has at least one child that is a collapsed block prop.
        at_least_one_collapsed_block_prop
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#follows-a-line-break>
    fn follows_a_line_break(&self) -> bool {
        // Step 1. Let offset be zero.
        let mut offset = 0;
        // Step 2. While (node, offset) is not a block boundary point:
        let mut node = DomRoot::from_ref(self);
        while !node.is_block_boundary_point(offset) {
            // Step 2.2. If offset is zero or node has no children, set offset to node's index, then set node to its parent.
            if offset == 0 || node.children_count() == 0 {
                offset = node.index();
                node = node.GetParentNode().expect("Must always have a parent");
                continue;
            }
            // Step 2.1. If node has a visible child with index offset minus one, return false.
            let child = node.children().nth(offset as usize - 1);
            let Some(child) = child else {
                return false;
            };
            if child.is_visible() {
                return false;
            }
            // Step 2.3. Otherwise, set node to its child with index offset minus one, then set offset to node's length.
            node = child;
            offset = node.len();
        }
        // Step 3. Return true.
        true
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#precedes-a-line-break>
    fn precedes_a_line_break(&self) -> bool {
        let mut node = DomRoot::from_ref(self);
        // Step 1. Let offset be node's length.
        let mut offset = node.len();
        // Step 2. While (node, offset) is not a block boundary point:
        while !node.is_block_boundary_point(offset) {
            // Step 2.1. If node has a visible child with index offset, return false.
            if node
                .children()
                .nth(offset as usize)
                .is_some_and(|child| child.is_visible())
            {
                return false;
            }
            // Step 2.2. If offset is node's length or node has no children, set offset to one plus node's index, then set node to its parent.
            if offset == node.len() || node.children_count() == 0 {
                offset = 1 + node.index();
                node = node.GetParentNode().expect("Must always have a parent");
                continue;
            }
            // Step 2.3. Otherwise, set node to its child with index offset and set offset to zero.
            let child = node.children().nth(offset as usize);
            node = match child {
                None => return false,
                Some(child) => child,
            };
            offset = 0;
        }
        // Step 3. Return true.
        true
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#canonical-space-sequence>
    fn canonical_space_sequence(
        n: usize,
        non_breaking_start: bool,
        non_breaking_end: bool,
    ) -> String {
        let mut n = n;
        // Step 1. If n is zero, return the empty string.
        if n == 0 {
            return String::new();
        }
        // Step 2. If n is one and both non-breaking start and non-breaking end are false, return a single space (U+0020).
        if n == 1 {
            if !non_breaking_start && !non_breaking_end {
                return "\u{0020}".to_owned();
            }
            // Step 3. If n is one, return a single non-breaking space (U+00A0).
            return "\u{00A0}".to_owned();
        }
        // Step 4. Let buffer be the empty string.
        let mut buffer = String::new();
        // Step 5. If non-breaking start is true, let repeated pair be U+00A0 U+0020. Otherwise, let it be U+0020 U+00A0.
        let repeated_pair = if non_breaking_start {
            "\u{00A0}\u{0020}"
        } else {
            "\u{0020}\u{00A0}"
        };
        // Step 6. While n is greater than three, append repeated pair to buffer and subtract two from n.
        while n > 3 {
            buffer.push_str(repeated_pair);
            n -= 2;
        }
        // Step 7. If n is three, append a three-code unit string to buffer depending on non-breaking start and non-breaking end:
        if n == 3 {
            buffer.push_str(match (non_breaking_start, non_breaking_end) {
                (false, false) => "\u{0020}\u{00A0}\u{0020}",
                (true, false) => "\u{00A0}\u{00A0}\u{0020}",
                (false, true) => "\u{0020}\u{00A0}\u{00A0}",
                (true, true) => "\u{00A0}\u{0020}\u{00A0}",
            });
        } else {
            // Step 8. Otherwise, append a two-code unit string to buffer depending on non-breaking start and non-breaking end:
            buffer.push_str(match (non_breaking_start, non_breaking_end) {
                (false, false) | (true, false) => "\u{00A0}\u{0020}",
                (false, true) => "\u{0020}\u{00A0}",
                (true, true) => "\u{00A0}\u{00A0}",
            });
        }
        // Step 9. Return buffer.
        buffer
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#canonicalize-whitespace>
    pub(crate) fn canonicalize_whitespace(&self, offset: u32, fix_collapsed_space: bool) {
        // Step 1. If node is neither editable nor an editing host, abort these steps.
        if !self.is_editable_or_editing_host() {
            return;
        }
        // Step 2. Let start node equal node and let start offset equal offset.
        let mut start_node = DomRoot::from_ref(self);
        let mut start_offset = offset;
        // Step 3. Repeat the following steps:
        loop {
            // Step 3.1. If start node has a child in the same editing host with index start offset minus one,
            // set start node to that child, then set start offset to start node's length.
            if start_offset > 0 {
                let child = start_node.children().nth(start_offset as usize - 1);
                if let Some(child) = child {
                    if start_node.same_editing_host(&child) {
                        start_node = child;
                        start_offset = start_node.len();
                        continue;
                    }
                };
            }
            // Step 3.2. Otherwise, if start offset is zero and start node does not follow a line break
            // and start node's parent is in the same editing host, set start offset to start node's index,
            // then set start node to its parent.
            if start_offset == 0 && !start_node.follows_a_line_break() {
                if let Some(parent) = start_node.GetParentNode() {
                    if parent.same_editing_host(&start_node) {
                        start_offset = start_node.index();
                        start_node = parent;
                    }
                }
            }
            // Step 3.3. Otherwise, if start node is a Text node and its parent's resolved
            // value for "white-space" is neither "pre" nor "pre-wrap" and start offset is not zero
            // and the (start offset − 1)st code unit of start node's data is a space (0x0020) or
            // non-breaking space (0x00A0), subtract one from start offset.
            if start_offset != 0 &&
                start_node.downcast::<Text>().is_some_and(|text| {
                    text.has_whitespace_and_has_parent_with_whitespace_preserve(
                        start_offset - 1,
                        &[&'\u{0020}', &'\u{00A0}'],
                    )
                })
            {
                start_offset -= 1;
            }
            // Step 3.4. Otherwise, break from this loop.
            break;
        }
        // Step 4. Let end node equal start node and end offset equal start offset.
        let mut end_node = start_node.clone();
        let mut end_offset = start_offset;
        // Step 5. Let length equal zero.
        let mut length = 0;
        // Step 6. Let collapse spaces be true if start offset is zero and start node follows a line break, otherwise false.
        let mut collapse_spaces = start_offset == 0 && start_node.follows_a_line_break();
        // Step 7. Repeat the following steps:
        loop {
            // Step 7.1. If end node has a child in the same editing host with index end offset,
            // set end node to that child, then set end offset to zero.
            if let Some(child) = end_node.children().nth(end_offset as usize) {
                if child.same_editing_host(&end_node) {
                    end_node = child;
                    end_offset = 0;
                    continue;
                }
            }
            // Step 7.2. Otherwise, if end offset is end node's length
            // and end node does not precede a line break
            // and end node's parent is in the same editing host,
            // set end offset to one plus end node's index, then set end node to its parent.
            if end_offset == end_node.len() && !end_node.precedes_a_line_break() {
                if let Some(parent) = end_node.GetParentNode() {
                    if parent.same_editing_host(&end_node) {
                        end_offset = 1 + end_node.index();
                        end_node = parent;
                    }
                }
                continue;
            }
            // Step 7.3. Otherwise, if end node is a Text node and its parent's resolved value for "white-space"
            // is neither "pre" nor "pre-wrap"
            // and end offset is not end node's length and the end offsetth code unit of end node's data
            // is a space (0x0020) or non-breaking space (0x00A0):
            if let Some(text) = end_node.downcast::<Text>() {
                if text.has_whitespace_and_has_parent_with_whitespace_preserve(
                    end_offset,
                    &[&'\u{0020}', &'\u{00A0}'],
                ) {
                    // Step 7.3.1. If fix collapsed space is true, and collapse spaces is true,
                    // and the end offsetth code unit of end node's data is a space (0x0020):
                    // call deleteData(end offset, 1) on end node, then continue this loop from the beginning.
                    let has_space_at_offset = text
                        .data()
                        .chars()
                        .nth(end_offset as usize)
                        .is_some_and(|c| c == '\u{0020}');
                    if fix_collapsed_space && collapse_spaces && has_space_at_offset {
                        if text
                            .upcast::<CharacterData>()
                            .DeleteData(end_offset, 1)
                            .is_err()
                        {
                            unreachable!("Invalid deletion for character at end offset");
                        }
                        continue;
                    }
                    // Step 7.3.2. Set collapse spaces to true if the end offsetth code unit of
                    // end node's data is a space (0x0020), false otherwise.
                    collapse_spaces = text
                        .data()
                        .chars()
                        .nth(end_offset as usize)
                        .is_some_and(|c| c == '\u{0020}');
                    // Step 7.3.3. Add one to end offset.
                    end_offset += 1;
                    // Step 7.3.4. Add one to length.
                    length += 1;
                    continue;
                }
            }
            // Step 7.4. Otherwise, break from this loop.
            break;
        }
        // Step 8. If fix collapsed space is true, then while (start node, start offset)
        // is before (end node, end offset):
        if fix_collapsed_space {
            while bp_position(&start_node, start_offset, &end_node, end_offset) ==
                Some(Ordering::Less)
            {
                // Step 8.1. If end node has a child in the same editing host with index end offset − 1,
                // set end node to that child, then set end offset to end node's length.
                if end_offset > 0 {
                    if let Some(child) = end_node.children().nth(end_offset as usize - 1) {
                        if child.same_editing_host(&end_node) {
                            end_node = child;
                            end_offset = end_node.len();
                            continue;
                        }
                    }
                }
                // Step 8.2. Otherwise, if end offset is zero and end node's parent is in the same editing host,
                // set end offset to end node's index, then set end node to its parent.
                if let Some(parent) = end_node.GetParentNode() {
                    if end_offset == 0 && parent.same_editing_host(&end_node) {
                        end_offset = end_node.index();
                        end_node = parent;
                        continue;
                    }
                }
                // Step 8.3. Otherwise, if end node is a Text node and its parent's resolved value for "white-space"
                // is neither "pre" nor "pre-wrap"
                // and end offset is end node's length and the last code unit of end node's data
                // is a space (0x0020) and end node precedes a line break:
                if let Some(text) = end_node.downcast::<Text>() {
                    if text.has_whitespace_and_has_parent_with_whitespace_preserve(
                        text.data().len() as u32,
                        &[&'\u{0020}'],
                    ) && end_node.precedes_a_line_break()
                    {
                        // Step 8.3.1. Subtract one from end offset.
                        end_offset -= 1;
                        // Step 8.3.2. Subtract one from length.
                        length -= 1;
                        // Step 8.3.3. Call deleteData(end offset, 1) on end node.
                        if text
                            .upcast::<CharacterData>()
                            .DeleteData(end_offset, 1)
                            .is_err()
                        {
                            unreachable!("Invalid deletion for character at end offset");
                        }
                        continue;
                    }
                }
                // Step 8.4. Otherwise, break from this loop.
                break;
            }
        }
        // Step 9. Let replacement whitespace be the canonical space sequence of length length.
        // non-breaking start is true if start offset is zero and start node follows a line break, and false otherwise.
        // non-breaking end is true if end offset is end node's length and end node precedes a line break, and false otherwise.
        let replacement_whitespace = Node::canonical_space_sequence(
            length,
            start_offset == 0 && start_node.follows_a_line_break(),
            end_offset == end_node.len() && end_node.precedes_a_line_break(),
        );
        let mut replacement_whitespace_chars = replacement_whitespace.chars();
        // Step 10. While (start node, start offset) is before (end node, end offset):
        while bp_position(&start_node, start_offset, &end_node, end_offset) == Some(Ordering::Less)
        {
            // Step 10.1. If start node has a child with index start offset, set start node to that child, then set start offset to zero.
            if let Some(child) = start_node.children().nth(start_offset as usize) {
                start_node = child;
                start_offset = 0;
                continue;
            }
            // Step 10.2. Otherwise, if start node is not a Text node or if start offset is start node's length,
            // set start offset to one plus start node's index, then set start node to its parent.
            let start_node_as_text = start_node.downcast::<Text>();
            if start_node_as_text.is_none() || start_offset == start_node.len() {
                start_offset = 1 + start_node.index();
                start_node = start_node
                    .GetParentNode()
                    .expect("Must always have a parent");
                continue;
            }
            let start_node_as_text =
                start_node_as_text.expect("Already verified none in previous statement");
            // Step 10.3. Otherwise:
            // Step 10.3.1. Remove the first code unit from replacement whitespace, and let element be that code unit.
            if let Some(element) = replacement_whitespace_chars.next() {
                // Step 10.3.2. If element is not the same as the start offsetth code unit of start node's data:
                if start_node_as_text.data().chars().nth(start_offset as usize) != Some(element) {
                    let character_data = start_node_as_text.upcast::<CharacterData>();
                    // Step 10.3.2.1. Call insertData(start offset, element) on start node.
                    if character_data
                        .InsertData(start_offset, element.to_string().into())
                        .is_err()
                    {
                        unreachable!("Invalid insertion for character at start offset");
                    }
                    // Step 10.3.2.2. Call deleteData(start offset + 1, 1) on start node.
                    if character_data.DeleteData(start_offset + 1, 1).is_err() {
                        unreachable!("Invalid deletion for character at start offset + 1");
                    }
                }
            }
            // Step 10.3.3. Add one to start offset.
            start_offset += 1;
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#remove-extraneous-line-breaks-before>
    fn remove_extraneous_line_breaks_before(&self, cx: &mut JSContext) {
        let parent = self.GetParentNode();
        // Step 1. Let ref be the previousSibling of node.
        let Some(mut ref_) = self.GetPreviousSibling() else {
            // Step 2. If ref is null, abort these steps.
            return;
        };
        // Step 3. While ref has children, set ref to its lastChild.
        while let Some(last_child) = ref_.children().last() {
            ref_ = last_child;
        }
        // Step 4. While ref is invisible but not an extraneous line break,
        // and ref does not equal node's parent, set ref to the node before it in tree order.
        loop {
            if ref_.is_invisible() &&
                ref_.downcast::<HTMLBRElement>()
                    .is_none_or(|br| !br.is_extraneous_line_break())
            {
                if let Some(parent) = parent.as_ref() {
                    if ref_ != *parent {
                        ref_ = match ref_.preceding_nodes(parent).nth(0) {
                            None => break,
                            Some(node) => node,
                        };
                        continue;
                    }
                }
            }
            break;
        }
        // Step 5. If ref is an editable extraneous line break, remove it from its parent.
        if ref_.is_editable() &&
            ref_.downcast::<HTMLBRElement>()
                .is_some_and(|br| br.is_extraneous_line_break())
        {
            assert!(ref_.has_parent());
            ref_.remove_self(cx);
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#remove-extraneous-line-breaks-at-the-end-of>
    pub(crate) fn remove_extraneous_line_breaks_at_the_end_of(&self, cx: &mut JSContext) {
        // Step 1. Let ref be node.
        let mut ref_ = DomRoot::from_ref(self);
        // Step 2. While ref has children, set ref to its lastChild.
        while let Some(last_child) = ref_.children().last() {
            ref_ = last_child;
        }
        // Step 3. While ref is invisible but not an extraneous line break, and ref does not equal node,
        // set ref to the node before it in tree order.
        loop {
            if ref_.is_invisible() &&
                *ref_ != *self &&
                ref_.downcast::<HTMLBRElement>()
                    .is_none_or(|br| !br.is_extraneous_line_break())
            {
                if let Some(parent_of_ref) = ref_.GetParentNode() {
                    ref_ = match ref_.preceding_nodes(&parent_of_ref).nth(0) {
                        None => break,
                        Some(node) => node,
                    };
                    continue;
                }
            }
            break;
        }
        // Step 4. If ref is an editable extraneous line break:
        if ref_.is_editable() &&
            ref_.downcast::<HTMLBRElement>()
                .is_some_and(|br| br.is_extraneous_line_break())
        {
            // Step 4.1. While ref's parent is editable and invisible, set ref to its parent.
            loop {
                if let Some(parent) = ref_.GetParentNode() {
                    if parent.is_editable() && parent.is_invisible() {
                        ref_ = parent;
                        continue;
                    }
                }
                break;
            }
            // Step 4.2. Remove ref from its parent.
            assert!(ref_.has_parent());
            ref_.remove_self(cx);
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#remove-extraneous-line-breaks-from>
    fn remove_extraneous_line_breaks_from(&self, cx: &mut JSContext) {
        // > To remove extraneous line breaks from a node, first remove extraneous line breaks before it,
        // > then remove extraneous line breaks at the end of it.
        self.remove_extraneous_line_breaks_before(cx);
        self.remove_extraneous_line_breaks_at_the_end_of(cx);
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#preserving-its-descendants>
    pub(crate) fn remove_preserving_its_descendants(&self, cx: &mut JSContext) {
        // > To remove a node node while preserving its descendants,
        // > split the parent of node's children if it has any.
        // > If it has no children, instead remove it from its parent.
        if self.children_count() == 0 {
            assert!(self.has_parent());
            self.remove_self(cx);
        } else {
            rooted_vec!(let children <- self.children().map(|child| DomRoot::as_traced(&child)));
            split_the_parent(cx, children.r());
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#effective-command-value>
    pub(crate) fn effective_command_value(&self, command: &CommandName) -> Option<DOMString> {
        // Step 1. If neither node nor its parent is an Element, return null.
        // Step 2. If node is not an Element, return the effective command value of its parent for command.
        let Some(element) = self.downcast::<Element>() else {
            return self
                .GetParentElement()
                .and_then(|parent| parent.upcast::<Node>().effective_command_value(command));
        };
        match command {
            // Step 3. If command is "createLink" or "unlink":
            CommandName::CreateLink | CommandName::Unlink => {
                // Step 3.1. While node is not null, and is not an a element that has an href attribute, set node to its parent.
                let mut current_node = Some(DomRoot::from_ref(self));
                while let Some(node) = current_node {
                    if let Some(anchor_value) =
                        node.downcast::<HTMLAnchorElement>().and_then(|anchor| {
                            anchor
                                .upcast::<Element>()
                                .get_attribute(&local_name!("href"))
                        })
                    {
                        // Step 3.3. Return the value of node's href attribute.
                        return Some(DOMString::from(&**anchor_value.value()));
                    }
                    current_node = node.GetParentNode();
                }
                // Step 3.2. If node is null, return null.
                None
            },
            // Step 4. If command is "backColor" or "hiliteColor":
            CommandName::BackColor | CommandName::HiliteColor => {
                // Step 4.1. While the resolved value of "background-color" on node is any fully transparent value,
                // and node's parent is an Element, set node to its parent.
                // TODO
                // Step 4.2. Return the resolved value of "background-color" for node.
                // TODO
                None
            },
            // Step 5. If command is "subscript" or "superscript":
            CommandName::Subscript | CommandName::Superscript => {
                // Step 5.1. Let affected by subscript and affected by superscript be two boolean variables,
                // both initially false.
                let mut affected_by_subscript = false;
                let mut affected_by_superscript = false;
                // Step 5.2. While node is an inline node:
                let mut current_node = Some(DomRoot::from_ref(self));
                while let Some(node) = current_node {
                    if !node.is_inline_node() {
                        break;
                    }
                    // Step 5.2.1. If node is a sub, set affected by subscript to true.
                    if *element.local_name() == local_name!("sub") {
                        affected_by_subscript = true;
                    } else if *element.local_name() == local_name!("sup") {
                        // Step 5.2.2. Otherwise, if node is a sup, set affected by superscript to true.
                        affected_by_superscript = true;
                    }
                    // Step 5.2.3. Set node to its parent.
                    current_node = node.GetParentNode();
                }
                Some(match (affected_by_subscript, affected_by_superscript) {
                    // Step 5.3. If affected by subscript and affected by superscript are both true,
                    // return the string "mixed".
                    (true, true) => "mixed".into(),
                    // Step 5.4. If affected by subscript is true, return "subscript".
                    (true, false) => "subscript".into(),
                    // Step 5.5. If affected by superscript is true, return "superscript".
                    (false, true) => "superscript".into(),
                    // Step 5.6. Return null.
                    (false, false) => return None,
                })
            },
            // Step 6. If command is "strikethrough",
            // and the "text-decoration" property of node or any of its ancestors has resolved value containing "line-through",
            // return "line-through". Otherwise, return null.
            // TODO
            CommandName::Strikethrough => None,
            // Step 7. If command is "underline",
            // and the "text-decoration" property of node or any of its ancestors has resolved value containing "underline",
            // return "underline". Otherwise, return null.
            CommandName::Underline => Some("underline".into()).filter(|_| {
                self.inclusive_ancestors(ShadowIncluding::No).any(|node| {
                    node.downcast::<Element>()
                        .and_then(|element| CommandName::Underline.resolved_value_for_node(element))
                        .is_some_and(|property| property.contains("underline"))
                })
            }),
            // Step 8. Return the resolved value for node of the relevant CSS property for command.
            _ => command.resolved_value_for_node(element),
        }
    }
}
