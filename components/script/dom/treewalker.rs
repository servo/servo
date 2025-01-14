/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;

use crate::dom::bindings::callback::ExceptionHandling::Rethrow;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::NodeFilterBinding::{NodeFilter, NodeFilterConstants};
use crate::dom::bindings::codegen::Bindings::TreeWalkerBinding::TreeWalkerMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutDom};
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::script_runtime::CanGc;

// https://dom.spec.whatwg.org/#interface-treewalker
#[dom_struct]
pub(crate) struct TreeWalker {
    reflector_: Reflector,
    root_node: Dom<Node>,
    current_node: MutDom<Node>,
    what_to_show: u32,
    #[ignore_malloc_size_of = "function pointers and Rc<T> are hard"]
    filter: Filter,
    active: Cell<bool>,
}

impl TreeWalker {
    fn new_inherited(root_node: &Node, what_to_show: u32, filter: Filter) -> TreeWalker {
        TreeWalker {
            reflector_: Reflector::new(),
            root_node: Dom::from_ref(root_node),
            current_node: MutDom::new(root_node),
            what_to_show,
            filter,
            active: Cell::new(false),
        }
    }

    pub(crate) fn new_with_filter(
        document: &Document,
        root_node: &Node,
        what_to_show: u32,
        filter: Filter,
    ) -> DomRoot<TreeWalker> {
        reflect_dom_object(
            Box::new(TreeWalker::new_inherited(root_node, what_to_show, filter)),
            document.window(),
            CanGc::note(),
        )
    }

    pub(crate) fn new(
        document: &Document,
        root_node: &Node,
        what_to_show: u32,
        node_filter: Option<Rc<NodeFilter>>,
    ) -> DomRoot<TreeWalker> {
        let filter = match node_filter {
            None => Filter::None,
            Some(jsfilter) => Filter::Dom(jsfilter),
        };
        TreeWalker::new_with_filter(document, root_node, what_to_show, filter)
    }
}

