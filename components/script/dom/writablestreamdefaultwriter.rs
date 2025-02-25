/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use crate::dom::bindings::codegen::Bindings::WritableStreamDefaultWriterBinding::WritableStreamDefaultWriterMethods;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::writablestream::WritableStream;
use crate::realms::InRealm;
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

    pub(crate) fn new(
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
    pub(crate) fn setup(
        &self,
        cx: SafeJSContext,
        stream: &WritableStream,
        can_gc: CanGc,
    ) -> Result<(), Error> {
        // If ! IsWritableStreamLocked(stream) is true, throw a TypeError exception.
        if stream.is_locked() {
            return Err(Error::Type("Stream is locked".to_string()));
        }

        // Set writer.[[stream]] to stream.
        self.stream.set(Some(stream));

        // Set stream.[[writer]] to writer.
        stream.set_writer(Some(self));

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
                self.ready_promise.borrow().resolve_native(&(), can_gc);
            }

            // Set writer.[[closedPromise]] to a new promise.
            // Done in `new_inherited`.
            return Ok(());
        }

        // Otherwise, if state is "erroring",
        if stream.is_erroring() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());

            // Set writer.[[readyPromise]] to a promise rejected with stream.[[storedError]].
            // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
            // Note: new promise created in `new_inherited`.
            let ready_promise = self.ready_promise.borrow();
            ready_promise.reject_native(&error.handle(), can_gc);
            ready_promise.set_promise_is_handled();

            // Set writer.[[closedPromise]] to a new promise.
            // Done in `new_inherited`.
            return Ok(());
        }

        // Otherwise, if state is "closed",
        if stream.is_closed() {
            // Set writer.[[readyPromise]] to a promise resolved with undefined.
            // Note: new promise created in `new_inherited`.
            self.ready_promise.borrow().resolve_native(&(), can_gc);

            // Set writer.[[closedPromise]] to a promise resolved with undefined.
            // Note: new promise created in `new_inherited`.
            self.closed_promise.borrow().resolve_native(&(), can_gc);
            return Ok(());
        }

        // Otherwise,
        // Assert: state is "errored".
        assert!(stream.is_errored());

        // Let storedError be stream.[[storedError]].
        rooted!(in(*cx) let mut error = UndefinedValue());
        stream.get_stored_error(error.handle_mut());

        // Set writer.[[readyPromise]] to a promise rejected with stream.[[storedError]].
        // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
        // Note: new promise created in `new_inherited`.
        let ready_promise = self.ready_promise.borrow();
        ready_promise.reject_native(&error.handle(), can_gc);
        ready_promise.set_promise_is_handled();

        // Set writer.[[closedPromise]] to a promise rejected with storedError.
        // Set writer.[[closedPromise]].[[PromiseIsHandled]] to true.
        // Note: new promise created in `new_inherited`.
        let ready_promise = self.closed_promise.borrow();
        ready_promise.reject_native(&error.handle(), can_gc);
        ready_promise.set_promise_is_handled();

        Ok(())
    }

    pub(crate) fn reject_closed_promise_with_stored_error(
        &self,
        error: &SafeHandleValue,
        can_gc: CanGc,
    ) {
        self.closed_promise.borrow().reject_native(error, can_gc);
    }

    pub(crate) fn set_close_promise_is_handled(&self) {
        self.closed_promise.borrow().set_promise_is_handled();
    }

    pub(crate) fn set_ready_promise(&self, promise: Rc<Promise>) {
        *self.ready_promise.borrow_mut() = promise;
    }

    pub(crate) fn resolve_ready_promise_with_undefined(&self, can_gc: CanGc) {
        self.ready_promise.borrow().resolve_native(&(), can_gc);
    }

    pub(crate) fn resolve_closed_promise_with_undefined(&self, can_gc: CanGc) {
        self.closed_promise.borrow().resolve_native(&(), can_gc);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-ensure-ready-promise-rejected>
    pub(crate) fn ensure_ready_promise_rejected(
        &self,
        global: &GlobalScope,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        let ready_promise = self.ready_promise.borrow().clone();

        // If writer.[[readyPromise]].[[PromiseState]] is "pending",
        if ready_promise.is_pending() {
            // reject writer.[[readyPromise]] with error.
            ready_promise.reject_native(&error, can_gc);

            // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
            ready_promise.set_promise_is_handled();
        } else {
            // Otherwise, set writer.[[readyPromise]] to a promise rejected with error.
            let promise = Promise::new(global, can_gc);
            promise.reject_native(&error, can_gc);

            // Set writer.[[readyPromise]].[[PromiseIsHandled]] to true.
            promise.set_promise_is_handled();
            *self.ready_promise.borrow_mut() = promise;
        }
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-ensure-closed-promise-rejected>
    pub(crate) fn ensure_closed_promise_rejected(
        &self,
        global: &GlobalScope,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        let closed_promise = self.closed_promise.borrow().clone();

        // If writer.[[closedPromise]].[[PromiseState]] is "pending",
        if closed_promise.is_pending() {
            // reject writer.[[closedPromise]] with error.
            closed_promise.reject_native(&error, can_gc);

            // Set writer.[[closedPromise]].[[PromiseIsHandled]] to true.
            closed_promise.set_promise_is_handled();
        } else {
            // Otherwise, set writer.[[closedPromise]] to a promise rejected with error.
            let promise = Promise::new(global, can_gc);
            promise.reject_native(&error, can_gc);

            // Set writer.[[closedPromise]].[[PromiseIsHandled]] to true.
            promise.set_promise_is_handled();
            *self.closed_promise.borrow_mut() = promise;
        }
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-abort>
    fn abort(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Let stream be writer.[[stream]].
        let Some(stream) = self.stream.get() else {
            // Assert: stream is not undefined.
            unreachable!("Stream should be set.");
        };

        // Return ! WritableStreamAbort(stream, reason).
        stream.abort(cx, global, reason, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-close>
    fn close(&self, cx: SafeJSContext, global: &GlobalScope, can_gc: CanGc) -> Rc<Promise> {
        // Let stream be writer.[[stream]].
        let Some(stream) = self.stream.get() else {
            // Assert: stream is not undefined.
            unreachable!("Stream should be set.");
        };

        // Return ! WritableStreamClose(stream).
        stream.close(cx, global, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-write>
    fn write(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Let stream be writer.[[stream]].
        let Some(stream) = self.stream.get() else {
            // Assert: stream is not undefined.
            unreachable!("Stream should be set.");
        };

        // Let controller be stream.[[controller]].
        // Note: asserting controller is some.
        let Some(controller) = stream.get_controller() else {
            unreachable!("Controller should be set.");
        };

        // Let chunkSize be ! WritableStreamDefaultControllerGetChunkSize(controller, chunk).
        let chunk_size = controller.get_chunk_size(cx, global, chunk, can_gc);

        // If stream is not equal to writer.[[stream]],
        // return a promise rejected with a TypeError exception.
        if !self
            .stream
            .get()
            .is_some_and(|current_stream| current_stream == stream)
        {
            let promise = Promise::new(global, can_gc);
            promise.reject_error(
                Error::Type("Stream is not equal to writer stream".to_string()),
                can_gc,
            );
            return promise;
        }

        // Let state be stream.[[state]].
        // If state is "errored",
        if stream.is_errored() {
            // return a promise rejected with stream.[[storedError]].
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            let promise = Promise::new(global, can_gc);
            promise.reject_native(&error.handle(), can_gc);
            return promise;
        }

        // If ! WritableStreamCloseQueuedOrInFlight(stream) is true
        // or state is "closed",
        if stream.close_queued_or_in_flight() || stream.is_closed() {
            // return a promise rejected with a TypeError exception
            // indicating that the stream is closing or closed
            let promise = Promise::new(global, can_gc);
            promise.reject_error(
                Error::Type("Stream has been closed, or has close queued or in-flight".to_string()),
                can_gc,
            );
            return promise;
        }

        // If state is "erroring",
        if stream.is_erroring() {
            // return a promise rejected with stream.[[storedError]].
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            let promise = Promise::new(global, can_gc);
            promise.reject_native(&error.handle(), can_gc);
            return promise;
        }

        // Assert: state is "writable".
        assert!(stream.is_writable());

        // Let promise be ! WritableStreamAddWriteRequest(stream).
        let promise = stream.add_write_request(global, can_gc);

        // Perform ! WritableStreamDefaultControllerWrite(controller, chunk, chunkSize).
        controller.write(cx, global, chunk, chunk_size, can_gc);

        // Return promise.
        promise
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-release>
    pub(crate) fn release(&self, cx: SafeJSContext, global: &GlobalScope, can_gc: CanGc) {
        // Let stream be this.[[stream]].
        let Some(stream) = self.stream.get() else {
            // Assert: stream is not undefined.
            unreachable!("Stream should be set.");
        };

        // Assert: stream.[[writer]] is writer.
        assert!(stream.get_writer().is_some_and(|writer| &*writer == self));

        // Let releasedError be a new TypeError.
        let released_error = Error::Type("Writer has been released".to_string());

        // Root the js val of the error.
        rooted!(in(*cx) let mut error = UndefinedValue());
        released_error.to_jsval(cx, global, error.handle_mut());

        // Perform ! WritableStreamDefaultWriterEnsureReadyPromiseRejected(writer, releasedError).
        self.ensure_ready_promise_rejected(global, error.handle(), can_gc);

        // Perform ! WritableStreamDefaultWriterEnsureClosedPromiseRejected(writer, releasedError).
        self.ensure_closed_promise_rejected(global, error.handle(), can_gc);

        // Set stream.[[writer]] to undefined.
        stream.set_writer(None);

        // Set this.[[stream]] to undefined.
        self.stream.set(None);
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
        let global = GlobalScope::from_safe_context(cx, realm);

        // If this.[[stream]] is undefined,
        if self.stream.get().is_none() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Stream is undefined".to_string()), can_gc);
            return promise;
        }

        // Return ! WritableStreamDefaultWriterAbort(this, reason).
        self.abort(cx, &global, reason, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#default-writer-close>
    fn Close(&self, in_realm: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        let global = GlobalScope::from_safe_context(cx, in_realm);
        let promise = Promise::new(&global, can_gc);

        // Let stream be this.[[stream]].
        let Some(stream) = self.stream.get() else {
            // If stream is undefined,
            // return a promise rejected with a TypeError exception.
            promise.reject_error(Error::Type("Stream is undefined".to_string()), can_gc);
            return promise;
        };

        // If ! WritableStreamCloseQueuedOrInFlight(stream) is true
        if stream.close_queued_or_in_flight() {
            // return a promise rejected with a TypeError exception.
            promise.reject_error(
                Error::Type("Stream has closed queued or in-flight".to_string()),
                can_gc,
            );
            return promise;
        }

        self.close(cx, &global, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#default-writer-release-lock>
    fn ReleaseLock(&self, can_gc: CanGc) {
        // Let stream be this.[[stream]].
        let Some(stream) = self.stream.get() else {
            // If stream is undefined, return.
            return;
        };

        // Assert: stream.[[writer]] is not undefined.
        assert!(stream.get_writer().is_some());

        let global = self.global();
        let cx = GlobalScope::get_cx();

        // Perform ! WritableStreamDefaultWriterRelease(this).
        self.release(cx, &global, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#default-writer-write>
    fn Write(
        &self,
        cx: SafeJSContext,
        chunk: SafeHandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let global = GlobalScope::from_safe_context(cx, realm);

        // If this.[[stream]] is undefined,
        if self.stream.get().is_none() {
            // return a promise rejected with a TypeError exception.
            let global = GlobalScope::from_safe_context(cx, realm);
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Stream is undefined".to_string()), can_gc);
            return promise;
        }

        // Return ! WritableStreamDefaultWriterWrite(this, chunk).
        self.write(cx, &global, chunk, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#default-writer-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        stream: &WritableStream,
    ) -> Result<DomRoot<WritableStreamDefaultWriter>, Error> {
        let writer = WritableStreamDefaultWriter::new(global, proto, can_gc);

        let cx = GlobalScope::get_cx();

        // Perform ? SetUpWritableStreamDefaultWriter(this, stream).
        writer.setup(cx, stream, can_gc)?;

        Ok(writer)
    }
}
