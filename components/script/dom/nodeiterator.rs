/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeIteratorBinding;
use dom::bindings::codegen::Bindings::NodeIteratorBinding::NodeIteratorMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};

#[jstraceable]
#[must_root]
pub struct NodeIterator {
    pub reflector_: Reflector
}

impl NodeIterator {
    pub fn new_inherited() -> NodeIterator {
        NodeIterator {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<NodeIterator> {
        reflect_dom_object(box NodeIterator::new_inherited(), global, NodeIteratorBinding::Wrap)
    }
}

impl<'a> NodeIteratorMethods for JSRef<'a, NodeIterator> {
}

impl Reflectable for NodeIterator {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
