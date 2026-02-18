/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use script_bindings::inheritance::Castable;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;
use style::values::specified::box_::DisplayOutside;

use crate::dom::abstractrange::bp_position;
use crate::dom::bindings::cell::Ref;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::inheritance::NodeTypeId;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::element::Element;
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmllielement::HTMLLIElement;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::range::Range;
use crate::dom::selection::Selection;
use crate::dom::text::Text;

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

impl Node {
    /// <https://w3c.github.io/editing/docs/execCommand/#in-the-same-editing-host>
    fn same_editing_host(&self, other: &Node) -> bool {
        // > Two nodes are in the same editing host if the editing host of the first is non-null and the same as the editing host of the second.
        self.editing_host_of()
            .is_some_and(|editing_host| other.editing_host_of() == Some(editing_host))
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#block-node>
    fn is_block_node(&self) -> bool {
        // > A block node is either an Element whose "display" property does not have resolved value "inline" or "inline-block" or "inline-table" or "none",
        if self.downcast::<Element>().is_some_and(|el| {
            !el.style()
                .is_none_or(|style| style.get_box().display.outside() == DisplayOutside::Inline)
        }) {
            return true;
        }
        // > or a document, or a DocumentFragment.
        matches!(
            self.type_id(),
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_)
        )
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#visible>
    fn is_visible(&self) -> bool {
        for parent in self.inclusive_ancestors(ShadowIncluding::Yes) {
            // > excluding any node with an inclusive ancestor Element whose "display" property has resolved value "none".
            if parent.is_display_none() {
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
                node = match node.GetParentNode() {
                    None => return false,
                    Some(node) => node,
                };
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
                node = match node.GetParentNode() {
                    None => return false,
                    Some(node) => node,
                };
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
                    collapse_spaces = has_space_at_offset;
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
                start_node = match start_node.GetParentNode() {
                    None => break,
                    Some(node) => node,
                };
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
}

pub(crate) trait BaseCommand {
    fn execute(&self, selection: &Selection, value: DOMString) -> bool;

    /// <https://w3c.github.io/editing/docs/execCommand/#delete-the-selection>
    fn delete_the_selection(&self, _selection: &Selection, active_range: &Range) {
        // Step 1. If the active range is null, abort these steps and do nothing.
        //
        // Always passed in as argument

        // Step 2. Canonicalize whitespace at the active range's start.
        active_range
            .start_container()
            .canonicalize_whitespace(active_range.start_offset(), true);

        // Step 3. Canonicalize whitespace at the active range's end.
        active_range
            .end_container()
            .canonicalize_whitespace(active_range.end_offset(), true);

        // Step 4. Let (start node, start offset) be the last equivalent point for the active range's start.
        // TODO

        // Step 5. Let (end node, end offset) be the first equivalent point for the active range's end.
        // TODO

        // Step 6. If (end node, end offset) is not after (start node, start offset):
        // TODO

        // Step 7. If start node is a Text node and start offset is 0, set start offset to the index of start node,
        // then set start node to its parent.
        // TODO

        // Step 8. If end node is a Text node and end offset is its length, set end offset to one plus the index of end node,
        // then set end node to its parent.
        // TODO

        // Step 9. Call collapse(start node, start offset) on the context object's selection.
        // TODO

        // Step 10. Call extend(end node, end offset) on the context object's selection.
        // TODO

        // Step 11.
        //
        // This step does not exist in the spec

        // Step 12. Let start block be the active range's start node.
        // TODO

        // Step 13. While start block's parent is in the same editing host and start block is an inline node, set start block to its parent.
        // TODO

        // Step 14. If start block is neither a block node nor an editing host, or "span" is not an allowed child of start block,
        // or start block is a td or th, set start block to null.
        // TODO

        // Step 15. Let end block be the active range's end node.
        // TODO

        // Step 16. While end block's parent is in the same editing host and end block is an inline node, set end block to its parent.
        // TODO

        // Step 17. If end block is neither a block node nor an editing host, or "span" is not an allowed child of end block,
        // or end block is a td or th, set end block to null.
        // TODO

        // Step 18.
        //
        // This step does not exist in the spec

        // Step 19. Record current states and values, and let overrides be the result.
        // TODO

        // Step 20.
        //
        // This step does not exist in the spec

        // Step 21. If start node and end node are the same, and start node is an editable Text node:
        //
        // As per the spec:
        // > NOTE: This whole piece of the algorithm is based on deleteContents() in DOM Range, copy-pasted and then adjusted to fit.
        let _ = active_range.DeleteContents();

        // Step 22. If start node is an editable Text node, call deleteData() on it, with start offset as
        // the first argument and (length of start node − start offset) as the second argument.
        // TODO

        // Step 23. Let node list be a list of nodes, initially empty.
        // TODO

        // Step 24. For each node contained in the active range, append node to node list if the
        // last member of node list (if any) is not an ancestor of node; node is editable;
        // and node is not a thead, tbody, tfoot, tr, th, or td.
        // TODO

        // Step 25. For each node in node list:
        // TODO

        // Step 26. If end node is an editable Text node, call deleteData(0, end offset) on it.
        // TODO

        // Step 27. Canonicalize whitespace at the active range's start, with fix collapsed space false.
        // TODO

        // Step 28. Canonicalize whitespace at the active range's end, with fix collapsed space false.
        // TODO

        // Step 29.
        //
        // This step does not exist in the spec

        // Step 30. If block merging is false, or start block or end block is null, or start block is not
        // in the same editing host as end block, or start block and end block are the same:
        // TODO

        // Step 31. If start block has one child, which is a collapsed block prop, remove its child from it.
        // TODO

        // Step 32. If start block is an ancestor of end block:
        // TODO

        // Step 33. Otherwise, if start block is a descendant of end block:
        // TODO

        // Step 34. Otherwise:
        // TODO

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
        // TODO

        // Step 40. Remove extraneous line breaks at the end of start block.
        // TODO

        // Step 41. Restore states and values from overrides.
        // TODO
    }
}
