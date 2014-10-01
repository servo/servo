/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::RethrowExceptions;
use dom::bindings::codegen::Bindings::TreeWalkerBinding;
use dom::bindings::codegen::Bindings::TreeWalkerBinding::TreeWalkerMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
// FIXME: Uncomment when codegen fix allows NodeFilterConstants
// to move to the NodeFilter binding file (#3149).
// For now, it is defined in this file.
// use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilterConstants;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, OptionalRootable, Temporary};
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::Document;
use dom::node::{Node, NodeHelpers};

use std::cell::Cell;

// http://dom.spec.whatwg.org/#interface-treewalker
#[jstraceable]
#[must_root]
pub struct TreeWalker {
    pub reflector_: Reflector,
    pub root_node: JS<Node>,
    pub current_node: Cell<JS<Node>>,
    pub what_to_show: u32,
    pub filter: Filter
}

impl TreeWalker {
    fn new_inherited(root_node: JSRef<Node>,
                         what_to_show: u32,
                         filter: Filter) -> TreeWalker {
        TreeWalker {
            reflector_: Reflector::new(),
            root_node: JS::from_rooted(root_node),
            current_node: Cell::new(JS::from_rooted(root_node)),
            what_to_show: what_to_show,
            filter: filter
        }
    }

    pub fn new_with_filter(document: JSRef<Document>,
                           root_node: JSRef<Node>,
                           what_to_show: u32,
                           filter: Filter) -> Temporary<TreeWalker> {
        let window = document.window.root();
        reflect_dom_object(box TreeWalker::new_inherited(root_node, what_to_show, filter),
                           &Window(*window),
                           TreeWalkerBinding::Wrap)
    }

    pub fn new(document: JSRef<Document>,
               root_node: JSRef<Node>,
               what_to_show: u32,
               node_filter: Option<NodeFilter>) -> Temporary<TreeWalker> {
        let filter = match node_filter {
            None => FilterNone,
            Some(jsfilter) => FilterJS(jsfilter)
        };
        TreeWalker::new_with_filter(document, root_node, what_to_show, filter)
    }
}

impl<'a> TreeWalkerMethods for JSRef<'a, TreeWalker> {
    fn Root(self) -> Temporary<Node> {
        Temporary::new(self.root_node)
    }

    fn WhatToShow(self) -> u32 {
        self.what_to_show
    }

    fn GetFilter(self) -> Option<NodeFilter> {
        match self.filter {
            FilterNone => None,
            FilterJS(nf) => Some(nf),
            FilterNative(_) => fail!("Cannot convert native node filter to DOM NodeFilter")
        }
    }

    fn CurrentNode(self) -> Temporary<Node> {
        Temporary::new(self.current_node.get())
    }

    fn SetCurrentNode(self, node: JSRef<Node>) -> ErrorResult {
        // XXX Future: check_same_origin(root_node, node) (throws)
        self.current_node.set(JS::from_rooted(node));
        Ok(())
    }

    fn ParentNode(self) -> Fallible<Option<Temporary<Node>>> {
        self.parent_node()
    }

    fn FirstChild(self) -> Fallible<Option<Temporary<Node>>> {
        self.first_child()
    }

    fn LastChild(self) -> Fallible<Option<Temporary<Node>>> {
        self.last_child()
    }

    fn PreviousSibling(self) -> Fallible<Option<Temporary<Node>>> {
        self.prev_sibling()
    }

    fn NextSibling(self) -> Fallible<Option<Temporary<Node>>> {
        self.next_sibling()
    }

    fn PreviousNode(self) -> Fallible<Option<Temporary<Node>>> {
        self.prev_node()
    }

    fn NextNode(self) -> Fallible<Option<Temporary<Node>>> {
        self.next_node()
    }
}

