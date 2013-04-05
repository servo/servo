/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content::content_task::task_from_context;
use dom::node::AbstractNode;
use dom::bindings::codegen::HTMLCollectionBinding;
use dom::bindings::utils::{DOMString, ErrorResult, OpaqueBindingReference};
use dom::bindings::utils::{CacheableWrapper, BindingObject, WrapperCache};
use js::jsapi::{JSObject, JSContext};

pub struct HTMLCollection {
    elements: ~[AbstractNode],
    wrapper: WrapperCache
}

pub impl HTMLCollection {
    fn new(elements: ~[AbstractNode]) -> HTMLCollection {
        HTMLCollection {
            elements: elements,
            wrapper: WrapperCache::new()
        }
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

impl BindingObject for HTMLCollection {
    fn GetParentObject(&self, cx: *JSContext) -> OpaqueBindingReference {
        let content = task_from_context(cx);
        unsafe { OpaqueBindingReference(Right((*content).window.get() as @CacheableWrapper)) }
    }
}

impl CacheableWrapper for HTMLCollection {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_unique(~self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        HTMLCollectionBinding::Wrap(cx, scope, self, &mut unused)
    }

    fn wrap_object_shared(@self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"nyi")
    }
}
