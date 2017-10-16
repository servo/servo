/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PromiseNativeHandlerBinding;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::trace::JSTraceable;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use heapsize::HeapSizeOf;
use js::jsapi::{JSContext, HandleValue};

pub trait Callback: JSTraceable + HeapSizeOf {
    fn callback(&self, cx: *mut JSContext, v: HandleValue);
}

#[dom_struct]
pub struct PromiseNativeHandler {
    reflector: Reflector,
    resolve: Option<Box<Callback>>,
    reject: Option<Box<Callback>>,
}

impl PromiseNativeHandler {
    pub fn new(global: &GlobalScope,
               resolve: Option<Box<Callback>>,
               reject: Option<Box<Callback>>)
               -> DomRoot<PromiseNativeHandler> {
        reflect_dom_object(Box::new(PromiseNativeHandler {
            reflector: Reflector::new(),
            resolve: resolve,
            reject: reject,
        }), global, PromiseNativeHandlerBinding::Wrap)
    }

    fn callback(callback: &Option<Box<Callback>>, cx: *mut JSContext, v: HandleValue) {
        if let Some(ref callback) = *callback {
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
