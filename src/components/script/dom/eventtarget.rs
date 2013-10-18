/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::EventTargetBinding;
use dom::bindings::utils::{Reflectable, Reflector};
use script_task::page_from_context;

use js::jsapi::{JSObject, JSContext};

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
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        EventTargetBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        // TODO(tkuehn): This only handles top-level pages. Needs to handle subframes.
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
