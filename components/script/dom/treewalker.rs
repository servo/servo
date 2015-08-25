/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Rethrow;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilterConstants;
use dom::bindings::codegen::Bindings::TreeWalkerBinding;
use dom::bindings::codegen::Bindings::TreeWalkerBinding::TreeWalkerMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::js::{JS, MutHeap};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::Node;
use std::rc::Rc;

// https://dom.spec.whatwg.org/#interface-treewalker
#[dom_struct]
pub struct TreeWalker {
    reflector_: Reflector,
    root_node: JS<Node>,
    current_node: MutHeap<JS<Node>>,
    what_to_show: u32,
    #[ignore_heap_size_of = "function pointers and Rc<T> are hard"]
    filter: Filter
}

impl TreeWalker {
    fn new_inherited(root_node: &Node,
                         what_to_show: u32,
                         filter: Filter) -> TreeWalker {
        TreeWalker {
            reflector_: Reflector::new(),
            root_node: JS::from_ref(root_node),
            current_node: MutHeap::new(JS::from_ref(root_node)),
            what_to_show: what_to_show,
            filter: filter
        }
    }

    pub fn new_with_filter(document: &Document,
                           root_node: &Node,
                           what_to_show: u32,
                           filter: Filter) -> Root<TreeWalker> {
        let window = document.window();
        reflect_dom_object(box TreeWalker::new_inherited(root_node, what_to_show, filter),
                           GlobalRef::Window(window.r()),
                           TreeWalkerBinding::Wrap)
    }

    pub fn new(document: &Document,
               root_node: &Node,
               what_to_show: u32,
               node_filter: Option<Rc<NodeFilter>>) -> Root<TreeWalker> {
        let filter = match node_filter {
            None => Filter::None,
            Some(jsfilter) => Filter::JS(jsfilter)
        };
        TreeWalker::new_with_filter(document, root_node, what_to_show, filter)
    }
}

