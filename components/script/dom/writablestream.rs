/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::{self};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::UnderlyingSinkBinding::UnderlyingSink;
use crate::dom::bindings::codegen::Bindings::WritableStreamBinding::WritableStreamMethods;
use crate::dom::bindings::conversions::{ConversionBehavior, ConversionResult};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, DomObject, Reflector,
};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::countqueuingstrategy::{extract_high_water_mark, extract_size_algorithm};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::writablestreamdefaultcontroller::WritableStreamDefaultController;
use crate::dom::writablestreamdefaultwriter::WritableStreamDefaultWriter;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// The fulfillment handler for the abort steps of
/// <https://streams.spec.whatwg.org/#writable-stream-finish-erroringr>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
struct AbortAlgorithmFulfillmentHandler {
    stream: Dom<WritableStream>,
    #[ignore_malloc_size_of = "Rc is hard"]
    abort_request_promise: Rc<Promise>,
}

impl Callback for AbortAlgorithmFulfillmentHandler {
    fn callback(&self, _cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Resolve abortRequest’s promise with undefined.
        self.abort_request_promise.resolve_native(&());

        // Perform ! WritableStreamRejectCloseAndClosedPromiseIfNeeded(stream).
        self.stream
            .as_rooted()
            .reject_close_and_closed_promise_if_needed();
    }
}

/// The rejection handler for the abort steps of
/// <https://streams.spec.whatwg.org/#writable-stream-finish-erroring>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
struct AbortAlgorithmRejectionHandler {
    stream: Dom<WritableStream>,
    #[ignore_malloc_size_of = "Rc is hard"]
    abort_request_promise: Rc<Promise>,
}

impl Callback for AbortAlgorithmRejectionHandler {
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, _can_gc: CanGc) {
        // Reject abortRequest’s promise with undefined.
        self.abort_request_promise.reject_native(&v);

        // Perform ! WritableStreamRejectCloseAndClosedPromiseIfNeeded(stream).
        self.stream
            .as_rooted()
            .reject_close_and_closed_promise_if_needed();
    }
}

/// <https://streams.spec.whatwg.org/#pending-abort-request>
#[derive(JSTraceable, MallocSizeOf)]
struct PendingAbortRequest {
    /// <https://streams.spec.whatwg.org/#pending-abort-request-promise>
    #[ignore_malloc_size_of = "Rc is hard"]
    promise: Rc<Promise>,

    /// <https://streams.spec.whatwg.org/#pending-abort-request-reason>
    #[ignore_malloc_size_of = "mozjs"]
    reason: Box<Heap<JSVal>>,

    /// <https://streams.spec.whatwg.org/#pending-abort-request-was-already-erroring>
    was_already_erroring: bool,
}

/// <https://streams.spec.whatwg.org/#pending-abort-request>
#[derive(Clone, Copy, Default, JSTraceable, MallocSizeOf)]
enum WritableStreamState {
    #[default]
    Writable,
    Closed,
    Erroring,
    Errored,
}

/// <https://streams.spec.whatwg.org/#ws-class>
#[dom_struct]
pub struct WritableStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#writablestream-backpressure>
    backpressure: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#writablestream-closerequest>
    #[ignore_malloc_size_of = "Rc is hard"]
    close_request: DomRefCell<Option<Rc<Promise>>>,

    /// <https://streams.spec.whatwg.org/#writablestream-controller>
    controller: MutNullableDom<WritableStreamDefaultController>,

    /// <https://streams.spec.whatwg.org/#writablestream-detached>
    detached: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#writablestream-inflightwriterequest>
    #[ignore_malloc_size_of = "Rc is hard"]
    in_flight_write_request: DomRefCell<Option<Rc<Promise>>>,

    /// <https://streams.spec.whatwg.org/#writablestream-inflightcloserequest>
    #[ignore_malloc_size_of = "Rc is hard"]
    in_flight_close_request: DomRefCell<Option<Rc<Promise>>>,

    /// <https://streams.spec.whatwg.org/#writablestream-pendingabortrequest>
    pending_abort_request: DomRefCell<Option<PendingAbortRequest>>,

    /// <https://streams.spec.whatwg.org/#writablestream-state>
    state: Cell<WritableStreamState>,

    /// <https://streams.spec.whatwg.org/#writablestream-storederror>
    #[ignore_malloc_size_of = "mozjs"]
    stored_error: Heap<JSVal>,

    /// <https://streams.spec.whatwg.org/#writablestream-writer>
    writer: MutNullableDom<WritableStreamDefaultWriter>,

    /// <https://streams.spec.whatwg.org/#writablestream-writerequests>
    #[ignore_malloc_size_of = "Rc is hard"]
    write_requests: DomRefCell<Vec<Rc<Promise>>>,
}

