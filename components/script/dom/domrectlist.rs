/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectListBinding;
use dom::bindings::codegen::Bindings::DOMRectListBinding::DOMRectListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::domrect::DOMRect;
use dom::window::Window;

#[dom_struct]
pub struct DOMRectList {
    reflector_: Reflector,
    rects: Vec<JS<DOMRect>>,
    window: JS<Window>,
}

impl DOMRectList {
    fn new_inherited<T>(window: &Window, rects: T) -> DOMRectList
                        where T: Iterator<Item=Root<DOMRect>> {
        DOMRectList {
            reflector_: Reflector::new(),
            rects: rects.map(|r| JS::from_rooted(&r)).collect(),
            window: JS::from_ref(window),
        }
    }

    pub fn new<T>(window: &Window, rects: T) -> Root<DOMRectList>
                  where T: Iterator<Item=Root<DOMRect>> {
        reflect_dom_object(box DOMRectList::new_inherited(window, rects),
                           GlobalRef::Window(window), DOMRectListBinding::Wrap)
    }
}

impl<'a> DOMRectListMethods for &'a DOMRectList {
    fn Length(self) -> u32 {
        self.rects.len() as u32
    }

    fn Item(self, index: u32) -> Option<Root<DOMRect>> {
        let rects = &self.rects;
        if index < rects.len() as u32 {
            Some(rects[index as usize].root())
        } else {
            None
        }
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Root<DOMRect>> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}

