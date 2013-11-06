/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLCollectionBinding;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::{DOMString, Fallible};
use dom::node::{AbstractNode, ScriptView};
use dom::window::Window;

use js::jsapi::{JSObject, JSContext};

use std::ptr;

pub struct HTMLCollection {
    elements: ~[AbstractNode<ScriptView>],
    reflector_: Reflector,
    window: @mut Window,
}

impl HTMLCollection {
    pub fn new_inherited(window: @mut Window,
                         elements: ~[AbstractNode<ScriptView>]) -> HTMLCollection {
        HTMLCollection {
            elements: elements,
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: @mut Window,
               elements: ~[AbstractNode<ScriptView>]) -> @mut HTMLCollection {
        reflect_dom_object(@mut HTMLCollection::new_inherited(window, elements),
                           window, HTMLCollectionBinding::Wrap)
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

impl Reflectable for HTMLCollection {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut Reflectable> {
        Some(self.window as @mut Reflectable)
    }
}
