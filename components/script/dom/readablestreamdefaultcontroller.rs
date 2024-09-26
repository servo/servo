/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleValue as SafeHandleValue, HandleValue};

use super::bindings::codegen::Bindings::QueuingStrategyBinding::{
    CountQueuingStrategyMethods, QueuingStrategy, QueuingStrategyInit, QueuingStrategySize,
};
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::{ReadableStream, ReadableStreamState};
use crate::dom::readablestreamdefaultreader::ReadRequest;
use crate::dom::underlyingsourcecontainer::{UnderlyingSourceContainer, UnderlyingSourceType};
use crate::js::conversions::ToJSValConvertible;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{JSContext, JSContext as SafeJSContext};

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmFulfillmentHandler {
    // TODO: check the validity of using Dom here.
    controller: Dom<ReadableStreamDefaultController>,
}

impl Callback for PullAlgorithmFulfillmentHandler {
    /// Handle fufillment of pull algo promise.
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        todo!();
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmRejectionHandler {
    // TODO: check the validity of using Dom here.
    controller: Dom<ReadableStreamDefaultController>,
}

impl Callback for PullAlgorithmRejectionHandler {
    /// Handle rejection of pull algo promise.
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        todo!();
    }
}

/// <https://streams.spec.whatwg.org/#value-with-size>
#[derive(JSTraceable)]
pub struct ValueWithSize {
    // TODO: check how to properly do this one.
    value: Heap<*mut JSObject>,
    size: f64,
}

/// <https://streams.spec.whatwg.org/#value-with-size>
#[derive(JSTraceable)]
#[allow(crown::unrooted_must_root)]
pub enum EnqueuedValue {
    /// A value enqueued from Rust.
    Native(Vec<u8>),
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
}

/// <https://streams.spec.whatwg.org/#queue-with-sizes>
#[derive(Default, JSTraceable, MallocSizeOf)]
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
        // TODO: If ! IsNonNegativeNumber(size) is false, throw a RangeError exception.
        // TODO: If size is +∞, throw a RangeError exception.

        self.total_size += value.size();
        self.queue.push_back(value);

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Only used with native sources.
    fn get_in_memory_bytes(&self) -> Vec<u8> {
        self.queue
            .iter()
            .flat_map(|value| {
                let EnqueuedValue::Native(chunk) = value else {
                    unreachable!(
                        "`get_in_memory_bytes` can only be called on a queue with native values."
                    )
                };
                chunk.clone()
            })
            .collect()
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
}

impl ReadableStreamDefaultController {
    fn new_inherited(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
        strategy_hwm: f64,
        strategy_size: Rc<QueuingStrategySize>,
    ) -> ReadableStreamDefaultController {
        ReadableStreamDefaultController {
            reflector_: Reflector::new(),
            queue: RefCell::new(Default::default()),
            stream: MutNullableDom::new(None),
            underlying_source: MutNullableDom::new(Some(&*UnderlyingSourceContainer::new(
                global,
                underlying_source_type,
            ))),
            strategy_hwm,
            strategy_size: RefCell::new(Some(strategy_size)),
            close_requested: Default::default(),
        }
    }
    pub fn new(
        global: &GlobalScope,
        underlying_source: UnderlyingSourceType,
        strategy_hwm: f64,
        strategy_size: Rc<QueuingStrategySize>,
    ) -> DomRoot<ReadableStreamDefaultController> {
        reflect_dom_object(
            Box::new(ReadableStreamDefaultController::new_inherited(
                global,
                underlying_source,
                strategy_hwm,
                strategy_size,
            )),
            global,
        )
    }

