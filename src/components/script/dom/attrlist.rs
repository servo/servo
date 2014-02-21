/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::AttrListBinding;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::node::{AbstractNode};
use dom::window::Window;

pub struct AttrList {
    reflector_: Reflector,
    window: @mut Window,
    owner: AbstractNode,
}

impl AttrList {
    pub fn new_inherited(window: @mut Window, elem: AbstractNode) -> AttrList {
        AttrList {
            reflector_: Reflector::new(),
            window: window,
            owner: elem
        }
    }

    pub fn new(window: @mut Window, elem: AbstractNode) -> @mut AttrList {
        reflect_dom_object(@mut AttrList::new_inherited(window, elem),
                           window, AttrListBinding::Wrap)
    }

    pub fn Length(&self) -> u32 {
        self.owner.with_imm_element(|elem| elem.attrs.len() as u32)
    }

    pub fn Item(&self, index: u32) -> Option<@mut Attr> {
        self.owner.with_imm_element(|elem| {
            elem.attrs.get_opt(index as uint).map(|&x| x)
        })
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<@mut Attr> {
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
