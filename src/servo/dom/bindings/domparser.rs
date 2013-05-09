/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMParserBinding;
use dom::bindings::utils::{CacheableWrapper, WrapperCache};
use dom::bindings::utils::{BindingObject, DerivedWrapper};
use dom::domparser::DOMParser;

use js::jsapi::{JSContext, JSObject, JSVal};
use js::glue::bindgen::{RUST_OBJECT_TO_JSVAL};

impl CacheableWrapper for DOMParser {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        DOMParserBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for DOMParser {
    fn GetParentObject(&self, _cx: *JSContext) -> @mut CacheableWrapper {
        return self.owner as @mut CacheableWrapper;
    }
}

impl DerivedWrapper for DOMParser {
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
