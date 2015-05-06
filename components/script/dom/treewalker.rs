/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Rethrow;
use dom::bindings::codegen::Bindings::TreeWalkerBinding;
use dom::bindings::codegen::Bindings::TreeWalkerBinding::TreeWalkerMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilterConstants;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, MutHeap, OptionalRootable, Rootable};
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::Node;

// https://dom.spec.whatwg.org/#interface-treewalker
#[dom_struct]
pub struct TreeWalker {
    reflector_: Reflector,
    root_node: JS<Node>,
    current_node: MutHeap<JS<Node>>,
    what_to_show: u32,
    filter: Filter
}

impl TreeWalker {
    fn new_inherited(root_node: JSRef<Node>,
                         what_to_show: u32,
                         filter: Filter) -> TreeWalker {
        TreeWalker {
            reflector_: Reflector::new(),
            root_node: JS::from_rooted(root_node),
            current_node: MutHeap::new(JS::from_rooted(root_node)),
            what_to_show: what_to_show,
            filter: filter
        }
    }

    pub fn new_with_filter(document: JSRef<Document>,
                           root_node: JSRef<Node>,
                           what_to_show: u32,
                           filter: Filter) -> Temporary<TreeWalker> {
        let window = document.window().root();
        reflect_dom_object(box TreeWalker::new_inherited(root_node, what_to_show, filter),
                           GlobalRef::Window(window.r()),
                           TreeWalkerBinding::Wrap)
    }

    pub fn new(document: JSRef<Document>,
               root_node: JSRef<Node>,
               what_to_show: u32,
               node_filter: Option<NodeFilter>) -> Temporary<TreeWalker> {
        let filter = match node_filter {
            None => Filter::None,
            Some(jsfilter) => Filter::JS(jsfilter)
        };
        TreeWalker::new_with_filter(document, root_node, what_to_show, filter)
    }
}

