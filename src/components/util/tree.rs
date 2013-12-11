/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper functions for garbage collected doubly-linked trees.

// Macros to make add_child etc. less painful to write.
// Code outside this module should instead implement TreeNode
// and use its default methods.
macro_rules! get(
    ($node:expr, $fun:ident) => (
        {
            let val: Option<Self> = TreeNodeRef::<Node>::$fun($node);
            val
        }
    )
)

macro_rules! set(
    ($node:expr, $fun:ident, $val:expr) => (
        {
            let val: Option<Self> = $val;
            TreeNodeRef::<Node>::$fun($node, val)
        }
    )
)

pub struct ChildIterator<Ref> {
    priv current: Option<Ref>,
}

impl<Node, Ref: TreeNodeRef<Node>> Iterator<Ref> for ChildIterator<Ref> {
    fn next(&mut self) -> Option<Ref> {
        if self.current.is_none() {
            return None;
        }

        // FIXME: Do we need two clones here?
        let x = self.current.get_ref().clone();
        self.current = TreeNodeRef::<Node>::next_sibling(x.node());
        Some(x.clone())
    }
}

pub struct AncestorIterator<Ref> {
    priv current: Option<Ref>,
}

impl<Node, Ref: TreeNodeRef<Node>> Iterator<Ref> for AncestorIterator<Ref> {
    fn next(&mut self) -> Option<Ref> {
        if self.current.is_none() {
            return None;
        }

        // FIXME: Do we need two clones here?
        let x = self.current.get_ref().clone();
        self.current = TreeNodeRef::<Node>::parent_node(x.node());
        Some(x.clone())
    }
}

// FIXME: Do this without precomputing a vector of refs.
// Easy for preorder; harder for postorder.
pub struct TreeIterator<Ref> {
    priv nodes: ~[Ref],
    priv index: uint,
}

impl<Ref> TreeIterator<Ref> {
    fn new(nodes: ~[Ref]) -> TreeIterator<Ref> {
        TreeIterator {
            nodes: nodes,
            index: 0,
        }
    }
}

impl<Ref: Clone> Iterator<Ref> for TreeIterator<Ref> {
    fn next(&mut self) -> Option<Ref> {
        if self.index >= self.nodes.len() {
            None
        } else {
            let v = self.nodes[self.index].clone();
            self.index += 1;
            Some(v)
        }
    }
}

/// A type implementing TreeNodeRef<Node> is a clonable reference to an underlying
/// node type Node.
///
/// We have to define both ref and node operations in the same trait, which makes
/// the latter more annoying to call (as static methods).  But we provide non-static
/// proxies in trait TreeNode below.
pub trait TreeNodeRef<Node>: Clone {

    // Fundamental operations on refs.

