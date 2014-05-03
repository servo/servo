/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::ClientRectListBinding;
use dom::bindings::js::{JS, JSRef, Temporary};
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
    pub fn new_inherited(window: &JSRef<Window>,
                         rects: Vec<JSRef<ClientRect>>) -> ClientRectList {
        ClientRectList {
            reflector_: Reflector::new(),
            rects: rects.iter().map(|rect| rect.unrooted()).collect(),
            window: window.unrooted(),
        }
    }

    pub fn new(window: &JSRef<Window>,
               rects: Vec<JSRef<ClientRect>>) -> Temporary<ClientRectList> {
        reflect_dom_object(~ClientRectList::new_inherited(window, rects),
                           window, ClientRectListBinding::Wrap)
    }
}

pub trait ClientRectListMethods {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<Temporary<ClientRect>>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<ClientRect>>;
}

impl<'a> ClientRectListMethods for JSRef<'a, ClientRectList> {
    fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    fn Item(&self, index: u32) -> Option<Temporary<ClientRect>> {
        if index < self.rects.len() as u32 {
            Some(Temporary::new(self.rects.get(index as uint).clone()))
        } else {
            None
        }
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<ClientRect>> {
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
