/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PromiseNativeHandlerBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use heapsize::HeapSizeOf;
use js::jsapi::{JSContext, HandleValue, JSTracer};

pub type CallbackType = Box<Callback>;

impl JSTraceable for Box<Callback> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc);
    }
}

pub trait Callback: JSTraceable + HeapSizeOf {
    fn callback(&self, cx: *mut JSContext, v: HandleValue);
}

#[dom_struct]
pub struct PromiseNativeHandler {
    reflector: Reflector,
    resolve: Option<CallbackType>,
    reject: Option<CallbackType>,
}

impl PromiseNativeHandler {
    pub fn new(global: GlobalRef,
           resolve: Option<CallbackType>,
           reject: Option<CallbackType>) -> Root<PromiseNativeHandler> {
        reflect_dom_object(box PromiseNativeHandler {
            reflector: Reflector::new(),
            resolve: resolve,
            reject: reject,
        }, global, PromiseNativeHandlerBinding::Wrap)
    }

    fn callback(callback: &Option<CallbackType>, cx: *mut JSContext, v: HandleValue) {
        if let &Some(ref callback) = callback {
            callback.callback(cx, v)
        }
    }

    pub fn resolved_callback(&self, cx: *mut JSContext, v: HandleValue) {
        PromiseNativeHandler::callback(&self.resolve, cx, v)
    }

    pub fn rejected_callback(&self, cx: *mut JSContext, v: HandleValue) {
        PromiseNativeHandler::callback(&self.reject, cx, v)
    }
}
