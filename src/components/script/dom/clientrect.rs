/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::ClientRectBinding;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;

use js::jsapi::{JSObject, JSContext};

pub struct ClientRect {
    reflector_: Reflector,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
    window: @mut Window,
}

impl ClientRect {
    pub fn new_inherited(window: @mut Window,
                         top: f32, bottom: f32,
                         left: f32, right: f32) -> ClientRect {
        ClientRect {
            top: top,
            bottom: bottom,
            left: left,
            right: right,
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: @mut Window,
               top: f32, bottom: f32,
               left: f32, right: f32) -> @mut ClientRect {
        let rect = ClientRect::new_inherited(window, top, bottom, left, right);
        reflect_dom_object(@mut rect, window, ClientRectBinding::Wrap)
    }

    pub fn Top(&self) -> f32 {
        self.top
    }

    pub fn Bottom(&self) -> f32 {
        self.bottom
    }

    pub fn Left(&self) -> f32 {
        self.left
    }

    pub fn Right(&self) -> f32 {
        self.right
    }

    pub fn Width(&self) -> f32 {
        (self.right - self.left).abs()
    }

    pub fn Height(&self) -> f32 {
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

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        unreachable!();
    }

    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut Reflectable> {
        Some(self.window as @mut Reflectable)
    }
}
