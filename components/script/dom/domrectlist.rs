/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMRectListBinding::DOMRectListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::domrect::DOMRect;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMRectList {
    reflector_: Reflector,
    rects: DomRefCell<Vec<Dom<DOMRect>>>,
}

impl DOMRectList {
    fn new_inherited(rects: Vec<DomRoot<DOMRect>>) -> DOMRectList {
        DOMRectList {
            reflector_: Reflector::new(),
            rects: DomRefCell::new(
                rects
                    .into_iter()
                    .map(|dom_root| dom_root.as_traced())
                    .collect(),
            ),
        }
    }

    pub(crate) fn new(
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

    pub(crate) fn first(&self) -> Option<DomRoot<DOMRect>> {
        self.rects.borrow().first().map(Dom::as_rooted)
    }
}

impl DOMRectListMethods<crate::DomTypeHolder> for DOMRectList {
    /// <https://drafts.fxtf.org/geometry/#DOMRectList>
    fn Item(&self, index: u32) -> Option<DomRoot<DOMRect>> {
        self.rects.borrow().get(index as usize).map(Dom::as_rooted)
    }

    /// <https://drafts.fxtf.org/geometry/#DOMRectList>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DOMRect>> {
        self.Item(index)
    }

    /// <https://drafts.fxtf.org/geometry/#DOMRectList>
    fn Length(&self) -> u32 {
        self.rects.borrow().len() as u32
    }
}
