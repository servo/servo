/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::ClientRectListBinding;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::clientrect::ClientRect;
use dom::window::Window;

pub struct ClientRectList {
    reflector_: Reflector,
    rects: ~[@mut ClientRect],
    window: @mut Window,
}

impl ClientRectList {
    pub fn new_inherited(window: @mut Window,
                         rects: ~[@mut ClientRect]) -> ClientRectList {
        ClientRectList {
            reflector_: Reflector::new(),
            rects: rects,
            window: window,
        }
    }

    pub fn new(window: @mut Window,
               rects: ~[@mut ClientRect]) -> @mut ClientRectList {
        reflect_dom_object(@mut ClientRectList::new_inherited(window, rects),
                           window, ClientRectListBinding::Wrap)
    }

    pub fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    pub fn Item(&self, index: u32) -> Option<@mut ClientRect> {
        if index < self.rects.len() as u32 {
            Some(self.rects[index])
        } else {
            None
        }
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<@mut ClientRect> {
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
