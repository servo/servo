/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TouchListBinding;
use dom::bindings::codegen::Bindings::TouchListBinding::TouchListMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::touch::Touch;
use dom::window::Window;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct TouchList<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    touches: Vec<Dom<Touch<TH>>>,
}

impl<TH: TypeHolderTrait> TouchList<TH> {
    fn new_inherited(touches: &[&Touch<TH>]) -> TouchList<TH> {
        TouchList {
            reflector_: Reflector::new(),
            touches: touches.iter().map(|touch| Dom::from_ref(*touch)).collect(),
        }
    }

    pub fn new(window: &Window<TH>, touches: &[&Touch<TH>]) -> DomRoot<TouchList<TH>> {
        reflect_dom_object(Box::new(TouchList::new_inherited(touches)),
                           window, TouchListBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> TouchListMethods<TH> for TouchList<TH> {
    /// <https://w3c.github.io/touch-events/#widl-TouchList-length>
    fn Length(&self) -> u32 {
        self.touches.len() as u32
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index>
    fn Item(&self, index: u32) -> Option<DomRoot<Touch<TH>>> {
        self.touches.get(index as usize).map(|js| DomRoot::from_ref(&**js))
    }

    /// <https://w3c.github.io/touch-events/#widl-TouchList-item-getter-Touch-unsigned-long-index>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Touch<TH>>> {
        self.Item(index)
    }
}
