/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeIteratorBinding;
use dom::bindings::codegen::Bindings::NodeIteratorBinding::NodeIteratorMethods;
use dom::bindings::codegen::Bindings::NodeFilterBinding::NodeFilter;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers};
use dom::node::{Node};

#[dom_struct]
pub struct NodeIterator {
    reflector_: Reflector,
    root_node: JS<Node>,
    what_to_show: u32,
    filter: Filter
}

impl NodeIterator {
    fn new_inherited(root_node: JSRef<Node>,
                         what_to_show: u32,
                         filter: Filter) -> NodeIterator {
        NodeIterator {
            reflector_: Reflector::new(),
            root_node: JS::from_rooted(root_node),
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
        Temporary::new(self.root_node)
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

}

#[jstraceable]
pub enum Filter {
    None,
    Native(fn (node: JSRef<Node>) -> u16),
    JS(NodeFilter)
}
