/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use crate::dom::Node;

/// The context of the binding to tree of a node.
pub(crate) struct BindContext<'a> {
    /// The parent of the inclusive ancestor that was inserted.
    pub(crate) parent: &'a Node,

    /// Whether the tree is connected.
    ///
    /// <https://dom.spec.whatwg.org/#connected>
    pub(crate) tree_connected: bool,

    /// Whether the tree's root is a document.
    ///
    /// <https://dom.spec.whatwg.org/#in-a-document-tree>
    pub(crate) tree_is_in_a_document_tree: bool,

    /// Whether the tree's root is a shadow root
    pub(crate) tree_is_in_a_shadow_tree: bool,

    /// Whether the root of the subtree that is being bound to the parent is a shadow root.
    ///
    /// This implies that all elements whose "bind_to_tree" method are called were already
    /// in a shadow tree beforehand.
    pub(crate) is_shadow_tree: IsShadowTree,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum IsShadowTree {
    Yes,
    No,
}

impl<'a> BindContext<'a> {
    /// Create a new `BindContext` value.
    pub(crate) fn new(parent: &'a Node, is_shadow_tree: IsShadowTree) -> Self {
        BindContext {
            parent,
            tree_connected: parent.is_connected(),
            tree_is_in_a_document_tree: parent.is_in_a_document_tree(),
            tree_is_in_a_shadow_tree: parent.is_in_a_shadow_tree(),
            is_shadow_tree,
        }
    }

    /// Return true iff the tree is inside either a document- or a shadow tree.
    pub(crate) fn is_in_tree(&self) -> bool {
        self.tree_is_in_a_document_tree || self.tree_is_in_a_shadow_tree
    }
}

/// The context of the unbinding from a tree of a node when one of its
/// inclusive ancestors is removed.
pub(crate) struct UnbindContext<'a> {
    /// The index of the inclusive ancestor that was removed.
    index: Cell<Option<u32>>,
    /// The parent of the inclusive ancestor that was removed.
    pub(crate) parent: &'a Node,
    /// The previous sibling of the inclusive ancestor that was removed.
    prev_sibling: Option<&'a Node>,
    /// The next sibling of the inclusive ancestor that was removed.
    pub(crate) next_sibling: Option<&'a Node>,

    /// Whether the tree is connected.
    ///
    /// <https://dom.spec.whatwg.org/#connected>
    pub(crate) tree_connected: bool,

    /// Whether the tree's root is a document.
    ///
    /// <https://dom.spec.whatwg.org/#in-a-document-tree>
    pub(crate) tree_is_in_a_document_tree: bool,

    /// Whether the tree's root is a shadow root
    pub(crate) tree_is_in_a_shadow_tree: bool,
}

impl<'a> UnbindContext<'a> {
    /// Create a new `UnbindContext` value.
    pub(crate) fn new(
        parent: &'a Node,
        prev_sibling: Option<&'a Node>,
        next_sibling: Option<&'a Node>,
        cached_index: Option<u32>,
    ) -> Self {
        UnbindContext {
            index: Cell::new(cached_index),
            parent,
            prev_sibling,
            next_sibling,
            tree_connected: parent.is_connected(),
            tree_is_in_a_document_tree: parent.is_in_a_document_tree(),
            tree_is_in_a_shadow_tree: parent.is_in_a_shadow_tree(),
        }
    }

    /// The index of the inclusive ancestor that was removed from the tree.
    pub(crate) fn index(&self) -> u32 {
        if let Some(index) = self.index.get() {
            return index;
        }
        let index = self.prev_sibling.map_or(0, |sibling| sibling.index() + 1);
        self.index.set(Some(index));
        index
    }
}

/// The context of the moving from a tree of a node when one of its
/// inclusive ancestors is moved.
pub(crate) struct MoveContext<'a> {
    /// The index of the inclusive ancestor that was moved.
    index: Cell<Option<u32>>,
    /// The old parent, if any, of the inclusive ancestor that was moved.
    pub(crate) old_parent: Option<&'a Node>,
    /// The previous sibling of the inclusive ancestor that was moved.
    prev_sibling: Option<&'a Node>,
}

impl<'a> MoveContext<'a> {
    /// Create a new `MoveContext` value.
    pub(crate) fn new(
        old_parent: Option<&'a Node>,
        prev_sibling: Option<&'a Node>,
        cached_index: Option<u32>,
    ) -> Self {
        MoveContext {
            index: Cell::new(cached_index),
            old_parent,
            prev_sibling,
        }
    }

    /// The index of the inclusive ancestor that was moved from the tree.
    pub(crate) fn index(&self) -> u32 {
        if let Some(index) = self.index.get() {
            return index;
        }
        let index = self.prev_sibling.map_or(0, |sibling| sibling.index() + 1);
        self.index.set(Some(index));
        index
    }
}
