/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::JS_GetPendingException;
use js::rust::{HandleObject, HandleValue as SafeHandleValue, HandleValue, MutableHandleValue};
use js::typedarray::Uint8;

use super::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::import::module::{throw_dom_exception, Error, Fallible};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::ReadableStream;
use crate::dom::readablestreamdefaultreader::ReadRequest;
use crate::dom::underlyingsourcecontainer::{UnderlyingSourceContainer, UnderlyingSourceType};
use crate::js::conversions::ToJSValConvertible;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext, JSContext as SafeJSContext};

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmFulfillmentHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
    controller: Trusted<ReadableStreamDefaultController>,
}

impl Callback for PullAlgorithmFulfillmentHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
    /// Upon fulfillment of pullPromise
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm, can_gc: CanGc) {
        let controller = self.controller.root();

        // Set controller.[[pulling]] to false.
        controller.pulling.set(false);

        // If controller.[[pullAgain]] is true,
        if controller.pull_again.get() {
            // Set controller.[[pullAgain]] to false.
            controller.pull_again.set(false);

            // Perform ! ReadableStreamDefaultControllerCallPullIfNeeded(controller).
            controller.call_pull_if_needed(can_gc);
        }
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmRejectionHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
    controller: Trusted<ReadableStreamDefaultController>,
}

impl Callback for PullAlgorithmRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
    /// Upon rejection of pullPromise with reason e.
    fn callback(&self, _cx: JSContext, v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
        let controller = self.controller.root();

        // Perform ! ReadableStreamDefaultControllerError(controller, e).
        controller.error(v);
    }
}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#dom-underlyingsource-start>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct StartAlgorithmFulfillmentHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
    controller: Trusted<ReadableStreamDefaultController>,
}

impl Callback for StartAlgorithmFulfillmentHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    /// Upon fulfillment of startPromise,
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm, can_gc: CanGc) {
        let controller = self.controller.root();

        // Set controller.[[started]] to true.
        controller.started.set(true);

        // Perform ! ReadableStreamDefaultControllerCallPullIfNeeded(controller).
        controller.call_pull_if_needed(can_gc);
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#dom-underlyingsource-start>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct StartAlgorithmRejectionHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
    controller: Trusted<ReadableStreamDefaultController>,
}

impl Callback for StartAlgorithmRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    /// Upon rejection of startPromise with reason r,
    fn callback(&self, _cx: JSContext, v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
        let controller = self.controller.root();

        // Perform ! ReadableStreamDefaultControllerError(controller, r).
        controller.error(v);
    }
}

/// <https://streams.spec.whatwg.org/#value-with-size>
#[derive(JSTraceable)]
#[crown::unrooted_must_root_lint::must_root]
pub struct ValueWithSize {
    value: Box<Heap<JSVal>>,
    size: f64,
}

/// <https://streams.spec.whatwg.org/#value-with-size>
#[derive(JSTraceable)]
#[crown::unrooted_must_root_lint::must_root]
pub enum EnqueuedValue {
    /// A value enqueued from Rust.
    Native(Box<[u8]>),
    /// A Js value.
    Js(ValueWithSize),
}

impl EnqueuedValue {
    fn size(&self) -> f64 {
        match self {
            EnqueuedValue::Native(v) => v.len() as f64,
            EnqueuedValue::Js(v) => v.size,
        }
    }

    #[allow(unsafe_code)]
    fn to_jsval(&self, cx: SafeJSContext, rval: MutableHandleValue) {
        match self {
            EnqueuedValue::Native(chunk) => {
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                create_buffer_source::<Uint8>(cx, chunk, array_buffer_ptr.handle_mut())
                    .expect("failed to create buffer source for native chunk.");
                unsafe { array_buffer_ptr.to_jsval(*cx, rval) };
            },
            EnqueuedValue::Js(value_with_size) => unsafe {
                value_with_size.value.to_jsval(*cx, rval);
            },
        }
    }
}

