/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(unsafe_code)]
#![deny(missing_docs)]

use layout_api::DangerousStyleNode;
use script_bindings::error::Fallible;
use servo_arc::Arc;
use style;
use style::dom::{NodeInfo, TNode};
use style::dom_apis::{MayUseInvalidation, SelectorQuery, query_selector};
use style::selector_parser::SelectorParser;
use style::stylesheets::UrlExtraData;
use url::Url;

use super::{ServoDangerousStyleDocument, ServoDangerousStyleShadowRoot};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::LayoutDom;
use crate::dom::node::{Node, NodeFlags};
use crate::layout_dom::{ServoDangerousStyleElement, ServoLayoutNode};

/// A wrapper around [`LayoutDom<_, Node>`] to be used with `stylo` and `selectors`.
///
/// Note: This should only be used for `stylo` or `selectors interaction.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct ServoDangerousStyleNode<'dom> {
    pub(crate) node: LayoutDom<'dom, Node>,
}

unsafe impl Send for ServoDangerousStyleNode<'_> {}
unsafe impl Sync for ServoDangerousStyleNode<'_> {}

impl<'dom> ServoDangerousStyleNode<'dom> {
    /// <https://dom.spec.whatwg.org/#scope-match-a-selectors-string>
    pub(crate) fn scope_match_a_selectors_string<Query>(
        self,
        document_url: Arc<Url>,
        selector: &str,
    ) -> Fallible<Query::Output>
    where
        Query: SelectorQuery<ServoDangerousStyleElement<'dom>>,
        Query::Output: Default,
    {
        let mut result = Query::Output::default();

        // Step 1. Let selector be the result of parse a selector selectors.
        let selector_or_error =
            SelectorParser::parse_author_origin_no_namespace(selector, &UrlExtraData(document_url));

        // Step 2. If selector is failure, then throw a "SyntaxError" DOMException.
        let Ok(selector_list) = selector_or_error else {
            return Err(Error::Syntax(None));
        };

        // Step 3. Return the result of match a selector against a tree with selector
        // and node’s root using scoping root node.
        query_selector::<ServoDangerousStyleElement<'dom>, Query>(
            self,
            &selector_list,
            &mut result,
            MayUseInvalidation::No,
        );

        Ok(result)
    }
}

impl<'dom> From<LayoutDom<'dom, Node>> for ServoDangerousStyleNode<'dom> {
    fn from(node: LayoutDom<'dom, Node>) -> Self {
        Self { node }
    }
}

impl<'dom> DangerousStyleNode<'dom> for ServoDangerousStyleNode<'dom> {
    type ConcreteLayoutNode = ServoLayoutNode<'dom>;

    fn layout_node(&self) -> Self::ConcreteLayoutNode {
        self.node.into()
    }
}

impl NodeInfo for ServoDangerousStyleNode<'_> {
    fn is_element(&self) -> bool {
        self.node.is_element_for_layout()
    }

    fn is_text_node(&self) -> bool {
        self.node.is_text_node_for_layout()
    }
}

impl<'dom> TNode for ServoDangerousStyleNode<'dom> {
    type ConcreteDocument = ServoDangerousStyleDocument<'dom>;
    type ConcreteElement = ServoDangerousStyleElement<'dom>;
    type ConcreteShadowRoot = ServoDangerousStyleShadowRoot<'dom>;

    fn parent_node(&self) -> Option<Self> {
        self.node.parent_node_ref().map(Into::into)
    }

    fn first_child(&self) -> Option<Self> {
        self.node.first_child_ref().map(Into::into)
    }

    fn last_child(&self) -> Option<Self> {
        self.node.last_child_ref().map(Into::into)
    }

    fn prev_sibling(&self) -> Option<Self> {
        self.node.prev_sibling_ref().map(Into::into)
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling_ref().map(Into::into)
    }

    fn owner_doc(&self) -> Self::ConcreteDocument {
        self.node.owner_doc_for_layout().into()
    }

    fn traversal_parent(&self) -> Option<ServoDangerousStyleElement<'dom>> {
        Some(self.node.traversal_parent()?.into())
    }

    fn opaque(&self) -> style::dom::OpaqueNode {
        self.node.opaque()
    }

    fn debug_id(self) -> usize {
        self.opaque().0
    }

    fn as_element(&self) -> Option<ServoDangerousStyleElement<'dom>> {
        Some(self.node.downcast()?.into())
    }

    fn as_document(&self) -> Option<ServoDangerousStyleDocument<'dom>> {
        self.node.downcast().map(Into::into)
    }

    fn as_shadow_root(&self) -> Option<ServoDangerousStyleShadowRoot<'dom>> {
        self.node.downcast().map(Into::into)
    }

    fn is_in_document(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_IN_A_DOCUMENT_TREE) }
    }
}
