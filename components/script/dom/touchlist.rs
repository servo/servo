/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TouchListBinding;
use dom::bindings::codegen::Bindings::TouchListBinding::TouchListMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::touch::Touch;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct TouchList {
    reflector_: Reflector,
    touches: Vec<JS<Touch>>,
}

impl TouchList {
    fn new_inherited(touches: &[&Touch]) -> TouchList {
        TouchList {
            reflector_: Reflector::new(),
            touches: touches.iter().map(|touch| JS::from_ref(*touch)).collect(),
        }
    }

    pub fn new(window: &Window, touches: &[&Touch]) -> Root<TouchList> {
        reflect_dom_object(box TouchList::new_inherited(touches),
                           window, TouchListBinding::Wrap)
    }
}

impl TouchListMethods for TouchList {
    /// https://w3c.github.io/touch-events/#widl-TouchList-length
    fn Length(&self) -> u32 {
        self.touches.len() as u32
    }

    /// https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index
    fn Item(&self, index: u32) -> Option<Root<Touch>> {
        self.touches.get(index as usize).map(|js| Root::from_ref(&**js))
    }

    /// https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index
    fn IndexedGetter(&self, index: u32) -> Option<Root<Touch>> {
        self.Item(index)
    }
}
