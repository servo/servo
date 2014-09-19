/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectListBinding;
use dom::bindings::codegen::Bindings::DOMRectListBinding::DOMRectListMethods;
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::domrect::DOMRect;
use dom::window::Window;

#[deriving(Encodable)]
#[must_root]
pub struct DOMRectList {
    reflector_: Reflector,
    rects: Vec<JS<DOMRect>>,
    window: JS<Window>,
}

impl DOMRectList {
    pub fn new_inherited(window: JSRef<Window>,
                         rects: Vec<JSRef<DOMRect>>) -> DOMRectList {
        let rects = rects.iter().map(|rect| JS::from_rooted(*rect)).collect();
        DOMRectList {
            reflector_: Reflector::new(),
            rects: rects,
            window: JS::from_rooted(window),
        }
    }

    pub fn new(window: JSRef<Window>,
               rects: Vec<JSRef<DOMRect>>) -> Temporary<DOMRectList> {
        reflect_dom_object(box DOMRectList::new_inherited(window, rects),
                           &Window(window), DOMRectListBinding::Wrap)
    }
}

impl<'a> DOMRectListMethods for JSRef<'a, DOMRectList> {
    fn Length(self) -> u32 {
        self.rects.len() as u32
    }

    fn Item(self, index: u32) -> Option<Temporary<DOMRect>> {
        let rects = &self.rects;
        if index < rects.len() as u32 {
            Some(Temporary::new(rects[index as uint].clone()))
        } else {
            None
        }
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Temporary<DOMRect>> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}

impl Reflectable for DOMRectList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