    pub fn set_stream(&self, stream: &ReadableStream) {
        self.stream.set(Some(stream))
    }

    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&self) -> EnqueuedValue {
        let mut queue = self.queue.borrow_mut();
        queue.dequeue_value()
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-should-call-pull>
    fn should_pull(&self) -> bool {
        // TODO: implement the algo.
        true
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
    fn call_pull_if_needed(&self) {
        if !self.should_pull() {
            return;
        }

        let global = self.global();
        let rooted_default_controller = DomRoot::from_ref(self);
        let controller =
            Controller::ReadableStreamDefaultController(rooted_default_controller.clone());

        let Some(underlying_source) = self.underlying_source.get() else {
            return;
        };

        if let Some(promise) = underlying_source.call_pull_algorithm(controller) {
            let fulfillment_handler = Box::new(PullAlgorithmFulfillmentHandler {
                controller: Dom::from_ref(&*rooted_default_controller),
            });
            let rejection_handler = Box::new(PullAlgorithmRejectionHandler {
                controller: Dom::from_ref(&*rooted_default_controller),
            });
            let handler = PromiseNativeHandler::new(
                &global,
                Some(fulfillment_handler),
                Some(rejection_handler),
            );

            let realm = enter_realm(&*global);
            let comp = InRealm::Entered(&realm);
            promise.append_native_handler(&handler, comp);
        }
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-private-pull>
    #[allow(unsafe_code)]
    pub fn perform_pull_steps(&self, read_request: ReadRequest) {
        // if queue contains bytes, perform chunk steps.
        if !self.queue.borrow().is_empty() {
            let chunk = self.dequeue_value();

            // If this.[[closeRequested]] is true and this.[[queue]] is empty
            if self.close_requested.get() && self.queue.borrow().is_empty() {
                // Perform ! ReadableStreamDefaultControllerClearAlgorithms(controller).
                self.clear_algorithms();

                // Perform ! ReadableStreamClose(stream).
                self.stream
                    .get()
                    .expect("Controller must have a stream when pull steps are called into.")
                    .close();
            }

            self.call_pull_if_needed();

            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            let result = RootedTraceableBox::new(Heap::default());
            match chunk {
                EnqueuedValue::Native(chunk) => unsafe {
                    chunk.to_jsval(*cx, rval.handle_mut());
                },
                EnqueuedValue::Js(value_with_size) => unsafe {
                    value_with_size.value.to_jsval(*cx, rval.handle_mut());
                },
            }
            result.set(*rval);
            read_request.chunk_steps(result);
        }

        // else, append read request to reader.
        self.stream
            .get()
            .expect("Controller must have a stream when pull steps are called into.")
            .add_read_request(read_request);

        self.call_pull_if_needed();
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    #[allow(unsafe_code)]
    pub fn enqueue(&self, cx: SafeJSContext, chunk: SafeHandleValue) -> Result<(), Error> {
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
            return Ok(());
        }

        // Let result be the result of performing controller.[[strategySizeAlgorithm]],
        // passing in chunk, and interpreting the result as a completion record.
        let size = if let Some(strategy_size) = self.strategy_size.borrow().as_ref() {
            let result = strategy_size.Call__(chunk, ExceptionHandling::Report);
            match result {
                // Let chunkSize be result.[[Value]].
                Ok(size) => size,
                Err(error) => {
                    // If result is an abrupt completion,
                    rooted!(in(*cx) let mut rval = UndefinedValue());
                    
                    // TODO: check if this is the right globalscope.
                    unsafe {
                        error
                            .clone()
                            .to_jsval(*cx, &*self.global(), rval.handle_mut())
                    };
                    
                    // Perform ! ReadableStreamDefaultControllerError(controller, result.[[Value]]).
                    self.error(rval.handle());
                    
                    // Return result.
                    return Err(error);
                },
            }
        } else {
            0.
        };

        // We create the value-with-size created inside
        // EnqueueValueWithSize here.
        rooted!(in(*cx) let object = chunk.to_object());
        let value_with_size = ValueWithSize {
            value: Heap::default(),
            size,
        };
        value_with_size.value.set(*object);

        // Let enqueueResult be EnqueueValueWithSize(controller, chunk, chunkSize).
        let mut queue = self.queue.borrow_mut();
        if let Err(error) = queue.enqueue_value_with_size(EnqueuedValue::Js(value_with_size)) {
            // If enqueueResult is an abrupt completion,
            
            rooted!(in(*cx) let mut rval = UndefinedValue());
            // TODO: check if this is the right globalscope.
            unsafe {
                error
                    .clone()
                    .to_jsval(*cx, &*self.global(), rval.handle_mut())
            };
            
            // Perform ! ReadableStreamDefaultControllerError(controller, enqueueResult.[[Value]]).
            self.error(rval.handle());
            
            // Return enqueueResult.
            return Err(error);
        }

        // Perform ! ReadableStreamDefaultControllerCallPullIfNeeded(controller).
        self.call_pull_if_needed();

        Ok(())
    }

    /// Native call to
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    #[allow(unsafe_code)]
    pub fn enqueue_native(&self, chunk: Vec<u8>) {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        unsafe { chunk.to_jsval(*cx, rval.handle_mut()) };
        self.enqueue(cx, rval.handle());
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
        let Some(underlying_source) = self.underlying_source.get() else {
            return None;
        };
        if underlying_source.in_memory() {
            return Some(self.queue.borrow().get_in_memory_bytes());
        }
        None
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-clear-algorithms>
    fn clear_algorithms(&self) {
        // Set controller.[[pullAlgorithm]] to undefined.
        // Set controller.[[cancelAlgorithm]] to undefined.
        self.underlying_source.set(None);

        // TODO: Set controller.[[strategySizeAlgorithm]] to undefined.
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
        let Some(stream) = self.stream.get() else {
            return None;
        };

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
        Some(self.strategy_hwm - queue.total_size)
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

impl ReadableStreamDefaultControllerMethods for ReadableStreamDefaultController {
    /// <https://streams.spec.whatwg.org/#rs-default-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        self.get_desired_size()
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-close>
    fn Close(&self) -> Fallible<()> {
        if !self.can_close_or_enqueue() {
            return Err(Error::NotFound);
        }

        self.close();

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-enqueue>
    fn Enqueue(&self, cx: SafeJSContext, chunk: SafeHandleValue) -> Fallible<()> {
        // If ! ReadableStreamDefaultControllerCanCloseOrEnqueue(this) is false, throw a TypeError exception.
        if !self.can_close_or_enqueue() {
            return Err(Error::Type("Stream cannot be enqueued to.".to_string()));
        }

        // Perform ? ReadableStreamDefaultControllerEnqueue(this, chunk).
        self.enqueue(cx, chunk)
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-error>
    fn Error(&self, _cx: SafeJSContext, e: SafeHandleValue) -> Fallible<()> {
        self.error(e);
        Ok(())
    }
}