/// <https://streams.spec.whatwg.org/#is-non-negative-number>
fn is_non_negative_number(value: &EnqueuedValue) -> bool {
    let value_with_size = match value {
        EnqueuedValue::Native(_) => return true,
        EnqueuedValue::Js(value_with_size) => value_with_size,
    };

    // If v is not a Number, return false.
    // Checked as part of the WebIDL.

    // If v is NaN, return false.
    if value_with_size.size.is_nan() {
        return false;
    }

    // If v < 0, return false.
    if value_with_size.size.is_sign_negative() {
        return false;
    }

    true
}

/// <https://streams.spec.whatwg.org/#queue-with-sizes>
#[derive(Default, JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct QueueWithSizes {
    #[ignore_malloc_size_of = "EnqueuedValue::Js"]
    queue: VecDeque<EnqueuedValue>,
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-queuetotalsize>
    total_size: f64,
}

impl QueueWithSizes {
    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&mut self) -> EnqueuedValue {
        let value = self
            .queue
            .pop_front()
            .expect("Buffer cannot be empty when dequeue value is called into.");
        self.total_size -= value.size();
        value
    }

    /// <https://streams.spec.whatwg.org/#enqueue-value-with-size>
    fn enqueue_value_with_size(&mut self, value: EnqueuedValue) -> Result<(), Error> {
        // If ! IsNonNegativeNumber(size) is false, throw a RangeError exception.
        if !is_non_negative_number(&value) {
            return Err(Error::Range(
                "The size of the enqueued chunk is not a non-negative number.".to_string(),
            ));
        }

        // If size is +∞, throw a RangeError exception.
        if value.size().is_infinite() {
            return Err(Error::Range(
                "The size of the enqueued chunk is infinite.".to_string(),
            ));
        }

        self.total_size += value.size();
        self.queue.push_back(value);

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Only used with native sources.
    fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        self.queue
            .iter()
            .try_fold(Vec::new(), |mut acc, value| match value {
                EnqueuedValue::Native(chunk) => {
                    acc.extend(chunk.iter().copied());
                    Some(acc)
                },
                _ => {
                    warn!("get_in_memory_bytes called on a controller with non-native source.");
                    None
                },
            })
    }

    /// <https://streams.spec.whatwg.org/#reset-queue>
    fn reset(&mut self) {
        self.queue.clear();
        self.total_size = Default::default();
    }
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
#[dom_struct]
pub struct ReadableStreamDefaultController {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-queue>
    queue: RefCell<QueueWithSizes>,

    /// A mutable reference to the underlying source is used to implement these two
    /// internal slots:
    ///
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-pullalgorithm>
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-cancelalgorithm>
    underlying_source: MutNullableDom<UnderlyingSourceContainer>,

    stream: MutNullableDom<ReadableStream>,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-strategyhwm>
    strategy_hwm: f64,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-strategysizealgorithm>
    #[ignore_malloc_size_of = "mozjs"]
    strategy_size: RefCell<Option<Rc<QueuingStrategySize>>>,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-closerequested>
    close_requested: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-started>
    started: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-pulling>
    pulling: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-pullagain>
    pull_again: Cell<bool>,
}

