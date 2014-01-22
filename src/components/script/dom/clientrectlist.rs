/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::ClientRectListBinding;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object2};
use dom::clientrect::ClientRect;
use dom::window::Window;

pub struct ClientRectList {
    reflector_: Reflector,
    rects: ~[JSManaged<ClientRect>],
    window: JSManaged<Window>,
    force_box_layout: @int
}

impl ClientRectList {
    pub fn new_inherited(window: JSManaged<Window>,
                         rects: ~[JSManaged<ClientRect>]) -> ClientRectList {
        ClientRectList {
            reflector_: Reflector::new(),
            rects: rects,
            window: window,
            force_box_layout: @1
        }
    }

    pub fn new(window: JSManaged<Window>,
               rects: ~[JSManaged<ClientRect>]) -> JSManaged<ClientRectList> {
        reflect_dom_object2(~ClientRectList::new_inherited(window, rects),
                            window.value(), ClientRectListBinding::Wrap)
    }

    pub fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    pub fn Item(&self, index: u32) -> Option<JSManaged<ClientRect>> {
        if index < self.rects.len() as u32 {
            Some(self.rects[index])
        } else {
            None
        }
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<JSManaged<ClientRect>> {
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