impl WritableStream {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://streams.spec.whatwg.org/#initialize-readable-stream>
    fn new_inherited() -> WritableStream {
        WritableStream {
            reflector_: Reflector::new(),
            backpressure: Default::default(),
            close_request: Default::default(),
            controller: Default::default(),
            detached: Default::default(),
            in_flight_write_request: Default::default(),
            in_flight_close_request: Default::default(),
            pending_abort_request: Default::default(),
            state: Default::default(),
            stored_error: Default::default(),
            writer: Default::default(),
            write_requests: Default::default(),
        }
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<WritableStream> {
        reflect_dom_object_with_proto(
            Box::new(WritableStream::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
    pub(crate) fn assert_no_controller(&self) {
        assert!(self.controller.get().is_none());
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
    pub(crate) fn set_default_controller(&self, controller: &WritableStreamDefaultController) {
        self.controller.set(Some(controller));
    }

    pub(crate) fn is_writable(&self) -> bool {
        matches!(self.state.get(), WritableStreamState::Writable)
    }

    pub(crate) fn is_erroring(&self) -> bool {
        matches!(self.state.get(), WritableStreamState::Erroring)
    }

    pub(crate) fn is_errored(&self) -> bool {
        matches!(self.state.get(), WritableStreamState::Errored)
    }

    pub(crate) fn is_closed(&self) -> bool {
        matches!(self.state.get(), WritableStreamState::Closed)
    }

    pub(crate) fn has_in_flight_write_request(&self) -> bool {
        self.in_flight_close_request.borrow().is_some()
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-has-operation-marked-in-flight>
    pub fn has_opertations_marked_inflight(&self) -> bool {
        let in_flight_write_requested = self.in_flight_close_request.borrow().is_some();
        let in_flight_close_requested = self.in_flight_close_request.borrow().is_some();

        in_flight_write_requested || in_flight_close_requested
    }

    /// <https://streams.spec.whatwg.org/#writablestream-storederror>
    pub fn get_stored_error(&self, mut handle_mut: SafeMutableHandleValue) {
        handle_mut.set(self.stored_error.get());
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-finish-erroring>
    pub(crate) fn finish_erroring(&self, global: &GlobalScope, can_gc: CanGc) {
        // Assert: stream.[[state]] is "erroring".
        assert!(self.is_erroring());

        // Assert: ! WritableStreamHasOperationMarkedInFlight(stream) is false.
        assert!(!self.has_opertations_marked_inflight());

        // Set stream.[[state]] to "errored".
        self.state.set(WritableStreamState::Errored);

        // Perform ! stream.[[controller]].[[ErrorSteps]]().
        let Some(controller) = self.controller.get() else {
            unreachable!("Stream should have a controller.");
        };
        controller.perform_error_steps();

        // Let storedError be stream.[[storedError]].
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut stored_error = UndefinedValue());
        self.get_stored_error(stored_error.handle_mut());

        // For each writeRequest of stream.[[writeRequests]]:
        for request in self.write_requests.borrow_mut().drain(..) {
            // Reject writeRequest with storedError.
            request.reject(cx, stored_error.handle());
        }

        // Set stream.[[writeRequests]] to an empty list.
        // Done above with `drain`.

        // If stream.[[pendingAbortRequest]] is undefined,
        if self.pending_abort_request.borrow().is_none() {
            // Perform ! WritableStreamRejectCloseAndClosedPromiseIfNeeded(stream).
            self.reject_close_and_closed_promise_if_needed();

            // Return.
            return;
        }

        // Let abortRequest be stream.[[pendingAbortRequest]].
        // Set stream.[[pendingAbortRequest]] to undefined.
        if let Some(pending_abort_request) = self.pending_abort_request.borrow_mut().take() {
            // If abortRequest’s was already erroring is true,
            if pending_abort_request.was_already_erroring {
                // Reject abortRequest’s promise with storedError.
                pending_abort_request
                    .promise
                    .reject(cx, stored_error.handle());

                // Perform ! WritableStreamRejectCloseAndClosedPromiseIfNeeded(stream).
                self.reject_close_and_closed_promise_if_needed();

                // Return.
                return;
            }

            // Let promise be ! stream.[[controller]].[[AbortSteps]](abortRequest’s reason).
            rooted!(in(*cx) let mut reason = UndefinedValue());
            reason.set(pending_abort_request.reason.get());
            let promise = controller.abort_steps(global, reason.handle(), can_gc);

            // Upon fulfillment of promise,
            let fulfillment_handler = Box::new(AbortAlgorithmFulfillmentHandler {
                stream: Dom::from_ref(self),
                abort_request_promise: pending_abort_request.promise.clone(),
            });

            // Upon rejection of promise with reason r,
            let rejection_handler = Box::new(AbortAlgorithmRejectionHandler {
                stream: Dom::from_ref(self),
                abort_request_promise: pending_abort_request.promise,
            });
            let handler = PromiseNativeHandler::new(
                global,
                Some(fulfillment_handler),
                Some(rejection_handler),
            );
            let realm = enter_realm(global);
            let comp = InRealm::Entered(&realm);
            promise.append_native_handler(&handler, comp, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-reject-close-and-closed-promise-if-needed>
    #[allow(unsafe_code)]
    fn reject_close_and_closed_promise_if_needed(&self) {
        // Assert: stream.[[state]] is "errored".
        assert!(self.is_errored());

        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut stored_error = UndefinedValue());
        self.get_stored_error(stored_error.handle_mut());

        // If stream.[[closeRequest]] is not undefined
        if let Some(close_request) = self.close_request.borrow_mut().take() {
            // Assert: stream.[[inFlightCloseRequest]] is undefined.
            assert!(self.in_flight_close_request.borrow().is_none());

            // Reject stream.[[closeRequest]] with stream.[[storedError]].
            close_request.reject_native(&stored_error.handle())

            // Set stream.[[closeRequest]] to undefined.
            // Done with `take` above.
        }

        // Let writer be stream.[[writer]].
        // If writer is not undefined,
        if let Some(writer) = self.writer.get() {
            // Reject writer.[[closedPromise]] with stream.[[storedError]].
            writer.reject_closed_promise_with_stored_error(&stored_error.handle());

            // Set writer.[[closedPromise]].[[PromiseIsHandled]] to true.
            writer.set_close_promise_is_handled();
        }
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-close-queued-or-in-flight>
    pub(crate) fn close_queued_or_in_flight(&self) -> bool {
        let close_requested = self.close_request.borrow().is_some();
        let in_flight_close_requested = self.in_flight_close_request.borrow().is_some();

        close_requested || in_flight_close_requested
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-finish-in-flight-write>
    pub(crate) fn finish_in_flight_write(&self) {
        let Some(in_flight_write_request) = self.in_flight_write_request.borrow_mut().take() else {
            // Assert: stream.[[inFlightWriteRequest]] is not undefined.
            unreachable!("Stream should have a write request");
        };

        // Resolve stream.[[inFlightWriteRequest]] with undefined.
        in_flight_write_request.resolve_native(&());

        // Set stream.[[inFlightWriteRequest]] to undefined.
        // Done above with `take`.
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-start-erroring>
    pub(crate) fn start_erroring(
        &self,
        global: &GlobalScope,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        // Assert: stream.[[storedError]] is undefined.
        assert!(self.stored_error.get().is_undefined());

        // Assert: stream.[[state]] is "writable".
        assert!(self.is_writable());

        // Let controller be stream.[[controller]].
        let Some(controller) = self.controller.get() else {
            // Assert: controller is not undefined.
            unreachable!("Stream should have a controller.");
        };

        // Set stream.[[state]] to "erroring".
        self.state.set(WritableStreamState::Writable);

        // Set stream.[[storedError]] to reason.
        self.stored_error.set(*error);

        // Let writer be stream.[[writer]].
        if let Some(writer) = self.writer.get() {
            // If writer is not undefined, perform ! WritableStreamDefaultWriterEnsureReadyPromiseRejected
            writer.ensure_ready_promise_rejected(global, &error, can_gc);
        }

        // If ! WritableStreamHasOperationMarkedInFlight(stream) is false and controller.[[started]] is true
        if !self.has_opertations_marked_inflight() && controller.started() {
            // perform ! WritableStreamFinishErroring
            self.finish_erroring(global, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-deal-with-rejection>
    pub(crate) fn deal_with_rejection(
        &self,
        global: &GlobalScope,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        // Let state be stream.[[state]].

        // If state is "writable",
        if self.is_writable() {
            // Perform ! WritableStreamStartErroring(stream, error).
            self.start_erroring(global, error, can_gc);

            // Return.
            return;
        }

        // Assert: state is "erroring".
        assert!(self.is_erroring());

        // Perform ! WritableStreamFinishErroring(stream).
        self.finish_erroring(global, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-mark-first-write-request-in-flight>
    pub(crate) fn mark_first_write_request_in_flight(&self) {
        let mut in_flight_write_request = self.in_flight_write_request.borrow_mut();
        let mut write_requests = self.write_requests.borrow_mut();

        // Assert: stream.[[inFlightWriteRequest]] is undefined.
        assert!(in_flight_write_request.is_none());

        // Assert: stream.[[writeRequests]] is not empty.
        assert!(!write_requests.is_empty());

        // Let writeRequest be stream.[[writeRequests]][0].
        // Remove writeRequest from stream.[[writeRequests]].
        let write_request = write_requests.remove(0);

        // Set stream.[[inFlightWriteRequest]] to writeRequest.
        *in_flight_write_request = Some(write_request);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-finish-in-flight-write-with-error>
    pub(crate) fn finish_in_flight_write_with_error(
        &self,
        global: &GlobalScope,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        let Some(mut in_flight_write_request) = self.in_flight_write_request.borrow_mut().take()
        else {
            // Assert: stream.[[inFlightWriteRequest]] is not undefined.
            unreachable!("Inflight write request must be defined.");
        };

        // Reject stream.[[inFlightWriteRequest]] with error.
        in_flight_write_request.reject_native(&error);

        // Set stream.[[inFlightWriteRequest]] to undefined.
        // Done above with `take`.

        // Assert: stream.[[state]] is "writable" or "erroring".
        assert!(self.is_erroring() || self.is_writable());

        // Perform ! WritableStreamDealWithRejection(stream, error).
        self.deal_with_rejection(&*global, error, can_gc);
    }

    pub(crate) fn get_writer(&self) -> Option<DomRoot<WritableStreamDefaultWriter>> {
        self.writer.get()
    }

    pub(crate) fn set_writer(&self, writer: Option<&WritableStreamDefaultWriter>) {
        self.writer.set(writer);
    }

    pub(crate) fn set_backpressure(&self, backpressure: bool) {
        self.backpressure.set(backpressure);
    }

    pub(crate) fn get_backpressure(&self) -> bool {
        self.backpressure.get()
    }

    /// <https://streams.spec.whatwg.org/#is-writable-stream-locked>
    pub(crate) fn is_locked(&self) -> bool {
        // If stream.[[writer]] is undefined, return false.
        // Return true.
        self.get_writer().is_some()
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-abort>
    #[allow(unsafe_code)]
    fn abort(
        &self,
        cx: SafeJSContext,
        mut reason: SafeMutableHandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let global = GlobalScope::from_safe_context(cx, realm);

        // If stream.[[state]] is "closed" or "errored",
        if self.is_closed() || self.is_errored() {
            rooted!(in(*cx) let mut rval = UndefinedValue());
            unsafe {
                Error::Type("Stream is closed or errored.".to_string()).to_jsval(
                    *cx,
                    &*global,
                    rval.handle_mut(),
                )
            };
            let promise = Promise::new(&global, can_gc);
            promise.reject_native(&rval.handle());
            return promise;
        }

        // TODO: Signal abort on stream.[[controller]].[[abortController]] with reason.

        // TODO: If state is "closed" or "errored", return a promise resolved with undefined.
        // Note: state may have changed because of signal above.

        // If stream.[[pendingAbortRequest]] is not undefined,
        if let Some(pending_abort_request) = &*self.pending_abort_request.borrow() {
            // return stream.[[pendingAbortRequest]]'s promise.
            return pending_abort_request.promise.clone();
        }

        // Assert: state is "writable" or "erroring".
        assert!(self.is_writable() || self.is_erroring());

        // Let wasAlreadyErroring be false.
        let mut was_already_erroring = false;

        // If state is "erroring",
        if self.is_erroring() {
            // Set wasAlreadyErroring to true.
            was_already_erroring = true;

            // Set reason to undefined.
            reason.set(UndefinedValue());
        }

        // Let promise be a new promise.
        let promise = Promise::new(&global, can_gc);

        // Set stream.[[pendingAbortRequest]] to a new pending abort request
        // whose promise is promise,
        // reason is reason,
        // and was already erroring is wasAlreadyErroring.
        *self.pending_abort_request.borrow_mut() = Some(PendingAbortRequest {
            promise: promise.clone(),
            reason: Heap::boxed(reason.get()),
            was_already_erroring,
        });

        // If wasAlreadyErroring is false,
        if !was_already_erroring {
            rooted!(in(*cx) let mut reason_clone = UndefinedValue());
            reason_clone.set(reason.get());

            // perform ! WritableStreamStartErroring(stream, reason)
            self.start_erroring(&*global, reason_clone.handle(), can_gc);
        }

        // Return promise.
        return promise;
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-close>
    fn close(&self, cx: SafeJSContext, realm: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let global = GlobalScope::from_safe_context(cx, realm);

        // Let state be stream.[[state]].

        // If state is "closed" or "errored",
        if self.is_closed() || self.is_errored() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Stream is closed or errored.".to_string()));
            return promise;
        }

        // Assert: state is "writable" or "erroring".
        assert!(self.is_writable() || self.is_erroring());

        // Assert: ! WritableStreamCloseQueuedOrInFlight(stream) is false.
        assert!(!self.close_queued_or_in_flight());

        // Let promise be a new promise.
        let promise = Promise::new(&global, can_gc);

        // Set stream.[[closeRequest]] to promise.
        *self.close_request.borrow_mut() = Some(promise.clone());

        // Let writer be stream.[[writer]].

        // If writer is not undefined,
        if let Some(writer) = self.writer.get() {
            // and stream.[[backpressure]] is true,
            // and state is "writable",
            if self.get_backpressure() || self.is_writable() {
                // resolve writer.[[readyPromise]] with undefined.
                writer.resolve_ready_promise();
            }
        }

        // Perform ! WritableStreamDefaultControllerClose(stream.[[controller]]).
        let Some(controller) = self.controller.get() else {
            unreachable!("Stream must have a controller.");
        };
        controller.close(&global, can_gc);

        // Return promise.
        promise
    }
}

impl WritableStreamMethods<crate::DomTypeHolder> for WritableStream {
    /// <https://streams.spec.whatwg.org/#ws-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        underlying_sink: Option<*mut JSObject>,
        strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<WritableStream>> {
        // If underlyingSink is missing, set it to null.
        rooted!(in(*cx) let underlying_sink_obj = underlying_sink.unwrap_or(ptr::null_mut()));

        // Let underlyingSinkDict be underlyingSink,
        // converted to an IDL value of type UnderlyingSink.
        let underlying_sink_dict = if !underlying_sink_obj.is_null() {
            rooted!(in(*cx) let obj_val = ObjectValue(underlying_sink_obj.get()));
            match UnderlyingSink::new(cx, obj_val.handle()) {
                Ok(ConversionResult::Success(val)) => val,
                Ok(ConversionResult::Failure(error)) => return Err(Error::Type(error.to_string())),
                _ => {
                    return Err(Error::JSFailed);
                },
            }
        } else {
            UnderlyingSink::empty()
        };

        if !underlying_sink_dict.type_.handle().is_null() {
            // If underlyingSinkDict["type"] exists, throw a RangeError exception.
            return Err(Error::Range("type is set".to_string()));
        }

        // Perform ! InitializeWritableStream(this).
        let stream = WritableStream::new_with_proto(global, proto, can_gc);

        // Let sizeAlgorithm be ! ExtractSizeAlgorithm(strategy).
        let size_algorithm = extract_size_algorithm(strategy);

        // Let highWaterMark be ? ExtractHighWaterMark(strategy, 1).
        let high_water_mark = extract_high_water_mark(strategy, 1.0)?;

        // Perform ? SetUpWritableStreamDefaultControllerFromUnderlyingSink
        let controller = WritableStreamDefaultController::new(
            global,
            &underlying_sink_dict,
            high_water_mark,
            size_algorithm,
            can_gc,
        );

        // Note: this must be done before `setup`,
        // otherwise `thisOb` is null in the start callback.
        controller.set_underlying_sink_this_object(underlying_sink_obj.handle());

        // Perform ? SetUpWritableStreamDefaultController
        controller.setup(&stream, &underlying_sink_dict.start, global, can_gc);

        Ok(stream)
    }

    /// <https://streams.spec.whatwg.org/#ws-locked>
    fn Locked(&self) -> bool {
        // Return ! IsWritableStreamLocked(this).
        self.is_locked()
    }

    /// <https://streams.spec.whatwg.org/#ws-abort>
    fn Abort(
        &self,
        cx: SafeJSContext,
        reason: SafeHandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let global = GlobalScope::from_safe_context(cx, realm);

        // If ! IsWritableStreamLocked(this) is true,
        if self.is_locked() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Stream is locked.".to_string()));
            return promise;
        }

        rooted!(in(*cx) let mut reason_clone = UndefinedValue());
        reason_clone.set(reason.get());

        // Return ! WritableStreamAbort(this, reason).
        self.abort(cx, reason_clone.handle_mut(), realm, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#ws-close>
    fn Close(&self, realm: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        let global = GlobalScope::from_safe_context(cx, realm);
        // If ! IsWritableStreamLocked(this) is true,
        if self.is_locked() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Stream is locked.".to_string()));
            return promise;
        }

        // If ! WritableStreamCloseQueuedOrInFlight(this) is true
        if self.close_queued_or_in_flight() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Stream close queued or in flight.".to_string()));
            return promise;
        }

        // Return ! WritableStreamClose(this).
        self.close(cx, realm, can_gc)
    }

    fn GetWriter(&self) -> DomRoot<WritableStreamDefaultWriter> {
        todo!()
    }
}
