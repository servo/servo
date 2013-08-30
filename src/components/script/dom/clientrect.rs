/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, DerivedWrapper};
use dom::bindings::codegen::ClientRectBinding;
use script_task::page_from_context;

use js::jsapi::{JSObject, JSContext, JSVal};
use js::glue::RUST_OBJECT_TO_JSVAL;

use std::cast;

pub struct ClientRect {
    wrapper: WrapperCache,
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
            wrapper: WrapperCache::new()
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

impl CacheableWrapper for ClientRect {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe {
            cast::transmute(&self.wrapper)
        }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for ClientRect {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}

impl DerivedWrapper for ClientRect {
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, _vp: *mut JSVal) -> i32 {
        fail!(~"nyi")
    }

    #[fixed_stack_segment]
    fn wrap_shared(@mut self, cx: *JSContext, scope: *JSObject, vp: *mut JSVal) -> i32 {
        let obj = self.wrap_object_shared(cx, scope);
        if obj.is_null() {
            return 0;
        } else {
            unsafe { *vp = RUST_OBJECT_TO_JSVAL(obj) };
            return 1;
        }
    }
}
