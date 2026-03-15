/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;
use std::ops::Deref;

use html5ever::local_name;
use script_bindings::inheritance::Castable;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::values::specified::box_::DisplayOutside;

use crate::dom::abstractrange::bp_position;
use crate::dom::bindings::cell::Ref;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, ElementCreationOptions,
};
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::SelectionBinding::SelectionMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrElementCreationOptions;
use crate::dom::bindings::inheritance::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::root::{DomRoot, DomSlice};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmllielement::HTMLLIElement;
use crate::dom::html::htmltablecellelement::HTMLTableCellElement;
use crate::dom::html::htmltablerowelement::HTMLTableRowElement;
use crate::dom::html::htmltablesectionelement::HTMLTableSectionElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::range::Range;
use crate::dom::selection::Selection;
use crate::dom::text::Text;
use crate::script_runtime::CanGc;

impl Text {
    /// <https://dom.spec.whatwg.org/#concept-cd-data>
    fn data(&self) -> Ref<'_, String> {
        self.upcast::<CharacterData>().data()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#whitespace-node>
    fn is_whitespace_node(&self) -> bool {
        // > A whitespace node is either a Text node whose data is the empty string;
        let data = self.data();
        if data.is_empty() {
            return true;
        }
        // > or a Text node whose data consists only of one or more tabs (0x0009), line feeds (0x000A),
        // > carriage returns (0x000D), and/or spaces (0x0020),
        // > and whose parent is an Element whose resolved value for "white-space" is "normal" or "nowrap";
        let Some(parent) = self.upcast::<Node>().GetParentElement() else {
            return false;
        };
        // TODO: Optimize the below to only do a traversal once and in the match handle the expected collapse value
        let Some(style) = parent.style() else {
            return false;
        };
        let white_space_collapse = style.get_inherited_text().white_space_collapse;
        if data
            .bytes()
            .all(|byte| matches!(byte, b'\t' | b'\n' | b'\r' | b' ')) &&
            // Note that for "normal" and "nowrap", the longhand "white-space-collapse: collapse" applies
            // https://www.w3.org/TR/css-text-4/#white-space-property
            white_space_collapse == WhiteSpaceCollapse::Collapse
        {
            return true;
        }
        // > or a Text node whose data consists only of one or more tabs (0x0009), carriage returns (0x000D),
        // > and/or spaces (0x0020), and whose parent is an Element whose resolved value for "white-space" is "pre-line".
        data.bytes()
            .all(|byte| matches!(byte, b'\t' | b'\r' | b' ')) &&
            // Note that for "pre-line", the longhand "white-space-collapse: preserve-breaks" applies
            // https://www.w3.org/TR/css-text-4/#white-space-property
            white_space_collapse == WhiteSpaceCollapse::PreserveBreaks
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#collapsed-whitespace-node>
    fn is_collapsed_whitespace_node(&self) -> bool {
        // Step 1. If node is not a whitespace node, return false.
        if !self.is_whitespace_node() {
            return false;
        }
        // Step 2. If node's data is the empty string, return true.
        if self.data().is_empty() {
            return true;
        }
        // Step 3. Let ancestor be node's parent.
        let node = self.upcast::<Node>();
        let Some(ancestor) = node.GetParentNode() else {
            // Step 4. If ancestor is null, return true.
            return true;
        };
        let mut resolved_ancestor = ancestor.clone();
        for parent in ancestor.ancestors() {
            // Step 5. If the "display" property of some ancestor of node has resolved value "none", return true.
            if parent.is_display_none() {
                return true;
            }
            // Step 6. While ancestor is not a block node and its parent is not null, set ancestor to its parent.
            //
            // Note that the spec is written as "while not". Since this is the end-condition, we need to invert
            // the condition to decide when to stop.
            if parent.is_block_node() {
                break;
            }
            resolved_ancestor = parent;
        }
        // Step 7. Let reference be node.
        // Step 8. While reference is a descendant of ancestor:
        // Step 8.1. Let reference be the node before it in tree order.
        for reference in node.preceding_nodes(&resolved_ancestor) {
            // Step 8.2. If reference is a block node or a br, return true.
            if reference.is_block_node() || reference.is::<HTMLBRElement>() {
                return true;
            }
            // Step 8.3. If reference is a Text node that is not a whitespace node, or is an img, break from this loop.
            if reference
                .downcast::<Text>()
                .is_some_and(|text| !text.is_whitespace_node()) ||
                reference.is::<HTMLImageElement>()
            {
                break;
            }
        }
        // Step 9. Let reference be node.
        // Step 10. While reference is a descendant of ancestor:
        // Step 10.1. Let reference be the node after it in tree order, or null if there is no such node.
        for reference in node.following_nodes(&resolved_ancestor) {
            // Step 10.2. If reference is a block node or a br, return true.
            if reference.is_block_node() || reference.is::<HTMLBRElement>() {
                return true;
            }
            // Step 10.3. If reference is a Text node that is not a whitespace node, or is an img, break from this loop.
            if reference
                .downcast::<Text>()
                .is_some_and(|text| !text.is_whitespace_node()) ||
                reference.is::<HTMLImageElement>()
            {
                break;
            }
        }
        // Step 11. Return false.
        false
    }

    /// Part of <https://w3c.github.io/editing/docs/execCommand/#canonicalize-whitespace>
    /// and deduplicated here, since we need to do this for both start and end nodes
    fn has_whitespace_and_has_parent_with_whitespace_preserve(
        &self,
        offset: u32,
        space_characters: &'static [&'static char],
    ) -> bool {
        // if node is a Text node and its parent's resolved value for "white-space" is neither "pre" nor "pre-wrap"
        // and start offset is not zero and the (start offset − 1)st code unit of start node's data is a space (0x0020) or
        // non-breaking space (0x00A0)
        let has_preserve_space = self
            .upcast::<Node>()
            .GetParentNode()
            .and_then(|parent| parent.style())
            .is_some_and(|style| {
                // Note that for "pre" and "pre-wrap", the longhand "white-space-collapse: preserve" applies
                // https://www.w3.org/TR/css-text-4/#white-space-property
                style.get_inherited_text().white_space_collapse != WhiteSpaceCollapse::Preserve
            });
        let has_space_character = self
            .data()
            .chars()
            .nth(offset as usize)
            .is_some_and(|c| space_characters.contains(&&c));
        has_preserve_space && has_space_character
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

impl Document {
    fn create_br_element(&self, cx: &mut js::context::JSContext) -> DomRoot<Element> {
        let element_options =
            StringOrElementCreationOptions::ElementCreationOptions(ElementCreationOptions {
                is: None,
            });
        match self.CreateElement(cx, "br".into(), element_options) {
            Err(_) => unreachable!("Must always be able to create br"),
            Ok(br) => br,
        }
    }
}

impl HTMLElement {
    fn local_name(&self) -> &str {
        self.upcast::<Element>().local_name()
    }
}

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

/// <https://w3c.github.io/editing/docs/execCommand/#allowed-child>
fn is_allowed_child(child: NodeOrString, parent: NodeOrString) -> bool {
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
pub(crate) fn split_the_parent<'a>(cx: &mut js::context::JSContext, node_list: &'a [&'a Node]) {
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
        for node in node_list.iter().rev() {
            // TODO: Preserving ranges
            if parent_of_original_parent
                .InsertBefore(cx, node, original_parent.GetNextSibling().as_deref())
                .is_err()
            {
                unreachable!("Must always have a parent");
            }
        }
        // Step 6.2. If precedes line break is true, and the last member of node list does not precede a line break,
        // call createElement("br") on the context object and insert the result immediately after the last member of node list.
        if precedes_line_break {
            if let Some(last) = node_list.last() {
                if !last.precedes_a_line_break() {
                    let br = context_object.create_br_element(cx);
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
                    // TODO: Preserving ranges
                    if cloned_parent.AppendChild(cx, &first_of_original).is_err() {
                        unreachable!("Must always have a parent");
                    }
                    continue;
                }
            }
            break;
        }
    }
    // Step 8. For each node in node list, insert node into the parent of original parent immediately before original parent, preserving ranges.
    for node in node_list.iter() {
        // TODO: Preserving ranges
        if parent_of_original_parent
            .InsertBefore(cx, node, Some(&original_parent))
            .is_err()
        {
            unreachable!("Must always have a parent");
        }
    }
    // Step 9. If follows line break is true, and the first member of node list does not follow a line break,
    // call createElement("br") on the context object and insert the result immediately before the first member of node list.
    if follows_line_break {
        if let Some(first) = node_list.first() {
            if !first.follows_a_line_break() {
                let br = context_object.create_br_element(cx);
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
                    let br = context_object.create_br_element(cx);
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

impl Node {
    fn resolved_display_value(&self) -> Option<DisplayOutside> {
        self.style().map(|style| style.get_box().display.outside())
    }
}

pub(crate) trait NodeExecCommandSupport {
    fn same_editing_host(&self, other: &Node) -> bool;
    fn is_block_node(&self) -> bool;
    fn is_inline_node(&self) -> bool;
    fn block_node_of(&self) -> Option<DomRoot<Node>>;
    fn is_visible(&self) -> bool;
    fn is_invisible(&self) -> bool;
    fn is_formattable(&self) -> bool;
    fn is_block_start_point(&self, offset: usize) -> bool;
    fn is_block_end_point(&self, offset: u32) -> bool;
    fn is_block_boundary_point(&self, offset: u32) -> bool;
    fn is_collapsed_block_prop(&self) -> bool;
    fn follows_a_line_break(&self) -> bool;
    fn precedes_a_line_break(&self) -> bool;
    fn canonical_space_sequence(
        n: usize,
        non_breaking_start: bool,
        non_breaking_end: bool,
    ) -> String;
    fn canonicalize_whitespace(&self, offset: u32, fix_collapsed_space: bool);
    fn remove_extraneous_line_breaks_before(&self, cx: &mut js::context::JSContext);
    fn remove_extraneous_line_breaks_at_the_end_of(&self, cx: &mut js::context::JSContext);
    fn remove_preserving_its_descendants(&self, cx: &mut js::context::JSContext);
    fn effective_command_value(&self, command_name: CommandName) -> Option<DOMString>;
}

impl NodeExecCommandSupport for Node {
    /// <https://w3c.github.io/editing/docs/execCommand/#in-the-same-editing-host>
    fn same_editing_host(&self, other: &Node) -> bool {
        // > Two nodes are in the same editing host if the editing host of the first is non-null and the same as the editing host of the second.
        self.editing_host_of()
            .is_some_and(|editing_host| other.editing_host_of() == Some(editing_host))
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-node>
    fn is_block_node(&self) -> bool {
        // > A block node is either an Element whose "display" property does not have resolved value "inline" or "inline-block" or "inline-table" or "none",
        if self.is::<Element>() &&
            self.resolved_display_value().is_some_and(|display| {
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
    fn is_inline_node(&self) -> bool {
        // > An inline node is a node that is not a block node.
        !self.is_block_node()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-node-of>
    fn block_node_of(&self) -> Option<DomRoot<Node>> {
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
    fn is_visible(&self) -> bool {
        for parent in self.inclusive_ancestors(ShadowIncluding::No) {
            // > excluding any node with an inclusive ancestor Element whose "display" property has resolved value "none".
            if parent.is::<Element>() &&
                parent
                    .resolved_display_value()
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
    fn is_invisible(&self) -> bool {
        // > Something is invisible if it is a node that is not visible.
        !self.is_visible()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#formattable-node>
    fn is_formattable(&self) -> bool {
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
    fn is_collapsed_block_prop(&self) -> bool {
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
    fn canonicalize_whitespace(&self, offset: u32, fix_collapsed_space: bool) {
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
    fn remove_extraneous_line_breaks_before(&self, cx: &mut js::context::JSContext) {
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
    fn remove_extraneous_line_breaks_at_the_end_of(&self, cx: &mut js::context::JSContext) {
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

    /// <https://w3c.github.io/editing/docs/execCommand/#preserving-its-descendants>
    fn remove_preserving_its_descendants(&self, cx: &mut js::context::JSContext) {
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
    fn effective_command_value(&self, command_name: CommandName) -> Option<DOMString> {
        // Step 1. If neither node nor its parent is an Element, return null.
        // Step 2. If node is not an Element, return the effective command value of its parent for command.
        if !self.is::<Element>() {
            return self.GetParentElement().and_then(|parent| {
                parent
                    .upcast::<Node>()
                    .effective_command_value(command_name)
            });
        }
        match command_name {
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
                    if let Some(element) = node.downcast::<Element>() {
                        // Step 5.2.1. If node is a sub, set affected by subscript to true.
                        if *element.local_name() == local_name!("sub") {
                            affected_by_subscript = true;
                        } else if *element.local_name() == local_name!("sup") {
                            // Step 5.2.2. Otherwise, if node is a sup, set affected by superscript to true.
                            affected_by_superscript = true;
                        }
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
            // TODO
            CommandName::Underline => None,
            // Step 8. Return the resolved value for node of the relevant CSS property for command.
            // TODO
            _ => None,
        }
    }
}

pub(crate) trait ContentEditableRange {
    fn handle_focus_state_for_contenteditable(&self, can_gc: CanGc);
}

impl ContentEditableRange for HTMLElement {
    /// There is no specification for this implementation. Instead, it is
    /// reverse-engineered based on the WPT test
    /// /selection/contenteditable/initial-selection-on-focus.tentative.html
    fn handle_focus_state_for_contenteditable(&self, can_gc: CanGc) {
        if !self.is_editing_host() {
            return;
        }
        let document = self.owner_document();
        let Some(selection) = document.GetSelection(can_gc) else {
            return;
        };
        let range = self
            .upcast::<Element>()
            .ensure_contenteditable_selection_range(&document, can_gc);
        // If the current range is already associated with this contenteditable
        // element, then we shouldn't do anything. This is important when focus
        // is lost and regained, but selection was changed beforehand. In that
        // case, we should maintain the selection as it were, by not creating
        // a new range.
        if selection
            .active_range()
            .is_some_and(|active| active == range)
        {
            return;
        }
        let node = self.upcast::<Node>();
        let mut selected_node = DomRoot::from_ref(node);
        let mut previous_eligible_node = DomRoot::from_ref(node);
        let mut previous_node = DomRoot::from_ref(node);
        let mut selected_offset = 0;
        for child in node.traverse_preorder(ShadowIncluding::Yes) {
            if let Some(text) = child.downcast::<Text>() {
                // Note that to consider it whitespace, it needs to take more
                // into account than simply "it has a non-whitespace" character.
                // Therefore, we need to first check if it is not a whitespace
                // node and only then can we find what the relevant character is.
                if !text.is_whitespace_node() {
                    // A node with "white-space: pre" set must select its first
                    // character, regardless if that's a whitespace character or not.
                    let is_pre_formatted_text_node = child
                        .GetParentElement()
                        .and_then(|parent| parent.style())
                        .is_some_and(|style| {
                            style.get_inherited_text().white_space_collapse ==
                                WhiteSpaceCollapse::Preserve
                        });
                    if !is_pre_formatted_text_node {
                        // If it isn't pre-formatted, then we should instead select the
                        // first non-whitespace character.
                        selected_offset = text
                            .data()
                            .find(|c: char| !c.is_whitespace())
                            .unwrap_or_default() as u32;
                    }
                    selected_node = child;
                    break;
                }
            }
            // For <input>, <textarea>, <hr> and <br> elements, we should select the previous
            // node, regardless if it was a block node or not
            if matches!(
                child.type_id(),
                NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLInputElement,
                )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLTextAreaElement,
                )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLHRElement,
                )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLBRElement,
                ))
            ) {
                selected_node = previous_node;
                break;
            }
            // When we encounter a non-contenteditable element, we should select the previous
            // eligible node
            if child
                .downcast::<HTMLElement>()
                .is_some_and(|el| el.ContentEditable().str() == "false")
            {
                selected_node = previous_eligible_node;
                break;
            }
            // We can only select block nodes as eligible nodes for the case of non-conenteditable
            // nodes
            if child.is_block_node() {
                previous_eligible_node = child.clone();
            }
            previous_node = child;
        }
        range.set_start(&selected_node, selected_offset);
        range.set_end(&selected_node, selected_offset);
        selection.AddRange(&range);
    }
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

enum BoolOrOptionalString {
    Bool(bool),
    OptionalString(Option<DOMString>),
}

impl From<Option<DOMString>> for BoolOrOptionalString {
    fn from(optional_string: Option<DOMString>) -> Self {
        Self::OptionalString(optional_string)
    }
}

impl From<bool> for BoolOrOptionalString {
    fn from(bool_: bool) -> Self {
        Self::Bool(bool_)
    }
}

struct RecordedStateOfNode {
    name: CommandName,
    value: BoolOrOptionalString,
}

impl Range {
    /// <https://w3c.github.io/editing/docs/execCommand/#effectively-contained>
    fn is_effectively_contained_node(&self, node: &Node) -> bool {
        assert!(!self.collapsed());
        // > A node node is effectively contained in a range range if range is not collapsed,
        // > and at least one of the following holds:
        // > node is range's start node, it is a Text node, and its length is different from range's start offset.
        let start_container = self.start_container();
        if *start_container == *node && node.is::<Text>() && node.len() != self.start_offset() {
            return true;
        }
        // > node is range's end node, it is a Text node, and range's end offset is not 0.
        let end_container = self.end_container();
        if *end_container == *node && node.is::<Text>() && self.end_offset() != 0 {
            return true;
        }
        // > node is contained in range.
        //
        // Already checked by caller

        // > node has at least one child; and all its children are effectively contained in range;
        node.children_count() > 0 && node.children().all(|child| self.is_effectively_contained_node(&child))
        // > and either range's start node is not a descendant of node or is not a Text node or range's start offset is zero;
        && (!node.is_ancestor_of(&start_container) || !start_container.is::<Text>() || self.start_offset() == 0)
        // > and either range's end node is not a descendant of node or is not a Text node or range's end offset is its end node's length.
        && (!node.is_ancestor_of(&end_container) || !end_container.is::<Text>() || self.end_offset() == end_container.len())
    }

    fn first_formattable_contained_node(&self) -> Option<DomRoot<Node>> {
        if self.collapsed() {
            return None;
        }
        let Ok(contained_children) = self.contained_children() else {
            unreachable!("Must always be able to obtain contained children");
        };
        contained_children
            .first_partially_contained_child
            .as_ref()
            .filter(|child| child.is_formattable() && self.is_effectively_contained_node(child))
            .or_else(|| {
                // We don't call `is_effectively_contained_node` here, since nodes are considered
                // effectively contained if they are in `contained_children`
                contained_children
                    .contained_children
                    .iter()
                    .find(|child| child.is_formattable())
            })
            .or_else(|| {
                contained_children
                    .last_partially_contained_child
                    .as_ref()
                    .filter(|child| {
                        child.is_formattable() && self.is_effectively_contained_node(child)
                    })
            })
            .cloned()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#record-current-states-and-values>
    fn record_current_states_and_values(&self) -> Vec<RecordedStateOfNode> {
        // Step 1. Let overrides be a list of (string, string or boolean) ordered pairs, initially empty.
        //
        // We return the vec in one go for the relevant values

        // Step 2. Let node be the first formattable node effectively contained in the active range,
        // or null if there is none.
        let Some(node) = self.first_formattable_contained_node() else {
            // Step 3. If node is null, return overrides.
            return vec![];
        };
        // Step 8. Return overrides.
        vec![
            // Step 4. Add ("createLink", node's effective command value for "createLink") to overrides.
            RecordedStateOfNode {
                name: CommandName::CreateLink,
                value: node.effective_command_value(CommandName::CreateLink).into(),
            },
            // Step 5. For each command in the list
            // "bold", "italic", "strikethrough", "subscript", "superscript", "underline", in order:
            // if node's effective command value for command is one of its inline command activated values,
            // add (command, true) to overrides, and otherwise add (command, false) to overrides.
            // TODO

            // Step 6. For each command in the list "fontName", "foreColor", "hiliteColor", in order:
            // add (command, command's value) to overrides.
            // TODO

            // Step 7. Add ("fontSize", node's effective command value for "fontSize") to overrides.
            RecordedStateOfNode {
                name: CommandName::FontSize,
                value: node.effective_command_value(CommandName::FontSize).into(),
            },
        ]
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#restore-states-and-values>
    fn restore_states_and_values(
        &self,
        context_object: &Document,
        overrides: Vec<RecordedStateOfNode>,
    ) {
        // Step 1. Let node be the first formattable node effectively contained in the active range,
        // or null if there is none.
        let Some(_node) = self.first_formattable_contained_node() else {
            // Step 3. Otherwise, for each (command, override) pair in overrides, in order:
            for override_state in overrides {
                // Step 3.1. If override is a boolean, set the state override for command to override.
                match override_state.value {
                    BoolOrOptionalString::Bool(bool_) => {
                        context_object.set_state_override(override_state.name, bool_)
                    },
                    // Step 3.2. If override is a string, set the value override for command to override.
                    BoolOrOptionalString::OptionalString(optional_string) => {
                        context_object.set_value_override(override_state.name, optional_string)
                    },
                }
            }
            return;
        };
        // Step 2. If node is not null, then for each (command, override) pair in overrides, in order:
        // TODO

        // Step 2.1. If override is a boolean, and queryCommandState(command)
        // returns something different from override, take the action for command,
        // with value equal to the empty string.
        // TODO

        // Step 2.2. Otherwise, if override is a string, and command is neither "createLink" nor "fontSize",
        // and queryCommandValue(command) returns something not equivalent to override,
        // take the action for command, with value equal to override.
        // TODO

        // Step 2.3. Otherwise, if override is a string; and command is "createLink";
        // and either there is a value override for "createLink" that is not equal to override,
        // or there is no value override for "createLink" and node's effective command value
        // for "createLink" is not equal to override: take the action for "createLink", with value equal to override.
        // TODO

        // Step 2.4. Otherwise, if override is a string; and command is "fontSize";
        // and either there is a value override for "fontSize" that is not equal to override,
        // or there is no value override for "fontSize" and node's effective command value for "fontSize"
        // is not loosely equivalent to override:
        // TODO

        // Step 2.5. Otherwise, continue this loop from the beginning.
        // TODO

        // Step 2.6. Set node to the first formattable node effectively contained in the active range, if there is one.
        // TODO
    }
}

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

pub(crate) trait SelectionExecCommandSupport {
    fn delete_the_selection(
        &self,
        cx: &mut js::context::JSContext,
        context_object: &Document,
        block_merging: SelectionDeletionBlockMerging,
        strip_wrappers: SelectionDeletionStripWrappers,
        direction: SelectionDeleteDirection,
    );
}

impl SelectionExecCommandSupport for Selection {
    /// <https://w3c.github.io/editing/docs/execCommand/#delete-the-selection>
    fn delete_the_selection(
        &self,
        cx: &mut js::context::JSContext,
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
                active_range.restore_states_and_values(context_object, overrides);

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
                let br = context_object.create_br_element(cx);
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
            active_range.restore_states_and_values(context_object, overrides);

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
        if start_block.is_ancestor_of(&end_block) {
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
                            let br = context_object.create_br_element(cx);
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
                active_range.restore_states_and_values(context_object, overrides);

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
                // TODO: Restore state
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
            // TODO

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
            // TODO

            // Step 33.9. For each node in nodes to move,
            // append node as the last child of start block, preserving ranges.
            for node in nodes_to_move.iter() {
                // TODO: Preserve ranges
                if start_block.AppendChild(cx, node).is_err() {
                    unreachable!("Must always be able to append");
                }
            }
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
            // TODO

            // Step 34.4. While end block has children,
            // append the first child of end block to start block, preserving ranges.
            loop {
                if let Some(first_child) = end_block.children().nth(0) {
                    // TODO: Preserve ranges
                    if start_block.AppendChild(cx, &first_child).is_err() {
                        unreachable!("Must always be able to append");
                    }
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
        }

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
        // TODO

        // Step 39. If start block has no children, call createElement("br") on the context object and
        // append the result as the last child of start block.
        if start_block.children_count() == 0 {
            let br = context_object.create_br_element(cx);
            if start_block.AppendChild(cx, br.upcast()).is_err() {
                unreachable!("Must always be able to append");
            }
        }

        // Step 40. Remove extraneous line breaks at the end of start block.
        start_block.remove_extraneous_line_breaks_at_the_end_of(cx);

        // Step 41. Restore states and values from overrides.
        active_range.restore_states_and_values(context_object, overrides);
    }
}
