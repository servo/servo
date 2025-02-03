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
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#writablestreamdefaultwriter>
#[dom_struct]
pub struct WritableStreamDefaultWriter {
    reflector_: Reflector,

    #[ignore_malloc_size_of = "Rc is hard"]
    ready_promise: RefCell<Rc<Promise>>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultwriter-closedpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    closed_promise: RefCell<Rc<Promise>>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultwriter-stream>
    stream: MutNullableDom<WritableStream>,
}

impl WritableStreamDefaultWriter {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-writer>
    /// The parts that create a new promise.
    fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> WritableStreamDefaultWriter {
        WritableStreamDefaultWriter {
            reflector_: Reflector::new(),
            stream: Default::default(),
            closed_promise: RefCell::new(Promise::new(global, can_gc)),
            ready_promise: RefCell::new(Promise::new(global, can_gc)),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<WritableStreamDefaultWriter> {
        reflect_dom_object_with_proto(
            Box::new(WritableStreamDefaultWriter::new_inherited(global, can_gc)),
            global,
            proto,
            can_gc,
        )
    }

    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-writer>
    /// Continuing from `new_inherited`, the rest.
    fn setup(
        &self,
        stream: &WritableStream,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Result<(), Error> {
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
                // Done in `new_inherited`.
            } else {
                // Otherwise, set writer.[[readyPromise]] to a promise resolved with undefined.
                // Note: new promise created in `new_inherited`.
                self.ready_promise.borrow().resolve_native(&());
            }

            // Set writer.[[closedPromise]] to a new promise.
            // Done in `new_inherited`.
            return Ok(());
        }

        // Otherwise, if state is "erroring",
        if stream.is_writable() {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());

            // Set writer.[[readyPromise]] to a promise rejected with stream.[[storedError]].
            // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
            // Note: new promise created in `new_inherited`.
            let ready_promise = self.ready_promise.borrow();
            ready_promise.reject_native(&error.handle());
            ready_promise.set_promise_is_handled();

            // Set writer.[[closedPromise]] to a new promise.
            // Done in `new_inherited`.
            return Ok(());
        }

        // Otherwise, if state is "closed",
        if stream.is_closed() {
            // Set writer.[[readyPromise]] to a promise resolved with undefined.
            // Note: new promise created in `new_inherited`.
            self.ready_promise.borrow().resolve_native(&());

            // Set writer.[[closedPromise]] to a promise resolved with undefined.
            // Note: new promise created in `new_inherited`.
            self.closed_promise.borrow().resolve_native(&());
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
        // Note: new promise created in `new_inherited`.
        let ready_promise = self.ready_promise.borrow();
        ready_promise.reject_native(&error.handle());
        ready_promise.set_promise_is_handled();

        // Set writer.[[closedPromise]] to a promise rejected with storedError.
        // Set writer.[[closedPromise]].[[PromiseIsHandled]] to true.
        // Note: new promise created in `new_inherited`.
        let ready_promise = self.closed_promise.borrow();
        ready_promise.reject_native(&error.handle());
        ready_promise.set_promise_is_handled();

        Ok(())
    }

    pub(crate) fn resolve_ready_promise(&self) {
        self.ready_promise.borrow().resolve_native(&());
    }

    pub(crate) fn reject_closed_promise_with_stored_error(&self, error: &SafeHandleValue) {
        self.closed_promise.borrow().reject_native(error);
    }

    pub(crate) fn set_close_promise_is_handled(&self) {
        self.closed_promise.borrow().set_promise_is_handled();
    }

    pub(crate) fn set_ready_promise(&self, promise: Rc<Promise>) {
        *self.ready_promise.borrow_mut() = promise;
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-ensure-ready-promise-rejected>
    pub(crate) fn ensure_ready_promise_rejected(
        &self,
        global: &GlobalScope,
        error: &SafeHandleValue,
        can_gc: CanGc,
    ) {
        let mut ready_promise = self.ready_promise.borrow_mut();
        // If writer.[[readyPromise]].[[PromiseState]] is "pending", reject writer.[[readyPromise]] with error.
        if ready_promise.is_pending() {
            ready_promise.reject_native(error);
        } else {
            // Otherwise, set writer.[[readyPromise]] to a promise rejected with error.
            let promise = Promise::new(global, can_gc);
            promise.reject_native(error);
            *ready_promise = promise;
        }
        // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
        ready_promise.set_promise_is_handled();
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-abort>
    fn abort(
        &self,
        cx: SafeJSContext,
        reason: SafeHandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Let stream be writer.[[stream]].
        let Some(stream) = self.stream.get() else {
            // Assert: stream is not undefined.
            unreachable!("Stream should be set.");
        };

        // Return ! WritableStreamAbort(stream, reason).
        rooted!(in(*cx) let mut reason_clone = UndefinedValue());
        reason_clone.set(reason.get());
        stream.abort(cx, reason_clone.handle_mut(), realm, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-close>
    fn close(&self, realm: InRealm, can_gc: CanGc) -> Rc<Promise> {
        // Let stream be writer.[[stream]].
        let Some(stream) = self.stream.get() else {
            // Assert: stream is not undefined.
            unreachable!("Stream should be set.");
        };

        let cx = GlobalScope::get_cx();

        // Return ! WritableStreamClose(stream).
        stream.close(cx, realm, can_gc)
    }
}

impl WritableStreamDefaultWriterMethods<crate::DomTypeHolder> for WritableStreamDefaultWriter {
    /// <https://streams.spec.whatwg.org/#default-writer-closed>
    fn Closed(&self) -> Rc<Promise> {
        // Return this.[[closedPromise]].
        return self.closed_promise.borrow().clone();
    }

    /// <https://streams.spec.whatwg.org/#default-writer-desired-size>
    fn GetDesiredSize(&self) -> Result<Option<f64>, Error> {
        // If this.[[stream]] is undefined, throw a TypeError exception.
        let Some(stream) = self.stream.get() else {
            return Err(Error::Type("Stream is undefined".to_string()));
        };

        // Return ! WritableStreamDefaultWriterGetDesiredSize(this).
        Ok(stream.get_desired_size())
    }

    /// <https://streams.spec.whatwg.org/#default-writer-ready>
    fn Ready(&self) -> Rc<Promise> {
        // Return this.[[readyPromise]].
        return self.ready_promise.borrow().clone();
    }

    /// <https://streams.spec.whatwg.org/#default-writer-abort>
    fn Abort(
        &self,
        cx: SafeJSContext,
        reason: SafeHandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // If this.[[stream]] is undefined,
        if self.stream.get().is_none() {
            // return a promise rejected with a TypeError exception.
            let global = GlobalScope::from_safe_context(cx, realm);
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Stream is undefined".to_string()));
            return promise;
        }

        // Return ! WritableStreamDefaultWriterAbort(this, reason).
        self.abort(cx, reason, realm, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#default-writer-close>
    fn Close(&self, realm: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();
        let promise = Promise::new(&global, can_gc);

        // Let stream be this.[[stream]].
        let Some(stream) = self.stream.get() else {
            // If stream is undefined,
            // return a promise rejected with a TypeError exception.
            promise.reject_error(Error::Type("Stream is undefined".to_string()));
            return promise;
        };

        // If ! WritableStreamCloseQueuedOrInFlight(stream) is true
        if stream.close_queued_or_in_flight() {
            // return a promise rejected with a TypeError exception.
            promise.reject_error(Error::Type(
                "Stream has closed queued or in-flight".to_string(),
            ));
            return promise;
        }

        return self.close(realm, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#default-writer-release-lock>
    fn ReleaseLock(&self) {
        // Let stream be this.[[stream]].
        let Some(stream) = self.stream.get() else {
            // If stream is undefined, return.
            return;
        };

        // Assert: stream.[[writer]] is not undefined.
        assert!(stream.get_writer().is_some());

        // Perform ! WritableStreamDefaultWriterRelease(this).
        stream.release();
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
