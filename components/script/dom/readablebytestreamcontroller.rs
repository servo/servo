/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::VecDeque;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{HandleValue as SafeHandleValue, HandleValue};
use js::typedarray::{ArrayBufferU8, TypedArray, Uint8};

use super::bindings::codegen::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use super::bindings::reflector::{reflect_dom_object, DomObject};
use super::promisenativehandler::Callback;
use super::readablestreambyobreader::ReadIntoRequest;
use super::readablestreamdefaultreader::ReadRequest;
use super::types::{GlobalScope, PromiseNativeHandler};
use crate::dom::bindings::buffer_source::{
    create_buffer_source, new_initialized_heap_buffer_source, HeapBufferSource, HeapTypedArrayInit,
};
use crate::dom::bindings::codegen::Bindings::ReadableByteStreamControllerBinding::ReadableByteStreamControllerMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::readablestream::ReadableStream;
use crate::dom::underlyingsourcecontainer::{UnderlyingSourceContainer, UnderlyingSourceType};
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{JSContext, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry>
#[derive(JSTraceable, MallocSizeOf)]
pub struct ReadableByteStreamQueueEntry {
    #[ignore_malloc_size_of = "HeapArrayBuffer"]
    buffer: HeapBufferSource<ArrayBufferU8>,
    byte_offset: usize,
    byte_length: usize,
}

/// <https://streams.spec.whatwg.org/#queue-with-sizes>
#[derive(Default, JSTraceable, MallocSizeOf)]
pub struct QueueWithSizes {
    total_size: usize,
    queue: VecDeque<ReadableByteStreamQueueEntry>,
}

impl QueueWithSizes {
    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&mut self) -> ReadableByteStreamQueueEntry {
        self.queue
            .pop_front()
            .expect("Buffer cannot be empty when dequeue value is called into.")
    }

    /// <https://streams.spec.whatwg.org/#enqueue-value-with-size>
    fn enqueue_value_with_size(&mut self, value: ReadableByteStreamQueueEntry) {
        self.queue.push_back(value);
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmFulfillmentHandler {
    // TODO: check the validity of using Dom here.
    controller: Dom<ReadableByteStreamController>,
}

impl Callback for PullAlgorithmFulfillmentHandler {
    /// Handle fufillment of pull algo promise.
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        todo!();
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmRejectionHandler {
    // TODO: check the validity of using Dom here.
    controller: Dom<ReadableByteStreamController>,
}

impl Callback for PullAlgorithmRejectionHandler {
    /// Handle rejection of pull algo promise.
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        todo!();
    }
}

// FIXME: this is similar to [`script::dom::readablestream::ReaderType`] but without reader
#[derive(JSTraceable, MallocSizeOf)]
#[allow(dead_code)]
enum ReaderType {
    BYOB,
    Default,
}

// TODO: temporary implementation, should change after having real usage
#[derive(JSTraceable, MallocSizeOf)]
#[allow(dead_code)]
pub enum ViewConstructorType {
    Int8Array,
    Int16Array,
    Int32Array,
    Uint8Array,
    Uint16Array,
    Uint32Array,
    Uint8ClampedArray,
    BigInt64Array,
    BigUint64Array,
    Float32Array,
    Float64Array,
    DataView,
    ArrayBuffer,
}

/// <https://streams.spec.whatwg.org/#pull-into-descriptor>
#[derive(JSTraceable, MallocSizeOf)]
struct PullIntoDescriptor {
    #[ignore_malloc_size_of = "HeapArrayBuffer"]
    buffer: HeapBufferSource<ArrayBufferU8>,
    buffer_byte_length: usize,
    byte_offset: usize,
    byte_length: usize,
    bytes_filled: usize,
    minimun_fill: usize,
    element_size: usize,
    view_constructor: ViewConstructorType,
    reader_type: Option<ReaderType>,
}

/// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
#[dom_struct]
pub struct ReadableByteStreamController {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-queue>
    queue: RefCell<QueueWithSizes>,

    underlying_source: Dom<UnderlyingSourceContainer>,

    stream: MutNullableDom<ReadableStream>,

    auto_allocate_chunk_size: Option<usize>,

    pending_pull_intos: RefCell<Vec<PullIntoDescriptor>>,

    closed_requested: bool,

    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-strategyhwm>
    strategy_hwm: f64,
}

impl ReadableByteStreamController {
    pub fn new_inherited(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
        strategy_hwm: f64,
    ) -> ReadableByteStreamController {
        ReadableByteStreamController {
            reflector_: Reflector::new(),
            queue: RefCell::new(Default::default()),
            underlying_source: Dom::from_ref(&*UnderlyingSourceContainer::new(
                global,
                underlying_source_type,
            )),
            stream: MutNullableDom::new(None),
            auto_allocate_chunk_size: None,
            pending_pull_intos: RefCell::new(Vec::new()),
            closed_requested: false,
            strategy_hwm,
        }
    }

    pub fn new(
        global: &GlobalScope,
        underlying_source: UnderlyingSourceType,
        strategy_hwm: f64,
    ) -> DomRoot<ReadableByteStreamController> {
        reflect_dom_object(
            Box::new(ReadableByteStreamController::new_inherited(
                global,
                underlying_source,
                strategy_hwm,
            )),
            global,
        )
    }

    pub fn set_stream(&self, stream: &ReadableStream) {
        self.stream.set(Some(stream))
    }

    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&self) -> ReadableByteStreamQueueEntry {
        let mut queue = self.queue.borrow_mut();
        queue.dequeue_value()
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-should-call-pull>
    fn should_pull(&self) -> bool {
        // TODO: implement the algo.
        true
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
    fn call_pull_if_needed(&self) {
        // step 1,2
        if !self.should_pull() {
            return;
        }

        let global = self.global();
        let rooted_byte_controller = DomRoot::from_ref(self);
        let controller = Controller::ReadableByteStreamController(rooted_byte_controller.clone());

        // TODO: missing step 3,4,5

        // step 6
        if let Some(promise) = self.underlying_source.call_pull_algorithm(controller) {
            // TODO: step 7
            let fulfillment_handler = Box::new(PullAlgorithmFulfillmentHandler {
                controller: Dom::from_ref(&*rooted_byte_controller),
            });
            // TODO: step 8
            let rejection_handler = Box::new(PullAlgorithmRejectionHandler {
                controller: Dom::from_ref(&*rooted_byte_controller),
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

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-pull-into>
    pub fn pull_into(&self, read_into_request: ReadIntoRequest) {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-private-pull>
    pub fn perform_pull_steps(&self, cx: SafeJSContext, read_request: ReadRequest) {
        // step 1
        let stream: DomRoot<ReadableStream> = self
            .stream
            .get()
            .expect("Controller must have a stream when pull steps are called into.");

        // step 2
        assert!(stream.has_default_reader());

        // step 3
        if self.queue.borrow().total_size > 0 {
            // step 3.1
            assert!(stream.get_num_read_requests() == 0);
            // step 3.2
            self.fill_read_request_from_queue(cx, &read_request);
            // step 3.3
            return;
        }

        // step 4,5
        if let Some(auto_allocate_chunk_size) = self.auto_allocate_chunk_size {
            // step 5.1
            let buffer =
                new_initialized_heap_buffer_source::<ArrayBufferU8>(HeapTypedArrayInit::Info {
                    len: auto_allocate_chunk_size as u32,
                    cx,
                });

            // step 5.2
            if buffer.is_err() {
                // TODO: step 5.2.1
                return;
            }

            // step 5.3
            let descriptor = PullIntoDescriptor {
                buffer: buffer.unwrap(),
                buffer_byte_length: auto_allocate_chunk_size,
                byte_offset: 0,
                byte_length: auto_allocate_chunk_size,
                bytes_filled: 0,
                minimun_fill: 1,
                element_size: 1,
                view_constructor: ViewConstructorType::Uint8Array,
                reader_type: Some(ReaderType::Default),
            };

            // step 5.4
            self.pending_pull_intos.borrow_mut().push(descriptor);
        }

        // step 6
        stream.add_read_request(read_request);
        // step 7
        self.call_pull_if_needed();
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerfillreadrequestfromqueue>
    #[allow(unsafe_code)]
    pub fn fill_read_request_from_queue(&self, cx: SafeJSContext, read_request: &ReadRequest) {
        // step 1
        assert!(self.queue.borrow().total_size > 0);

        // step 2,3
        let entry = self.dequeue_value();

        // step 4
        self.queue.borrow_mut().total_size -= entry.byte_length;

        // step 5
        self.handle_queue_drain();

        // step 6
        let buffer = entry
            .buffer
            .get_buffer()
            .expect("Getting buffer should not fail");
        let data: &[u8] = unsafe {
            let start = entry.byte_offset;
            let end = entry.byte_offset + entry.byte_length;
            buffer
                .as_slice()
                .get(start..end)
                .expect("ArrayBuffer slice should not overflow")
        };
        rooted!(in (*cx) let mut array = std::ptr::null_mut::<JSObject>());
        let view: TypedArray<Uint8, *mut JSObject> =
            create_buffer_source(cx, data, array.handle_mut())
                .expect("Converting to Uint8Array should never fail");

        rooted!(in(*cx) let mut view_jsval = UndefinedValue());
        unsafe { view.to_jsval(*cx, view_jsval.handle_mut()) }
        let chunk = RootedTraceableBox::new(Heap::default());
        chunk.set(*view_jsval);

        // step 7
        read_request.chunk_steps(chunk);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-handle-queue-drain>
    fn handle_queue_drain(&self) {
        let stream: DomRoot<ReadableStream> = self
            .stream
            .get()
            .expect("Controller must have a stream when pull steps are called into.");
        // step 1
        assert!(stream.is_readable());

        // step 2
        if self.queue.borrow().total_size == 0 && self.closed_requested {
            // TODO: step 2.1
            self.clear_algorithm();
            // step 2.2
            stream.close();
        } else {
            // step 3
            self.call_pull_if_needed();
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-clear-algorithms>
    fn clear_algorithm(&self) {
        todo!()
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