impl<'a> TreeWalkerMethods for JSRef<'a, TreeWalker> {
    // https://dom.spec.whatwg.org/#dom-treewalker-root
    fn Root(self) -> Temporary<Node> {
        Temporary::from_rooted(self.root_node)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-whattoshow
    fn WhatToShow(self) -> u32 {
        self.what_to_show
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-filter
    fn GetFilter(self) -> Option<NodeFilter> {
        match self.filter {
            Filter::None => None,
            Filter::JS(nf) => Some(nf),
            Filter::Native(_) => panic!("Cannot convert native node filter to DOM NodeFilter")
        }
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-currentnode
    fn CurrentNode(self) -> Temporary<Node> {
        Temporary::from_rooted(self.current_node.get())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-currentnode
    fn SetCurrentNode(self, node: JSRef<Node>) {
        self.current_node.set(JS::from_rooted(node));
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-parentnode
    fn ParentNode(self) -> Fallible<Option<Temporary<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().get_unsound_ref_forever();
        // "2. While node is not null and is not root, run these substeps:"
        while !self.is_root_node(node) {
            // "1. Let node be node's parent."
            match node.GetParentNode() {
                Some(n) => {
                    node = n.root().get_unsound_ref_forever();
                    // "2. If node is not null and filtering node returns FILTER_ACCEPT,
                    //     then set the currentNode attribute to node, return node."
                    match try!(self.accept_node(node)) {
                        NodeFilterConstants::FILTER_ACCEPT => {
                            self.current_node.set(JS::from_rooted(node));
                            return Ok(Some(Temporary::from_rooted(node)))
                        },
                        _ => {}
                    }
                },
                None => break,
            }
        }
        // "3. Return null."
        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-firstchild
    fn FirstChild(self) -> Fallible<Option<Temporary<Node>>> {
        // "The firstChild() method must traverse children of type first."
        self.traverse_children(|node| node.GetFirstChild(),
                               |node| node.GetNextSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-lastchild
    fn LastChild(self) -> Fallible<Option<Temporary<Node>>> {
        // "The lastChild() method must traverse children of type last."
        self.traverse_children(|node| node.GetLastChild(),
                               |node| node.GetPreviousSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-previoussibling
    fn PreviousSibling(self) -> Fallible<Option<Temporary<Node>>> {
        // "The nextSibling() method must traverse siblings of type next."
        self.traverse_siblings(|node| node.GetLastChild(),
                               |node| node.GetPreviousSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-nextsibling
    fn NextSibling(self) -> Fallible<Option<Temporary<Node>>> {
        // "The previousSibling() method must traverse siblings of type previous."
        self.traverse_siblings(|node| node.GetFirstChild(),
                               |node| node.GetNextSibling())
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-previousnode
    fn PreviousNode(self) -> Fallible<Option<Temporary<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().get_unsound_ref_forever();
        // "2. While node is not root, run these substeps:"
        while !self.is_root_node(node) {
            // "1. Let sibling be the previous sibling of node."
            let mut sibling_op = node.GetPreviousSibling();
            // "2. While sibling is not null, run these subsubsteps:"
            while sibling_op.is_some() {
                // "1. Set node to sibling."
                node = sibling_op.unwrap().root().get_unsound_ref_forever();
                // "2. Filter node and let result be the return value."
                // "3. While result is not FILTER_REJECT and node has a child,
                //     set node to its last child and then filter node and
                //     set result to the return value."
                // "4. If result is FILTER_ACCEPT, then
                //     set the currentNode attribute to node and return node."
                loop {
                    let result = try!(self.accept_node(node));
                    match result {
                        NodeFilterConstants::FILTER_REJECT => break,
                        _ if node.GetFirstChild().is_some() =>
                            node = node.GetLastChild().unwrap().root()
                                       .get_unsound_ref_forever(),
                        NodeFilterConstants::FILTER_ACCEPT => {
                            self.current_node.set(JS::from_rooted(node));
                            return Ok(Some(Temporary::from_rooted(node)))
                        },
                        _ => break
                    }
                }
                // "5. Set sibling to the previous sibling of node."
                sibling_op = node.GetPreviousSibling()
            }
            // "3. If node is root or node's parent is null, return null."
            if self.is_root_node(node) || node.GetParentNode().is_none() {
                return Ok(None)
            }
            // "4. Set node to its parent."
            match node.GetParentNode() {
                None =>
                    // This can happen if the user set the current node to somewhere
                    // outside of the tree rooted at the original root.
                    return Ok(None),
                Some(n) => node = n.root().get_unsound_ref_forever()
            }
            // "5. Filter node and if the return value is FILTER_ACCEPT, then
            //     set the currentNode attribute to node and return node."
            match try!(self.accept_node(node)) {
                NodeFilterConstants::FILTER_ACCEPT => {
                    self.current_node.set(JS::from_rooted(node));
                    return Ok(Some(Temporary::from_rooted(node)))
                },
                _ => {}
            }
        }
        // "6. Return null."
        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-treewalker-nextnode
    fn NextNode(self) -> Fallible<Option<Temporary<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().get_unsound_ref_forever();
        // "2. Let result be FILTER_ACCEPT."
        let mut result = NodeFilterConstants::FILTER_ACCEPT;
        // "3. Run these substeps:"
        loop {
            // "1. While result is not FILTER_REJECT and node has a child, run these subsubsteps:"
            loop {
                match result {
                    NodeFilterConstants::FILTER_REJECT => break,
                    _ => {}
                }
                match node.GetFirstChild() {
                    None => break,
                    Some (child) => {
                        // "1. Set node to its first child."
                        node = child.root().get_unsound_ref_forever();
                        // "2. Filter node and set result to the return value."
                        result = try!(self.accept_node(node));
                        // "3. If result is FILTER_ACCEPT, then
                        //     set the currentNode attribute to node and return node."
                        match result {
                            NodeFilterConstants::FILTER_ACCEPT => {
                                self.current_node.set(JS::from_rooted(node));
                                return Ok(Some(Temporary::from_rooted(node)))
                            },
                            _ => {}
                        }
                    }
                }
            }
            // "2. If a node is following node and is not following root,
            //     set node to the first such node."
            // "Otherwise, return null."
            match self.first_following_node_not_following_root(node) {
                None => return Ok(None),
                Some(n) => {
                    node = n.root().get_unsound_ref_forever();
                    // "3. Filter node and set result to the return value."
                    result = try!(self.accept_node(node));
                    // "4. If result is FILTER_ACCEPT, then
                    //     set the currentNode attribute to node and return node."
                    match result {
                        NodeFilterConstants::FILTER_ACCEPT => {
                            self.current_node.set(JS::from_rooted(node));
                            return Ok(Some(Temporary::from_rooted(node)))
                        },
                        _ => {}
                    }
                }
            }
            // "5. Run these substeps again."
        }
    }
}

type NodeAdvancer<'a> = Fn(JSRef<'a, Node>) -> Option<Temporary<Node>> + 'a;

trait PrivateTreeWalkerHelpers {
    fn traverse_children<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Temporary<Node>>>
        where F: Fn(JSRef<Node>) -> Option<Temporary<Node>>,
              G: Fn(JSRef<Node>) -> Option<Temporary<Node>>;
    fn traverse_siblings<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Temporary<Node>>>
        where F: Fn(JSRef<Node>) -> Option<Temporary<Node>>,
              G: Fn(JSRef<Node>) -> Option<Temporary<Node>>;
    fn is_root_node(self, node: JSRef<Node>) -> bool;
    fn is_current_node(self, node: JSRef<Node>) -> bool;
    fn first_following_node_not_following_root(self, node: JSRef<Node>)
                                               -> Option<Temporary<Node>>;
    fn accept_node(self, node: JSRef<Node>) -> Fallible<u16>;
}

impl<'a> PrivateTreeWalkerHelpers for JSRef<'a, TreeWalker> {
    // https://dom.spec.whatwg.org/#concept-traverse-children
    fn traverse_children<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Temporary<Node>>>
        where F: Fn(JSRef<Node>) -> Option<Temporary<Node>>,
              G: Fn(JSRef<Node>) -> Option<Temporary<Node>>
    {
        // "To **traverse children** of type *type*, run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        // "2. Set node to node's first child if type is first, and node's last child if type is last."
        let cur = self.current_node.get().root();
        let mut node_op: Option<JSRef<Node>> = next_child(cur.r()).map(|node| node.root().get_unsound_ref_forever());

        // 3. Main: While node is not null, run these substeps:
        'main: loop {
            match node_op {
                None => break,
                Some(node) => {
                    // "1. Filter node and let result be the return value."
                    let result = try!(self.accept_node(node));
                    match result {
                        // "2. If result is FILTER_ACCEPT, then set the currentNode
                        //     attribute to node and return node."
                        NodeFilterConstants::FILTER_ACCEPT => {
                            self.current_node.set(JS::from_rooted(node));
                            return Ok(Some(Temporary::from_rooted(node)))
                        },
                        // "3. If result is FILTER_SKIP, run these subsubsteps:"
                        NodeFilterConstants::FILTER_SKIP => {
                            // "1. Let child be node's first child if type is first,
                            //     and node's last child if type is last."
                            match next_child(node) {
                                // "2. If child is not null, set node to child and goto Main."
                                Some(child) => {
                                    node_op = Some(child.root().get_unsound_ref_forever());
                                    continue 'main
                                },
                                None => {}
                            }
                        },
                        _ => {}
                    }
                    // "4. While node is not null, run these substeps:"
                    loop {
                        match node_op {
                            None => break,
                            Some(node) => {
                                // "1. Let sibling be node's next sibling if type is next,
                                //     and node's previous sibling if type is previous."
                                match next_sibling(node) {
                                    // "2. If sibling is not null,
                                    //     set node to sibling and goto Main."
                                    Some(sibling) => {
                                        node_op = Some(sibling.root().get_unsound_ref_forever());
                                        continue 'main
                                    },
                                    None => {
                                        // "3. Let parent be node's parent."
                                        match node.GetParentNode().map(|p| p.root().get_unsound_ref_forever()) {
                                            // "4. If parent is null, parent is root,
                                            //     or parent is currentNode attribute's value,
                                            //     return null."
                                            None => return Ok(None),
                                            Some(parent) if self.is_root_node(parent)
                                                            || self.is_current_node(parent) =>
                                                             return Ok(None),
                                            // "5. Otherwise, set node to parent."
                                            Some(parent) => node_op = Some(parent)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // "4. Return null."
        Ok(None)
    }

    // https://dom.spec.whatwg.org/#concept-traverse-siblings
    fn traverse_siblings<F, G>(self,
                               next_child: F,
                               next_sibling: G)
                               -> Fallible<Option<Temporary<Node>>>
        where F: Fn(JSRef<Node>) -> Option<Temporary<Node>>,
              G: Fn(JSRef<Node>) -> Option<Temporary<Node>>
    {
        // "To **traverse siblings** of type *type* run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().get_unsound_ref_forever();
        // "2. If node is root, return null."
        if self.is_root_node(node) {
            return Ok(None)
        }
        // "3. Run these substeps:"
        loop {
            // "1. Let sibling be node's next sibling if type is next,
            //  and node's previous sibling if type is previous."
            let mut sibling_op = next_sibling(node);
            // "2. While sibling is not null, run these subsubsteps:"
            while sibling_op.is_some() {
                // "1. Set node to sibling."
                node = sibling_op.unwrap().root().get_unsound_ref_forever();
                // "2. Filter node and let result be the return value."
                let result = try!(self.accept_node(node));
                // "3. If result is FILTER_ACCEPT, then set the currentNode
                //     attribute to node and return node."
                match result {
                    NodeFilterConstants::FILTER_ACCEPT => {
                        self.current_node.set(JS::from_rooted(node));
                        return Ok(Some(Temporary::from_rooted(node)))
                    },
                    _ => {}
                }
                // "4. Set sibling to node's first child if type is next,
                //     and node's last child if type is previous."
                sibling_op = next_child(node);
                // "5. If result is FILTER_REJECT or sibling is null,
                //     then set sibling to node's next sibling if type is next,
                //     and node's previous sibling if type is previous."
                match (result, &sibling_op) {
                    (NodeFilterConstants::FILTER_REJECT, _)
                    | (_, &None) => sibling_op = next_sibling(node),
                    _ => {}
                }
            }
            // "3. Set node to its parent."
            match node.GetParentNode().map(|p| p.root().get_unsound_ref_forever()) {
                // "4. If node is null or is root, return null."
                None => return Ok(None),
                Some(n) if self.is_root_node(n) => return Ok(None),
                // "5. Filter node and if the return value is FILTER_ACCEPT, then return null."
                Some(n) => {
                    node = n;
                    match try!(self.accept_node(node)) {
                        NodeFilterConstants::FILTER_ACCEPT => return Ok(None),
                        _ => {}
                    }
                }
            }
            // "6. Run these substeps again."
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-following
    fn first_following_node_not_following_root(self, node: JSRef<Node>)
                                               -> Option<Temporary<Node>> {
        // "An object A is following an object B if A and B are in the same tree
        //  and A comes after B in tree order."
        match node.GetNextSibling() {
            None => {
                let mut candidate = node;
                while !self.is_root_node(candidate) && candidate.GetNextSibling().is_none() {
                    match candidate.GetParentNode() {
                        None =>
                            // This can happen if the user set the current node to somewhere
                            // outside of the tree rooted at the original root.
                            return None,
                        Some(n) => candidate = n.root().get_unsound_ref_forever()
                    }
                }
                if self.is_root_node(candidate) {
                    None
                } else {
                    candidate.GetNextSibling()
                }
            },
            it => it
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-filter
    fn accept_node(self, node: JSRef<Node>) -> Fallible<u16> {
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
            Filter::JS(callback) => callback.AcceptNode_(self, node, Rethrow)
        }
    }

    fn is_root_node(self, node: JSRef<Node>) -> bool {
        JS::from_rooted(node) == self.root_node
    }

    fn is_current_node(self, node: JSRef<Node>) -> bool {
        JS::from_rooted(node) == self.current_node.get()
    }
}

impl<'a> Iterator for JSRef<'a, TreeWalker> {
    type Item = JSRef<'a, Node>;

    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        match self.NextNode() {
            Ok(node) => node.map(|n| n.root().get_unsound_ref_forever()),
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

#[jstraceable]
pub enum Filter {
    None,
    Native(fn (node: JSRef<Node>) -> u16),
    JS(NodeFilter)
}
