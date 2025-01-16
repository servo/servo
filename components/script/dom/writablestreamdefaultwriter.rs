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
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
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

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultwriter-stream>
    stream: MutNullableDom<WritableStream>,
}

impl WritableStreamDefaultWriter {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited() -> WritableStreamDefaultWriter {
        WritableStreamDefaultWriter {
            reflector_: Reflector::new(),
            stream: Default::default(),
            closed_promise: Default::default(),
            ready_promise: Default::default(),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<WritableStreamDefaultWriter> {
        reflect_dom_object_with_proto(
            Box::new(WritableStreamDefaultWriter::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-writer>
    fn setup(&self, stream: &WritableStream, global: &GlobalScope, can_gc: CanGc) -> Result<(), Error> {
        // If ! IsWritableStreamLocked(stream) is true, throw a TypeError exception.
        if stream.is_locked() {
            return Err(Error::Type("Stream is locked".to_string()));
        }

        // Set writer.[[stream]] to stream.
        self.stream.set(Some(stream));
        
        // Set stream.[[writer]] to writer.
        stream.set_writer(Some(&self));
        
        // Let state be stream.[[state]].
        
        // If state is "writable",
        if stream.is_writable() {
            // If ! WritableStreamCloseQueuedOrInFlight(stream) is false 
            // and stream.[[backpressure]] is true, 
            if !stream.close_queued_or_in_flight() && stream.get_backpressure() {
                // set writer.[[readyPromise]] to a new promise.
                let promise = Promise::new(global, can_gc);
                self.set_ready_promise(promise);
            } else {
                // Otherwise, set writer.[[readyPromise]] to a promise resolved with undefined.
                let promise = Promise::new(global, can_gc);
                promise.resolve_native(&());
                self.set_ready_promise(promise);
            }
            
            // Set writer.[[closedPromise]] to a new promise.
            let promise = Promise::new(global, can_gc);
            self.set_closed_promise(promise);
            return Ok(());
        }
        
        // Otherwise, if state is "erroring",
        if stream.is_writable() {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            
            // Set writer.[[readyPromise]] to a promise rejected with stream.[[storedError]].
            // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
            let promise = Promise::new(global, can_gc);
            promise.reject_native(&error.handle());
            promise.set_promise_is_handled();
            self.set_ready_promise(promise);
            
            // Set writer.[[closedPromise]] to a new promise.
            let promise = Promise::new(global, can_gc);
            self.set_closed_promise(promise);
            return Ok(());
        }
        
        // Otherwise, if state is "closed",
         if stream.is_closed() {
            // Set writer.[[readyPromise]] to a promise resolved with undefined.
            let promise = Promise::new(global, can_gc);
            promise.resolve_native(&());
            self.set_ready_promise(promise);
            
            // Set writer.[[closedPromise]] to a promise resolved with undefined.
            let promise = Promise::new(global, can_gc);
            promise.resolve_native(&());
            self.set_closed_promise(promise);
            return Ok(());
        }
        
        // Otherwise,
        // Assert: state is "errored".
        assert!(stream.is_errored());
        
        // Let storedError be stream.[[storedError]].
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error = UndefinedValue());
        stream.get_stored_error(error.handle_mut());
        
        // Set writer.[[readyPromise]] to a promise rejected with stream.[[storedError]].
        // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
        let promise = Promise::new(global, can_gc);
        promise.reject_native(&error.handle());
        promise.set_promise_is_handled();
        self.set_ready_promise(promise);
        
        // Set writer.[[closedPromise]] to a promise rejected with storedError.
        // Set writer.[[closedPromise]].[[PromiseIsHandled]] to true.
        let promise = Promise::new(global, can_gc);
        promise.reject_native(&error.handle());
        promise.set_promise_is_handled();
        self.set_closed_promise(promise);

        Ok(())
    }

    pub(crate) fn set_ready_promise(&self, promise: Rc<Promise>) {
        *self.ready_promise.borrow_mut() = Some(promise);
    }
    
    pub(crate) fn set_closed_promise(&self, promise: Rc<Promise>) {
        *self.closed_promise.borrow_mut() = Some(promise);
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

    /// <https://streams.spec.whatwg.org/#default-writer-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        stream: &WritableStream,
    ) -> DomRoot<WritableStreamDefaultWriter> {
        let writer = WritableStreamDefaultWriter::new(global, proto, can_gc);

        // Perform ? SetUpWritableStreamDefaultWriter(this, stream).
        writer.setup(stream, global, can_gc);

        writer
    }
}
