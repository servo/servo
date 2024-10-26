/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMRectListBinding::DOMRectListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::domrect::DOMRect;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct DOMRectList {
    reflector_: Reflector,
    rects: DomRefCell<Vec<DomRoot<DOMRect>>>,
}

impl DOMRectList {
    fn new_inherited(rects: Vec<DomRoot<DOMRect>>) -> DOMRectList {
        DOMRectList {
            reflector_: Reflector::new(),
            rects: DomRefCell::new(rects),
        }
    }

    pub fn new(
        window: &Window,
        rects: Vec<DomRoot<DOMRect>>,
        can_gc: CanGc,
    ) -> DomRoot<DOMRectList> {
        reflect_dom_object_with_proto(
            Box::new(DOMRectList::new_inherited(rects)),
            &*window.global(),
            None,
            can_gc,
        )
    }

    #[allow(dead_code)]
    pub fn empty(window: &Window, can_gc: CanGc) -> DomRoot<DOMRectList> {
        DOMRectList::new(window, vec![], can_gc)
    }

    pub fn first(&self) -> Option<DomRoot<DOMRect>> {
        self.rects.borrow().first().cloned()
    }
}

impl DOMRectListMethods for DOMRectList {
    // Implement both Item and IndexedGetter
    fn Item(&self, index: u32) -> Option<DomRoot<DOMRect>> {
        self.rects.borrow().get(index as usize).cloned()
    }

    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DOMRect>> {
        self.Item(index)
    }

    fn Length(&self) -> u32 {
        self.rects.borrow().len() as u32
    }
}
