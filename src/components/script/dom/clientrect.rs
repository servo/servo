/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, DerivedWrapper};
use dom::bindings::codegen::ClientRectBinding;
use script_task::{task_from_context, global_script_context};

use js::jsapi::{JSObject, JSContext, JSVal};
use js::glue::RUST_OBJECT_TO_JSVAL;

use std::cast;
use std::f32;

pub struct ClientRect {
    wrapper: WrapperCache,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl ClientRect {
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> @mut ClientRect {
        let rect = @mut ClientRect {
            top: top,
            bottom: bottom,
            left: left,
            right: right,
            wrapper: WrapperCache::new()
        };
        rect.init_wrapper();
        rect
    }

    pub fn init_wrapper(@mut self) {
        let script_context = global_script_context();
        let cx = script_context.js_compartment.cx.ptr;
        let owner = script_context.root_frame.get_ref().window;
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
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
        f32::abs(self.right - self.left)
    }

    pub fn Height(&self) -> f32 {
        f32::abs(self.bottom - self.top)
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
    fn GetParentObject(&self, cx: *JSContext) -> @mut CacheableWrapper {
        let script_context = task_from_context(cx);
        unsafe {
            (*script_context).root_frame.get_ref().window as @mut CacheableWrapper
        }
    }
}

impl DerivedWrapper for ClientRect {
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, _vp: *mut JSVal) -> i32 {
        fail!(~"nyi")
    }

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
