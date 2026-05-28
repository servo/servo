/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::NoGC;
use rustc_hash::FxHashSet;
use script_bindings::root::Dom;
use servo_config::pref;

use crate::dom::Node;

#[derive(Clone, Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct AccessibilityData {
    /// Nodes which have been unbound from the DOM but may not yet have been removed from the
    /// accessibility tree. This is cleared after each reflow.
    rooted_nodes: FxHashSet<Dom<Node>>,
}

impl AccessibilityData {
    /// Root a node which has been removed from the DOM but which may still have an associated
    /// accessibility tree node. It will be unrooted after the next reflow, since the accessibility
    /// tree is updated as part of the reflow process.
    ///
    /// Longer explanation:
    /// - The accessibility tree doesn't hold strong references to DOM nodes, but uses
    ///   [`OpaqueNode`]s as a way of mapping from an incoming DOM node to an existing accessibility
    ///   tree node. This allows us to cache previously computed accessibility data, and update it
    ///   based on the current DOM node state, which is passed in to the update function.
    /// - If a DOM node is garbage collected before its corresponding node is removed from the
    ///   accessibility tree, there is a risk that another new DOM node may be created at the same
    ///   memory address, causing it to have an identical `OpaqueNode`. If this `OpaqueNode` was
    ///   used to look up a node in the accessibility tree, we would get the stale accessibility
    ///   node corresponding to the node which was removed.
    /// - A DOM node is prevented from being garbage collected while it's connected to the document;
    ///   it's kept alive by strong references in its parent, child and/or sibling [`Node`]s (and in
    ///   the case of the document itself, by a strong reference in the [`Window`]). See
    ///   [`Node::first_child`], [`Node::next_sibling`], etc.
    /// - After a node is removed from the tree, those strong references are removed, and it _may_
    ///   become a candidate for GC if its DOM object isn't held (directly or indirectly) in script
    ///   and it isn't immediately inserted elsewhere in the DOM.
    /// - To make sure the node isn't GCed before the next accessibility update occurs, we
    ///   temporarily root it here in between its removal from the tree and the subsequent reflow.
    /// - During reflow, the accessibility tree is updated, and all stale accessibility nodes are
    ///   removed.
    /// - After reflow, we can safely un-root these nodes by dropping all the strong references
    ///   being held here, and allow them to potentially be GCed.
    ///   See [`Self::unroot_all_nodes_for_accessibility()`].
    pub(crate) fn root_removed_node_for_accessibility(
        &mut self,
        _no_gc: &NoGC,
        node_to_root: &Node,
    ) {
        debug_assert!(pref!(accessibility_enabled));

        self.rooted_nodes.insert(Dom::from_ref(node_to_root));
    }

    /// Clear all nodes which were rooted using [`Self::root_removed_node_for_accessibility()`].
    /// This should be called at the end of reflow.
    pub(crate) fn unroot_all_nodes_for_accessibility(&mut self) {
        debug_assert!(pref!(accessibility_enabled));

        self.rooted_nodes.clear();
    }
}
