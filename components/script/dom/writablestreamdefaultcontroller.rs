/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, IsPromiseObject, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{
    Handle as SafeHandle, HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    IntoHandle, MutableHandleValue as SafeMutableHandleValue,
};

use super::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::UnderlyingSinkBinding::{
    UnderlyingSink, UnderlyingSinkAbortCallback, UnderlyingSinkCloseCallback,
    UnderlyingSinkStartCallback, UnderlyingSinkWriteCallback,
};
use crate::dom::bindings::codegen::Bindings::WritableStreamDefaultControllerBinding::WritableStreamDefaultControllerMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, DomObject, Reflector,
};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestreamdefaultcontroller::QueueWithSizes;
use crate::dom::writablestream::WritableStream;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct StartAlgorithmFulfillmentHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
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
        controller.advance_queue_if_needed(&*global, can_gc)
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct StartAlgorithmRejectionHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
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
        stream.deal_with_rejection(&*global, can_gc);
    }
}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-write>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct WriteAlgorithmFulfillmentHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
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
        rooted!(in(*cx) let mut rval = UndefinedValue());
        controller.dequeue_value(cx, rval.handle_mut());

        let global = GlobalScope::from_safe_context(cx, realm);

        // If ! WritableStreamCloseQueuedOrInFlight(stream) is false and state is "writable",
        if !stream.close_queued_or_in_flight() && stream.is_writable() {
            // Let backpressure be ! WritableStreamDefaultControllerGetBackpressure(controller).
            let backpressure = controller.get_backpressure();

            // Perform ! WritableStreamUpdateBackpressure(stream, backpressure).
            controller.update_backpressure(&stream, backpressure, &*global, can_gc);
        }

        // Perform ! WritableStreamDefaultControllerAdvanceQueueIfNeeded(controller).
        controller.advance_queue_if_needed(&*global, can_gc)
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-write>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct WriteAlgorithmRejectionHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
    controller: Dom<WritableStreamDefaultController>,
}

impl Callback for WriteAlgorithmRejectionHandler {
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, _can_gc: CanGc) {
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

        // Perform ! WritableStreamFinishInFlightWriteWithError(stream, reason).
        stream.finish_in_flight_write_with_error(v);
    }
}

/// <https://streams.spec.whatwg.org/#ws-class>
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
    #[ignore_malloc_size_of = "mozjs"]
    strategy_size: RefCell<Option<Rc<QueuingStrategySize>>>,

    /// <https://streams.spec.whatwg.org/#writablestreamdefaultcontroller-stream>
    stream: MutNullableDom<WritableStream>,
}

impl WritableStreamDefaultController {
    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controller-from-underlying-sink>
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(
        global: &GlobalScope,
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
        // TODO: Perform ? SetUpWritableStreamDefaultController
    }

