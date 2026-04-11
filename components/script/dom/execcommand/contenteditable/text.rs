/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::inheritance::Castable;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;

use crate::dom::bindings::cell::Ref;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::characterdata::CharacterData;
use crate::dom::element::Element;
use crate::dom::html::htmlbrelement::HTMLBRElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::node::Node;
use crate::dom::text::Text;

impl Text {
    /// <https://dom.spec.whatwg.org/#concept-cd-data>
    pub(crate) fn data(&self) -> Ref<'_, String> {
        self.upcast::<CharacterData>().data()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#whitespace-node>
    pub(crate) fn is_whitespace_node(&self) -> bool {
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
    pub(crate) fn is_collapsed_whitespace_node(&self) -> bool {
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
            if parent
                .downcast::<Element>()
                .is_some_and(Element::is_display_none)
            {
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
    pub(crate) fn has_whitespace_and_has_parent_with_whitespace_preserve(
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
            .and_then(|parent_node| parent_node.downcast::<Element>().and_then(Element::style))
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
