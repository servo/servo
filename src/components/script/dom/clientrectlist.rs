/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::ClientRectListBinding;
use dom::bindings::js::{JS, JSRef, Unrooted};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::clientrect::ClientRect;
use dom::window::Window;

#[deriving(Encodable)]
pub struct ClientRectList {
    pub reflector_: Reflector,
    pub rects: Vec<JS<ClientRect>>,
    pub window: JS<Window>,
}

impl ClientRectList {
    pub fn new_inherited(window: JS<Window>,
                         rects: Vec<JSRef<ClientRect>>) -> ClientRectList {
        ClientRectList {
            reflector_: Reflector::new(),
            rects: rects.iter().map(|rect| rect.unrooted()).collect(),
            window: window,
        }
    }

    pub fn new(window: &JSRef<Window>,
               rects: Vec<JSRef<ClientRect>>) -> Unrooted<ClientRectList> {
        reflect_dom_object(~ClientRectList::new_inherited(window.unrooted(), rects),
                           window, ClientRectListBinding::Wrap)
    }

    pub fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    pub fn Item(&self, index: u32) -> Option<Unrooted<ClientRect>> {
        if index < self.rects.len() as u32 {
            Some(Unrooted::new(self.rects.get(index as uint).clone()))
        } else {
            None
        }
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Unrooted<ClientRect>> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}

impl Reflectable for ClientRectList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
