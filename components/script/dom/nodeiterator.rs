/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Rethrow;
use dom::bindings::codegen::Bindings::NodeIteratorBinding;
use dom::bindings::codegen::Bindings::NodeIteratorBinding::NodeIteratorMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilterConstants;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, MutNullableHeap, OptionalRootable, Temporary, Rootable};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::Node;

use std::cell::Cell;

#[dom_struct]
pub struct NodeIterator {
    reflector_: Reflector,
    root_node: JS<Node>,
    reference_node: MutNullableHeap<JS<Node>>,
    pointer_before_reference_node: Cell<bool>,
    what_to_show: u32,
    filter: Filter,
}

impl NodeIterator {
    fn new_inherited(root_node: JSRef<Node>,
                         what_to_show: u32,
                         filter: Filter) -> NodeIterator {
        NodeIterator {
            reflector_: Reflector::new(),
            root_node: JS::from_rooted(root_node),
            reference_node: MutNullableHeap::new(Some(JS::from_rooted(root_node))),
            pointer_before_reference_node: Cell::new(true),
            what_to_show: what_to_show,
            filter: filter
        }
    }

    pub fn new_with_filter(document: JSRef<Document>,
                           root_node: JSRef<Node>,
                           what_to_show: u32,
                           filter: Filter) -> Temporary<NodeIterator> {
        let window = document.window().root();
        reflect_dom_object(box NodeIterator::new_inherited(root_node, what_to_show, filter),
                           GlobalRef::Window(window.r()),
                           NodeIteratorBinding::Wrap)
    }

    pub fn new(document: JSRef<Document>,
               root_node: JSRef<Node>,
               what_to_show: u32,
               node_filter: Option<NodeFilter>) -> Temporary<NodeIterator> {
        let filter = match node_filter {
            None => Filter::None,
            Some(jsfilter) => Filter::JS(jsfilter)
        };
        NodeIterator::new_with_filter(document, root_node, what_to_show, filter)
    }
}

