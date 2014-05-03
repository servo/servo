/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::ClientRectBinding;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use servo_util::geometry::Au;

#[deriving(Encodable)]
pub struct ClientRect {
    pub reflector_: Reflector,
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    pub window: JS<Window>,
}

impl ClientRect {
    pub fn new_inherited(window: &JSRef<Window>,
                         top: Au, bottom: Au,
                         left: Au, right: Au) -> ClientRect {
        ClientRect {
            top: top.to_nearest_px() as f32,
            bottom: bottom.to_nearest_px() as f32,
            left: left.to_nearest_px() as f32,
            right: right.to_nearest_px() as f32,
            reflector_: Reflector::new(),
            window: window.unrooted(),
        }
    }

    pub fn new(window: &JSRef<Window>,
               top: Au, bottom: Au,
               left: Au, right: Au) -> Temporary<ClientRect> {
        let rect = ClientRect::new_inherited(window, top, bottom, left, right);
        reflect_dom_object(~rect, window, ClientRectBinding::Wrap)
    }
}

pub trait ClientRectMethods {
    fn Top(&self) -> f32;
    fn Bottom(&self) -> f32;
    fn Left(&self) -> f32;
    fn Right(&self) -> f32;
    fn Width(&self) -> f32;
    fn Height(&self) -> f32;
}

impl<'a> ClientRectMethods for JSRef<'a, ClientRect> {
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

impl Reflectable for ClientRect {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
