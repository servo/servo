/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLCollectionBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::element::Element;
use dom::window::Window;
use servo_util::str::DOMString;

use js::jsapi::{JSObject, JSContext};

use std::ptr;

#[deriving(Encodable)]
pub struct HTMLCollection {
    elements: ~[JS<Element>],
    reflector_: Reflector,
    window: JS<Window>,
}

impl HTMLCollection {
    pub fn new_inherited(window: JS<Window>, elements: ~[JS<Element>]) -> HTMLCollection {
        HTMLCollection {
            elements: elements,
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: &JS<Window>, elements: ~[JS<Element>]) -> JS<HTMLCollection> {
        reflect_dom_object(~HTMLCollection::new_inherited(window.clone(), elements),
                           window.get(), HTMLCollectionBinding::Wrap)
    }
    
    pub fn Length(&self) -> u32 {
        self.elements.len() as u32
    }

    pub fn Item(&self, index: u32) -> Option<JS<Element>> {
        if index < self.Length() {
            Some(self.elements[index].clone())
        } else {
            None
        }
    }

    pub fn NamedItem(&self, _cx: *JSContext, _name: DOMString) -> Fallible<*JSObject> {
        Ok(ptr::null())
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<JS<Element>> {
        *found = true;
        self.Item(index)
    }

    pub fn NamedGetter(&self, _cx: *JSContext, _name: Option<DOMString>, _found: &mut bool) -> Fallible<*JSObject> {
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
}