    /// Borrows this node as immutable.
    fn node<'a>(&'a self) -> &'a Node;
    /// Borrows this node as mutable.
    fn mut_node<'a>(&'a self) -> &'a mut Node;

    // Fundamental operations on nodes.

    /// Returns the parent of this node.
    fn parent_node(node: &Node) -> Option<Self>;

    /// Returns the first child of this node.
    fn first_child(node: &Node) -> Option<Self>;

    /// Returns the last child of this node.
    fn last_child(node: &Node) -> Option<Self>;

    /// Returns the previous sibling of this node.
    fn prev_sibling(node: &Node) -> Option<Self>;

    /// Returns the next sibling of this node.
    fn next_sibling(node: &Node) -> Option<Self>;

    /// Sets the parent of this node.
    fn set_parent_node(node: &mut Node, new_parent: Option<Self>);

    /// Sets the first child of this node.
    fn set_first_child(node: &mut Node, new_first_child: Option<Self>);

    /// Sets the last child of this node.
    fn set_last_child(node: &mut Node, new_last_child: Option<Self>);

    /// Sets the previous sibling of this node.
    fn set_prev_sibling(node: &mut Node, new_prev_sibling: Option<Self>);

    /// Sets the next sibling of this node.
    fn set_next_sibling(node: &mut Node, new_next_sibling: Option<Self>);


    // The tree utilities, operating on refs mostly.

    /// Returns true if this node is disconnected from the tree or has no children.
    fn is_leaf(&self) -> bool {
        (get!(self.node(), first_child)).is_none()
    }

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&self, new_child: Self, before: Option<Self>) {
        let this_node = self.mut_node();
        let new_child_node = new_child.mut_node();
        assert!((get!(new_child_node, parent_node)).is_none());
        assert!((get!(new_child_node, prev_sibling)).is_none());
        assert!((get!(new_child_node, next_sibling)).is_none());
        match before {
            Some(before) => {
                let before_node = before.mut_node();
                // XXX Should assert that parent is self.
                assert!((get!(before_node, parent_node)).is_some());
                set!(before_node, set_prev_sibling, Some(new_child.clone()));
                set!(new_child_node, set_next_sibling, Some(before.clone()));
                match get!(before_node, prev_sibling) {
                    None => {
                        // XXX Should assert that before is the first child of
                        //     self.
                        set!(this_node, set_first_child, Some(new_child.clone()));
                    },
                    Some(prev_sibling) => {
                        let prev_sibling_node = prev_sibling.mut_node();
                        set!(prev_sibling_node, set_next_sibling, Some(new_child.clone()));
                        set!(new_child_node, set_prev_sibling, Some(prev_sibling.clone()));
                    },
                }
            },
            None => {
                match get!(this_node, last_child) {
                    None => set!(this_node, set_first_child, Some(new_child.clone())),
                    Some(last_child) => {
                        let last_child_node = last_child.mut_node();
                        assert!((get!(last_child_node, next_sibling)).is_none());
                        set!(last_child_node, set_next_sibling, Some(new_child.clone()));
                        set!(new_child_node, set_prev_sibling, Some(last_child.clone()));
                    }
                }

                set!(this_node, set_last_child, Some(new_child.clone()));
            },
        }

        set!(new_child_node, set_parent_node, Some((*self).clone()));
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node. (FIXME: This is not yet checked.)
    fn remove_child(&self, child: Self) {
        let this_node = self.mut_node();
        let child_node = child.mut_node();
        assert!((get!(child_node, parent_node)).is_some());

        match get!(child_node, prev_sibling) {
            None => set!(this_node, set_first_child, get!(child_node, next_sibling)),
            Some(prev_sibling) => {
                let prev_sibling_node = prev_sibling.mut_node();
                set!(prev_sibling_node, set_next_sibling, get!(child_node, next_sibling));
            }
        }

        match get!(child_node, next_sibling) {
            None => set!(this_node, set_last_child, get!(child_node, prev_sibling)),
            Some(next_sibling) => {
                let next_sibling_node = next_sibling.mut_node();
                set!(next_sibling_node, set_prev_sibling, get!(child_node, prev_sibling));
            }
        }

        set!(child_node, set_prev_sibling, None);
        set!(child_node, set_next_sibling, None);
        set!(child_node, set_parent_node,  None);
    }

    /// Iterates over all children of this node.
    fn children(&self) -> ChildIterator<Self> {
        ChildIterator {
            current: get!(self.node(), first_child),
        }
    }

    /// Iterates over all ancestors of this node.
    fn ancestors(&self) -> AncestorIterator<Self> {
        AncestorIterator {
            current: get!(self.node(), parent_node),
        }
    }

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder(&self) -> TreeIterator<Self> {
        self.traverse_preorder_prune(|_| false)
    }

    /// Iterates over this node and all its descendants, in postorder.
    fn traverse_postorder(&self) -> TreeIterator<Self> {
        self.traverse_postorder_prune(|_| false)
    }

    /// Like traverse_preorder but calls 'prune' first on each node.  If it returns true then we
    /// skip the whole subtree but continue iterating.
    fn traverse_preorder_prune(&self, prune: &fn(&Self) -> bool) -> TreeIterator<Self> {
        let mut nodes = ~[];
        gather(self, &mut nodes, false, prune);
        TreeIterator::new(nodes)
    }

    /// Like traverse_postorder but calls 'prune' first on each node.  If it returns true then we
    /// skip the whole subtree but continue iterating.
    ///
    /// NB: 'prune' is called *before* traversing children, even though this is a
    /// postorder traversal.
    fn traverse_postorder_prune(&self, prune: &fn(&Self) -> bool) -> TreeIterator<Self> {
        let mut nodes = ~[];
        gather(self, &mut nodes, true, prune);
        TreeIterator::new(nodes)
    }

    fn is_element(&self) -> bool;

    fn is_document(&self) -> bool;
}

