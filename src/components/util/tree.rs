/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper functions for garbage collected doubly-linked trees.

/// The basic trait. This function is meant to encapsulate a clonable reference to a tree node.
pub trait TreeNodeRef<N> : Clone {
    /// Borrows this node as immutable.
    fn with_base<R>(&self, callback: &fn(&N) -> R) -> R;

    /// Borrows this node as mutable.
    fn with_mut_base<R>(&self, callback: &fn(&mut N) -> R) -> R;
}

/// The contents of a tree node.
pub trait TreeNode<NR> {
    /// Returns the parent of this node.
    fn parent_node(&self) -> Option<NR>;

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<NR>;

    /// Returns the last child of this node.
    fn last_child(&self) -> Option<NR>;

    /// Returns the previous sibling of this node.
    fn prev_sibling(&self) -> Option<NR>;

    /// Returns the next sibling of this node.
    fn next_sibling(&self) -> Option<NR>;

    /// Sets the parent of this node.
    fn set_parent_node(&mut self, new_parent: Option<NR>);

    /// Sets the first child of this node.
    fn set_first_child(&mut self, new_first_child: Option<NR>);

    /// Sets the last child of this node.
    fn set_last_child(&mut self, new_last_child: Option<NR>);

    /// Sets the previous sibling of this node.
    fn set_prev_sibling(&mut self, new_prev_sibling: Option<NR>);

    /// Sets the next sibling of this node.
    fn set_next_sibling(&mut self, new_next_sibling: Option<NR>);
}

/// A set of helper functions useful for operating on trees.
pub trait TreeUtils {
    /// Returns true if this node is disconnected from the tree or has no children.
    fn is_leaf(&self) -> bool;

    /// Returns the number of children the given node has.
    fn num_children(&self) -> uint;

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&self, new_child: Self);

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node. (FIXME: This is not yet checked.)
    fn remove_child(&self, child: Self);

    /// Iterates over all children of this node.
    fn each_child(&self, callback: &fn(Self) -> bool) -> bool;

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder(&self, callback: &fn(Self) -> bool) -> bool;

    /// Iterates over this node and all its descendants, in postorder.
    fn traverse_postorder(&self, callback: &fn(Self) -> bool) -> bool;

    /// Like traverse_preorder but calls 'prune' first on each node.  If it returns true then we
    /// skip the whole subtree but continue iterating.
    ///
    /// 'prune' is a separate function a) for compatibility with the 'for' protocol,
    /// b) so that the postorder version can still prune before traversing.
    fn traverse_preorder_prune(&self, prune: &fn(&Self) -> bool, callback: &fn(Self) -> bool) -> bool;

    /// Like traverse_postorder but calls 'prune' first on each node.  If it returns true then we
    /// skip the whole subtree but continue iterating.
    ///
    /// NB: 'prune' is called *before* traversing children, even though this is a
    /// postorder traversal.
    fn traverse_postorder_prune(&self, prune: &fn(&Self) -> bool, callback: &fn(Self) -> bool) -> bool;
}

impl<NR:TreeNodeRef<N>,N:TreeNode<NR>> TreeUtils for NR {
    fn is_leaf(&self) -> bool {
        do self.with_base |this_node| {
            this_node.first_child().is_none()
        }
    }

    //TODO: make this O(1)
    fn num_children(&self) -> uint {
        let mut result = 0;
        for self.each_child |_| {
            result += 1;
        }
        result
    }

    fn add_child(&self, new_child: NR) {
        do self.with_mut_base |this_node| {
            do new_child.with_mut_base |new_child_node| {
                assert!(new_child_node.parent_node().is_none());
                assert!(new_child_node.prev_sibling().is_none());
                assert!(new_child_node.next_sibling().is_none());

                match this_node.last_child() {
                    None => this_node.set_first_child(Some(new_child.clone())),
                    Some(last_child) => {
                        do last_child.with_mut_base |last_child_node| {
                            assert!(last_child_node.next_sibling().is_none());
                            last_child_node.set_next_sibling(Some(new_child.clone()));
                            new_child_node.set_prev_sibling(Some(last_child.clone()));
                        }
                    }
                }

                this_node.set_last_child(Some(new_child.clone()));
                new_child_node.set_parent_node(Some((*self).clone()));
            }
        }
    }

    fn remove_child(&self, child: NR) {
        do self.with_mut_base |this_node| {
            do child.with_mut_base |child_node| {
                assert!(child_node.parent_node().is_some());

                match child_node.prev_sibling() {
                    None => this_node.set_first_child(child_node.next_sibling()),
                    Some(prev_sibling) => {
                        do prev_sibling.with_mut_base |prev_sibling_node| {
                            prev_sibling_node.set_next_sibling(child_node.next_sibling());
                        }
                    }
                }

                match child_node.next_sibling() {
                    None => this_node.set_last_child(child_node.prev_sibling()),
                    Some(next_sibling) => {
                        do next_sibling.with_mut_base |next_sibling_node| {
                            next_sibling_node.set_prev_sibling(child_node.prev_sibling());
                        }
                    }
                }

                child_node.set_prev_sibling(None);
                child_node.set_next_sibling(None);
                child_node.set_parent_node(None);
            }
        }
    }

    fn each_child(&self, callback: &fn(NR) -> bool) -> bool {
        let mut maybe_current = self.with_base(|n| n.first_child());
        while !maybe_current.is_none() {
            let current = maybe_current.get_ref().clone();
            if !callback(current.clone()) {
                break;
            }

            maybe_current = current.with_base(|n| n.next_sibling());
        }

        true
    }

    fn traverse_preorder_prune(&self, prune: &fn(&NR) -> bool, callback: &fn(NR) -> bool) -> bool {
        // prune shouldn't mutate, so don't clone
        if prune(self) {
            return true;
        }

        if !callback((*self).clone()) {
            return false;
        }

        for self.each_child |kid| {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !kid.traverse_preorder_prune(|a| prune(a), |a| callback(a)) {
                return false;
            }
        }

        true
    }

    fn traverse_postorder_prune(&self, prune: &fn(&NR) -> bool, callback: &fn(NR) -> bool) -> bool {
        // prune shouldn't mutate, so don't clone
        if prune(self) {
            return true;
        }

        for self.each_child |kid| {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !kid.traverse_postorder_prune(|a| prune(a), |a| callback(a)) {
                return false;
            }
        }

        callback((*self).clone())
    }

    fn traverse_preorder(&self, callback: &fn(NR) -> bool) -> bool {
        self.traverse_preorder_prune(|_| false, callback)
    }

    fn traverse_postorder(&self, callback: &fn(NR) -> bool) -> bool {
        self.traverse_postorder_prune(|_| false, callback)
    }
}

