/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::codegen::ClientRectBinding;
use script_task::page_from_context;

use js::jsapi::{JSObject, JSContext};

pub struct ClientRect {
    reflector_: Reflector,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl ClientRect {
    pub fn new(top: f32, bottom: f32, left: f32, right: f32, cx: *JSContext, scope: *JSObject) -> @mut ClientRect {
        let rect = @mut ClientRect {
            top: top,
            bottom: bottom,
            left: left,
            right: right,
            reflector_: Reflector::new()
        };
        rect.init_wrapper(cx, scope);
        rect
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
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

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        ClientRectBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
