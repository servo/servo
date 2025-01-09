/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::TouchListBinding::TouchListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::touch::Touch;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct TouchList {
    reflector_: Reflector,
    touches: Vec<Dom<Touch>>,
}

impl TouchList {
    fn new_inherited(touches: &[&Touch]) -> TouchList {
        TouchList {
            reflector_: Reflector::new(),
            touches: touches.iter().map(|touch| Dom::from_ref(*touch)).collect(),
        }
    }

    pub(crate) fn new(window: &Window, touches: &[&Touch]) -> DomRoot<TouchList> {
        reflect_dom_object(
            Box::new(TouchList::new_inherited(touches)),
            window,
            CanGc::note(),
        )
    }
}

impl TouchListMethods<crate::DomTypeHolder> for TouchList {
    /// <https://w3c.github.io/touch-events/#widl-TouchList-length>
    fn Length(&self) -> u32 {
        self.touches.len() as u32
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index>
    fn Item(&self, index: u32) -> Option<DomRoot<Touch>> {
        self.touches
            .get(index as usize)
            .map(|js| DomRoot::from_ref(&**js))
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Touch>> {
        self.Item(index)
    }
}
