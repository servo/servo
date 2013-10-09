/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::EventTargetBinding;
use dom::bindings::utils::{Reflectable, Reflector, BindingObject, DerivedWrapper};
use script_task::page_from_context;

use js::glue::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSObject, JSContext, JSVal};

use std::cast;

pub struct EventTarget {
    reflector_: Reflector
}

impl EventTarget {
    pub fn new() -> ~EventTarget {
        ~EventTarget {
            reflector_: Reflector::new()
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }
}

impl Reflectable for EventTarget {
    fn reflector(&mut self) -> &mut Reflector {
        unsafe { cast::transmute(&self.reflector_) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        EventTargetBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for EventTarget {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        // TODO(tkuehn): This only handles top-level pages. Needs to handle subframes.
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
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