pub trait TreeNodeRefAsElement<Node, E: ElementLike>: TreeNodeRef<Node> {
    fn with_imm_element_like<R>(&self, f: &fn(&E) -> R) -> R;
}

fn gather<Node, Ref: TreeNodeRef<Node>>(cur: &Ref, refs: &mut ~[Ref],
                                        postorder: bool, prune: &fn(&Ref) -> bool) {
    // prune shouldn't mutate, so don't clone
    if prune(cur) {
        return;
    }

    if !postorder {
        refs.push(cur.clone());
    }
    for kid in cur.children() {
        // FIXME: Work around rust#2202. We should be able to pass the callback directly.
        gather(&kid, refs, postorder, |a| prune(a))
    }
    if postorder {
        refs.push(cur.clone());
    }
}


/// Access the fields of a node without a static TreeNodeRef method call.
/// If you make an impl TreeNodeRef<Node> for Ref then you should also make
/// impl TreeNode<Ref> for Node with an empty body.
pub trait TreeNode<Ref: TreeNodeRef<Self>> {
    /// Returns the parent of this node.
    fn parent_node(&self) -> Option<Ref> {
        TreeNodeRef::<Self>::parent_node(self)
    }

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<Ref> {
        TreeNodeRef::<Self>::first_child(self)
    }

    /// Returns the last child of this node.
    fn last_child(&self) -> Option<Ref> {
        TreeNodeRef::<Self>::last_child(self)
    }

    /// Returns the previous sibling of this node.
    fn prev_sibling(&self) -> Option<Ref> {
        TreeNodeRef::<Self>::prev_sibling(self)
    }

    /// Returns the next sibling of this node.
    fn next_sibling(&self) -> Option<Ref> {
        TreeNodeRef::<Self>::next_sibling(self)
    }

    /// Sets the parent of this node.
    fn set_parent_node(&mut self, new_parent: Option<Ref>) {
        TreeNodeRef::<Self>::set_parent_node(self, new_parent)
    }

    /// Sets the first child of this node.
    fn set_first_child(&mut self, new_first_child: Option<Ref>) {
        TreeNodeRef::<Self>::set_first_child(self, new_first_child)
    }

    /// Sets the last child of this node.
    fn set_last_child(&mut self, new_last_child: Option<Ref>) {
        TreeNodeRef::<Self>::set_last_child(self, new_last_child)
    }

    /// Sets the previous sibling of this node.
    fn set_prev_sibling(&mut self, new_prev_sibling: Option<Ref>) {
        TreeNodeRef::<Self>::set_prev_sibling(self, new_prev_sibling)
    }

    /// Sets the next sibling of this node.
    fn set_next_sibling(&mut self, new_next_sibling: Option<Ref>) {
        TreeNodeRef::<Self>::set_next_sibling(self, new_next_sibling)
    }
}


pub trait ElementLike {
    fn get_local_name<'a>(&'a self) -> &'a str;
    fn get_namespace_url<'a>(&'a self) -> &'a str;
    fn get_attr(&self, ns_url: Option<~str>, name: &str) -> Option<~str>;
    fn get_link(&self) -> Option<~str>;
}
