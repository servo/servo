/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::htmlcollection::HTMLCollection;
use dom::bindings::utils::{DOMString, WrapperCache, str};
use dom::node::AbstractNode;


pub struct Document {
    root: AbstractNode,
    wrapper: WrapperCache
}

pub fn Document(root: AbstractNode) -> Document {
    Document {
        root: root,
        wrapper: WrapperCache::new()
    }
}

pub impl Document {
    fn getElementsByTagName(&self, tag: DOMString) -> Option<@mut HTMLCollection> {
        let mut elements = ~[];
        let tag = match tag {
          str(s) => s,
          _ => ~""
        };
        let _ = for self.root.traverse_preorder |child| {
            if child.is_element() {
                do child.with_imm_element |elem| {
                    if elem.tag_name == tag {
                        elements.push(child);
                    }
                }
            }
        };
        Some(@mut HTMLCollection::new(elements))
    }
}