impl<'a> NodeIteratorMethods for JSRef<'a, NodeIterator> {
    // https://dom.spec.whatwg.org/#dom-nodeiterator-root
    fn Root(self) -> Temporary<Node> {
        Temporary::from_rooted(self.root_node)
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-whattoshow
    fn WhatToShow(self) -> u32 {
        self.what_to_show
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-filter
    fn GetFilter(self) -> Option<NodeFilter> {
        match self.filter {
            Filter::None => None,
            Filter::JS(nf) => Some(nf),
            Filter::Native(_) => panic!("Cannot convert native node filter to DOM NodeFilter")
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-referencenode
    fn GetReferenceNode(self) -> Option<Temporary<Node>> {
        self.reference_node.get().map(Temporary::from_rooted)
    }

    fn PointerBeforeReferenceNode(self) -> bool {
        self.pointer_before_reference_node.get()
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-previousnode
    fn PreviousNode(self) -> Fallible<Option<Temporary<Node>>> {
        self.prev_node()
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-nextnode
    fn NextNode(self) -> Fallible<Option<Temporary<Node>>> {
        self.next_node()
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-detach
    fn Detach(self) {
        // This method intentionally left blank.
    }
}

trait PrivateNodeIteratorHelpers {
    fn is_root_node(self, node: JSRef<Node>) -> bool;
    fn first_following_node_not_following_root(self, node: JSRef<Node>) -> Option<Temporary<Node>>;
    fn first_preceding_node_not_preceding_root(self, node: JSRef<Node>) -> Option<Temporary<Node>>;
    fn accept_node(self, node: JSRef<Node>) -> Fallible<u16>;
}

impl<'a> PrivateNodeIteratorHelpers for JSRef<'a, NodeIterator> {

    // https://dom.spec.whatwg.org/#concept-tree-following
    fn first_following_node_not_following_root(self, node: JSRef<Node>)
                                               -> Option<Temporary<Node>> {
        // "An object A is following an object B if A and B are in the same tree
        //  and A comes after B in tree order."
        match node.GetFirstChild() {
            Some (child) => { return Some(Temporary::from_rooted(child.root().get_unsound_ref_forever())) },
            None => {}
        }
        match node.GetNextSibling() {
            None => {
                let mut candidate = node;
                while !self.is_root_node(candidate) && candidate.GetNextSibling().is_none() {
                    match candidate.GetParentNode() {
                        None =>
                            // This can happen if the user set the current node to somewhere
                            // outside of the tree rooted at the original root.
                            return None,
                        Some(n) => { candidate = n.root().get_unsound_ref_forever(); }
                    }
                }
                if self.is_root_node(candidate) {
                    None
                } else {
                    candidate.GetNextSibling()
                }
            },
            it => {
                if self.is_root_node(node) {
                    return None
                } else {
                    it
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-preceding
    fn first_preceding_node_not_preceding_root(self, node: JSRef<Node>)
                                               -> Option<Temporary<Node>> {
        // "An object A is preceding an object B if A and B are in the same tree
        //  and A comes before B in tree order."
        match node.GetPreviousSibling() {
            None => {
                let mut candidate = node;
                while !self.is_root_node(candidate) && candidate.GetPreviousSibling().is_none() {
                    match candidate.GetParentNode() {
                        None =>
                            // This can happen if the user set the current node to somewhere
                            // outside of the tree rooted at the original root.
                            return None,
                        Some(n) => { candidate = n.root().get_unsound_ref_forever(); }
                    }
                }
                if self.is_root_node(candidate) {
                    None
                } else {
                    candidate.GetPreviousSibling()
                }
            },
            it => {
                let candidate = node;
                if self.is_root_node(candidate) {
                    return None
                } else {
                    it
                }
            }
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
}

pub trait NodeIteratorHelpers {
    fn next_node(self) -> Fallible<Option<Temporary<Node>>>;
    fn prev_node(self) -> Fallible<Option<Temporary<Node>>>;
    fn traverse(self, direction: Direction) -> Fallible<Option<Temporary<Node>>>;
}

impl<'a> NodeIteratorHelpers for JSRef<'a, NodeIterator> {
    // https://dom.spec.whatwg.org/#dom-nodeiterator-nextnode
    fn next_node(self) -> Fallible<Option<Temporary<Node>>> {
        self.traverse(Direction::Next)
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-previousnode
    fn prev_node(self) -> Fallible<Option<Temporary<Node>>> {
        self.traverse(Direction::Previous)
    }

    fn traverse(self, direction: Direction) -> Fallible<Option<Temporary<Node>>> {
        // "1. Let node be the value of the referenceNode attribute."
        let mut node = self.reference_node.get();
        // "2. Let before node be the value of the pointerBeforeReferenceNode attribute."
        let mut before_node = self.pointer_before_reference_node.get();
        // "3. Run these substeps:
        loop {
            match direction {
                // "1. If direction is next"
                Direction::Next => {
                    // "If before node is false, let node be the first node following node in the iterator collection.
                    //  If there is no such node return null. If before node is true, set it to false."
                    if !before_node {
                        match node {
                            None => return Ok(None),
                            Some(n) => {
                                match self.first_following_node_not_following_root(n.root().get_unsound_ref_forever()) {
                                    None => return Ok(None),
                                    Some(n) => node = Some(JS::from_rooted(n))
                                }
                            }
                        }
                    }
                    else {
                        before_node = false;
                    }
                }
                // "If direction is previous"
                Direction::Previous => {
                    // "If before node is true, let node be the first node preceding node in the iterator collection.
                    //  If there is no such node return null. If before node is false, set it to true.
                    if before_node {
                        match node {
                            None => return Ok(None),
                            Some(n) => {
                                match self.first_preceding_node_not_preceding_root(n.root().get_unsound_ref_forever()) {
                                    None => return Ok(None),
                                    Some(n) => node = Some(JS::from_rooted(n))
                                }
                            }
                        }
                    }
                    else {
                        before_node = true;
                    }
                }
            }
            // "2. Filter node and let result be the return value."
            let result = try!(self.accept_node(node.unwrap().root().get_unsound_ref_forever()));

            // "3. If result is FILTER_ACCEPT, go to the next step in the overall set of steps. Otherwise, run these substeps again."
            match result {
                NodeFilterConstants::FILTER_ACCEPT => break,
                _ => {}
            }

        }
        // "4. Set the referenceNode attribute to node, set the pointerBeforeReferenceNode attribute to before node, and return node."
        self.reference_node.set(Some(JS::from_rooted(node.unwrap())));
        self.pointer_before_reference_node.set(before_node);

        Ok(Some(Temporary::from_rooted(node.unwrap())))
    }
}

pub enum Direction {
    Next,
    Previous
}

#[jstraceable]
pub enum Filter {
    None,
    Native(fn (node: JSRef<Node>) -> u16),
    JS(NodeFilter)
}
