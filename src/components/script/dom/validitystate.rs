/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{WrapperCache, BindingObject, CacheableWrapper};
use dom::bindings::codegen::ValidityStateBinding;
use js::jsapi::{JSContext, JSObject};
use std::cast;

pub struct ValidityState {
    wrapper: WrapperCache,
    state: u8
}

impl ValidityState {
    pub fn valid() -> ValidityState {
        ValidityState {
            wrapper: WrapperCache::new(),
            state: 0
        }
    }
}

impl ValidityState {
    pub fn ValueMissing(&self) -> bool {
        false
    }

    pub fn TypeMismatch(&self) -> bool {
        false
    }

    pub fn PatternMismatch(&self) -> bool {
        false
    }

    pub fn TooLong(&self) -> bool {
        false
    }

    pub fn RangeUnderflow(&self) -> bool {
        false
    }

    pub fn RangeOverflow(&self) -> bool {
        false
    }

    pub fn StepMismatch(&self) -> bool {
        false
    }

    pub fn CustomError(&self) -> bool {
        false
    }

    pub fn Valid(&self) -> bool {
        true
    }
}

impl CacheableWrapper for ValidityState {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        ValidityStateBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for ValidityState {
    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut CacheableWrapper> {
        None
    }
}
