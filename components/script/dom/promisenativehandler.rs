/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::JSContext;
use js::rust::HandleValue;
use malloc_size_of::MallocSizeOf;

use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// Types that implement the `Callback` trait follow the same rooting requirements
/// as types that use the `#[dom_struct]` attribute.
/// Prefer storing `Dom<T>` members inside them instead of `DomRoot<T>`
/// to minimize redundant work by the garbage collector.
pub(crate) trait Callback: JSTraceable + MallocSizeOf {
    fn callback(&self, cx: SafeJSContext, v: HandleValue, realm: InRealm, can_gc: CanGc);
}

#[dom_struct]
pub(crate) struct PromiseNativeHandler {
    reflector: Reflector,
    resolve: Option<Box<dyn Callback>>,
    reject: Option<Box<dyn Callback>>,
}

impl PromiseNativeHandler {
    pub(crate) fn new(
        global: &GlobalScope,
        resolve: Option<Box<dyn Callback>>,
        reject: Option<Box<dyn Callback>>,
    ) -> DomRoot<PromiseNativeHandler> {
        reflect_dom_object(
            Box::new(PromiseNativeHandler {
                reflector: Reflector::new(),
                resolve,
                reject,
            }),
            global,
            CanGc::note(),
        )
    }

    #[allow(unsafe_code)]
    fn callback(
        callback: &Option<Box<dyn Callback>>,
        cx: *mut JSContext,
        v: HandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        let cx = unsafe { SafeJSContext::from_ptr(cx) };
        if let Some(ref callback) = *callback {
            callback.callback(cx, v, realm, can_gc)
        }
    }

    pub(crate) fn resolved_callback(
        &self,
        cx: *mut JSContext,
        v: HandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        PromiseNativeHandler::callback(&self.resolve, cx, v, realm, can_gc)
    }

    pub(crate) fn rejected_callback(
        &self,
        cx: *mut JSContext,
        v: HandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        PromiseNativeHandler::callback(&self.reject, cx, v, realm, can_gc)
    }
}