impl ReadableStreamDefaultController {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
        strategy_hwm: f64,
        strategy_size: Rc<QueuingStrategySize>,
        can_gc: CanGc,
    ) -> ReadableStreamDefaultController {
        ReadableStreamDefaultController {
            reflector_: Reflector::new(),
            queue: RefCell::new(Default::default()),
            stream: MutNullableDom::new(None),
            underlying_source: MutNullableDom::new(Some(&*UnderlyingSourceContainer::new(
                global,
                underlying_source_type,
                can_gc,
            ))),
            strategy_hwm,
            strategy_size: RefCell::new(Some(strategy_size)),
            close_requested: Default::default(),
            started: Default::default(),
            pulling: Default::default(),
            pull_again: Default::default(),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        underlying_source: UnderlyingSourceType,
        strategy_hwm: f64,
        strategy_size: Rc<QueuingStrategySize>,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStreamDefaultController> {
        reflect_dom_object(
            Box::new(ReadableStreamDefaultController::new_inherited(
                global,
                underlying_source,
                strategy_hwm,
                strategy_size,
                can_gc,
            )),
            global,
        )
    }

    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    #[allow(unsafe_code)]
    pub fn setup(&self, stream: DomRoot<ReadableStream>, can_gc: CanGc) -> Result<(), Error> {
        // Assert: stream.[[controller]] is undefined
        stream.assert_no_controller();

        // Set controller.[[stream]] to stream.
        self.stream.set(Some(&stream));

        let global = &*self.global();
        let rooted_default_controller = DomRoot::from_ref(self);

        // Perform ! ResetQueue(controller).
        // Set controller.[[started]], controller.[[closeRequested]],
        // controller.[[pullAgain]], and controller.[[pulling]] to false.
        // Set controller.[[strategySizeAlgorithm]] to sizeAlgorithm
        // and controller.[[strategyHWM]] to highWaterMark.
        // Set controller.[[strategySizeAlgorithm]] to sizeAlgorithm
        // and controller.[[strategyHWM]] to highWaterMark.
        // Set controller.[[cancelAlgorithm]] to cancelAlgorithm.

        // Note: the above steps are done in `new`.

        // Set stream.[[controller]] to controller.
        stream.set_default_controller(&rooted_default_controller);

        if let Some(underlying_source) = rooted_default_controller.underlying_source.get() {
            // Let startResult be the result of performing startAlgorithm. (This might throw an exception.)
            let start_result = underlying_source
                .call_start_algorithm(
                    Controller::ReadableStreamDefaultController(rooted_default_controller.clone()),
                    can_gc,
                )
                .unwrap_or_else(|| {
                    let promise = Promise::new(global, can_gc);
                    promise.resolve_native(&());
                    Ok(promise)
                });

            // Let startPromise be a promise resolved with startResult.
            let start_promise = start_result?;

            // Upon fulfillment of startPromise,
            let fulfillment_handler = Box::new(StartAlgorithmFulfillmentHandler {
                controller: Trusted::new(&*rooted_default_controller),
            });

            // Upon rejection of startPromise with reason r,
            let rejection_handler = Box::new(StartAlgorithmRejectionHandler {
                controller: Trusted::new(&*rooted_default_controller),
            });
            let handler = PromiseNativeHandler::new(
                global,
                Some(fulfillment_handler),
                Some(rejection_handler),
            );
            let realm = enter_realm(global);
            let comp = InRealm::Entered(&realm);
            start_promise.append_native_handler(&handler, comp, can_gc);
        };

        Ok(())
    }

    /// Setting the JS object after the heap has settled down.
    pub fn set_underlying_source_this_object(&self, this_object: HandleObject) {
        if let Some(underlying_source) = self.underlying_source.get() {
            underlying_source.set_underlying_source_this_object(this_object);
        }
    }

    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&self) -> EnqueuedValue {
        let mut queue = self.queue.borrow_mut();
        queue.dequeue_value()
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-should-call-pull>
    fn should_call_pull(&self) -> bool {
        // Let stream be controller.[[stream]].
        // Note: the spec does not assert that stream is not undefined here,
        // so we return false if it is.
        let Some(stream) = self.stream.get() else {
            debug!("`should_call_pull` called on a controller without a stream.");
            return false;
        };

        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(controller) is false, return.
        if !self.can_close_or_enqueue() {
            return false;
        }

        // If controller.[[started]] is false, return false.
        if !self.started.get() {
            return false;
        }

        // If ! IsReadableStreamLocked(stream) is true
        // and ! ReadableStreamGetNumReadRequests(stream) > 0, return true.
        if stream.is_locked() && stream.get_num_read_requests() > 0 {
            return true;
        }

        // Let desiredSize be ! ReadableStreamDefaultControllerGetDesiredSize(controller).
        // Assert: desiredSize is not null.
        let desired_size = self.get_desired_size().expect("desiredSize is not null.");

        if desired_size > 0. {
            return true;
        }

        false
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
    #[allow(unsafe_code)]
    fn call_pull_if_needed(&self, can_gc: CanGc) {
        if !self.should_call_pull() {
            return;
        }

        // If controller.[[pulling]] is true,
        if self.pulling.get() {
            // Set controller.[[pullAgain]] to true.
            self.pull_again.set(true);

            return;
        }

        // Set controller.[[pulling]] to true.
        self.pulling.set(true);

        // Let pullPromise be the result of performing controller.[[pullAlgorithm]].
        // Continues into the resolve and reject handling of the native handler.
        let global = self.global();
        let rooted_default_controller = DomRoot::from_ref(self);
        let controller =
            Controller::ReadableStreamDefaultController(rooted_default_controller.clone());

        let Some(underlying_source) = self.underlying_source.get() else {
            return;
        };

        let fulfillment_handler = Box::new(PullAlgorithmFulfillmentHandler {
            controller: Trusted::new(&*rooted_default_controller),
        });
        let rejection_handler = Box::new(PullAlgorithmRejectionHandler {
            controller: Trusted::new(&*rooted_default_controller),
        });
        let handler =
            PromiseNativeHandler::new(&global, Some(fulfillment_handler), Some(rejection_handler));

        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        let result = underlying_source
            .call_pull_algorithm(controller)
            .unwrap_or_else(|| {
                let promise = Promise::new(&global, can_gc);
                promise.resolve_native(&());
                Ok(promise)
            });
        let promise = result.unwrap_or_else(|error| {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            // TODO: check if `self.global()` is the right globalscope.
            unsafe {
                error
                    .clone()
                    .to_jsval(*cx, &self.global(), rval.handle_mut())
            };
            let promise = Promise::new(&global, can_gc);
            promise.reject_native(&rval.handle());
            promise
        });
        promise.append_native_handler(&handler, comp, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-private-cancel>
    #[allow(unsafe_code)]
    pub fn perform_cancel_steps(&self, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        // Perform ! ResetQueue(this).
        self.queue.borrow_mut().reset();

        let underlying_source = self
            .underlying_source
            .get()
            .expect("Controller should have a source when the cancel steps are called into.");
        let global = self.global();

        // Let result be the result of performing this.[[cancelAlgorithm]], passing reason.
        let result = underlying_source
            .call_cancel_algorithm(reason, can_gc)
            .unwrap_or_else(|| {
                let promise = Promise::new(&global, can_gc);
                promise.resolve_native(&());
                Ok(promise)
            });
        let promise = result.unwrap_or_else(|error| {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            // TODO: check if `self.global()` is the right globalscope.
            unsafe {
                error
                    .clone()
                    .to_jsval(*cx, &self.global(), rval.handle_mut())
            };
            let promise = Promise::new(&global, can_gc);
            promise.reject_native(&rval.handle());
            promise
        });

        // Perform ! ReadableStreamDefaultControllerClearAlgorithms(this).
        self.clear_algorithms();

        // Return result(the promise).
        promise
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-private-pull>
    pub fn perform_pull_steps(&self, read_request: ReadRequest) {
        // Let stream be this.[[stream]].
        // Note: the spec does not assert that there is a stream.
        let Some(stream) = self.stream.get() else {
            return;
        };

        // if queue contains bytes, perform chunk steps.
        if !self.queue.borrow().is_empty() {
            let chunk = self.dequeue_value();

            // If this.[[closeRequested]] is true and this.[[queue]] is empty
            if self.close_requested.get() && self.queue.borrow().is_empty() {
                // Perform ! ReadableStreamDefaultControllerClearAlgorithms(controller).
                self.clear_algorithms();

                // Perform ! ReadableStreamClose(stream).
                stream.close();
            } else {
                // Otherwise, perform ! ReadableStreamDefaultControllerCallPullIfNeeded(this).
                self.call_pull_if_needed(CanGc::note());
            }

            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            let result = RootedTraceableBox::new(Heap::default());
            chunk.to_jsval(cx, rval.handle_mut());
            result.set(*rval);
            // Perform readRequest’s chunk steps, given chunk.
            read_request.chunk_steps(result);
        } else {
            // Perform ! ReadableStreamAddReadRequest(stream, readRequest).
            stream.add_read_request(read_request);

            // Perform ! ReadableStreamDefaultControllerCallPullIfNeeded(this).
            self.call_pull_if_needed(CanGc::note());
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-abstract-opdef-readablestreamcontroller-releasesteps>
    pub fn perform_release_steps(&self) {
        // step 1 - Return.
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    #[allow(unsafe_code)]
    pub fn enqueue(
        &self,
        cx: SafeJSContext,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Result<(), Error> {
        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(controller) is false, return.
        if !self.can_close_or_enqueue() {
            return Ok(());
        }

        let stream = self
            .stream
            .get()
            .expect("Controller must have a stream when a chunk is enqueued.");

        // If ! IsReadableStreamLocked(stream) is true
        // and ! ReadableStreamGetNumReadRequests(stream) > 0,
        // perform ! ReadableStreamFulfillReadRequest(stream, chunk, false).
        if stream.is_locked() && stream.get_num_read_requests() > 0 {
            stream.fulfill_read_request(chunk, false);
        } else {
            // Otherwise,
            // Let result be the result of performing controller.[[strategySizeAlgorithm]],
            // passing in chunk, and interpreting the result as a completion record.
            // Note: the clone is necessary to prevent potential re-borrow panics.
            let strategy_size = {
                let reference = self.strategy_size.borrow();
                reference.clone()
            };
            let size = if let Some(strategy_size) = strategy_size {
                // Note: the Rethrow exception handling is necessary,
                // otherwise returning JSFailed will panic because no exception is pending.
                let result = strategy_size.Call__(chunk, ExceptionHandling::Rethrow);
                match result {
                    // Let chunkSize be result.[[Value]].
                    Ok(size) => size,
                    Err(error) => {
                        // If result is an abrupt completion,
                        rooted!(in(*cx) let mut rval = UndefinedValue());
                        unsafe { assert!(JS_GetPendingException(*cx, rval.handle_mut())) };

                        // Perform ! ReadableStreamDefaultControllerError(controller, result.[[Value]]).
                        self.error(rval.handle());

                        // Return result.
                        // Note: we need to return a type error, because no exception is pending.
                        return Err(error);
                    },
                }
            } else {
                0.
            };

            // We create the value-with-size created inside
            // EnqueueValueWithSize here.
            let value_with_size = ValueWithSize {
                value: Heap::boxed(chunk.get()),
                size,
            };

            {
                // Let enqueueResult be EnqueueValueWithSize(controller, chunk, chunkSize).
                let res = {
                    let mut queue = self.queue.borrow_mut();
                    queue.enqueue_value_with_size(EnqueuedValue::Js(value_with_size))
                };
                if let Err(error) = res {
                    // If enqueueResult is an abrupt completion,

                    // First, throw the exception.
                    // Note: this must be done manually here,
                    // because `enqueue_value_with_size` does not call into JS.
                    throw_dom_exception(cx, &self.global(), error);

                    // Then, get a handle to the JS val for the exception,
                    // and use that to error the stream.
                    rooted!(in(*cx) let mut rval = UndefinedValue());
                    unsafe { assert!(JS_GetPendingException(*cx, rval.handle_mut())) };

                    // Perform ! ReadableStreamDefaultControllerError(controller, enqueueResult.[[Value]]).
                    self.error(rval.handle());

                    // Return enqueueResult.
                    // Note: because we threw the exception above,
                    // there is a pending exception and we can return JSFailed.
                    return Err(Error::JSFailed);
                }
            }
        }

        // Perform ! ReadableStreamDefaultControllerCallPullIfNeeded(controller).
        self.call_pull_if_needed(can_gc);

        Ok(())
    }

    /// Native call to
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    pub fn enqueue_native(&self, chunk: Vec<u8>) {
        let stream = self
            .stream
            .get()
            .expect("Controller must have a stream when a chunk is enqueued.");
        if stream.is_locked() && stream.get_num_read_requests() > 0 {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            let enqueued_chunk = EnqueuedValue::Native(chunk.into_boxed_slice());
            enqueued_chunk.to_jsval(cx, rval.handle_mut());
            stream.fulfill_read_request(rval.handle(), false);
        } else {
            let mut queue = self.queue.borrow_mut();
            queue
                .enqueue_value_with_size(EnqueuedValue::Native(chunk.into_boxed_slice()))
                .expect("Enqueuing a chunk from Rust should not fail.");
        }
    }

    /// Does the stream have all data in memory?
    pub fn in_memory(&self) -> bool {
        let Some(underlying_source) = self.underlying_source.get() else {
            return false;
        };
        underlying_source.in_memory()
    }

    /// Return bytes synchronously if the stream has all data in memory.
    pub fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        let underlying_source = self.underlying_source.get()?;
        if underlying_source.in_memory() {
            return self.queue.borrow().get_in_memory_bytes();
        }
        None
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-clear-algorithms>
    fn clear_algorithms(&self) {
        // Set controller.[[pullAlgorithm]] to undefined.
        // Set controller.[[cancelAlgorithm]] to undefined.
        self.underlying_source.set(None);

        // Set controller.[[strategySizeAlgorithm]] to undefined.
        *self.strategy_size.borrow_mut() = None;
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-close>
    pub fn close(&self) {
        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(controller) is false, return.
        if !self.can_close_or_enqueue() {
            return;
        }

        let Some(stream) = self.stream.get() else {
            return;
        };

        // Set controller.[[closeRequested]] to true.
        self.close_requested.set(true);

        if self.queue.borrow().is_empty() {
            // Perform ! ReadableStreamDefaultControllerClearAlgorithms(controller).
            self.clear_algorithms();

            // Perform ! ReadableStreamClose(stream).
            stream.close();
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-get-desired-size>
    fn get_desired_size(&self) -> Option<f64> {
        let stream = self.stream.get()?;

        // If state is "errored", return null.
        if stream.is_errored() {
            return None;
        }

        // If state is "closed", return 0.
        if stream.is_closed() {
            return Some(0.0);
        }

        // Return controller.[[strategyHWM]] − controller.[[queueTotalSize]].
        let queue = self.queue.borrow();
        let desired_size = self.strategy_hwm - queue.total_size.clamp(0.0, f64::MAX);
        Some(desired_size.clamp(desired_size, self.strategy_hwm))
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-can-close-or-enqueue>
    fn can_close_or_enqueue(&self) -> bool {
        let Some(stream) = self.stream.get() else {
            return false;
        };

        // If controller.[[closeRequested]] is false and state is "readable", return true.
        if !self.close_requested.get() && stream.is_readable() {
            return true;
        }

        // Otherwise, return false.
        false
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-error>
    pub fn error(&self, e: SafeHandleValue) {
        let Some(stream) = self.stream.get() else {
            return;
        };

        // If stream.[[state]] is not "readable", return.
        if !stream.is_readable() {
            return;
        }

        // Perform ! ResetQueue(controller).
        self.queue.borrow_mut().reset();

        // Perform ! ReadableStreamDefaultControllerClearAlgorithms(controller).
        self.clear_algorithms();

        stream.error(e);
    }
}

impl ReadableStreamDefaultControllerMethods<crate::DomTypeHolder>
    for ReadableStreamDefaultController
{
    /// <https://streams.spec.whatwg.org/#rs-default-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        self.get_desired_size()
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-close>
    fn Close(&self) -> Fallible<()> {
        if !self.can_close_or_enqueue() {
            // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(this) is false,
            // throw a TypeError exception.
            return Err(Error::Type("Stream cannot be closed.".to_string()));
        }

        // Perform ! ReadableStreamDefaultControllerClose(this).
        self.close();

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-enqueue>
    fn Enqueue(&self, cx: SafeJSContext, chunk: SafeHandleValue, can_gc: CanGc) -> Fallible<()> {
        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(this) is false, throw a TypeError exception.
        if !self.can_close_or_enqueue() {
            return Err(Error::Type("Stream cannot be enqueued to.".to_string()));
        }

        // Perform ? ReadableStreamDefaultControllerEnqueue(this, chunk).
        self.enqueue(cx, chunk, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-error>
    fn Error(&self, _cx: SafeJSContext, e: SafeHandleValue) -> Fallible<()> {
        self.error(e);
        Ok(())
    }
}
