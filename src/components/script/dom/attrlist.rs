/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::BindingDeclarations::AttrListBinding;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::element::Element;
use dom::window::Window;

#[deriving(Encodable)]
pub struct AttrList {
    pub reflector_: Reflector,
    pub window: JS<Window>,
    pub owner: JS<Element>,
}

impl AttrList {
    pub fn new_inherited(window: &JSRef<Window>, elem: &JSRef<Element>) -> AttrList {
        AttrList {
            reflector_: Reflector::new(),
            window: window.unrooted(),
            owner: elem.unrooted(),
        }
    }

    pub fn new(window: &JSRef<Window>, elem: &JSRef<Element>) -> Temporary<AttrList> {
        reflect_dom_object(~AttrList::new_inherited(window, elem),
                           window, AttrListBinding::Wrap)
    }
}

pub trait AttrListMethods {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<Temporary<Attr>>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<Attr>>;
}

impl<'a> AttrListMethods for JSRef<'a, AttrList> {
    fn Length(&self) -> u32 {
        self.owner.root().attrs.len() as u32
    }

    fn Item(&self, index: u32) -> Option<Temporary<Attr>> {
        self.owner.root().attrs.as_slice().get(index as uint).map(|x| Temporary::new(x.clone()))
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<Attr>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

impl Reflectable for AttrList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
