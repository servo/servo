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
use crate::dom::writablestreamdefaultcontroller::WritableStreamDefaultController;
use crate::dom::writablestreamdefaultwriter::WritableStreamDefaultWriter;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#pending-abort-request>
#[derive(JSTraceable, MallocSizeOf)]
struct PendingAbortRequest {
    /// <https://streams.spec.whatwg.org/#pending-abort-request-promise>
    #[ignore_malloc_size_of = "Rc is hard"]
    promise: Rc<Promise>,

    /// <https://streams.spec.whatwg.org/#pending-abort-request-reason>
    #[ignore_malloc_size_of = "mozjs"]
    reason: Heap<JSVal>,

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
    #[allow(crown::unrooted_must_root)]
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
    pub(crate) fn finish_erroring(&self) {
        // Assert: stream.[[state]] is "erroring".
        assert!(self.is_erroring());

        // Assert: ! WritableStreamHasOperationMarkedInFlight(stream) is false.
        assert!(!self.has_opertations_marked_inflight());

        // Set stream.[[state]] to "errored".
        self.state.set(WritableStreamState::Errored);

        // TODO: Perform ! stream.[[controller]].[[ErrorSteps]]().
        let Some(controller) = self.controller.get() else {
            unreachable!("Stream should have a controller.");
        };
        controller.perform_error_steps();

        // Let storedError be stream.[[storedError]].
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error = UndefinedValue());
        let stored_error = self.get_stored_error(error.handle_mut());

        // For each writeRequest of stream.[[writeRequests]]:
        for request in self.write_requests.borrow_mut().drain(..) {
            // Reject writeRequest with storedError.
            request.reject_native(&error.handle());
        }

        // Set stream.[[writeRequests]] to an empty list.
        // Done above with `drain`.

        // If stream.[[pendingAbortRequest]] is undefined,
        if self.pending_abort_request.borrow().is_none() {
            // Perform ! WritableStreamRejectCloseAndClosedPromiseIfNeeded(stream).
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

    pub(crate) fn get_writer(&self) -> Option<DomRoot<WritableStreamDefaultWriter>> {
        self.writer.get()
    }

    pub(crate) fn set_backpressure(&self, backpressure: bool) {
        self.backpressure.set(backpressure);
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

        Ok(stream)
    }

    /// <https://streams.spec.whatwg.org/#ws-locked>
    fn Locked(&self) -> bool {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-abort>
    fn Abort(&self, cx: SafeJSContext, reason: SafeHandleValue, _can_gc: CanGc) -> Rc<Promise> {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-close>
    fn Close(&self, _can_gc: CanGc) -> Rc<Promise> {
        todo!()
    }

    fn GetWriter(&self) -> DomRoot<WritableStreamDefaultWriter> {
        todo!()
    }
}
