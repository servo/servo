/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::reflector::Reflectable;
use dom::eventtarget::EventTarget;
use js::jsapi::{JS_GetContext, JS_GetObjectRuntime, JSContext};

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
}

impl GlobalScope {
    pub fn new_inherited() -> GlobalScope {
        GlobalScope {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> *mut JSContext {
        unsafe {
            let runtime = JS_GetObjectRuntime(
                self.reflector().get_jsobject().get());
            assert!(!runtime.is_null());
            let context = JS_GetContext(runtime);
            assert!(!context.is_null());
            context
        }
    }
}
