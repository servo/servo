/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::WrapperCache;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::node::{AbstractNode, ScriptView};

use js::jsapi::{JSObject, JSContext};

pub struct HTMLCollection {
    elements: ~[AbstractNode<ScriptView>],
    wrapper: WrapperCache
}

pub impl HTMLCollection {
    fn new(elements: ~[AbstractNode<ScriptView>]) -> @mut HTMLCollection {
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

    fn Item(&self, index: u32) -> Option<AbstractNode<ScriptView>> {
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

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<AbstractNode<ScriptView>> {
        *found = true;
        self.Item(index)
    }
}
