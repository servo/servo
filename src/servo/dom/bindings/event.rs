/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content::content_task::{task_from_context, global_content};
use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, DerivedWrapper};
use dom::bindings::codegen::EventBinding;
use dom::event::Event_;
use js::jsapi::{JSObject, JSContext, JSVal};
use js::glue::bindgen::RUST_OBJECT_TO_JSVAL;

pub impl Event_ {
    pub fn init_wrapper(@mut self) {
        let content = global_content();
        let cx = content.compartment.get().cx.ptr;
        let owner = content.window.get();
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        self.wrap_object_shared(cx, scope);
    }
}

impl CacheableWrapper for Event_ {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        EventBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Event_ {
    fn GetParentObject(&self, cx: *JSContext) -> @mut CacheableWrapper {
        let content = task_from_context(cx);
        unsafe { (*content).window.get() as @mut CacheableWrapper }
    }
}

impl DerivedWrapper for Event_ {
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
