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
use crate::dom::bindings::codegen::Bindings::ReadableByteStreamControllerBinding::ReadableByteStreamControllerMethods;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::{
    ReadableStreamController, UnderlyingSource,
};
use crate::dom::bindings::import::module::{Error, ExceptionHandling, Fallible, InRealm};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::{ReadableStream, StreamState};
use crate::dom::readablestreambyobrequest::ReadableStreamBYOBRequest;
use crate::realms::enter_realm;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
#[dom_struct]
pub struct ReadableByteStreamController {
    reflector_: Reflector,
    /// A positive integer, when the automatic buffer allocation feature is enabled. In that case, this value specifies
    /// the size of buffer to allocate. It is undefined otherwise.
    auto_allocate_chunk_size: Cell<Option<u64>>,
    /// A ReadableStreamBYOBRequest instance representing the current BYOB pull request, or null if
    /// there are no pending requests.
    byob_request: MutNullableDom<ReadableStreamBYOBRequest>,
    /// All algoritems packed together:
    /// - Cancel algorithm: A promise-returning algorithm, taking one argument (the cancel reason), which communicates
    ///   a requested cancelation to the underlying byte source
    /// - Pull algorithm: A promise-returning algorithm that pulls data from the underlying byte source
    algorithms: DomRefCell<ControllerAlgorithms>,
    /// A boolean flag indicating whether the stream has been closed by its underlying byte source, but still has
    /// chunks in its internal queue that have not yet been read
    close_requested: Cell<bool>,
    /// A boolean flag set to true if the stream’s mechanisms requested a call to the underlying byte source's pull
    /// algorithm to pull more data, but the pull could not yet be done since a previous call is still executing
    pull_again: Cell<bool>,
    /// A boolean flag set to true while the underlying byte source's pull algorithm is executing and the returned
    /// promise has not yet fulfilled, used to prevent reentrant calls
    pulling: Cell<bool>,
    /// A list of pull-into descriptors
    #[ignore_malloc_size_of = "Defined in mozjs"]
    pending_pull_intos: DomRefCell<VecDeque<Heap<JSVal>>>,
    /// A list of readable byte stream queue entries representing the stream’s internal queue of chunks
    #[ignore_malloc_size_of = "Defined in mozjs"]
    queue: DomRefCell<VecDeque<Heap<JSVal>>>,
    /// A boolean flag indicating whether the underlying byte source has finished starting
    started: Cell<bool>,
    /// A number supplied to the constructor as part of the stream’s queuing strategy, indicating the point at which
    /// the stream will apply backpressure to its underlying byte source
    strategy_highwatermark: Cell<f64>,
    /// The ReadableStream instance controlled
    stream: DomRoot<ReadableStream>,
}

impl ReadableByteStreamController {
    fn new_inherited(stream: DomRoot<ReadableStream>) -> Self {
        Self {
            reflector_: Reflector::new(),
            auto_allocate_chunk_size: Cell::new(None),
            byob_request: MutNullableDom::new(None),
            pending_pull_intos: Default::default(),
            queue: Default::default(),
            close_requested: Cell::new(false),
            pull_again: Cell::new(false),
            pulling: Cell::new(false),
            started: Cell::new(false),
            strategy_highwatermark: Cell::new(0.),
            algorithms: DomRefCell::new(ControllerAlgorithms::Undefined),
            stream,
        }
    }

