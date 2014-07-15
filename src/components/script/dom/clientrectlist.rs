/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ClientRectListBinding;
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::clientrect::ClientRect;
use dom::window::Window;

#[deriving(Encodable)]
pub struct ClientRectList {
    reflector_: Reflector,
    rects: Vec<JS<ClientRect>>,
    window: JS<Window>,
}

impl ClientRectList {
    pub fn new_inherited(window: &JSRef<Window>,
                         rects: Vec<JSRef<ClientRect>>) -> ClientRectList {
        let rects = rects.iter().map(|rect| JS::from_rooted(rect)).collect();
        ClientRectList {
            reflector_: Reflector::new(),
            rects: rects,
            window: JS::from_rooted(window),
        }
    }

    pub fn new(window: &JSRef<Window>,
               rects: Vec<JSRef<ClientRect>>) -> Temporary<ClientRectList> {
        reflect_dom_object(box ClientRectList::new_inherited(window, rects),
                           &Window(*window), ClientRectListBinding::Wrap)
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
        let rects = &self.rects;
        if index < rects.len() as u32 {
            Some(Temporary::new(rects.get(index as uint).clone()))
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
}