    pub fn new(
        global: &GlobalScope,
        underlying_sink: &UnderlyingSink,
        strategy_hwm: f64,
        strategy_size: Rc<QueuingStrategySize>,
        can_gc: CanGc,
    ) -> DomRoot<WritableStreamDefaultController> {
        reflect_dom_object(
            Box::new(WritableStreamDefaultController::new_inherited(
                global,
                underlying_sink,
                strategy_hwm,
                strategy_size,
            )),
            global,
            can_gc,
        )
    }

    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&self, cx: SafeJSContext, rval: SafeMutableHandleValue) {
        let mut queue = self.queue.borrow_mut();
        queue.dequeue_value(cx, rval);
    }

    /// Setting the JS object after the heap has settled down.
    pub fn set_underlying_sink_this_object(&self, this_object: SafeHandleObject) {
        self.underlying_sink_obj.set(*this_object);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-clear-algorithms>
    fn clear_algorithms(&self) {
        self.abort.borrow_mut().take();
        self.write.borrow_mut().take();
        self.close.borrow_mut().take();
    }

    /// <https://streams.spec.whatwg.org/#set-up-writable-stream-default-controllerr>
    #[allow(unsafe_code)]
    pub fn setup(
        &self,
        stream: &WritableStream,
        start: &Option<Rc<UnderlyingSinkStartCallback>>,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Result<(), Error> {
        // Assert: stream implements WritableStream.
        // Implied by stream type.

        // Assert: stream.[[controller]] is undefined.
        stream.assert_no_controller();

        // Set controller.[[stream]] to stream.
        self.stream.set(Some(&stream));

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
        self.update_backpressure(&stream, backpressure, global, can_gc);

        // Let startResult be the result of performing startAlgorithm. (This may throw an exception.)
        // Let startPromise be a promise resolved with startResult.
        let start_promise = if let Some(start) = start {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut result_object = ptr::null_mut::<JSObject>());
            rooted!(in(*cx) let mut result: JSVal);
            unsafe {
                let _ = start.Call_(
                    &SafeHandle::from_raw(self.underlying_sink_obj.handle()),
                    self,
                    result.handle_mut(),
                    ExceptionHandling::Rethrow,
                );
            }
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
                let promise = Promise::new(&self.global(), can_gc);
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
        let fulfillment_handler = Box::new(StartAlgorithmFulfillmentHandler {
            controller: Dom::from_ref(&rooted_default_controller),
        });

        // Upon rejection of startPromise with reason r,
        let rejection_handler = Box::new(StartAlgorithmRejectionHandler {
            controller: Dom::from_ref(&rooted_default_controller),
        });
        let handler =
            PromiseNativeHandler::new(global, Some(fulfillment_handler), Some(rejection_handler));
        let realm = enter_realm(global);
        let comp = InRealm::Entered(&realm);
        start_promise.append_native_handler(&handler, comp, can_gc);

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#ref-for-abstract-opdef-writablestreamcontroller-abortsteps>
    #[allow(unsafe_code)]
    pub(crate) fn abort_steps(
        &self,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
        this_object.set(self.underlying_sink_obj.get());
        let algo = self.abort.borrow().clone();
        let result = if let Some(algo) = algo {
            unsafe {
                algo.Call_(
                    &this_object.handle(),
                    Some(reason),
                    ExceptionHandling::Rethrow,
                )
            }
        } else {
            let promise = Promise::new(&global, can_gc);
            promise.resolve_native(&());
            Ok(promise)
        };
        result.unwrap_or_else(|_| {
            let promise = Promise::new(&global, can_gc);
            promise.resolve_native(&());
            promise
        })
    }

    #[allow(unsafe_code)]
    pub(crate) fn call_write_algorithm(
        &self,
        chunk: SafeHandleValue,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut this_object = ptr::null_mut::<JSObject>());
        this_object.set(self.underlying_sink_obj.get());
        let algo = self.write.borrow().clone();
        let result = if let Some(algo) = algo {
            unsafe {
                algo.Call_(
                    &this_object.handle(),
                    chunk,
                    self,
                    ExceptionHandling::Rethrow,
                )
            }
        } else {
            let promise = Promise::new(&global, can_gc);
            promise.resolve_native(&());
            Ok(promise)
        };
        result.unwrap_or_else(|_| {
            let promise = Promise::new(&global, can_gc);
            promise.resolve_native(&());
            promise
        })
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-advance-queue-if-needed>
    fn advance_queue_if_needed(&self, global: &GlobalScope, can_gc: CanGc) {
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
        assert!(stream.is_errored() || stream.is_closed());

        // If state is "erroring",
        if stream.is_erroring() {
            // Perform ! WritableStreamFinishErroring(stream).
            stream.finish_erroring(global, can_gc);

            // Return.
            return;
        }

        let queue = self.queue.borrow_mut();

        // If controller.[[queue]] is empty, return.
        if queue.is_empty() {
            return;
        }

        // Let value be ! PeekQueueValue(controller).
        let value = queue.peek_queue_value();

        // TODO: If value is the close sentinel, perform ! WritableStreamDefaultControllerProcessClose(controller).

        // Otherwise, perform ! WritableStreamDefaultControllerProcessWrite(controller, value).
        self.process_write(&stream, value, global, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#ws-default-controller-private-error>
    pub(crate) fn perform_error_steps(&self) {
        // Perform ! ResetQueue(this).
        self.queue.borrow_mut().reset();
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-process-write>
    fn process_write(
        &self,
        stream: &WritableStream,
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
        let sink_write_promise = self.call_write_algorithm(chunk, global, can_gc);
    }

    fn update_backpressure(
        &self,
        stream: &WritableStream,
        backpressure: bool,
        global: &GlobalScope,
        can_gc: CanGc,
    ) {
        // Assert: stream.[[state]] is "writable".
        stream.is_writable();

        // Assert: ! WritableStreamCloseQueuedOrInFlight(stream) is false.
        assert!(!stream.close_queued_or_in_flight());

        // Let writer be stream.[[writer]].
        if let Some(writer) = stream.get_writer() {
            if backpressure {
                // If backpressure is true, set writer.[[readyPromise]] to a new promise.
                let promise = Promise::new(global, can_gc);
                writer.set_ready_promise(promise);
            }
            // Otherwise,
            // Assert: backpressure is false.
            assert!(backpressure);

            // Resolve writer.[[readyPromise]] with undefined.
            writer.resolve_ready_promise();
        };

        // Set stream.[[backpressure]] to backpressure.
        stream.set_backpressure(backpressure);
    }

    /// <https://streams.spec.whatwg.org/#writable-stream-default-controller-get-desired-size>
    fn get_desired_size(&self) -> f64 {
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
        desired_size.is_sign_positive()
    }
}

impl WritableStreamDefaultControllerMethods<crate::DomTypeHolder>
    for WritableStreamDefaultController
{
    fn Error(&self, cx: SafeJSContext, e: SafeHandleValue) {
        todo!()
    }
}
