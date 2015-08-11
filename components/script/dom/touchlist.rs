/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TouchListBinding;
use dom::bindings::codegen::Bindings::TouchListBinding::TouchListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::touch::Touch;
use dom::window::Window;

#[dom_struct]
pub struct TouchList {
    reflector_: Reflector,
    touches: Vec<JS<Touch>>,
}

impl TouchList {
    fn new_inherited(touches: Vec<JS<Touch>>) -> TouchList {
        TouchList {
            reflector_: Reflector::new(),
            touches: touches,
        }
    }

    pub fn new(window: &Window, touches: Vec<JS<Touch>>) -> Root<TouchList> {
        reflect_dom_object(box TouchList::new_inherited(touches),
                           GlobalRef::Window(window), TouchListBinding::Wrap)
    }
}

impl TouchListMethods for TouchList {
    /// https://w3c.github.io/touch-events/#widl-TouchList-length
    fn Length(&self) -> u32 {
        self.touches.len() as u32
    }

    /// https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index
    fn Item(&self, index: u32) -> Option<Root<Touch>> {
        self.touches.get(index as usize).map(|x| Root::from_rooted(*x))
    }

    /// https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<Touch>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}
