/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::AttrListBinding;
use dom::bindings::js::JS;
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
    pub fn new_inherited(window: JS<Window>, elem: JS<Element>) -> AttrList {
        AttrList {
            reflector_: Reflector::new(),
            window: window,
            owner: elem
        }
    }

    pub fn new(window: &JS<Window>, elem: &JS<Element>) -> JS<AttrList> {
        reflect_dom_object(~AttrList::new_inherited(window.clone(), elem.clone()),
                           window, AttrListBinding::Wrap)
    }

    pub fn Length(&self) -> u32 {
        self.owner.get().attrs.len() as u32
    }

    pub fn Item(&self, index: u32) -> Option<JS<Attr>> {
        self.owner.get().attrs.as_slice().get(index as uint).map(|x| x.clone())
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<JS<Attr>> {
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
