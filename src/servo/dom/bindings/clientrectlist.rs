/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content::content_task::{task_from_context, global_content};
use dom::bindings::codegen::ClientRectListBinding;
use dom::bindings::utils::{WrapperCache, CacheableWrapper, BindingObject};
use dom::clientrectlist::ClientRectList;
use js::jsapi::{JSObject, JSContext};

pub impl ClientRectList {
    fn init_wrapper(@mut self) {
        let content = global_content();
        let cx = content.compartment.get().cx.ptr;
        let owner = content.window.get();
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        self.wrap_object_shared(cx, scope);
    }
}

impl CacheableWrapper for ClientRectList {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ClientRectListBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for ClientRectList {
    fn GetParentObject(&self, cx: *JSContext) -> @mut CacheableWrapper {
        let content = task_from_context(cx);
        unsafe { (*content).window.get() as @mut CacheableWrapper }
    }
}
