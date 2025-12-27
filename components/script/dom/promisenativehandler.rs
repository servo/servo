/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::realm::CurrentRealm;
use js::rust::HandleValue;
use malloc_size_of::MallocSizeOf;

use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// Types that implement the `Callback` trait follow the same rooting requirements
/// as types that use the `#[dom_struct]` attribute.
/// Prefer storing `Dom<T>` members inside them instead of `DomRoot<T>`
/// to minimize redundant work by the garbage collector.
pub(crate) trait Callback: JSTraceable + MallocSizeOf {
    fn callback(&self, cx: &mut CurrentRealm, v: HandleValue);
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
        can_gc: CanGc,
    ) -> DomRoot<PromiseNativeHandler> {
        reflect_dom_object(
            Box::new(PromiseNativeHandler {
                reflector: Reflector::new(),
                resolve,
                reject,
            }),
            global,
            can_gc,
        )
    }

    fn callback(callback: &Option<Box<dyn Callback>>, cx: &mut CurrentRealm, v: HandleValue) {
        if let Some(ref callback) = *callback {
            callback.callback(cx, v)
        }
    }

    pub(crate) fn resolved_callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        PromiseNativeHandler::callback(&self.resolve, cx, v)
    }

    pub(crate) fn rejected_callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        PromiseNativeHandler::callback(&self.reject, cx, v)
    }
}
