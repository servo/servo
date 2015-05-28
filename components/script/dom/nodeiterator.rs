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
use dom::bindings::js::{JS, JSRef, MutHeap, OptionalRootable, Temporary, Rootable, RootedReference};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::{Node, NodeHelpers};

use std::cell::Cell;

#[dom_struct]
pub struct NodeIterator {
    reflector_: Reflector,
    root_node: JS<Node>,
    reference_node: MutHeap<JS<Node>>,
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
            reference_node: MutHeap::new(JS::from_rooted(root_node)),
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
            Some(jsfilter) => Filter::Callback(jsfilter)
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
            Filter::Callback(nf) => Some(nf),
            Filter::Native(_) => panic!("Cannot convert native node filter to DOM NodeFilter")
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-referencenode
    fn ReferenceNode(self) -> Temporary<Node> {
        Temporary::from_rooted(self.reference_node.get())
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-pointerbeforereferencenode
    fn PointerBeforeReferenceNode(self) -> bool {
        self.pointer_before_reference_node.get()
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-nextnode
    fn NextNode(self) -> Fallible<Option<Temporary<Node>>> {
        // https://dom.spec.whatwg.org/#concept-NodeIterator-traverse
        // Step 1.
        let node = self.reference_node.get().root();

        // Step 2.
        let mut before_node = self.pointer_before_reference_node.get();

        // Step 3-1.
        if before_node {
            before_node = false;

            // Step 3-2.
            let result = try!(self.accept_node(node.r()));

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(JS::from_rooted(node.r()));
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(Temporary::from_rooted(node.r())));
            }
        }

        // Step 3-1.
        for following_node in node.r().following_nodes(self.root_node.root().r()) {
            let following_node = following_node.root();

            // Step 3-2.
            let result = try!(self.accept_node(following_node.r()));

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(JS::from_rooted(following_node.r()));
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(Temporary::from_rooted(following_node.r())));
            }
        }

        return Ok(None);
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-previousnode
    fn PreviousNode(self) -> Fallible<Option<Temporary<Node>>> {
        // https://dom.spec.whatwg.org/#concept-NodeIterator-traverse
        // Step 1.
        let node = self.reference_node.get().root();

        // Step 2.
        let mut before_node = self.pointer_before_reference_node.get();

        // Step 3-1.
        if !before_node {
            before_node = true;

            // Step 3-2.
            let result = try!(self.accept_node(node.r()));

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(JS::from_rooted(node.r()));
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(Temporary::from_rooted(node.r())));
            }
        }

        // Step 3-1.
        for preceding_node in node.r().preceding_nodes(self.root_node.root().r()) {
            let preceding_node = preceding_node.root();

            // Step 3-2.
            let result = try!(self.accept_node(preceding_node.r()));

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(JS::from_rooted(preceding_node.r()));
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(Temporary::from_rooted(preceding_node.r())));
            }
        }

        return Ok(None);
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-detach
    fn Detach(self) {
        // This method intentionally left blank.
    }
}

trait PrivateNodeIteratorHelpers {
    fn accept_node(self, node: JSRef<Node>) -> Fallible<u16>;
    fn is_root_node(self, node: JSRef<Node>) -> bool;
}

impl<'a> PrivateNodeIteratorHelpers for JSRef<'a, NodeIterator> {
    // https://dom.spec.whatwg.org/#concept-node-filter
    fn accept_node(self, node: JSRef<Node>) -> Fallible<u16> {
        // Step 1.
        let n = node.NodeType() - 1;
        // Step 2.
        if (self.what_to_show & (1 << n)) == 0 {
            return Ok(NodeFilterConstants::FILTER_SKIP)
        }
        // Step 3-5.
        match self.filter {
            Filter::None => Ok(NodeFilterConstants::FILTER_ACCEPT),
            Filter::Native(f) => Ok((f)(node)),
            Filter::Callback(callback) => callback.AcceptNode_(self, node, Rethrow)
        }
    }

    fn is_root_node(self, node: JSRef<Node>) -> bool {
        JS::from_rooted(node) == self.root_node
    }
}


#[jstraceable]
pub enum Filter {
    None,
    Native(fn (node: JSRef<Node>) -> u16),
    Callback(NodeFilter)
}
