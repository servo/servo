/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::element::{Element, ElementHelpers};
use dom::window::Window;

#[dom_struct]
pub struct NamedNodeMap {
    reflector_: Reflector,
    owner: JS<Element>,
}

impl NamedNodeMap {
    fn new_inherited(elem: JSRef<Element>) -> NamedNodeMap {
        NamedNodeMap {
            reflector_: Reflector::new(),
            owner: JS::from_rooted(elem),
        }
    }

    pub fn new(window: JSRef<Window>, elem: JSRef<Element>) -> Temporary<NamedNodeMap> {
        reflect_dom_object(box NamedNodeMap::new_inherited(elem),
                           GlobalRef::Window(window), NamedNodeMapBinding::Wrap)
    }
}

impl<'a> NamedNodeMapMethods for JSRef<'a, NamedNodeMap> {
    fn Length(self) -> u32 {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        let attrs = owner.attrs();
        attrs.len() as u32
    }

    fn Item(self, index: u32) -> Option<Temporary<Attr>> {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        let attrs = owner.attrs();
        attrs.as_slice().get(index as usize).map(|x| Temporary::new(x.clone()))
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Temporary<Attr>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

