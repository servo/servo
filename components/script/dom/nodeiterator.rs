/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeIteratorBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflector, reflect_dom_object};

#[dom_struct]
pub struct NodeIterator {
    reflector_: Reflector
}

impl NodeIterator {
    fn new_inherited() -> NodeIterator {
        NodeIterator {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: GlobalRef) -> Temporary<NodeIterator> {
        reflect_dom_object(box NodeIterator::new_inherited(), global, NodeIteratorBinding::Wrap)
    }
}
