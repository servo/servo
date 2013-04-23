use dom::bindings::utils::WrapperCache;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::node::AbstractNode;

use js::jsapi::{JSObject, JSContext};

pub struct HTMLCollection {
    elements: ~[AbstractNode],
    wrapper: WrapperCache
}

pub impl HTMLCollection {
    fn new(elements: ~[AbstractNode]) -> @mut HTMLCollection {
        let collection = @mut HTMLCollection {
            elements: elements,
            wrapper: WrapperCache::new()
        };
        collection.init_wrapper();
        collection
    }
    
    fn Length(&self) -> u32 {
        self.elements.len() as u32
    }

    fn Item(&self, index: u32) -> Option<AbstractNode> {
        if index < self.Length() {
            Some(self.elements[index])
        } else {
            None
        }
    }

    fn NamedItem(&self, _cx: *JSContext, _name: DOMString, rv: &mut ErrorResult) -> *JSObject {
        *rv = Ok(());
        ptr::null()
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<AbstractNode> {
        *found = true;
        self.Item(index)
    }
}
