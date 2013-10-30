/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Defines traversals for DOM nodes. These are currently sequential, but parallel traversals are
//! intended in the future.

use dom::node::{AbstractNode, LayoutView};

use servo_util::tree::TreeNodeRef;

/// A bottom-up traversal.
pub trait PostorderNodeTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&mut self, node: AbstractNode<LayoutView>) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune(&mut self, _node: AbstractNode<LayoutView>) -> bool {
        false
    }
}

pub trait NodeTraversalMethods {
    fn traverse_postorder<T:PostorderNodeTraversal>(self, traversal: &mut T) -> bool;
}

impl NodeTraversalMethods for AbstractNode<LayoutView> {
    fn traverse_postorder<T:PostorderNodeTraversal>(self, traversal: &mut T) -> bool {
        if traversal.should_prune(self) {
            return true
        }

        for kid in self.children() {
            if !kid.traverse_postorder(traversal) {
                return false
            }
        }

        traversal.process(self)
    }
}

