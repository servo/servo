/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectBinding;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use servo_util::geometry::Au;

#[deriving(Encodable)]
pub struct DOMRect {
    reflector_: Reflector,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
    window: JS<Window>,
}

impl DOMRect {
    pub fn new_inherited(window: &JSRef<Window>,
                         top: Au, bottom: Au,
                         left: Au, right: Au) -> DOMRect {
        DOMRect {
            top: top.to_nearest_px() as f32,
            bottom: bottom.to_nearest_px() as f32,
            left: left.to_nearest_px() as f32,
            right: right.to_nearest_px() as f32,
            reflector_: Reflector::new(),
            window: JS::from_rooted(window),
        }
    }

    pub fn new(window: &JSRef<Window>,
               top: Au, bottom: Au,
               left: Au, right: Au) -> Temporary<DOMRect> {
        let rect = DOMRect::new_inherited(window, top, bottom, left, right);
        reflect_dom_object(box rect, window, DOMRectBinding::Wrap)
    }
}

pub trait DOMRectMethods {
    fn Top(&self) -> f32;
    fn Bottom(&self) -> f32;
    fn Left(&self) -> f32;
    fn Right(&self) -> f32;
    fn Width(&self) -> f32;
    fn Height(&self) -> f32;
}

impl<'a> DOMRectMethods for JSRef<'a, DOMRect> {
    fn Top(&self) -> f32 {
        self.top
    }

    fn Bottom(&self) -> f32 {
        self.bottom
    }

    fn Left(&self) -> f32 {
        self.left
    }

    fn Right(&self) -> f32 {
        self.right
    }

    fn Width(&self) -> f32 {
        (self.right - self.left).abs()
    }

    fn Height(&self) -> f32 {
        (self.bottom - self.top).abs()
    }
}

impl Reflectable for DOMRect {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
