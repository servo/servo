/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{WrapperCache, BindingObject, CacheableWrapper};
use dom::bindings::codegen::BlobBinding;
use script_task::{page_from_context};

use js::jsapi::{JSContext, JSObject};

use std::cast;

pub struct Blob {
    wrapper: WrapperCache
}

impl Blob {
    pub fn new() -> @mut Blob {
        @mut Blob {
            wrapper: WrapperCache::new()
        }
    }
}

impl CacheableWrapper for Blob {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        BlobBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Blob {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}
