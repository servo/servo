/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::WritableStreamDefaultWriterBinding::WritableStreamDefaultWriterMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, DomObject, Reflector,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::writablestream::WritableStream;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#writablestreamdefaultwriter>
#[dom_struct]
pub struct WritableStreamDefaultWriter {
    reflector_: Reflector,

    #[ignore_malloc_size_of = "Rc is hard"]
    ready_promise: RefCell<Option<Rc<Promise>>>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultwriter-closedpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    closed_promise: RefCell<Option<Rc<Promise>>>,
}

impl WritableStreamDefaultWriter {
    pub(crate) fn set_ready_promise(&self, promise: Rc<Promise>) {
        *self.ready_promise.borrow_mut() = Some(promise);
    }

    pub(crate) fn resolve_ready_promise(&self) {
        let Some(promise) = &*self.ready_promise.borrow() else {
            unreachable!("Promise should have been set.");
        };
        promise.resolve_native(&());
    }

    pub(crate) fn reject_closed_promise_with_stored_error(&self, error: &SafeHandleValue) {
        let Some(promise) = &*self.closed_promise.borrow() else {
            unreachable!("Promise should have been set.");
        };
        promise.reject_native(error);
    }

    pub(crate) fn set_close_promise_is_handled(&self) {
        let Some(promise) = &*self.closed_promise.borrow() else {
            unreachable!("Promise should have been set.");
        };
        promise.set_promise_is_handled();
    }

    fn set_ready_promise_is_handled(&self) {
        let Some(promise) = &*self.ready_promise.borrow() else {
            unreachable!("Promise should have been set.");
        };
        promise.set_promise_is_handled();
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-ensure-ready-promise-rejected>
    pub(crate) fn ensure_ready_promise_rejected(
        &self,
        global: &GlobalScope,
        error: &SafeHandleValue,
        can_gc: CanGc,
    ) {
        {
            let mut ready_promise = self.ready_promise.borrow_mut();
            // If writer.[[readyPromise]].[[PromiseState]] is "pending", reject writer.[[readyPromise]] with error.
            if let Some(promise) = &*ready_promise {
                if promise.is_pending() {
                    promise.reject_native(error);
                }
            } else {
                // Otherwise, set writer.[[readyPromise]] to a promise rejected with error.
                let promise = Promise::new(global, can_gc);
                promise.reject_native(error);
                *ready_promise = Some(promise.clone());
            }
        }
        // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
        self.set_ready_promise_is_handled();
    }
}

impl WritableStreamDefaultWriterMethods<crate::DomTypeHolder> for WritableStreamDefaultWriter {
    fn Closed(&self) -> Rc<Promise> {
        todo!()
    }

    fn GetDesiredSize(&self) -> Option<f64> {
        todo!()
    }

    fn Ready(&self) -> Rc<Promise> {
        todo!()
    }

    fn Abort(&self, cx: SafeJSContext, reason: SafeHandleValue) -> Rc<Promise> {
        todo!()
    }

    fn Close(&self) -> Rc<Promise> {
        todo!()
    }

    fn ReleaseLock(&self) {
        todo!()
    }

    fn Write(&self, cx: SafeJSContext, chunk: SafeHandleValue) -> Rc<Promise> {
        todo!()
    }

    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        stream: &WritableStream,
    ) -> DomRoot<WritableStreamDefaultWriter> {
        todo!()
    }
}
