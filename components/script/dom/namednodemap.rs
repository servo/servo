/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::element::Element;
use dom::window::Window;

#[jstraceable]
#[must_root]
pub struct NamedNodeMap {
    reflector_: Reflector,
    owner: JS<Element>,
}

impl NamedNodeMap {
    pub fn new_inherited(elem: JSRef<Element>) -> NamedNodeMap {
        NamedNodeMap {
            reflector_: Reflector::new(),
            owner: JS::from_rooted(elem),
        }
    }

    pub fn new(window: JSRef<Window>, elem: JSRef<Element>) -> Temporary<NamedNodeMap> {
        reflect_dom_object(box NamedNodeMap::new_inherited(elem),
                           &Window(window), NamedNodeMapBinding::Wrap)
    }
}

impl<'a> NamedNodeMapMethods for JSRef<'a, NamedNodeMap> {
    fn Length(self) -> u32 {
        self.owner.root().attrs.borrow().len() as u32
    }

    fn Item(self, index: u32) -> Option<Temporary<Attr>> {
        self.owner.root().attrs.borrow().as_slice().get(index as uint).map(|x| Temporary::new(x.clone()))
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Temporary<Attr>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

impl Reflectable for NamedNodeMap {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
