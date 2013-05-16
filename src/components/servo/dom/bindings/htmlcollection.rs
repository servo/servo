/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLCollectionBinding;
use dom::bindings::utils::{CacheableWrapper, BindingObject, WrapperCache};
use dom::htmlcollection::HTMLCollection;
use js::jsapi::{JSObject, JSContext};
use scripting::script_task::{task_from_context, global_script_context};

pub impl HTMLCollection {
    fn init_wrapper(@mut self) {
        let script_context = global_script_context();
        let cx = script_context.compartment.get().cx.ptr;
        let owner = script_context.window.get();
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        self.wrap_object_shared(cx, scope);
    }
}

impl BindingObject for HTMLCollection {
    fn GetParentObject(&self, cx: *JSContext) -> @mut CacheableWrapper {
        let script_context = task_from_context(cx);
        unsafe { (*script_context).window.get() as @mut CacheableWrapper }
    }
}

impl CacheableWrapper for HTMLCollection {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        HTMLCollectionBinding::Wrap(cx, scope, self, &mut unused)
    }
}
