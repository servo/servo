/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLCollectionBinding;
use dom::bindings::utils::{CacheableWrapper, BindingObject, WrapperCache};
use dom::bindings::utils::{DOMString, Fallible};
use dom::node::{AbstractNode, ScriptView};
use script_task::page_from_context;

use js::jsapi::{JSObject, JSContext};

use std::cast;
use std::ptr;

pub struct HTMLCollection {
    elements: ~[AbstractNode<ScriptView>],
    wrapper: WrapperCache
}

impl HTMLCollection {
    pub fn new(elements: ~[AbstractNode<ScriptView>], cx: *JSContext, scope: *JSObject) -> @mut HTMLCollection {
        let collection = @mut HTMLCollection {
            elements: elements,
            wrapper: WrapperCache::new()
        };
        collection.init_wrapper(cx, scope);
        collection
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }
    
    pub fn Length(&self) -> u32 {
        self.elements.len() as u32
    }

    pub fn Item(&self, index: u32) -> Option<AbstractNode<ScriptView>> {
        if index < self.Length() {
            Some(self.elements[index])
        } else {
            None
        }
    }

    pub fn NamedItem(&self, _cx: *JSContext, _name: &DOMString) -> Fallible<*JSObject> {
        Ok(ptr::null())
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<AbstractNode<ScriptView>> {
        *found = true;
        self.Item(index)
    }

    pub fn NamedGetter(&self, _cx: *JSContext, _name: &DOMString, _found: &mut bool) -> Fallible<*JSObject> {
        Ok(ptr::null())
    }
}

impl BindingObject for HTMLCollection {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        // TODO(tkuehn): This only handles the top-level frame. Need to grab subframes.
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}

impl CacheableWrapper for HTMLCollection {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe {
            cast::transmute(&self.wrapper)
        }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        HTMLCollectionBinding::Wrap(cx, scope, self, &mut unused)
    }
}