impl TreeWalkerMethods<crate::DomTypeHolder> for TreeWalker {
    // https://dom.spec.whatwg.org/#dom-treewalker-root
    fn Root(&self) -> DomRoot<Node> {
        DomRoot::from_ref(&*self.root_node)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-whattoshow
    fn WhatToShow(&self) -> u32 {
        self.what_to_show
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-filter
    fn GetFilter(&self) -> Option<Rc<NodeFilter>> {
        match self.filter {
            Filter::None => None,
            Filter::Dom(ref nf) => Some(nf.clone()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-currentnode
    fn CurrentNode(&self) -> DomRoot<Node> {
        self.current_node.get()
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-currentnode
    fn SetCurrentNode(&self, node: &Node) {
        self.current_node.set(node);
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-parentnode
    fn ParentNode(&self) -> Fallible<Option<DomRoot<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get();
        // "2. While node is not null and is not root, run these substeps:"
        while !self.is_root_node(&node) {
            // "1. Let node be node's parent."
            match node.GetParentNode() {
                Some(n) => {
                    node = n;
                    // "2. If node is not null and filtering node returns FILTER_ACCEPT,
                    //     then set the currentNode attribute to node, return node."
                    if NodeFilterConstants::FILTER_ACCEPT == self.accept_node(&node)? {
                        self.current_node.set(&node);
                        return Ok(Some(node));
                    }
                },
                None => break,
            }
        }
        // "3. Return null."
        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-firstchild
    fn FirstChild(&self) -> Fallible<Option<DomRoot<Node>>> {
        // "The firstChild() method must traverse children of type first."
        self.traverse_children(|node| node.GetFirstChild(), |node| node.GetNextSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-lastchild
    fn LastChild(&self) -> Fallible<Option<DomRoot<Node>>> {
        // "The lastChild() method must traverse children of type last."
        self.traverse_children(|node| node.GetLastChild(), |node| node.GetPreviousSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-previoussibling
    fn PreviousSibling(&self) -> Fallible<Option<DomRoot<Node>>> {
        // "The nextSibling() method must traverse siblings of type next."
        self.traverse_siblings(|node| node.GetLastChild(), |node| node.GetPreviousSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-nextsibling
    fn NextSibling(&self) -> Fallible<Option<DomRoot<Node>>> {
        // "The previousSibling() method must traverse siblings of type previous."
        self.traverse_siblings(|node| node.GetFirstChild(), |node| node.GetNextSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-previousnode
    fn PreviousNode(&self) -> Fallible<Option<DomRoot<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get();
        // "2. While node is not root, run these substeps:"
        while !self.is_root_node(&node) {
            // "1. Let sibling be the previous sibling of node."
            let mut sibling_op = node.GetPreviousSibling();
            // "2. While sibling is not null, run these subsubsteps:"
            while sibling_op.is_some() {
                // "1. Set node to sibling."
                node = sibling_op.unwrap();
                // "2. Filter node and let result be the return value."
                // "3. While result is not FILTER_REJECT and node has a child,
                //     set node to its last child and then filter node and
                //     set result to the return value."
                // "4. If result is FILTER_ACCEPT, then
                //     set the currentNode attribute to node and return node."
                loop {
                    let result = self.accept_node(&node)?;
                    match result {
                        NodeFilterConstants::FILTER_REJECT => break,
                        _ if node.GetFirstChild().is_some() => node = node.GetLastChild().unwrap(),
                        NodeFilterConstants::FILTER_ACCEPT => {
                            self.current_node.set(&node);
                            return Ok(Some(node));
                        },
                        _ => break,
                    }
                }
                // "5. Set sibling to the previous sibling of node."
                sibling_op = node.GetPreviousSibling()
            }
            // "3. If node is root or node's parent is null, return null."
            if self.is_root_node(&node) || node.GetParentNode().is_none() {
                return Ok(None);
            }
            // "4. Set node to its parent."
            match node.GetParentNode() {
                None =>
                // This can happen if the user set the current node to somewhere
                // outside of the tree rooted at the original root.
                {
                    return Ok(None);
                },
                Some(n) => node = n,
            }
            // "5. Filter node and if the return value is FILTER_ACCEPT, then
            //     set the currentNode attribute to node and return node."
            if NodeFilterConstants::FILTER_ACCEPT == self.accept_node(&node)? {
                self.current_node.set(&node);
                return Ok(Some(node));
            }
        }
        // "6. Return null."
        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-nextnode
    fn NextNode(&self) -> Fallible<Option<DomRoot<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get();
        // "2. Let result be FILTER_ACCEPT."
        let mut result = NodeFilterConstants::FILTER_ACCEPT;
        // "3. Run these substeps:"
        loop {
            // "1. While result is not FILTER_REJECT and node has a child, run these subsubsteps:"
            loop {
                if NodeFilterConstants::FILTER_REJECT == result {
                    break;
                }
                match node.GetFirstChild() {
                    None => break,
                    Some(child) => {
                        // "1. Set node to its first child."
                        node = child;
                        // "2. Filter node and set result to the return value."
                        result = self.accept_node(&node)?;
                        // "3. If result is FILTER_ACCEPT, then
                        //     set the currentNode attribute to node and return node."
                        if NodeFilterConstants::FILTER_ACCEPT == result {
                            self.current_node.set(&node);
                            return Ok(Some(node));
                        }
                    },
                }
            }
            // "2. If a node is following node and is not following root,
            //     set node to the first such node."
            // "Otherwise, return null."
            match self.first_following_node_not_following_root(&node) {
                None => return Ok(None),
                Some(n) => {
                    node = n;
                    // "3. Filter node and set result to the return value."
                    result = self.accept_node(&node)?;
                    // "4. If result is FILTER_ACCEPT, then
                    //     set the currentNode attribute to node and return node."
                    if NodeFilterConstants::FILTER_ACCEPT == result {
                        self.current_node.set(&node);
                        return Ok(Some(node));
                    }
                },
            }
            // "5. Run these substeps again."
        }
    }
}

impl TreeWalker {
    // https://dom.spec.whatwg.org/#concept-traverse-children
    fn traverse_children<F, G>(
        &self,
        next_child: F,
        next_sibling: G,
    ) -> Fallible<Option<DomRoot<Node>>>
    where
        F: Fn(&Node) -> Option<DomRoot<Node>>,
        G: Fn(&Node) -> Option<DomRoot<Node>>,
    {
        // "To **traverse children** of type *type*, run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        let cur = self.current_node.get();

        // "2. Set node to node's first child if type is first, and node's last child if type is last."
        // "3. If node is null, return null."
        let mut node = match next_child(&cur) {
            Some(node) => node,
            None => return Ok(None),
        };

        // 4. Main: Repeat these substeps:
        'main: loop {
            // "1. Filter node and let result be the return value."
            let result = self.accept_node(&node)?;
            match result {
                // "2. If result is FILTER_ACCEPT, then set the currentNode
                //     attribute to node and return node."
                NodeFilterConstants::FILTER_ACCEPT => {
                    self.current_node.set(&node);
                    return Ok(Some(DomRoot::from_ref(&node)));
                },
                // "3. If result is FILTER_SKIP, run these subsubsteps:"
                NodeFilterConstants::FILTER_SKIP => {
                    // "1. Let child be node's first child if type is first,
                    //     and node's last child if type is last."
                    if let Some(child) = next_child(&node) {
                        // "2. If child is not null, set node to child and goto Main."
                        node = child;
                        continue 'main;
                    }
                },
                _ => {},
            }
            // "4. Repeat these subsubsteps:"
            loop {
                // "1. Let sibling be node's next sibling if type is next,
                //     and node's previous sibling if type is previous."
                match next_sibling(&node) {
                    // "2. If sibling is not null,
                    //     set node to sibling and goto Main."
                    Some(sibling) => {
                        node = sibling;
                        continue 'main;
                    },
                    None => {
                        // "3. Let parent be node's parent."
                        match node.GetParentNode() {
                            // "4. If parent is null, parent is root,
                            //     or parent is currentNode attribute's value,
                            //     return null."
                            None => return Ok(None),
                            Some(ref parent)
                                if self.is_root_node(parent) || self.is_current_node(parent) =>
                            {
                                return Ok(None);
                            },
                            // "5. Otherwise, set node to parent."
                            Some(parent) => node = parent,
                        }
                    },
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#concept-traverse-siblings
    fn traverse_siblings<F, G>(
        &self,
        next_child: F,
        next_sibling: G,
    ) -> Fallible<Option<DomRoot<Node>>>
    where
        F: Fn(&Node) -> Option<DomRoot<Node>>,
        G: Fn(&Node) -> Option<DomRoot<Node>>,
    {
        // "To **traverse siblings** of type *type* run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get();
        // "2. If node is root, return null."
        if self.is_root_node(&node) {
            return Ok(None);
        }
        // "3. Run these substeps:"
        loop {
            // "1. Let sibling be node's next sibling if type is next,
            //  and node's previous sibling if type is previous."
            let mut sibling_op = next_sibling(&node);
            // "2. While sibling is not null, run these subsubsteps:"
            while sibling_op.is_some() {
                // "1. Set node to sibling."
                node = sibling_op.unwrap();
                // "2. Filter node and let result be the return value."
                let result = self.accept_node(&node)?;
                // "3. If result is FILTER_ACCEPT, then set the currentNode
                //     attribute to node and return node."
                if NodeFilterConstants::FILTER_ACCEPT == result {
                    self.current_node.set(&node);
                    return Ok(Some(node));
                }

                // "4. Set sibling to node's first child if type is next,
                //     and node's last child if type is previous."
                sibling_op = next_child(&node);
                // "5. If result is FILTER_REJECT or sibling is null,
                //     then set sibling to node's next sibling if type is next,
                //     and node's previous sibling if type is previous."
                match (result, &sibling_op) {
                    (NodeFilterConstants::FILTER_REJECT, _) | (_, &None) => {
                        sibling_op = next_sibling(&node)
                    },
                    _ => {},
                }
            }
            // "3. Set node to its parent."
            match node.GetParentNode() {
                // "4. If node is null or is root, return null."
                None => return Ok(None),
                Some(ref n) if self.is_root_node(n) => return Ok(None),
                // "5. Filter node and if the return value is FILTER_ACCEPT, then return null."
                Some(n) => {
                    node = n;
                    if NodeFilterConstants::FILTER_ACCEPT == self.accept_node(&node)? {
                        return Ok(None);
                    }
                },
            }
            // "6. Run these substeps again."
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-following
    fn first_following_node_not_following_root(&self, node: &Node) -> Option<DomRoot<Node>> {
        // "An object A is following an object B if A and B are in the same tree
        //  and A comes after B in tree order."
        match node.GetNextSibling() {
            None => {
                let mut candidate = DomRoot::from_ref(node);
                while !self.is_root_node(&candidate) && candidate.GetNextSibling().is_none() {
                    // This can return None if the user set the current node to somewhere
                    // outside of the tree rooted at the original root.
                    candidate = candidate.GetParentNode()?;
                }
                if self.is_root_node(&candidate) {
                    None
                } else {
                    candidate.GetNextSibling()
                }
            },
            it => it,
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-filter
    fn accept_node(&self, node: &Node) -> Fallible<u16> {
        // Step 1.
        if self.active.get() {
            return Err(Error::InvalidState);
        }
        // Step 2.
        let n = node.NodeType() - 1;
        // Step 3.
        if (self.what_to_show & (1 << n)) == 0 {
            return Ok(NodeFilterConstants::FILTER_SKIP);
        }
        match self.filter {
            // Step 4.
            Filter::None => Ok(NodeFilterConstants::FILTER_ACCEPT),
            Filter::Dom(ref callback) => {
                // Step 5.
                self.active.set(true);
                // Step 6.
                let result = callback.AcceptNode_(self, node, Rethrow);
                // Step 7.
                self.active.set(false);
                // Step 8.
                result
            },
        }
    }

    fn is_root_node(&self, node: &Node) -> bool {
        Dom::from_ref(node) == self.root_node
    }

    fn is_current_node(&self, node: &Node) -> bool {
        node == &*self.current_node.get()
    }
}

impl Iterator for &TreeWalker {
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<DomRoot<Node>> {
        match self.NextNode() {
            Ok(node) => node,
            Err(_) =>
            // The Err path happens only when a JavaScript
            // NodeFilter throws an exception. This iterator
            // is meant for internal use from Rust code, which
            // will probably be using a native Rust filter,
            // which cannot produce an Err result.
            {
                unreachable!()
            },
        }
    }
}

#[derive(JSTraceable)]
pub(crate) enum Filter {
    None,
    Dom(Rc<NodeFilter>),
}