impl<'a> TreeWalkerMethods for &'a TreeWalker {
    // https://dom.spec.whatwg.org/#dom-treewalker-root
    fn Root(self) -> Root<Node> {
        self.root_node.root()
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-whattoshow
    fn WhatToShow(self) -> u32 {
        self.what_to_show
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-filter
    fn GetFilter(self) -> Option<Rc<NodeFilter>> {
        match self.filter {
            Filter::None => None,
            Filter::JS(ref nf) => Some(nf.clone()),
            Filter::Native(_) => panic!("Cannot convert native node filter to DOM NodeFilter")
        }
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-currentnode
    fn CurrentNode(self) -> Root<Node> {
        self.current_node.get().root()
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-currentnode
    fn SetCurrentNode(self, node: &Node) {
        self.current_node.set(JS::from_ref(node));
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-parentnode
    fn ParentNode(self) -> Fallible<Option<Root<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root();
        // "2. While node is not null and is not root, run these substeps:"
        while !self.is_root_node(node.r()) {
            // "1. Let node be node's parent."
            match node.GetParentNode() {
                Some(n) => {
                    node = n;
                    // "2. If node is not null and filtering node returns FILTER_ACCEPT,
                    //     then set the currentNode attribute to node, return node."
                    if NodeFilterConstants::FILTER_ACCEPT == try!(self.accept_node(node.r())) {
                        self.current_node.set(JS::from_rooted(&node));
                        return Ok(Some(node))
                    }
                },
                None => break,
            }
        }
        // "3. Return null."
        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-firstchild
    fn FirstChild(self) -> Fallible<Option<Root<Node>>> {
        // "The firstChild() method must traverse children of type first."
        self.traverse_children(|node| node.GetFirstChild(),
                               |node| node.GetNextSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-lastchild
    fn LastChild(self) -> Fallible<Option<Root<Node>>> {
        // "The lastChild() method must traverse children of type last."
        self.traverse_children(|node| node.GetLastChild(),
                               |node| node.GetPreviousSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-previoussibling
    fn PreviousSibling(self) -> Fallible<Option<Root<Node>>> {
        // "The nextSibling() method must traverse siblings of type next."
        self.traverse_siblings(|node| node.GetLastChild(),
                               |node| node.GetPreviousSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-nextsibling
    fn NextSibling(self) -> Fallible<Option<Root<Node>>> {
        // "The previousSibling() method must traverse siblings of type previous."
        self.traverse_siblings(|node| node.GetFirstChild(),
                               |node| node.GetNextSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-previousnode
    fn PreviousNode(self) -> Fallible<Option<Root<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root();
        // "2. While node is not root, run these substeps:"
        while !self.is_root_node(node.r()) {
            // "1. Let sibling be the previous sibling of node."
            let mut sibling_op = node.r().GetPreviousSibling();
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
                    let result = try!(self.accept_node(node.r()));
                    match result {
                        NodeFilterConstants::FILTER_REJECT => break,
                        _ if node.GetFirstChild().is_some() =>
                            node = node.GetLastChild().unwrap(),
                        NodeFilterConstants::FILTER_ACCEPT => {
                            self.current_node.set(JS::from_rooted(&node));
                            return Ok(Some(node))
                        },
                        _ => break
                    }
                }
                // "5. Set sibling to the previous sibling of node."
                sibling_op = node.GetPreviousSibling()
            }
            // "3. If node is root or node's parent is null, return null."
            if self.is_root_node(node.r()) || node.GetParentNode().is_none() {
                return Ok(None)
            }
            // "4. Set node to its parent."
            match node.r().GetParentNode() {
                None =>
                    // This can happen if the user set the current node to somewhere
                    // outside of the tree rooted at the original root.
                    return Ok(None),
                Some(n) => node = n
            }
            // "5. Filter node and if the return value is FILTER_ACCEPT, then
            //     set the currentNode attribute to node and return node."
            if NodeFilterConstants::FILTER_ACCEPT == try!(self.accept_node(node.r())) {
                self.current_node.set(JS::from_rooted(&node));
                return Ok(Some(node))
            }
        }
        // "6. Return null."
        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-nextnode
    fn NextNode(self) -> Fallible<Option<Root<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root();
        // "2. Let result be FILTER_ACCEPT."
        let mut result = NodeFilterConstants::FILTER_ACCEPT;
        // "3. Run these substeps:"
        loop {
            // "1. While result is not FILTER_REJECT and node has a child, run these subsubsteps:"
            loop {
                if NodeFilterConstants::FILTER_REJECT == result {
                    break;
                }
                match node.r().GetFirstChild() {
                    None => break,
                    Some (child) => {
                        // "1. Set node to its first child."
                        node = child;
                        // "2. Filter node and set result to the return value."
                        result = try!(self.accept_node(node.r()));
                        // "3. If result is FILTER_ACCEPT, then
                        //     set the currentNode attribute to node and return node."
                        if NodeFilterConstants::FILTER_ACCEPT == result {
                            self.current_node.set(JS::from_rooted(&node));
                            return Ok(Some(node))
                        }
                    }
                }
            }
            // "2. If a node is following node and is not following root,
            //     set node to the first such node."
            // "Otherwise, return null."
            match self.first_following_node_not_following_root(node.r()) {
                None => return Ok(None),
                Some(n) => {
                    node = n;
                    // "3. Filter node and set result to the return value."
                    result = try!(self.accept_node(node.r()));
                    // "4. If result is FILTER_ACCEPT, then
                    //     set the currentNode attribute to node and return node."
                    if NodeFilterConstants::FILTER_ACCEPT == result {
                        self.current_node.set(JS::from_rooted(&node));
                        return Ok(Some(node))
                    }
                }
            }
            // "5. Run these substeps again."
        }
    }
}

type NodeAdvancer<'a> = Fn(&Node) -> Option<Root<Node>> + 'a;

trait PrivateTreeWalkerHelpers {
    fn traverse_children<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Root<Node>>>
        where F: Fn(&Node) -> Option<Root<Node>>,
              G: Fn(&Node) -> Option<Root<Node>>;
    fn traverse_siblings<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Root<Node>>>
        where F: Fn(&Node) -> Option<Root<Node>>,
              G: Fn(&Node) -> Option<Root<Node>>;
    fn is_root_node(self, node: &Node) -> bool;
    fn is_current_node(self, node: &Node) -> bool;
    fn first_following_node_not_following_root(self, node: &Node)
                                               -> Option<Root<Node>>;
    fn accept_node(self, node: &Node) -> Fallible<u16>;
}

impl<'a> PrivateTreeWalkerHelpers for &'a TreeWalker {
    // https://dom.spec.whatwg.org/#concept-traverse-children
    fn traverse_children<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Root<Node>>>
        where F: Fn(&Node) -> Option<Root<Node>>,
              G: Fn(&Node) -> Option<Root<Node>>
    {
        // "To **traverse children** of type *type*, run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        let cur = self.current_node.get().root();

        // "2. Set node to node's first child if type is first, and node's last child if type is last."
        // "3. If node is null, return null."
        let mut node = match next_child(cur.r()) {
            Some(node) => node,
            None => return Ok(None),
        };

        // 4. Main: Repeat these substeps:
        'main: loop {
            // "1. Filter node and let result be the return value."
            let result = try!(self.accept_node(node.r()));
            match result {
                // "2. If result is FILTER_ACCEPT, then set the currentNode
                //     attribute to node and return node."
                NodeFilterConstants::FILTER_ACCEPT => {
                    self.current_node.set(JS::from_rooted(&node));
                    return Ok(Some(Root::from_ref(node.r())))
                },
                // "3. If result is FILTER_SKIP, run these subsubsteps:"
                NodeFilterConstants::FILTER_SKIP => {
                    // "1. Let child be node's first child if type is first,
                    //     and node's last child if type is last."
                    match next_child(node.r()) {
                        // "2. If child is not null, set node to child and goto Main."
                        Some(child) => {
                            node = child;
                            continue 'main
                        },
                        None => {}
                    }
                },
                _ => {}
            }
            // "4. Repeat these subsubsteps:"
            loop {
                // "1. Let sibling be node's next sibling if type is next,
                //     and node's previous sibling if type is previous."
                match next_sibling(node.r()) {
                    // "2. If sibling is not null,
                    //     set node to sibling and goto Main."
                    Some(sibling) => {
                        node = sibling;
                        continue 'main
                    },
                    None => {
                        // "3. Let parent be node's parent."
                        match node.GetParentNode() {
                            // "4. If parent is null, parent is root,
                            //     or parent is currentNode attribute's value,
                            //     return null."
                            None => return Ok(None),
                            Some(ref parent) if self.is_root_node(parent.r())
                                            || self.is_current_node(parent.r()) =>
                                             return Ok(None),
                            // "5. Otherwise, set node to parent."
                            Some(parent) => node = parent
                        }
                    }
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#concept-traverse-siblings
    fn traverse_siblings<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Root<Node>>>
        where F: Fn(&Node) -> Option<Root<Node>>,
              G: Fn(&Node) -> Option<Root<Node>>
    {
        // "To **traverse siblings** of type *type* run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root();
        // "2. If node is root, return null."
        if self.is_root_node(node.r()) {
            return Ok(None)
        }
        // "3. Run these substeps:"
        loop {
            // "1. Let sibling be node's next sibling if type is next,
            //  and node's previous sibling if type is previous."
            let mut sibling_op = next_sibling(node.r());
            // "2. While sibling is not null, run these subsubsteps:"
            while sibling_op.is_some() {
                // "1. Set node to sibling."
                node = sibling_op.unwrap();
                // "2. Filter node and let result be the return value."
                let result = try!(self.accept_node(node.r()));
                // "3. If result is FILTER_ACCEPT, then set the currentNode
                //     attribute to node and return node."
                if NodeFilterConstants::FILTER_ACCEPT == result {
                    self.current_node.set(JS::from_rooted(&node));
                    return Ok(Some(node))
                }

                // "4. Set sibling to node's first child if type is next,
                //     and node's last child if type is previous."
                sibling_op = next_child(node.r());
                // "5. If result is FILTER_REJECT or sibling is null,
                //     then set sibling to node's next sibling if type is next,
                //     and node's previous sibling if type is previous."
                match (result, &sibling_op) {
                    (NodeFilterConstants::FILTER_REJECT, _)
                    | (_, &None) => sibling_op = next_sibling(node.r()),
                    _ => {}
                }
            }
            // "3. Set node to its parent."
            match node.GetParentNode() {
                // "4. If node is null or is root, return null."
                None => return Ok(None),
                Some(ref n) if self.is_root_node(n.r()) => return Ok(None),
                // "5. Filter node and if the return value is FILTER_ACCEPT, then return null."
                Some(n) => {
                    node = n;
                    if NodeFilterConstants::FILTER_ACCEPT == try!(self.accept_node(node.r())) {
                        return Ok(None)
                    }
                }
            }
            // "6. Run these substeps again."
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-following
    fn first_following_node_not_following_root(self, node: &Node)
                                               -> Option<Root<Node>> {
        // "An object A is following an object B if A and B are in the same tree
        //  and A comes after B in tree order."
        match node.GetNextSibling() {
            None => {
                let mut candidate = Root::from_ref(node);
                while !self.is_root_node(candidate.r()) && candidate.GetNextSibling().is_none() {
                    match candidate.GetParentNode() {
                        None =>
                            // This can happen if the user set the current node to somewhere
                            // outside of the tree rooted at the original root.
                            return None,
                        Some(n) => candidate = n
                    }
                }
                if self.is_root_node(candidate.r()) {
                    None
                } else {
                    candidate.GetNextSibling()
                }
            },
            it => it
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-filter
    fn accept_node(self, node: &Node) -> Fallible<u16> {
        // "To filter node run these steps:"
        // "1. Let n be node's nodeType attribute value minus 1."
        let n = node.NodeType() - 1;
        // "2. If the nth bit (where 0 is the least significant bit) of whatToShow is not set,
        //     return FILTER_SKIP."
        if (self.what_to_show & (1 << n)) == 0 {
            return Ok(NodeFilterConstants::FILTER_SKIP)
        }
        // "3. If filter is null, return FILTER_ACCEPT."
        // "4. Let result be the return value of invoking filter."
        // "5. If an exception was thrown, re-throw the exception."
        // "6. Return result."
        match self.filter {
            Filter::None => Ok(NodeFilterConstants::FILTER_ACCEPT),
            Filter::Native(f) => Ok((f)(node)),
            Filter::JS(ref callback) => callback.AcceptNode_(self, node, Rethrow)
        }
    }

    fn is_root_node(self, node: &Node) -> bool {
        JS::from_ref(node) == self.root_node
    }

    fn is_current_node(self, node: &Node) -> bool {
        JS::from_ref(node) == self.current_node.get()
    }
}

impl<'a> Iterator for &'a TreeWalker {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        match self.NextNode() {
            Ok(node) => node,
            Err(_) =>
                // The Err path happens only when a JavaScript
                // NodeFilter throws an exception. This iterator
                // is meant for internal use from Rust code, which
                // will probably be using a native Rust filter,
                // which cannot produce an Err result.
                unreachable!()
       }
    }
}

#[derive(JSTraceable)]
pub enum Filter {
    None,
    Native(fn (node: &Node) -> u16),
    JS(Rc<NodeFilter>)
}
