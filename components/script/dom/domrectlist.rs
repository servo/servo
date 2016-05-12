/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectListBinding;
use dom::bindings::codegen::Bindings::DOMRectListBinding::DOMRectListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::domrect::DOMRect;
use dom::window::Window;

pub struct Iter<'a> {
    rects: &'a DOMRectList,
    position: usize
}

#[dom_struct]
pub struct DOMRectList {
    reflector_: Reflector,
    rects: Vec<JS<DOMRect>>,
}

impl DOMRectList {
    fn new_inherited<T>(rects: T) -> DOMRectList
        where T: Iterator<Item = Root<DOMRect>>
    {
        DOMRectList {
            reflector_: Reflector::new(),
            rects: rects.map(|r| JS::from_rooted(&r)).collect(),
        }
    }

    pub fn new<T>(window: &Window, rects: T) -> Root<DOMRectList>
        where T: Iterator<Item = Root<DOMRect>>
    {
        reflect_dom_object(box DOMRectList::new_inherited(rects),
                           GlobalRef::Window(window),
                           DOMRectListBinding::Wrap)
    }

    pub fn iter(&self) -> Iter {
        Iter { rects: self, position: 0 }
    }
}

impl DOMRectListMethods for DOMRectList {
    // https://drafts.fxtf.org/geometry/#dom-domrectlist-length
    fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectlist-item
    fn Item(&self, index: u32) -> Option<Root<DOMRect>> {
        let rects = &self.rects;
        if index < rects.len() as u32 {
            Some(Root::from_ref(&*rects[index as usize]))
        } else {
            None
        }
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<DOMRect>> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Root<DOMRect>;

    fn next(&mut self) -> Option<Root<DOMRect>> {
        let rv = self.rects.Item(self.position as u32);
        match rv {
            Some(rv) => {
                self.position = self.position + 1;
                Some(rv)
            },
            None => None
        }
    }
}