impl Reflectable for TreeWalker {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

type NodeAdvancer<'a> = |node: JSRef<'a, Node>|: 'a -> Option<Temporary<Node>>;

trait PrivateTreeWalkerHelpers<'a> {
    fn traverse_children(self,
                         next_child: NodeAdvancer<'a>,
                         next_sibling: NodeAdvancer<'a>)
                         -> Fallible<Option<Temporary<Node>>>;
    fn traverse_siblings(self,
                         next_child: NodeAdvancer<'a>,
                         next_sibling: NodeAdvancer<'a>)
                         -> Fallible<Option<Temporary<Node>>>;
    fn is_root_node(self, node: JSRef<'a, Node>) -> bool;
    fn is_current_node(self, node: JSRef<'a, Node>) -> bool;
    fn first_following_node_not_following_root(self, node: JSRef<'a, Node>)
                                               -> Option<Temporary<Node>>;
    fn accept_node(self, node: JSRef<'a, Node>) -> Fallible<u16>;
}

impl<'a> PrivateTreeWalkerHelpers<'a> for JSRef<'a, TreeWalker> {
    // http://dom.spec.whatwg.org/#concept-traverse-children
    fn traverse_children(self,
                         next_child: NodeAdvancer<'a>,
                         next_sibling: NodeAdvancer<'a>)
                         -> Fallible<Option<Temporary<Node>>> {
        // "To **traverse children** of type *type*, run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        // "2. Set node to node's first child if type is first, and node's last child if type is last."
        let cur = self.current_node.get().root();
        let mut node_op: Option<JSRef<Node>> = next_child(*cur).map(|node| node.root().clone());

        // 3. Main: While node is not null, run these substeps:
        'main: loop {
            match node_op {
                None => break,
                Some(node) => {
                    // "1. Filter node and let result be the return value."
                    match self.accept_node(node) {
                        Err(e) => return Err(e),
                        // "2. If result is FILTER_ACCEPT, then set the currentNode
                        //     attribute to node and return node."
                        Ok(NodeFilterConstants::FILTER_ACCEPT) => {
                            self.current_node.set(JS::from_rooted(node));
                            return Ok(Some(Temporary::from_rooted(node)))
                        },
                        // "3. If result is FILTER_SKIP, run these subsubsteps:"
                        Ok(NodeFilterConstants::FILTER_SKIP) => {
                            // "1. Let child be node's first child if type is first,
                            //     and node's last child if type is last."
                            match next_child(node) {
                                // "2. If child is not null, set node to child and goto Main."
                                Some(child) => {
                                    node_op = Some(child.root().clone());
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
                                        node_op = Some(sibling.root().clone());
                                        continue 'main
                                    },
                                    None => {
                                        // "3. Let parent be node's parent."
                                        match node.parent_node().map(|p| p.root().clone()) {
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

    // http://dom.spec.whatwg.org/#concept-traverse-siblings
    fn traverse_siblings(self,
                         next_child: NodeAdvancer<'a>,
                         next_sibling: NodeAdvancer<'a>)
                         -> Fallible<Option<Temporary<Node>>> {
        // "To **traverse siblings** of type *type* run these steps:"
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().clone();
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
                node = sibling_op.unwrap().root().clone();
                // "2. Filter node and let result be the return value."
                let result = self.accept_node(node);
                // "3. If result is FILTER_ACCEPT, then set the currentNode
                //     attribute to node and return node."
                match result {
                    Err(e) => return Err(e),
                    Ok(NodeFilterConstants::FILTER_ACCEPT) => {
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
                match (result, sibling_op) {
                    (Ok(NodeFilterConstants::FILTER_REJECT), _)
                    | (_, None) => sibling_op = next_sibling(node),
                    _ => {}
                }
            }
            // "3. Set node to its parent."
            match node.parent_node().map(|p| p.root().clone()) {
                // "4. If node is null or is root, return null."
                None => return Ok(None),
                Some(n) if self.is_root_node(n) => return Ok(None),
                // "5. Filter node and if the return value is FILTER_ACCEPT, then return null."
                Some(n) => {
                    node = n;
                    match self.accept_node(node) {
                        Err(e) => return Err(e),
                        Ok(NodeFilterConstants::FILTER_ACCEPT) => return Ok(None),
                        _ => {}
                    }
                }
            }
            // "6. Run these substeps again."
        }
    }

    // http://dom.spec.whatwg.org/#concept-tree-following
    fn first_following_node_not_following_root(self, node: JSRef<'a, Node>)
                                               -> Option<Temporary<Node>> {
        // "An object A is following an object B if A and B are in the same tree
        //  and A comes after B in tree order."
        match node.next_sibling() {
            None => {
                let mut candidate = node;
                while !self.is_root_node(candidate) && candidate.next_sibling().is_none() {
                    match candidate.parent_node() {
                        None =>
                            // This can happen if the user set the current node to somewhere
                            // outside of the tree rooted at the original root.
                            return None,
                        Some(n) => candidate = n.root().clone()
                    }
                }
                if self.is_root_node(candidate) {
                    None
                } else {
                    candidate.next_sibling()
                }
            },
            it => it
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-filter
    fn accept_node(self, node: JSRef<'a, Node>) -> Fallible<u16> {
        // "To filter node run these steps:"
        // "1. Let n be node's nodeType attribute value minus 1."
        let n: uint = node.NodeType() as uint - 1;
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
            FilterNone => Ok(NodeFilterConstants::FILTER_ACCEPT),
            FilterNative(f) => Ok((*f)(node)),
            FilterJS(callback) => callback.AcceptNode_(self, node, RethrowExceptions)
        }
    }

    fn is_root_node(self, node: JSRef<'a, Node>) -> bool {
        JS::from_rooted(node) == self.root_node
    }

    fn is_current_node(self, node: JSRef<'a, Node>) -> bool {
        JS::from_rooted(node) == self.current_node.get()
    }
}

pub trait TreeWalkerHelpers<'a> {
    fn parent_node(self) -> Fallible<Option<Temporary<Node>>>;
    fn first_child(self) -> Fallible<Option<Temporary<Node>>>;
    fn last_child(self) -> Fallible<Option<Temporary<Node>>>;
    fn next_sibling(self) -> Fallible<Option<Temporary<Node>>>;
    fn prev_sibling(self) -> Fallible<Option<Temporary<Node>>>;
    fn next_node(self) -> Fallible<Option<Temporary<Node>>>;
    fn prev_node(self) -> Fallible<Option<Temporary<Node>>>;
}

impl<'a> TreeWalkerHelpers<'a> for JSRef<'a, TreeWalker> {
    // http://dom.spec.whatwg.org/#dom-treewalker-parentnode
    fn parent_node(self) -> Fallible<Option<Temporary<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().clone();
        // "2. While node is not null and is not root, run these substeps:"
        while !self.is_root_node(node) {
            // "1. Let node be node's parent."
            match node.parent_node() {
                Some(n) => {
                    node = n.root().clone();
                    // "2. If node is not null and filtering node returns FILTER_ACCEPT,
                    //     then set the currentNode attribute to node, return node."
                    match self.accept_node(node) {
                        Err(e) => return Err(e),
                        Ok(NodeFilterConstants::FILTER_ACCEPT) => {
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

    // http://dom.spec.whatwg.org/#dom-treewalker-firstchild
    fn first_child(self) -> Fallible<Option<Temporary<Node>>> {
        // "The firstChild() method must traverse children of type first."
        self.traverse_children(|node| node.first_child(),
                               |node| node.next_sibling())
    }

    // http://dom.spec.whatwg.org/#dom-treewalker-lastchild
    fn last_child(self) -> Fallible<Option<Temporary<Node>>> {
        // "The lastChild() method must traverse children of type last."
        self.traverse_children(|node| node.last_child(),
                               |node| node.prev_sibling())
    }

    // http://dom.spec.whatwg.org/#dom-treewalker-nextsibling
    fn next_sibling(self) -> Fallible<Option<Temporary<Node>>> {
        // "The nextSibling() method must traverse siblings of type next."
        self.traverse_siblings(|node| node.first_child(),
                               |node| node.next_sibling())
    }

    // http://dom.spec.whatwg.org/#dom-treewalker-previoussibling
    fn prev_sibling(self) -> Fallible<Option<Temporary<Node>>> {
        // "The previousSibling() method must traverse siblings of type previous."
        self.traverse_siblings(|node| node.last_child(),
                               |node| node.prev_sibling())
    }

    // http://dom.spec.whatwg.org/#dom-treewalker-previousnode
    fn prev_node(self) -> Fallible<Option<Temporary<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().clone();
        // "2. While node is not root, run these substeps:"
        while !self.is_root_node(node) {
            // "1. Let sibling be the previous sibling of node."
            let mut sibling_op = node.prev_sibling();
            // "2. While sibling is not null, run these subsubsteps:"
            while sibling_op.is_some() {
                // "1. Set node to sibling."
                node = sibling_op.unwrap().root().clone();
                // "2. Filter node and let result be the return value."
                // "3. While result is not FILTER_REJECT and node has a child,
                //     set node to its last child and then filter node and
                //     set result to the return value."
                // "4. If result is FILTER_ACCEPT, then
                //     set the currentNode attribute to node and return node."
                loop {
                    match self.accept_node(node) {
                        Err(e) => return Err(e),
                        Ok(NodeFilterConstants::FILTER_REJECT) => break,
                        _ if node.first_child().is_some() =>
                            node = node.last_child().unwrap().root().clone(),
                        Ok(NodeFilterConstants::FILTER_ACCEPT) => {
                            self.current_node.set(JS::from_rooted(node));
                            return Ok(Some(Temporary::from_rooted(node)))
                        },
                        _ => break
                    }
                }
                // "5. Set sibling to the previous sibling of node."
                sibling_op = node.prev_sibling()
            }
            // "3. If node is root or node's parent is null, return null."
            if self.is_root_node(node) || node.parent_node() == None {
                return Ok(None)
            }
            // "4. Set node to its parent."
            match node.parent_node() {
                None =>
                    // This can happen if the user set the current node to somewhere
                    // outside of the tree rooted at the original root.
                    return Ok(None),
                Some(n) => node = n.root().clone()
            }
            // "5. Filter node and if the return value is FILTER_ACCEPT, then
            //     set the currentNode attribute to node and return node."
            match self.accept_node(node) {
                Err(e) => return Err(e),
                Ok(NodeFilterConstants::FILTER_ACCEPT) => {
                    self.current_node.set(JS::from_rooted(node));
                    return Ok(Some(Temporary::from_rooted(node)))
                },
                _ => {}
            }
        }
        // "6. Return null."
        Ok(None)
    }

    // http://dom.spec.whatwg.org/#dom-treewalker-nextnode
    fn next_node(self) -> Fallible<Option<Temporary<Node>>> {
        // "1. Let node be the value of the currentNode attribute."
        let mut node = self.current_node.get().root().clone();
        // "2. Let result be FILTER_ACCEPT."
        let mut result = Ok(NodeFilterConstants::FILTER_ACCEPT);
        // "3. Run these substeps:"
        loop {
            // "1. While result is not FILTER_REJECT and node has a child, run these subsubsteps:"
            loop {
                match result {
                    Ok(NodeFilterConstants::FILTER_REJECT) => break,
                    _ => {}
                }
                match node.first_child() {
                    None => break,
                    Some (child) => {
                        // "1. Set node to its first child."
                        node = child.root().clone();
                        // "2. Filter node and set result to the return value."
                        result = self.accept_node(node);
                        // "3. If result is FILTER_ACCEPT, then
                        //     set the currentNode attribute to node and return node."
                        match result {
                            Err(e) => return Err(e),
                            Ok(NodeFilterConstants::FILTER_ACCEPT) => {
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
                    node = n.root().clone();
                    // "3. Filter node and set result to the return value."
                    result = self.accept_node(node);
                    // "4. If result is FILTER_ACCEPT, then
                    //     set the currentNode attribute to node and return node."
                    match result {
                        Err(e) => return Err(e),
                        Ok(NodeFilterConstants::FILTER_ACCEPT) => {
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

impl<'a> Iterator<JSRef<'a, Node>> for JSRef<'a, TreeWalker> {
   fn next(&mut self) -> Option<JSRef<'a, Node>> {
       match self.next_node() {
           Ok(node) => node.map(|n| n.root().clone()),
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
    FilterNone,
    FilterNative(Untraceable<fn (node: JSRef<Node>) -> u16>),
    FilterJS(NodeFilter)
}

// FIXME: NodeFilterConstants will be defined in NodeFilterBindings.rs
// when codegen supports a callback interface with constants (#3149).
pub mod NodeFilterConstants {
  pub static FILTER_ACCEPT: u16 = 1;
  pub static FILTER_REJECT: u16 = 2;
  pub static FILTER_SKIP: u16 = 3;
  pub static SHOW_ALL: u32 = 4294967295;
  pub static SHOW_ELEMENT: u32 = 1;
  pub static SHOW_ATTRIBUTE: u32 = 2;
  pub static SHOW_TEXT: u32 = 4;
  pub static SHOW_CDATA_SECTION: u32 = 8;
  pub static SHOW_ENTITY_REFERENCE: u32 = 16;
  pub static SHOW_ENTITY: u32 = 32;
  pub static SHOW_PROCESSING_INSTRUCTION: u32 = 64;
  pub static SHOW_COMMENT: u32 = 128;
  pub static SHOW_DOCUMENT: u32 = 256;
  pub static SHOW_DOCUMENT_TYPE: u32 = 512;
  pub static SHOW_DOCUMENT_FRAGMENT: u32 = 1024;
  pub static SHOW_NOTATION: u32 = 2048;
} // mod NodeFilterConstants
