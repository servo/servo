/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{CacheableWrapper, WrapperCache};
use script_task::global_script_context;

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

    pub fn init_wrapper(@mut self) {
        let script_context = global_script_context();
        let cx = script_context.js_compartment.cx.ptr;
        let owner = script_context.root_frame.get_ref().window;
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        self.wrap_object_shared(cx, scope);
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
