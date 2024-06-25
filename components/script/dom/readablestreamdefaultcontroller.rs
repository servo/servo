/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::gc::MutableHandleValue;
use js::jsapi::{HandleValue, Heap, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::{
    ReadableStreamController, UnderlyingSource,
};
use crate::dom::bindings::import::module::{ExceptionHandling, Fallible, InRealm};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::{ReadableStream, StreamState};
use crate::realms::enter_realm;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
#[dom_struct]
pub struct ReadableStreamDefaultController {
    reflector_: Reflector,
    /// All algoritems packed together:
    /// - Cancel algorithm: A promise-returning algorithm, taking one argument (the cancel reason), which communicates
    ///   a requested cancelation to the underlying source
    /// - Pull algorithm: A promise-returning algorithm that pulls data from the underlying source
    algorithms: DomRefCell<ControllerAlgorithms>,
    /// A boolean flag indicating whether the stream has been closed by its underlying source, but still has chunks in
    /// its internal queue that have not yet been read
    close_requested: Cell<bool>,
    /// A boolean flag set to true if the stream’s mechanisms requested a call to the underlying source's pull
    /// algorithm to pull more data, but the pull could not yet be done since a previous call is still executing
    pull_again: Cell<bool>,
    /// A boolean flag set to true while the underlying source's pull algorithm is executing and the returned promise
    /// has not yet fulfilled, used to prevent reentrant calls
    pulling: Cell<bool>,
    /// A list representing the stream’s internal queue of chunks
    #[ignore_malloc_size_of = "Defined in mozjs"]
    queue: DomRefCell<VecDeque<Heap<JSVal>>>,
    /// A boolean flag indicating whether the underlying source has finished starting
    started: Cell<bool>,
    /// A number supplied to the constructor as part of the stream’s queuing strategy, indicating the point at which
    /// the stream will apply backpressure to its underlying source
    strategy_highwatermark: Cell<f64>,
    /// An algorithm to calculate the size of enqueued chunks, as part of the stream’s queuing strategy
    ///
    /// If missing use default value (1) per https://streams.spec.whatwg.org/#make-size-algorithm-from-size-function
    #[ignore_malloc_size_of = "Rc is hard"]
    strategy_size_algorithm: DomRefCell<Option<Rc<QueuingStrategySize>>>,
    /// The ReadableStream instance controlled
    stream: DomRoot<ReadableStream>,
}

impl ReadableStreamDefaultController {
    fn new_inherited(stream: DomRoot<ReadableStream>) -> Self {
        Self {
            reflector_: Reflector::new(),
            queue: Default::default(),
            close_requested: Cell::new(false),
            pull_again: Cell::new(false),
            pulling: Cell::new(false),
            started: Cell::new(false),
            strategy_highwatermark: Cell::new(0.),
            algorithms: DomRefCell::new(ControllerAlgorithms::Undefined),
            strategy_size_algorithm: DomRefCell::new(None),
            stream,
        }
    }

    fn new(global: &GlobalScope, stream: DomRoot<ReadableStream>) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(stream)), global)
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-should-call-pull>
    fn should_call_pull(&self) -> bool {
        // Step 1
        let stream = &self.stream;
        // Step 2
        if !self.can_close_or_enqueue() {
            false
        // Step 3
        } else if !self.started.get() {
            false
        // Step 4
        } else if stream.is_locked() && stream.get_num_read_requests() > 0 {
            return true;
        // Step 5 ~ 7
        } else if self.get_desired_size().unwrap() > 0. {
            true
        // Step 8
        } else {
            false
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-can-close-or-enqueue>
    fn can_close_or_enqueue(&self) -> bool {
        // Step 1
        let state = self.stream.state();
        // Step 2 & 3
        if !self.close_requested.get() && state == StreamState::Readable {
            return true;
        } else {
            return false;
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-get-desired-size>
    pub fn get_desired_size(&self) -> Option<f64> {
        // Step 1
        let state = self.stream.state();
        match state {
            // Step 2
            StreamState::Errored => None,
            // Step 3
            StreamState::Closed => Some(0.),
            // Step 4
            StreamState::Readable => {
                Some(self.strategy_highwatermark.get() - self.queue.borrow().len() as f64)
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-error>
    fn error(&self, _e: SafeHandleValue) {
        // TODO
    }
}

impl ReadableStreamDefaultControllerMethods for ReadableStreamDefaultController {
    /// <https://streams.spec.whatwg.org/#rs-default-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        // TODO
        None
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-close>
    fn Close(&self) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-enqueue>
    fn Enqueue(&self, _cx: SafeJSContext, _chunk: SafeHandleValue) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-error>
    fn Error(&self, _cx: SafeJSContext, _e: SafeHandleValue) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }
}

/// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller-from-underlying-source>
pub fn setup_readable_stream_default_controller_from_underlying_source(
    cx: SafeJSContext,
    stream: DomRoot<ReadableStream>,
    underlying_source_obj: SafeHandleObject,
    underlying_source_dict: UnderlyingSource,
    highwatermark: f64,
    size_algorithm: Rc<QueuingStrategySize>,
) -> Fallible<()> {
    // Step 2. - 7. See UnderlyingSourceAlgorithms
    let algorithms = UnderlyingSourceAlgorithms::new(underlying_source_dict, underlying_source_obj);

    // Step 1
    let controller = ReadableStreamDefaultController::new(&*stream.global(), stream);

    // Step 8
    set_up_readable_stream_default_controller(
        cx,
        controller,
        ControllerAlgorithms::UnderlyingSource(algorithms),
        highwatermark,
        size_algorithm,
    )
}

/// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
fn set_up_readable_stream_default_controller(
    cx: SafeJSContext,
    controller: DomRoot<ReadableStreamDefaultController>,
    algorithms: ControllerAlgorithms,
    highwatermark: f64,
    size_algorithm: Rc<QueuingStrategySize>,
) -> Fallible<()> {
    // Step 1
    assert!(controller.stream.controller().is_none());
    // Step 2 is done in ReadableStreamDefaultController::new already.
    // Step 3 Perform ! ResetQueue(controller).
    controller.queue.borrow_mut().clear();
    // Step 4
    controller.started.set(false);
    controller.close_requested.set(false);
    controller.pull_again.set(false);
    controller.pulling.set(false);
    // Step 5
    *controller.strategy_size_algorithm.borrow_mut() = Some(size_algorithm);
    controller.strategy_highwatermark.set(highwatermark);
    // Step 6 & 7
    *controller.algorithms.borrow_mut() = algorithms;
    // Step 8
    controller
        .stream
        .set_controller(ReadableStreamController::ReadableStreamDefaultController(
            controller.clone(),
        ));
    // Step 9
    rooted!(in(*cx) let mut start_result = UndefinedValue());
    controller.algorithms.borrow().start(
        cx,
        ReadableStreamController::ReadableStreamDefaultController(controller.clone()),
        start_result.handle_mut(),
    )?;
    // Step 10
    let global = &*controller.stream.global();
    let realm = enter_realm(&*global);
    let comp = InRealm::Entered(&realm);
    let start_promise = Promise::new_resolved(global, cx, start_result.handle())?;
    // Step 11 & 12
    start_promise.append_native_handler(
        &PromiseNativeHandler::new(
            global,
            Some(ResolveHandler::new(controller.clone())),
            Some(RejectHandler::new(controller)),
        ),
        comp,
    );

    #[derive(JSTraceable, MallocSizeOf)]
    struct ResolveHandler {
        controller: DomRoot<ReadableStreamDefaultController>,
    }

    impl ResolveHandler {
        pub fn new(controller: DomRoot<ReadableStreamDefaultController>) -> Box<dyn Callback> {
            Box::new(Self { controller })
        }
    }

    impl Callback for ResolveHandler {
        fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm) {
            // Step 11.1
            self.controller.started.set(true);
            // Step 11.2
            assert!(!self.controller.pulling.get());
            // Step 11.3
            assert!(!self.controller.pull_again.get());
            // Step 11.4
            assert!(readable_stream_default_controller_call_pull_if_needed(
                cx,
                self.controller.clone()
            )
            .is_ok());
        }
    }

    Ok(())
}

/// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
fn readable_stream_default_controller_call_pull_if_needed(
    cx: SafeJSContext,
    controller: DomRoot<ReadableStreamDefaultController>,
) -> Fallible<()> {
    // Step 1 & 2
    if controller.should_call_pull() {
        // Step 3
        if controller.pulling.get() {
            controller.pull_again.set(true);
        } else {
            // Step 4
            assert!(!controller.pull_again.get());
            // Step 5
            controller.pulling.set(true);
            // Step 6
            let pull_promise = controller.algorithms.borrow().pull(
                cx,
                ReadableStreamController::ReadableStreamDefaultController(controller.clone()),
            )?;
            let global = &*controller.global();
            let realm = enter_realm(&*global);
            let comp = InRealm::Entered(&realm);
            pull_promise.append_native_handler(
                &PromiseNativeHandler::new(
                    global,
                    Some(ResolveHandler::new(controller.clone())),
                    Some(RejectHandler::new(controller)),
                ),
                comp,
            );
        }
    }

    #[derive(JSTraceable, MallocSizeOf)]
    struct ResolveHandler {
        controller: DomRoot<ReadableStreamDefaultController>,
    }

    impl ResolveHandler {
        pub fn new(controller: DomRoot<ReadableStreamDefaultController>) -> Box<dyn Callback> {
            Box::new(Self { controller })
        }
    }

    impl Callback for ResolveHandler {
        fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm) {
            // Step 7.1
            self.controller.pulling.set(false);
            // Step 7.2
            if self.controller.pull_again.get() {
                self.controller.pull_again.set(false);
                assert!(readable_stream_default_controller_call_pull_if_needed(
                    cx,
                    self.controller.clone()
                )
                .is_ok());
            }
        }
    }
    Ok(())
}

/// Algorithms for [setup_readable_stream_default_controller_from_underlying_source]
#[derive(JSTraceable, MallocSizeOf)]
pub enum ControllerAlgorithms {
    UnderlyingSource(UnderlyingSourceAlgorithms),
    Undefined,
}

impl ControllerAlgorithms {
    fn start(
        &self,
        cx: SafeJSContext,
        controller: ReadableStreamController,
        retval: MutableHandleValue,
    ) -> Fallible<()> {
        match self {
            ControllerAlgorithms::UnderlyingSource(s) => s.start(cx, controller, retval),
            ControllerAlgorithms::Undefined => unreachable!(),
        }
    }

    fn pull(
        &self,
        cx: SafeJSContext,
        controller: ReadableStreamController,
    ) -> Fallible<Rc<Promise>> {
        match self {
            ControllerAlgorithms::UnderlyingSource(s) => s.pull(cx, controller),
            ControllerAlgorithms::Undefined => unreachable!(),
        }
    }

    fn cancel(&self, cx: SafeJSContext, reason: Option<HandleValue>) -> Fallible<Rc<Promise>> {
        match self {
            ControllerAlgorithms::UnderlyingSource(s) => s.cancel(cx, reason),
            ControllerAlgorithms::Undefined => unreachable!(),
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct UnderlyingSourceAlgorithms {
    #[ignore_malloc_size_of = "bindings from mozjs"]
    underlying_source_dict: UnderlyingSource,
    #[ignore_malloc_size_of = "mozjs"]
    underlying_source_obj: Heap<*mut JSObject>,
}

impl UnderlyingSourceAlgorithms {
    pub fn new(underlying_source_dict: UnderlyingSource, obj: SafeHandleObject) -> Self {
        let underlying_source_obj = Heap::default();
        underlying_source_obj.set(obj.get());
        Self {
            underlying_source_dict,
            underlying_source_obj,
        }
    }
}

impl UnderlyingSourceAlgorithms {
    fn start(
        &self,
        cx: SafeJSContext,
        controller: ReadableStreamController,
        mut retval: MutableHandleValue,
    ) -> Fallible<()> {
        // Step 2
        rooted!(in(*cx) let mut val = UndefinedValue());
        // Step 5
        if let Some(callback) = &self.underlying_source_dict.start {
            val.set(callback.call_with_existing_obj(
                &self.underlying_source_obj,
                controller,
                ExceptionHandling::Rethrow,
            )?);
        }

        retval.set(val.get());
        Ok(())
    }

    fn pull(
        &self,
        cx: SafeJSContext,
        controller: ReadableStreamController,
    ) -> Fallible<Rc<Promise>> {
        // Step 3 & 6
        if let Some(callback) = &self.underlying_source_dict.pull {
            callback.call_with_existing_obj(
                &self.underlying_source_obj,
                controller,
                ExceptionHandling::Rethrow,
            )
        } else {
            Promise::new_resolved(
                &GlobalScope::current().expect("No current global"),
                cx,
                SafeHandleValue::undefined(),
            )
        }
    }

    fn cancel(&self, cx: SafeJSContext, reason: Option<HandleValue>) -> Fallible<Rc<Promise>> {
        // Step 4 & 7
        if let Some(callback) = &self.underlying_source_dict.cancel {
            callback.call_with_existing_obj(
                &self.underlying_source_obj,
                reason,
                ExceptionHandling::Rethrow,
            )
        } else {
            Promise::new_resolved(
                &GlobalScope::current().expect("No current global"),
                cx,
                SafeHandleValue::undefined(),
            )
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct RejectHandler {
    controller: DomRoot<ReadableStreamDefaultController>,
}

impl RejectHandler {
    pub fn new(controller: DomRoot<ReadableStreamDefaultController>) -> Box<dyn Callback> {
        Box::new(Self { controller })
    }
}

impl Callback for RejectHandler {
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm) {
        self.controller.error(v);
    }
}
