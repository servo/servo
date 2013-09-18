/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{CacheableWrapper, BindingObject, DerivedWrapper};
use dom::bindings::utils::{WrapperCache, DOMString, null_str_as_empty};
use dom::bindings::codegen::FormDataBinding;
use dom::blob::Blob;
use script_task::{page_from_context};

use js::jsapi::{JSObject, JSContext, JSVal};
use js::glue::RUST_OBJECT_TO_JSVAL;

use std::cast;
use std::hashmap::HashMap;

enum FormDatum {
    StringData(DOMString),
    BlobData { blob: @mut Blob, name: DOMString }
}

pub struct FormData {
    data: HashMap<~str, FormDatum>,
    wrapper: WrapperCache
}

impl FormData {
    pub fn new() -> @mut FormData {
        @mut FormData {
            data: HashMap::new(),
            wrapper: WrapperCache::new()
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn Append(&mut self, name: &DOMString, value: @mut Blob, filename: Option<DOMString>) {
        let blob = BlobData {
            blob: value,
            name: filename.unwrap_or_default(Some(~"default"))
        };
        self.data.insert(null_str_as_empty(name), blob);
    }

    pub fn Append_(&mut self, name: &DOMString, value: &DOMString) {
        self.data.insert(null_str_as_empty(name), StringData((*value).clone()));
    }
}

impl CacheableWrapper for FormData {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe {
            cast::transmute(&self.wrapper)
        }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        FormDataBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for FormData {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}

impl DerivedWrapper for FormData {
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
