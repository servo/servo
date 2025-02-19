/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, IsPromiseObject, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, IntoHandle};

use super::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::UnderlyingSinkBinding::{
    UnderlyingSink, UnderlyingSinkAbortCallback, UnderlyingSinkCloseCallback,
    UnderlyingSinkStartCallback, UnderlyingSinkWriteCallback,
};
use crate::dom::bindings::codegen::Bindings::WritableStreamDefaultControllerBinding::WritableStreamDefaultControllerMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestreamdefaultcontroller::{EnqueuedValue, QueueWithSizes, ValueWithSize};
use crate::dom::writablestream::WritableStream;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

impl js::gc::Rootable for CloseAlgorithmFulfillmentHandler {}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-close>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct CloseAlgorithmFulfillmentHandler {
    stream: Dom<WritableStream>,
}

impl Callback for CloseAlgorithmFulfillmentHandler {
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, _can_gc: CanGc) {
        let stream = self.stream.as_rooted();

        // Perform ! WritableStreamFinishInFlightClose(stream).
        stream.finish_in_flight_close(cx);
    }
}

impl js::gc::Rootable for CloseAlgorithmRejectionHandler {}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-close>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct CloseAlgorithmRejectionHandler {
    stream: Dom<WritableStream>,
}

impl Callback for CloseAlgorithmRejectionHandler {
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        let stream = self.stream.as_rooted();

        let global = GlobalScope::from_safe_context(cx, realm);

        // Perform ! WritableStreamFinishInFlightCloseWithError(stream, reason).
        stream.finish_in_flight_close_with_error(cx, &global, v, can_gc);
    }
}

impl js::gc::Rootable for StartAlgorithmFulfillmentHandler {}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct StartAlgorithmFulfillmentHandler {
    controller: Dom<WritableStreamDefaultController>,
}

impl Callback for StartAlgorithmFulfillmentHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
    /// Upon fulfillment of startPromise,
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        let controller = self.controller.as_rooted();
        let stream = controller
            .stream
            .get()
            .expect("Controller should have a stream.");

        // Assert: stream.[[state]] is "writable" or "erroring".
        assert!(stream.is_erroring() || stream.is_writable());

        // Set controller.[[started]] to true.
        controller.started.set(true);

        let global = GlobalScope::from_safe_context(cx, realm);

        // Perform ! WritableStreamDefaultControllerAdvanceQueueIfNeeded(controller).
        controller.advance_queue_if_needed(cx, &global, can_gc)
    }
}

impl js::gc::Rootable for StartAlgorithmRejectionHandler {}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct StartAlgorithmRejectionHandler {
    controller: Dom<WritableStreamDefaultController>,
}

impl Callback for StartAlgorithmRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
    /// Upon rejection of startPromise with reason r,
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        let controller = self.controller.as_rooted();
        let stream = controller
            .stream
            .get()
            .expect("Controller should have a stream.");

        // Assert: stream.[[state]] is "writable" or "erroring".
        assert!(stream.is_erroring() || stream.is_writable());

        // Set controller.[[started]] to true.
        controller.started.set(true);

        let global = GlobalScope::from_safe_context(cx, realm);

        // Perform ! WritableStreamDealWithRejection(stream, r).
        stream.deal_with_rejection(cx, &global, v, can_gc);
    }
}

impl js::gc::Rootable for WriteAlgorithmFulfillmentHandler {}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-write>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct WriteAlgorithmFulfillmentHandler {
    controller: Dom<WritableStreamDefaultController>,
}

impl Callback for WriteAlgorithmFulfillmentHandler {
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        let controller = self.controller.as_rooted();
        let stream = controller
            .stream
            .get()
            .expect("Controller should have a stream.");

        // Perform ! WritableStreamFinishInFlightWrite(stream).
        stream.finish_in_flight_write();

        // Let state be stream.[[state]].
        // Assert: state is "writable" or "erroring".
        assert!(stream.is_erroring() || stream.is_writable());

        // Perform ! DequeueValue(controller).
        {
            rooted!(in(*cx) let mut rval = UndefinedValue());
            let mut queue = controller.queue.borrow_mut();
            queue.dequeue_value(cx, Some(rval.handle_mut()));
        }

