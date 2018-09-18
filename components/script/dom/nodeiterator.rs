/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Rethrow;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilterConstants;
use dom::bindings::codegen::Bindings::NodeIteratorBinding;
use dom::bindings::codegen::Bindings::NodeIteratorBinding::NodeIteratorMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, MutDom};
use dom::document::Document;
use dom::node::Node;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::rc::Rc;

#[dom_struct]
pub struct NodeIterator {
    reflector_: Reflector,
    root_node: Dom<Node>,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    reference_node: MutDom<Node>,
    pointer_before_reference_node: Cell<bool>,
    what_to_show: u32,
    #[ignore_malloc_size_of = "Can't measure due to #6870"]
    filter: Filter,
    active: Cell<bool>,
}

impl NodeIterator {
    fn new_inherited(root_node: &Node, what_to_show: u32, filter: Filter) -> NodeIterator {
        NodeIterator {
            reflector_: Reflector::new(),
            root_node: Dom::from_ref(root_node),
            reference_node: MutDom::new(root_node),
            pointer_before_reference_node: Cell::new(true),
            what_to_show: what_to_show,
            filter: filter,
            active: Cell::new(false),
        }
    }

    pub fn new_with_filter(
        document: &Document,
        root_node: &Node,
        what_to_show: u32,
        filter: Filter,
    ) -> DomRoot<NodeIterator> {
        reflect_dom_object(
            Box::new(NodeIterator::new_inherited(root_node, what_to_show, filter)),
            document.window(),
            NodeIteratorBinding::Wrap,
        )
    }

    pub fn new(
        document: &Document,
        root_node: &Node,
        what_to_show: u32,
        node_filter: Option<Rc<NodeFilter>>,
    ) -> DomRoot<NodeIterator> {
        let filter = match node_filter {
            None => Filter::None,
            Some(jsfilter) => Filter::Callback(jsfilter),
        };
        NodeIterator::new_with_filter(document, root_node, what_to_show, filter)
    }
}

impl NodeIteratorMethods for NodeIterator {
    // https://dom.spec.whatwg.org/#dom-nodeiterator-root
    fn Root(&self) -> DomRoot<Node> {
        DomRoot::from_ref(&*self.root_node)
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-whattoshow
    fn WhatToShow(&self) -> u32 {
        self.what_to_show
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-filter
    fn GetFilter(&self) -> Option<Rc<NodeFilter>> {
        match self.filter {
            Filter::None => None,
            Filter::Callback(ref nf) => Some((*nf).clone()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-referencenode
    fn ReferenceNode(&self) -> DomRoot<Node> {
        self.reference_node.get()
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-pointerbeforereferencenode
    fn PointerBeforeReferenceNode(&self) -> bool {
        self.pointer_before_reference_node.get()
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-nextnode
    fn NextNode(&self) -> Fallible<Option<DomRoot<Node>>> {
        // https://dom.spec.whatwg.org/#concept-NodeIterator-traverse
        // Step 1.
        let node = self.reference_node.get();

        // Step 2.
        let mut before_node = self.pointer_before_reference_node.get();

        // Step 3-1.
        if before_node {
            before_node = false;

            // Step 3-2.
            let result = self.accept_node(&node)?;

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(&node);
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(node));
            }
        }

        // Step 3-1.
        for following_node in node.following_nodes(&self.root_node) {
            // Step 3-2.
            let result = self.accept_node(&following_node)?;

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(&following_node);
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(following_node));
            }
        }

        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-previousnode
    fn PreviousNode(&self) -> Fallible<Option<DomRoot<Node>>> {
        // https://dom.spec.whatwg.org/#concept-NodeIterator-traverse
        // Step 1.
        let node = self.reference_node.get();

        // Step 2.
        let mut before_node = self.pointer_before_reference_node.get();

        // Step 3-1.
        if !before_node {
            before_node = true;

            // Step 3-2.
            let result = self.accept_node(&node)?;

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(&node);
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(node));
            }
        }

        // Step 3-1.
        for preceding_node in node.preceding_nodes(&self.root_node) {
            // Step 3-2.
            let result = self.accept_node(&preceding_node)?;

            // Step 3-3.
            if result == NodeFilterConstants::FILTER_ACCEPT {
                // Step 4.
                self.reference_node.set(&preceding_node);
                self.pointer_before_reference_node.set(before_node);

                return Ok(Some(preceding_node));
            }
        }

        Ok(None)
    }

    // https://dom.spec.whatwg.org/#dom-nodeiterator-detach
    fn Detach(&self) {
        // This method intentionally left blank.
    }
}

impl NodeIterator {
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
            Filter::Callback(ref callback) => {
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
}

#[derive(JSTraceable)]
pub enum Filter {
    None,
    Callback(Rc<NodeFilter>),
}
