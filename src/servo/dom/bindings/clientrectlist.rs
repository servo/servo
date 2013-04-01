/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content::content_task::task_from_context;
use dom::bindings::clientrect::{ClientRect, ClientRectImpl};
use dom::bindings::codegen::ClientRectListBinding;
use dom::bindings::utils::{WrapperCache, CacheableWrapper, BindingObject};
use js::jsapi::{JSObject, JSContext};

pub trait ClientRectList {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<@mut ClientRectImpl>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<@mut ClientRectImpl>;
}

pub struct ClientRectListImpl {
    wrapper: WrapperCache,
    rects: ~[(f32, f32, f32, f32)]
}

impl ClientRectList for ClientRectListImpl {
    fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    fn Item(&self, index: u32) -> Option<@mut ClientRectImpl> {
        if index < self.rects.len() as u32 {
            let (top, bottom, left, right) = self.rects[index];
            Some(@mut ClientRect(top, bottom, left, right))
        } else {
            None
        }
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<@mut ClientRectImpl> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}

pub impl ClientRectListImpl {
    fn new() -> ClientRectListImpl {
        ClientRectListImpl {
            wrapper: WrapperCache::new(),
            rects: ~[(5.6, 80.2, 3.7, 4.8), (800.1, 8001.1, -50.000001, -45.01)]
        }
    }
}

impl CacheableWrapper for ClientRectListImpl {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_unique(~self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"nyi")
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectListBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for ClientRectListImpl {
    fn GetParentObject(&self, cx: *JSContext) -> @mut CacheableWrapper {
        let content = task_from_context(cx);
        unsafe { (*content).window.get() as @mut CacheableWrapper }
    }
}
