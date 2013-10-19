/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::utils::Fallible;
use dom::bindings::codegen::BlobBinding;
use dom::window::Window;

use js::jsapi::{JSContext, JSObject};

pub struct Blob {
    reflector_: Reflector,
    window: @mut Window,
}

impl Blob {
    pub fn new_inherited(window: @mut Window) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: @mut Window) -> @mut Blob {
        let blob = @mut Blob::new_inherited(window);
        let cx = window.get_cx();
        let scope = window.reflector().get_jsobject();
        if BlobBinding::Wrap(cx, scope, blob).is_null() {
            fail!("BlobBinding::Wrap failed");
        }
        assert!(blob.reflector().get_jsobject().is_not_null());
        blob
    }
}

impl Blob {
    pub fn Constructor(window: @mut Window) -> Fallible<@mut Blob> {
        Ok(Blob::new(window))
    }
}

impl Reflectable for Blob {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        unreachable!();
    }

    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut Reflectable> {
        Some(self.window as @mut Reflectable)
    }
}
