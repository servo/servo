/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::PromiseNativeHandlerBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
//use js::jsapi::JSContext;
use crate::script_runtime::JSContext;
use js::rust::HandleValue;
use malloc_size_of::MallocSizeOf;

pub trait Callback: JSTraceable + MallocSizeOf {
    fn callback(&self, cx: *mut JSContext, v: HandleValue);
}

#[dom_struct]
pub struct PromiseNativeHandler {
    reflector: Reflector,
    resolve: Option<Box<dyn Callback>>,
    reject: Option<Box<dyn Callback>>,
}

impl PromiseNativeHandler {
    pub fn new(
        global: &GlobalScope,
        resolve: Option<Box<dyn Callback>>,
        reject: Option<Box<dyn Callback>>,
    ) -> DomRoot<PromiseNativeHandler> {
        reflect_dom_object(
            Box::new(PromiseNativeHandler {
                reflector: Reflector::new(),
                resolve: resolve,
                reject: reject,
            }),
            global,
            PromiseNativeHandlerBinding::Wrap,
        )
    }

    fn callback(callback: &Option<Box<dyn Callback>>, cx: *mut JSContext, v: HandleValue) {
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
