/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::EventTargetBinding;
use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, DerivedWrapper};
use script_task::page_from_context;

use js::glue::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSObject, JSContext, JSVal};

use std::cast;

pub struct EventTarget {
    wrapper: WrapperCache
}

impl EventTarget {
    pub fn new() -> ~EventTarget {
        ~EventTarget {
            wrapper: WrapperCache::new()
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }
}

impl CacheableWrapper for EventTarget {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        EventTargetBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for EventTarget {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        // TODO(tkuehn): This only handles top-level pages. Needs to handle subframes.
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}

impl DerivedWrapper for EventTarget {
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, _vp: *mut JSVal) -> i32 {
        fail!(~"nyi")
    }

    #[fixed_stack_segment]
    fn wrap_shared(@mut self, cx: *JSContext, scope: *JSObject, vp: *mut JSVal) -> i32 {
        let obj = self.wrap_object_shared(cx, scope);
        if obj.is_null() {
            return 0;
        } else {
            unsafe {
                *vp = RUST_OBJECT_TO_JSVAL(obj)
            };
            return 1;
        }
    }
}
