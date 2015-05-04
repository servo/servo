/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectBinding;
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::num::Finite;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::window::Window;
use util::geometry::Au;

#[dom_struct]
pub struct DOMRect {
    reflector_: Reflector,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl DOMRect {
    fn new_inherited(top: Au, bottom: Au,
                         left: Au, right: Au) -> DOMRect {
        DOMRect {
            top: top.to_nearest_px() as f32,
            bottom: bottom.to_nearest_px() as f32,
            left: left.to_nearest_px() as f32,
            right: right.to_nearest_px() as f32,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: JSRef<Window>,
               top: Au, bottom: Au,
               left: Au, right: Au) -> Temporary<DOMRect> {
        reflect_dom_object(box DOMRect::new_inherited(top, bottom, left, right),
                           GlobalRef::Window(window), DOMRectBinding::Wrap)
    }
}

impl<'a> DOMRectMethods for JSRef<'a, DOMRect> {
    fn Top(self) -> Finite<f32> {
        Finite::wrap(self.top)
    }

    fn Bottom(self) -> Finite<f32> {
        Finite::wrap(self.bottom)
    }

    fn Left(self) -> Finite<f32> {
        Finite::wrap(self.left)
    }

    fn Right(self) -> Finite<f32> {
        Finite::wrap(self.right)
    }

    fn Width(self) -> Finite<f32> {
        let result = (self.right - self.left).abs();
        Finite::wrap(result)
    }

    fn Height(self) -> Finite<f32> {
        let result = (self.bottom - self.top).abs();
        Finite::wrap(result)
    }
}

