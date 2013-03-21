use dom::bindings::htmlcollection::HTMLCollection;
use dom::bindings::utils::{DOMString, WrapperCache, str};
use dom::node::AbstractNode;
use newcss::stylesheet::Stylesheet;

use std::arc::ARC;

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
    fn getElementsByTagName(&self, tag: DOMString) -> Option<~HTMLCollection> {
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
        Some(~HTMLCollection::new(elements))
    }
}