        let global = GlobalScope::from_safe_context(cx, realm);

        // If ! WritableStreamCloseQueuedOrInFlight(stream) is false and state is "writable",
        if !stream.close_queued_or_in_flight() && stream.is_writable() {
            // Let backpressure be ! WritableStreamDefaultControllerGetBackpressure(controller).
            let backpressure = controller.get_backpressure();

            // Perform ! WritableStreamUpdateBackpressure(stream, backpressure).
            stream.update_backpressure(backpressure, &global, can_gc);
        }

        // Perform ! WritableStreamDefaultControllerAdvanceQueueIfNeeded(controller).
        controller.advance_queue_if_needed(cx, &global, can_gc)
    }
}

impl js::gc::Rootable for WriteAlgorithmRejectionHandler {}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-write>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct WriteAlgorithmRejectionHandler {
    controller: Dom<WritableStreamDefaultController>,
}

impl Callback for WriteAlgorithmRejectionHandler {
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        let controller = self.controller.as_rooted();
        let stream = controller
            .stream
            .get()
            .expect("Controller should have a stream.");

        // If stream.[[state]] is "writable",
        if stream.is_writable() {
            // perform ! WritableStreamDefaultControllerClearAlgorithms(controller).
            controller.clear_algorithms();
        }

        let global = GlobalScope::from_safe_context(cx, realm);

        // Perform ! WritableStreamFinishInFlightWriteWithError(stream, reason).
        stream.finish_in_flight_write_with_error(cx, &global, v, can_gc);
    }
}

/// <https://streams.spec.whatwg.org/#ws-default-controller-class>
#[dom_struct]
pub struct WritableStreamDefaultController {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-abortalgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    abort: RefCell<Option<Rc<UnderlyingSinkAbortCallback>>>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-closealgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    close: RefCell<Option<Rc<UnderlyingSinkCloseCallback>>>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-writealgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    write: RefCell<Option<Rc<UnderlyingSinkWriteCallback>>>,

    /// The JS object used as `this` when invoking sink algorithms.
    #[ignore_malloc_size_of = "mozjs"]
    underlying_sink_obj: Heap<*mut JSObject>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-queue>
    queue: RefCell<QueueWithSizes>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-started>
    started: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-strategyhwm>
    strategy_hwm: f64,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-strategysizealgorithm>
    #[ignore_malloc_size_of = "Rc is hard"]
    strategy_size: RefCell<Option<Rc<QueuingStrategySize>>>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-stream>
    stream: MutNullableDom<WritableStream>,
}