    fn new(global: &GlobalScope, stream: DomRoot<ReadableStream>) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(stream)), global)
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-should-call-pull>
    fn should_call_pull(&self) -> bool {
        // Step 1
        let stream = &self.stream;
        // Step 2
        if stream.state() == StreamState::Readable {
            false
        // Step 3
        } else if self.close_requested.get() {
            false
        // Step 4
        } else if !self.started.get() {
            false
        // Step 5
        } else if stream.has_default_reader() && stream.get_num_read_requests() > 0 {
            true
        // Step 6
        } else if stream.has_byob_reader() && stream.get_num_read_into_requests() > 0 {
            true
        // Step 7 ~ 9
        } else if self.get_desired_size().unwrap() > 0. {
            true
        // Step 10
        } else {
            false
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-get-desired-size>
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

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-error>
    fn error(&self, _e: SafeHandleValue) {
        // TODO
    }
}

impl ReadableByteStreamControllerMethods for ReadableByteStreamController {
    /// <https://streams.spec.whatwg.org/#rbs-controller-byob-request>
    fn GetByobRequest(
        &self,
    ) -> Fallible<Option<DomRoot<super::readablestreambyobrequest::ReadableStreamBYOBRequest>>>
    {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        // TODO
        None
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-close>
    fn Close(&self) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-enqueue>
    fn Enqueue(
        &self,
        _chunk: js::gc::CustomAutoRooterGuard<js::typedarray::ArrayBufferView>,
    ) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-error>
    fn Error(&self, _cx: SafeJSContext, _e: SafeHandleValue) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }
}

/// <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller-from-underlying-source>
pub fn setup_readable_byte_stream_controller_from_underlying_source(
    cx: SafeJSContext,
    stream: DomRoot<ReadableStream>,
    underlying_source_obj: SafeHandleObject,
    underlying_source_dict: UnderlyingSource,
    highwatermark: f64,
) -> Fallible<()> {
    // Step 8
    let auto_allocate_chunk_size = underlying_source_dict.autoAllocateChunkSize;
    // Step 9
    if let Some(size) = auto_allocate_chunk_size {
        if size == 0 {
            return Err(Error::Type("autoAllocateChunkSize can't be 0".to_string()));
        }
    }

    // Step 2. - 7. See UnderlyingSourceAlgorithms
    let algorithms = UnderlyingSourceAlgorithms::new(underlying_source_dict, underlying_source_obj);

    // Step 1
    let controller = ReadableByteStreamController::new(&*stream.global(), stream);

    set_up_readable_byte_stream_controller(
        cx,
        controller,
        ControllerAlgorithms::UnderlyingSource(algorithms),
        highwatermark,
        auto_allocate_chunk_size,
    )
}

/// <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller>
fn set_up_readable_byte_stream_controller(
    cx: SafeJSContext,
    controller: DomRoot<ReadableByteStreamController>,
    algorithms: ControllerAlgorithms,
    highwatermark: f64,
    auto_allocate_chunk_size: Option<u64>,
) -> Fallible<()> {
    // Step 1
    assert!(controller.stream.controller().is_none());
    // Step 2
    if let Some(size) = auto_allocate_chunk_size {
        assert!(size > 0);
    }
    // Step 3 is done in ReadableStreamDefaultController::new already.
    // Step 4
    controller.pull_again.set(false);
    controller.pulling.set(false);
    // Step 5 is done in ReadableStreamDefaultController::new already.
    // Step 6
    controller.queue.borrow_mut().clear();
    // Step 7
    controller.started.set(false);
    controller.close_requested.set(false);
    // Step 8
    controller.strategy_highwatermark.set(highwatermark);
    // Step 9 & 10
    *controller.algorithms.borrow_mut() = algorithms;
    // Step 11
    controller
        .auto_allocate_chunk_size
        .set(auto_allocate_chunk_size);
    // Step 12
    controller.pending_pull_intos.borrow_mut().clear();
    // Step 13
    controller
        .stream
        .set_controller(ReadableStreamController::ReadableByteStreamController(
            controller.clone(),
        ));
    // Step 14
    rooted!(in(*cx) let mut start_result = UndefinedValue());
    controller.algorithms.borrow().start(
        cx,
        ReadableStreamController::ReadableByteStreamController(controller.clone()),
        start_result.handle_mut(),
    )?;
    // Step 15
    let global = &*controller.stream.global();
    let realm = enter_realm(&*global);
    let comp = InRealm::Entered(&realm);
    let start_promise = Promise::new_resolved(global, cx, start_result.handle())?;
    // Step 16 & 17
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
        controller: DomRoot<ReadableByteStreamController>,
    }

    impl ResolveHandler {
        pub fn new(controller: DomRoot<ReadableByteStreamController>) -> Box<dyn Callback> {
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
            assert!(readable_byte_stream_controller_call_pull_if_needed(
                cx,
                self.controller.clone()
            )
            .is_ok());
        }
    }

    Ok(())
}

/// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
fn readable_byte_stream_controller_call_pull_if_needed(
    cx: SafeJSContext,
    controller: DomRoot<ReadableByteStreamController>,
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
                ReadableStreamController::ReadableByteStreamController(controller.clone()),
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
        controller: DomRoot<ReadableByteStreamController>,
    }

    impl ResolveHandler {
        pub fn new(controller: DomRoot<ReadableByteStreamController>) -> Box<dyn Callback> {
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
                assert!(readable_byte_stream_controller_call_pull_if_needed(
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
    controller: DomRoot<ReadableByteStreamController>,
}

impl RejectHandler {
    pub fn new(controller: DomRoot<ReadableByteStreamController>) -> Box<dyn Callback> {
        Box::new(Self { controller })
    }
}

impl Callback for RejectHandler {
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm) {
        self.controller.error(v);
    }
}
