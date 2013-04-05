/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content::content_task::task_from_context;
use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject, OpaqueBindingReference};
use dom::bindings::codegen::ClientRectBinding;
use js::jsapi::{JSObject, JSContext};

pub trait ClientRect {
    fn Top(&self) -> f32;
    fn Bottom(&self) -> f32;
    fn Left(&self) -> f32;
    fn Right(&self) -> f32;
    fn Width(&self) -> f32;
    fn Height(&self) -> f32;
}

pub struct ClientRectImpl {
    wrapper: WrapperCache,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl ClientRect for ClientRectImpl {
    fn Top(&self) -> f32 {
        self.top
    }

    fn Bottom(&self) -> f32 {
        self.bottom
    }

    fn Left(&self) -> f32 {
        self.left
    }

    fn Right(&self) -> f32 {
        self.right
    }

    fn Width(&self) -> f32 {
        f32::abs(self.right - self.left)
    }

    fn Height(&self) -> f32 {
        f32::abs(self.bottom - self.top)
    }
}

pub fn ClientRect(top: f32, bottom: f32, left: f32, right: f32) -> ClientRectImpl {
    ClientRectImpl {
        top: top, bottom: bottom, left: left, right: right,
        wrapper: WrapperCache::new()
    }
}

impl CacheableWrapper for ClientRectImpl {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_unique(~self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectBinding::Wrap(cx, scope, self, &mut unused)
    }

    fn wrap_object_shared(@self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"nyi")
    }
}

impl BindingObject for ClientRectImpl {
    fn GetParentObject(&self, cx: *JSContext) -> OpaqueBindingReference {
        let content = task_from_context(cx);
        unsafe { OpaqueBindingReference(Right((*content).window.get() as @CacheableWrapper)) }
    }
}