impl WritableStreamDefaultController {
    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller-from-underlying-sink>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        underlying_sink: &UnderlyingSink,
        strategy_hwm: f64,
        strategy_size: Rc<QueuingStrategySize>,
    ) -> WritableStreamDefaultController {
        WritableStreamDefaultController {
            reflector_: Reflector::new(),
            queue: Default::default(),
            stream: Default::default(),
            abort: RefCell::new(underlying_sink.abort.clone()),
            close: RefCell::new(underlying_sink.close.clone()),
            write: RefCell::new(underlying_sink.write.clone()),
            underlying_sink_obj: Default::default(),
            strategy_hwm,
            strategy_size: RefCell::new(Some(strategy_size)),
            started: Default::default(),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        underlying_sink: &UnderlyingSink,
        strategy_hwm: f64,
        strategy_size: Rc<QueuingStrategySize>,
        can_gc: CanGc,
    ) -> DomRoot<WritableStreamDefaultController> {
        reflect_dom_object(
            Box::new(WritableStreamDefaultController::new_inherited(
                underlying_sink,
                strategy_hwm,
                strategy_size,
            )),
            global,
            can_gc,
        )
    }

    pub(crate) fn started(&self) -> bool {
        self.started.get()
    }

    /// Setting the JS object after the heap has settled down.
    pub(crate) fn set_underlying_sink_this_object(&self, this_object: SafeHandleObject) {
        self.underlying_sink_obj.set(*this_object);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-clear-algorithms>
    fn clear_algorithms(&self) {
        // Set controller.[[writeAlgorithm]] to undefined.
        self.write.borrow_mut().take();

        // Set controller.[[closeAlgorithm]] to undefined.
        self.close.borrow_mut().take();

        // Set controller.[[abortAlgorithm]] to undefined.
        self.abort.borrow_mut().take();

        // Set controller.[[strategySizeAlgorithm]] to undefined.
        self.strategy_size.borrow_mut().take();
    }

    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controllerr>
    #[allow(unsafe_code)]
    pub(crate) fn setup(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        stream: &WritableStream,
        start: &Option<Rc<UnderlyingSinkStartCallback>>,
        can_gc: CanGc,
    ) -> Result<(), Error> {
        // Assert: stream implements WritableStream.
        // Implied by stream type.

        // Assert: stream.[[controller]] is undefined.
        stream.assert_no_controller();

        // Set controller.[[stream]] to stream.
        self.stream.set(Some(stream));

        // Set stream.[[controller]] to controller.
        stream.set_default_controller(self);

        // Perform ! ResetQueue(controller).

        // Set controller.[[abortController]] to a new AbortController.

        // Set controller.[[started]] to false.

        // Set controller.[[strategySizeAlgorithm]] to sizeAlgorithm.

        // Set controller.[[strategyHWM]] to highWaterMark.

        // Set controller.[[writeAlgorithm]] to writeAlgorithm.

        // Set controller.[[closeAlgorithm]] to closeAlgorithm.

        // Set controller.[[abortAlgorithm]] to abortAlgorithm.

        // Note: above steps are done in `new_inherited`.

        // Let backpressure be ! WritableStreamDefaultControllerGetBackpressure(controller).
        let backpressure = self.get_backpressure();

        // Perform ! WritableStreamUpdateBackpressure(stream, backpressure).
        stream.update_backpressure(backpressure, global, can_gc);

        // Let startResult be the result of performing startAlgorithm. (This may throw an exception.)
        // Let startPromise be a promise resolved with startResult.
        let start_promise = if let Some(start) = start {
            rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
            rooted!(in(*cx) let mut result: JSVal);
            rooted!(in(*cx) let this_object = self.underlying_sink_obj.get());
            start.Call_(
                &this_object.handle(),
                self,
                result.handle_mut(),
                ExceptionHandling::Rethrow,
            )?;
            let is_promise = unsafe {
                if result.is_object() {
                    result_object.set(result.to_object());
                    IsPromiseObject(result_object.handle().into_handle())
                } else {
                    false
                }
            };
            if is_promise {
                let promise = Promise::new_with_js_promise(result_object.handle(), cx);
                promise
            } else {
                let promise = Promise::new(global, can_gc);
                promise.resolve_native(&result.get());
                promise
            }
        } else {
            let promise = Promise::new(global, can_gc);
            promise.resolve_native(&());
            promise
        };

        let rooted_default_controller = DomRoot::from_ref(self);

        // Upon fulfillment of startPromise,
        rooted!(in(*cx) let mut fulfillment_handler = Some(StartAlgorithmFulfillmentHandler {
            controller: Dom::from_ref(&rooted_default_controller),
        }));

        // Upon rejection of startPromise with reason r,
        rooted!(in(*cx) let mut rejection_handler = Some(StartAlgorithmRejectionHandler {
            controller: Dom::from_ref(&rooted_default_controller),
        }));

        let handler = PromiseNativeHandler::new(
            global,
            fulfillment_handler.take().map(|h| Box::new(h) as Box<_>),
            rejection_handler.take().map(|h| Box::new(h) as Box<_>),
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        start_promise.append_native_handler(&handler, comp, can_gc);

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-close>
    pub(crate) fn close(&self, cx: SafeJSContext, global: &GlobalScope, can_gc: CanGc) {
        // Perform ! EnqueueValueWithSize(controller, close sentinel, 0).
        {
            let mut queue = self.queue.borrow_mut();
            queue
                .enqueue_value_with_size(EnqueuedValue::CloseSentinel)
                .expect("Enqueuing the close sentinel should not fail.");
        }
        // Perform ! WritableStreamDefaultControllerAdvanceQueueIfNeeded(controller).
        self.advance_queue_if_needed(cx, global, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#ref-for-abstract-opdef-writablestreamcontroller-abortsteps>
    pub(crate) fn abort_steps(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        rooted!(in(*cx) let this_object = self.underlying_sink_obj.get());
        let algo = self.abort.borrow().clone();
        let result = if let Some(algo) = algo {
            algo.Call_(
                &this_object.handle(),
                Some(reason),
                ExceptionHandling::Rethrow,
            )
        } else {
            let promise = Promise::new(global, can_gc);
            promise.resolve_native(&());
            Ok(promise)
        };
        result.unwrap_or_else(|e| {
            let promise = Promise::new(global, can_gc);
            promise.reject_error(e);
            promise
        })
    }

    pub(crate) fn call_write_algorithm(
        &self,
        cx: SafeJSContext,
        chunk: SafeHandleValue,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        rooted!(in(*cx) let this_object = self.underlying_sink_obj.get());
        let algo = self.write.borrow().clone();
        let result = if let Some(algo) = algo {
            algo.Call_(
                &this_object.handle(),
                chunk,
                self,
                ExceptionHandling::Rethrow,
            )
        } else {
            let promise = Promise::new(global, can_gc);
            promise.resolve_native(&());
            Ok(promise)
        };
        result.unwrap_or_else(|e| {
            let promise = Promise::new(global, can_gc);
            promise.reject_error(e);
            promise
        })
    }

    fn call_close_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
        this_object.set(self.underlying_sink_obj.get());
        let algo = self.close.borrow().clone();
        let result = if let Some(algo) = algo {
            algo.Call_(&this_object.handle(), ExceptionHandling::Rethrow)
        } else {
            let promise = Promise::new(global, can_gc);
            promise.resolve_native(&());
            Ok(promise)
        };
        result.unwrap_or_else(|e| {
            let promise = Promise::new(global, can_gc);
            promise.reject_error(e);
            promise
        })
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-close>
    pub(crate) fn process_close(&self, cx: SafeJSContext, global: &GlobalScope, can_gc: CanGc) {
        // Let stream be controller.[[stream]].
        let Some(stream) = self.stream.get() else {
            unreachable!("Controller should have a stream");
        };

        // Perform ! WritableStreamMarkCloseRequestInFlight(stream).
        stream.mark_close_request_in_flight();

        // Perform ! DequeueValue(controller).
        {
            let mut queue = self.queue.borrow_mut();
            queue.dequeue_value(cx, None);
        }

        // Assert: controller.[[queue]] is empty.
        assert!(self.queue.borrow().is_empty());

        // Let sinkClosePromise be the result of performing controller.[[closeAlgorithm]].
        let sink_close_promise = self.call_close_algorithm(cx, global, can_gc);

        // Perform ! WritableStreamDefaultControllerClearAlgorithms(controller).
        self.clear_algorithms();

        // Upon fulfillment of sinkClosePromise,
        rooted!(in(*cx) let mut fulfillment_handler = Some(CloseAlgorithmFulfillmentHandler {
            stream: Dom::from_ref(&stream),
        }));

        // Upon rejection of sinkClosePromise with reason reason,
        rooted!(in(*cx) let mut rejection_handler = Some(CloseAlgorithmRejectionHandler {
            stream: Dom::from_ref(&stream),
        }));

        // Attach handlers to the promise.
        let handler = PromiseNativeHandler::new(
            global,
            fulfillment_handler.take().map(|h| Box::new(h) as Box<_>),
            rejection_handler.take().map(|h| Box::new(h) as Box<_>),
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        sink_close_promise.append_native_handler(&handler, comp, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-advance-queue-if-needed>
    fn advance_queue_if_needed(&self, cx: SafeJSContext, global: &GlobalScope, can_gc: CanGc) {
        // Let stream be controller.[[stream]].
        let Some(stream) = self.stream.get() else {
            unreachable!("Controller should have a stream");
        };

        // If controller.[[started]] is false, return.
        if !self.started.get() {
            return;
        }

        // If stream.[[inFlightWriteRequest]] is not undefined, return.
        if stream.has_in_flight_write_request() {
            return;
        }

        // Let state be stream.[[state]].

        // Assert: state is not "closed" or "errored".
        assert!(!(stream.is_errored() || stream.is_closed()));

        // If state is "erroring",
        if stream.is_erroring() {
            // Perform ! WritableStreamFinishErroring(stream).
            stream.finish_erroring(cx, global, can_gc);

            // Return.
            return;
        }

        // Let value be ! PeekQueueValue(controller).
        rooted!(in(*cx) let mut value = UndefinedValue());
        let is_closed = {
            let queue = self.queue.borrow_mut();

            // If controller.[[queue]] is empty, return.
            if queue.is_empty() {
                return;
            }
            queue.peek_queue_value(cx, value.handle_mut())
        };

        if is_closed {
            // If value is the close sentinel, perform ! WritableStreamDefaultControllerProcessClose(controller).
            self.process_close(cx, global, can_gc);
        } else {
            // Otherwise, perform ! WritableStreamDefaultControllerProcessWrite(controller, value).
            self.process_write(cx, value.handle(), global, can_gc);
        };
    }

    /// <https://streams.spec.whatwg.org/#ws-default-controller-private-error>
    pub(crate) fn perform_error_steps(&self) {
        // Perform ! ResetQueue(this).
        self.queue.borrow_mut().reset();
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-write>
    fn process_write(
        &self,
        cx: SafeJSContext,
        chunk: SafeHandleValue,
        global: &GlobalScope,
        can_gc: CanGc,
    ) {
        // Let stream be controller.[[stream]].
        let Some(stream) = self.stream.get() else {
            unreachable!("Controller should have a stream");
        };

        // Perform ! WritableStreamMarkFirstWriteRequestInFlight(stream).
        stream.mark_first_write_request_in_flight();

        // Let sinkWritePromise be the result of performing controller.[[writeAlgorithm]], passing in chunk.
        let sink_write_promise = self.call_write_algorithm(cx, chunk, global, can_gc);

        // Upon fulfillment of sinkWritePromise,
        rooted!(in(*cx) let mut fulfillment_handler = Some(WriteAlgorithmFulfillmentHandler {
            controller: Dom::from_ref(self),
        }));

        // Upon rejection of sinkWritePromise with reason,
        rooted!(in(*cx) let mut rejection_handler = Some(WriteAlgorithmRejectionHandler {
            controller: Dom::from_ref(self),
        }));

        // Attach handlers to the promise.
        let handler = PromiseNativeHandler::new(
            global,
            fulfillment_handler.take().map(|h| Box::new(h) as Box<_>),
            rejection_handler.take().map(|h| Box::new(h) as Box<_>),
        );
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        sink_write_promise.append_native_handler(&handler, comp, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-get-desired-size>
    pub(crate) fn get_desired_size(&self) -> f64 {
        // Return controller.[[strategyHWM]] − controller.[[queueTotalSize]].
        let queue = self.queue.borrow();
        let desired_size = self.strategy_hwm - queue.total_size.clamp(0.0, f64::MAX);
        desired_size.clamp(desired_size, self.strategy_hwm)
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-get-backpressure>
    fn get_backpressure(&self) -> bool {
        // Let desiredSize be ! WritableStreamDefaultControllerGetDesiredSize(controller).
        let desired_size = self.get_desired_size();

        // Return true if desiredSize ≤ 0, or false otherwise.
        desired_size == 0.0 || desired_size.is_sign_negative()
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-get-chunk-size>
    pub(crate) fn get_chunk_size(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> f64 {
        // If controller.[[strategySizeAlgorithm]] is undefined,
        let Some(strategy_size) = self.strategy_size.borrow().clone() else {
            // Assert: controller.[[stream]].[[state]] is "erroring" or "errored".
            let Some(stream) = self.stream.get() else {
                unreachable!("Controller should have a stream");
            };
            assert!(stream.is_erroring() || stream.is_errored());

            // Return 1.
            return 1.0;
        };

        // Let returnValue be the result of performing controller.[[strategySizeAlgorithm]],
        // passing in chunk, and interpreting the result as a completion record.
        let result = strategy_size.Call__(chunk, ExceptionHandling::Rethrow);

        match result {
            // Let chunkSize be result.[[Value]].
            Ok(size) => size,
            Err(error) => {
                // If result is an abrupt completion,

                // Perform ! WritableStreamDefaultControllerErrorIfNeeded(controller, returnValue.[[Value]]).
                // Create a rooted value for the error.
                rooted!(in(*cx) let mut rooted_error = UndefinedValue());
                error.to_jsval(cx, global, rooted_error.handle_mut());
                self.error_if_needed(cx, rooted_error.handle(), global, can_gc);

                // Return 1.
                1.0
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-write>
    pub(crate) fn write(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        chunk_size: f64,
        can_gc: CanGc,
    ) {
        // Let enqueueResult be EnqueueValueWithSize(controller, chunk, chunkSize).
        let enqueue_result = {
            let mut queue = self.queue.borrow_mut();
            queue.enqueue_value_with_size(EnqueuedValue::Js(ValueWithSize {
                value: Heap::boxed(chunk.get()),
                size: chunk_size,
            }))
        };

        // If enqueueResult is an abrupt completion,
        if let Err(error) = enqueue_result {
            // Perform ! WritableStreamDefaultControllerErrorIfNeeded(controller, enqueueResult.[[Value]]).
            // Create a rooted value for the error.
            rooted!(in(*cx) let mut rooted_error = UndefinedValue());
            error.to_jsval(cx, global, rooted_error.handle_mut());
            self.error_if_needed(cx, rooted_error.handle(), global, can_gc);

            // Return.
            return;
        }

        // Let stream be controller.[[stream]].
        let Some(stream) = self.stream.get() else {
            unreachable!("Controller should have a stream");
        };

        // If ! WritableStreamCloseQueuedOrInFlight(stream) is false and stream.[[state]] is "writable",
        if !stream.close_queued_or_in_flight() && stream.is_writable() {
            // Let backpressure be ! WritableStreamDefaultControllerGetBackpressure(controller).
            let backpressure = self.get_backpressure();

            // Perform ! WritableStreamUpdateBackpressure(stream, backpressure).
            stream.update_backpressure(backpressure, global, can_gc);
        }

        // Perform ! WritableStreamDefaultControllerAdvanceQueueIfNeeded(controller).
        self.advance_queue_if_needed(cx, global, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-error-if-needed>
    pub(crate) fn error_if_needed(
        &self,
        cx: SafeJSContext,
        error: SafeHandleValue,
        global: &GlobalScope,
        can_gc: CanGc,
    ) {
        // Let stream be controller.[[stream]].
        let Some(stream) = self.stream.get() else {
            unreachable!("Controller should have a stream");
        };

        // If stream.[[state]] is "writable",
        if stream.is_writable() {
            // Perform ! WritableStreamDefaultControllerError(controller, e).
            self.error(&stream, cx, error, global, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-error>
    fn error(
        &self,
        stream: &WritableStream,
        cx: SafeJSContext,
        e: SafeHandleValue,
        global: &GlobalScope,
        can_gc: CanGc,
    ) {
        // Let stream be controller.[[stream]].
        // Done above with the argument.

        // Assert: stream.[[state]] is "writable".
        assert!(stream.is_writable());

        // Perform ! WritableStreamDefaultControllerClearAlgorithms(controller).
        self.clear_algorithms();

        // Perform ! WritableStreamStartErroring(stream, error).
        stream.start_erroring(cx, global, e, can_gc);
    }
}

impl WritableStreamDefaultControllerMethods<crate::DomTypeHolder>
    for WritableStreamDefaultController
{
    /// <https://streams.spec.whatwg.org/#ws-default-controller-error>
    fn Error(&self, cx: SafeJSContext, e: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        // Let state be this.[[stream]].[[state]].
        let Some(stream) = self.stream.get() else {
            unreachable!("Controller should have a stream");
        };

        // If state is not "writable", return.
        if !stream.is_writable() {
            return;
        }

        let global = GlobalScope::from_safe_context(cx, realm);

        // Perform ! WritableStreamDefaultControllerError(this, e).
        self.error(&stream, cx, e, &global, can_gc);
    }
}
