/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{CacheableWrapper, WrapperCache, BindingObject};
use script_task::page_from_context;

use js::jsapi::{JSContext, JSObject};

pub struct WindowProxy {
    wrapper: WrapperCache
}

impl WindowProxy {
    pub fn new() -> @mut WindowProxy {
        @mut WindowProxy {
            wrapper: WrapperCache::new()
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }
}

impl BindingObject for WindowProxy {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}

impl CacheableWrapper for WindowProxy {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        return self.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!("not yet implemented")
    }
}
