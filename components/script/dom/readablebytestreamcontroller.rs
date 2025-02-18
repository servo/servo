/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JS_ClearPendingException};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::wrappers::JS_GetPendingException;
use js::rust::{HandleObject, HandleValue as SafeHandleValue, HandleValue};
use js::typedarray::{ArrayBufferU8, ArrayBufferViewU8};

use super::bindings::buffer_source::HeapBufferSource;
use super::bindings::cell::DomRefCell;
use super::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderReadOptions;
use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::Dom;
use super::readablestreambyobreader::ReadIntoRequest;
use super::readablestreamdefaultreader::ReadRequest;
use super::underlyingsourcecontainer::{UnderlyingSourceContainer, UnderlyingSourceType};
use crate::dom::bindings::buffer_source::{
    byte_size, create_buffer_source_with_constructor, create_uint8_array_with_buffer, BufferSource,
    Constructor,
};
use crate::dom::bindings::codegen::Bindings::ReadableByteStreamControllerBinding::ReadableByteStreamControllerMethods;
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::ReadableStream;
use crate::dom::readablestreambyobrequest::ReadableStreamBYOBRequest;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct QueueEntry {
    /// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry-buffer>
    #[ignore_malloc_size_of = "HeapBufferSource"]
    buffer: HeapBufferSource<ArrayBufferU8>,
    /// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry-byte-offset>
    byte_offset: RefCell<usize>,
    /// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry-byte-length>
    byte_length: RefCell<usize>,
}

impl QueueEntry {
    pub(crate) fn new(
        buffer: HeapBufferSource<ArrayBufferU8>,
        byte_offset: usize,
        byte_length: usize,
    ) -> QueueEntry {
        QueueEntry {
            buffer,
            byte_offset: RefCell::new(byte_offset),
            byte_length: RefCell::new(byte_length),
        }
    }

    pub(crate) fn byte_length(&self) -> usize {
        *self.byte_length.borrow()
    }

    pub(crate) fn byte_offset(&self) -> usize {
        *self.byte_offset.borrow()
    }

    pub(crate) fn set_byte_offset(&self, byte_offset: usize) {
        *self.byte_offset.borrow_mut() = byte_offset;
    }

    pub(crate) fn set_byte_length(&self, byte_length: usize) {
        *self.byte_length.borrow_mut() = byte_length;
    }
}

#[derive(Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ReaderType {
    /// <https://streams.spec.whatwg.org/#readablestreambyobreader>
    BYOB,
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
    Default,
}

/// <https://streams.spec.whatwg.org/#pull-into-descriptor>
#[derive(Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct PullIntoDescriptor {
    #[ignore_malloc_size_of = "HeapBufferSource"]
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-buffer>
    buffer: RefCell<HeapBufferSource<ArrayBufferU8>>,
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-buffer-byte-length>
    buffer_byte_length: usize,
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-byte-offset>
    byte_offset: usize,
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-byte-length>
    byte_length: usize,
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-bytes-filled>
    bytes_filled: RefCell<usize>,
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-minimum-fill>
    minimum_fill: usize,
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-element-size>
    element_size: usize,
    #[ignore_malloc_size_of = "HeapBufferSource"]
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-view-constructor>
    view_constructor: Constructor,
    /// <https://streams.spec.whatwg.org/#pull-into-descriptor-reader-type>
    reader_type: Option<ReaderType>,
}

impl PullIntoDescriptor {
    pub(crate) fn set_buffer(&self, buffer: HeapBufferSource<ArrayBufferU8>) {
        *self.buffer.borrow_mut() = buffer;
    }

    pub(crate) fn set_bytes_filled(&self, bytes_filled: usize) {
        *self.bytes_filled.borrow_mut() = bytes_filled;
    }

    pub(crate) fn bytes_filled(&self) -> usize {
        *self.bytes_filled.borrow()
    }
}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#dom-underlyingsource-start>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct StartAlgorithmFulfillmentHandler {
    controller: Dom<ReadableByteStreamController>,
}

impl Callback for StartAlgorithmFulfillmentHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller>
    /// Upon fulfillment of startPromise,
    fn callback(&self, _cx: SafeJSContext, _v: HandleValue, _realm: InRealm, can_gc: CanGc) {
        // Set controller.[[started]] to true.
        self.controller.started.set(true);

        // Assert: controller.[[pulling]] is false.
        assert!(!self.controller.pulling.get());

        // Assert: controller.[[pullAgain]] is false.
        assert!(!self.controller.pull_again.get());

        // Perform ! ReadableByteStreamControllerCallPullIfNeeded(controller).
        self.controller.call_pull_if_needed(can_gc);
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#dom-underlyingsource-start>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct StartAlgorithmRejectionHandler {
    controller: Dom<ReadableByteStreamController>,
}

impl Callback for StartAlgorithmRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller>
    /// Upon rejection of startPromise with reason r,
    fn callback(&self, _cx: SafeJSContext, v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
        // Perform ! ReadableByteStreamControllerError(controller, r).
        self.controller.error(v);
    }
}

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PullAlgorithmFulfillmentHandler {
    controller: Dom<ReadableByteStreamController>,
}

impl Callback for PullAlgorithmFulfillmentHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
    /// Upon fulfillment of pullPromise
    fn callback(&self, _cx: SafeJSContext, _v: HandleValue, _realm: InRealm, can_gc: CanGc) {
        // Set controller.[[pulling]] to false.
        self.controller.pulling.set(false);

        // If controller.[[pullAgain]] is true,
        if self.controller.pull_again.get() {
            // Set controller.[[pullAgain]] to false.
            self.controller.pull_again.set(false);

            // Perform ! ReadableByteStreamControllerCallPullIfNeeded(controller).
            self.controller.call_pull_if_needed(can_gc);
        }
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PullAlgorithmRejectionHandler {
    controller: Dom<ReadableByteStreamController>,
}

impl Callback for PullAlgorithmRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#readable-stream-byte-controller-call-pull-if-needed>
    /// Upon rejection of pullPromise with reason e.
    fn callback(&self, _cx: SafeJSContext, v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
        // Perform ! ReadableByteStreamControllerError(controller, e).
        self.controller.error(v);
    }
}

/// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
#[dom_struct]
pub(crate) struct ReadableByteStreamController {
    reflector_: Reflector,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-autoallocatechunksize>
    auto_allocate_chunk_size: Option<u64>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-stream>
    stream: MutNullableDom<ReadableStream>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-strategyhwm>
    strategy_hwm: f64,
    /// A mutable reference to the underlying source is used to implement these two
    /// internal slots:
    ///
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-pullalgorithm>
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-cancelalgorithm>
    underlying_source: MutNullableDom<UnderlyingSourceContainer>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-queue>
    queue: DomRefCell<VecDeque<QueueEntry>>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-queuetotalsize>
    queue_total_size: Cell<f64>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-byobrequest>
    byob_request: MutNullableDom<ReadableStreamBYOBRequest>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-pendingpullintos>
    pending_pull_intos: DomRefCell<Vec<PullIntoDescriptor>>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-closerequested>
    close_requested: Cell<bool>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-started>
    started: Cell<bool>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-pulling>
    pulling: Cell<bool>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-pullalgorithm>
    pull_again: Cell<bool>,
}

impl ReadableByteStreamController {
    fn new_inherited(
        underlying_source_type: UnderlyingSourceType,
        strategy_hwm: f64,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> ReadableByteStreamController {
        let underlying_source_container =
            UnderlyingSourceContainer::new(global, underlying_source_type, can_gc);
        let auto_allocate_chunk_size = underlying_source_container.auto_allocate_chunk_size();
        ReadableByteStreamController {
            reflector_: Reflector::new(),
            byob_request: MutNullableDom::new(None),
            stream: MutNullableDom::new(None),
            underlying_source: MutNullableDom::new(Some(&*underlying_source_container)),
            auto_allocate_chunk_size,
            pending_pull_intos: DomRefCell::new(Vec::new()),
            strategy_hwm,
            close_requested: Default::default(),
            queue: DomRefCell::new(Default::default()),
            queue_total_size: Default::default(),
            started: Default::default(),
            pulling: Default::default(),
            pull_again: Default::default(),
        }
    }

    pub(crate) fn new(
        underlying_source_type: UnderlyingSourceType,
        strategy_hwm: f64,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<ReadableByteStreamController> {
        reflect_dom_object(
            Box::new(ReadableByteStreamController::new_inherited(
                underlying_source_type,
                strategy_hwm,
                global,
                can_gc,
            )),
            global,
            can_gc,
        )
    }

    pub(crate) fn set_stream(&self, stream: &ReadableStream) {
        self.stream.set(Some(stream))
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-pull-into>
    #[allow(unsafe_code)]
    pub(crate) fn perform_pull_into(
        &self,
        read_into_request: &ReadIntoRequest,
        view: HeapBufferSource<ArrayBufferViewU8>,
        options: &ReadableStreamBYOBReaderReadOptions,
        can_gc: CanGc,
    ) {
        let cx = GlobalScope::get_cx();
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().unwrap();

        // Let elementSize be 1.
        let mut element_size = 1;

        // Let ctor be %DataView%.
        let mut ctor = Constructor::DataView;

        // If view has a [[TypedArrayName]] internal slot (i.e., it is not a DataView),
        if view.has_typed_array_name() {
            // Set elementSize to the element size specified in the typed array constructors table for view.[[TypedArrayName]].
            let view_typw = view.get_array_buffer_view_type();
            element_size = byte_size(view_typw);

            // Set ctor to the constructor specified in the typed array constructors table for view.[[TypedArrayName]].
            ctor = Constructor::Name(view_typw);
        }

        // Let minimumFill be min × elementSize.
        let minimum_fill = (options.min as usize) * element_size;

        // Assert: minimumFill ≥ 0 and minimumFill ≤ view.[[ByteLength]].
        assert!(minimum_fill >= 0 && minimum_fill <= view.byte_length());

        // Assert: the remainder after dividing minimumFill by elementSize is 0.
        assert_eq!(minimum_fill % element_size, 0);

        // Let byteOffset be view.[[ByteOffset]].
        let byte_offset = view.get_byte_offset(cx);

        // Let byteLength be view.[[ByteLength]].
        let byte_length = view.byte_length();

        // Let bufferResult be TransferArrayBuffer(view.[[ViewedArrayBuffer]]).
        let buffer_result = view.transfer_array_buffer(cx);

        // If bufferResult is an abrupt completion,
        if buffer_result.is_none() {
            // Perform readIntoRequest’s error steps, given bufferResult.[[Value]].
            unsafe {
                rooted!(in(*cx) let mut rval = UndefinedValue());
                assert!(JS_GetPendingException(*cx, rval.handle_mut()));
                JS_ClearPendingException(*cx);

                read_into_request.error_steps(rval.handle());
            };

            // Return.
            return;
        }

        // Let buffer be bufferResult.[[Value]].
        let array_type = buffer_result
            .unwrap()
            .get_typed_array()
            .expect("can get typed array");

        let buffer = HeapBufferSource::<ArrayBufferU8>::new(BufferSource::ArrayBuffer(
            Heap::boxed(unsafe { *array_type.underlying_object() }),
        ));

        let buffer_byte_length = buffer.byte_length();
        // Let pullIntoDescriptor be a new pull-into descriptor with
        // buffer   buffer
        // buffer byte length   buffer.[[ArrayBufferByteLength]]
        // byte offset  byteOffset
        // byte length  byteLength
        // bytes filled  0
        // minimum fill minimumFill
        // element size elementSize
        // view constructor ctor
        // reader type  "byob"

        let pull_into_descriptor = PullIntoDescriptor {
            buffer: RefCell::new(buffer),
            buffer_byte_length,
            byte_offset,
            byte_length,
            bytes_filled: RefCell::new(0),
            minimum_fill: minimum_fill as usize,
            element_size,
            view_constructor: ctor.clone(),
            reader_type: Some(ReaderType::BYOB),
        };

        // If controller.[[pendingPullIntos]] is not empty,
        let pending_pull_intos = self.pending_pull_intos.borrow();
        if !pending_pull_intos.is_empty() {
            // Append pullIntoDescriptor to controller.[[pendingPullIntos]].
            self.pending_pull_intos
                .borrow_mut()
                .push(pull_into_descriptor);

            // Perform ! ReadableStreamAddReadIntoRequest(stream, readIntoRequest).
            stream.add_read_into_request(read_into_request);

            // Return.
            return;
        }

        // If stream.[[state]] is "closed",
        if stream.is_closed() {
            // Let emptyView be ! Construct(ctor, « pullIntoDescriptor’s buffer, pullIntoDescriptor’s byte offset, 0 »).
            let empty_view = create_buffer_source_with_constructor(
                cx,
                ctor,
                &pull_into_descriptor.buffer.borrow(),
                pull_into_descriptor.byte_offset,
                0,
            );

            // Perform readIntoRequest’s close steps, given emptyView.
            let result = RootedTraceableBox::new(Heap::default());
            unsafe {
                result.set(ObjectValue(
                    *empty_view.get_typed_array().unwrap().underlying_object(),
                ));
            }
            read_into_request.close_steps(Some(result));

            // Return.
            return;
        }

        // If controller.[[queueTotalSize]] > 0,
        if self.queue_total_size.get() > 0.0 {
            // If ! ReadableByteStreamControllerFillPullIntoDescriptorFromQueue(controller, pullIntoDescriptor) is true,
            if self.fill_pull_into_descriptor_from_queue(&pull_into_descriptor) {
                // Let filledView be ! ReadableByteStreamControllerConvertPullIntoDescriptor(pullIntoDescriptor).
                let filled_view = self.convert_pull_into_descriptor(&pull_into_descriptor);

                // Perform ! ReadableByteStreamControllerHandleQueueDrain(controller).
                self.handle_queue_drain(can_gc);

                // Perform readIntoRequest’s chunk steps, given filledView.
                let result = RootedTraceableBox::new(Heap::default());
                unsafe {
                    result.set(ObjectValue(
                        *filled_view.get_typed_array().unwrap().underlying_object(),
                    ));
                }
                read_into_request.chunk_steps(result);

                // Return.
                return;
            }

            // If controller.[[closeRequested]] is true,
            if self.close_requested.get() {
                // Let e be a new TypeError exception.
                rooted!(in(*cx) let mut error = UndefinedValue());
                unsafe {
                    Error::Type("close requested".to_owned()).to_jsval(
                        *cx,
                        &self.global(),
                        error.handle_mut(),
                    )
                };

                // Perform ! ReadableByteStreamControllerError(controller, e).
                self.error(error.handle());

                // Perform readIntoRequest’s error steps, given e.
                read_into_request.error_steps(error.handle());

                // Return.
                return;
            }
        }

        // Append pullIntoDescriptor to controller.[[pendingPullIntos]].
        self.pending_pull_intos
            .borrow_mut()
            .push(pull_into_descriptor);

        // Perform ! ReadableStreamAddReadIntoRequest(stream, readIntoRequest).
        stream.add_read_into_request(read_into_request);

        // Perform ! ReadableByteStreamControllerCallPullIfNeeded(controller).
        self.call_pull_if_needed(can_gc);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond>
    pub(crate) fn respond(&self, bytes_written: u64) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        // Assert: controller.[[pendingPullIntos]] is not empty.
        let pending_pull_intos = self.pending_pull_intos.borrow();
        assert!(!pending_pull_intos.is_empty());

        // Let firstDescriptor be controller.[[pendingPullIntos]][0].
        let first_descriptor = pending_pull_intos.first().unwrap();

        // Let state be controller.[[stream]].[[state]].
        let stream = self.stream.get().unwrap();

        // If state is "closed",
        // If bytesWritten is not 0, throw a TypeError exception.
        if stream.is_closed() && bytes_written != 0 {
            return Err(Error::Type("bytesWritten is not 0".to_owned()));
        } else {
            // Assert: state is "readable".
            assert!(stream.is_readable());

            // If bytesWritten is 0, throw a TypeError exception.
            if bytes_written == 0 {
                return Err(Error::Type("bytesWritten is 0".to_owned()));
            }

            // If firstDescriptor’s bytes filled + bytesWritten > firstDescriptor’s byte length, throw a RangeError exception.
            if first_descriptor.bytes_filled() + bytes_written as usize >
                first_descriptor.byte_length
            {
                return Err(Error::Range(
                    "bytes filled + bytesWritten > byte length".to_owned(),
                ));
            }

            // Set firstDescriptor’s buffer to ! TransferArrayBuffer(firstDescriptor’s buffer).
            first_descriptor.set_buffer(
                first_descriptor
                    .buffer
                    .borrow()
                    .transfer_array_buffer(cx)
                    .ok_or(Error::Type("can't transfer array buffer".to_owned()))?,
            );

            // Perform ? ReadableByteStreamControllerRespondInternal(controller, bytesWritten).
            self.respond_internal(bytes_written);
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond-internal>
    pub(crate) fn respond_internal(&self, bytes_written: u64) {
        let cx = GlobalScope::get_cx();
        // Let firstDescriptor be controller.[[pendingPullIntos]][0].
        let pending_pull_intos = self.pending_pull_intos.borrow();
        let first_descriptor = pending_pull_intos.first().unwrap();

        // Assert: ! CanTransferArrayBuffer(firstDescriptor’s buffer) is true
        assert!(first_descriptor
            .buffer
            .borrow()
            .can_transfer_array_buffer(cx));

        // Perform ! ReadableByteStreamControllerInvalidateBYOBRequest(controller).
        self.invalidate_byob_request();

        // Let state be controller.[[stream]].[[state]].
        let stream = self.stream.get().unwrap();

        // If state is "closed",
        if stream.is_closed() {
            // Assert: bytesWritten is 0.
            assert_eq!(bytes_written, 0);

            // Perform ! ReadableByteStreamControllerRespondInClosedState(controller, firstDescriptor).
            self.respond_in_closed_state(first_descriptor);
        } else {
            // Assert: state is "readable".
            assert!(stream.is_readable());

            // Assert: bytesWritten > 0.
            assert!(bytes_written > 0);

            // Perform ? ReadableByteStreamControllerRespondInReadableState(controller, bytesWritten, firstDescriptor).
            self.respond_in_readable_state(bytes_written, first_descriptor)
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond-in-closed-state>
    pub(crate) fn respond_in_closed_state(&self, first_descriptor: &PullIntoDescriptor) {
        // Assert: the remainder after dividing firstDescriptor’s bytes filled by firstDescriptor’s element size is 0.
        assert_eq!(
            first_descriptor.bytes_filled() % first_descriptor.element_size,
            0
        );

        // If firstDescriptor’s reader type is "none", perform ! ReadableByteStreamControllerShiftPendingPullInto(controller).
        if first_descriptor.reader_type.is_none() {
            self.shift_pending_pull_into();
        }

        // Let stream be controller.[[stream]].
        let stream = self.stream.get().unwrap();

        // If ! ReadableStreamHasBYOBReader(stream) is true,
        if stream.has_byob_reader() {
            // Let filledPullIntos be a new empty list.
            let mut filled_pull_intos = Vec::new();

            // While filledPullIntos’s size < ! ReadableStreamGetNumReadIntoRequests(stream),
            while filled_pull_intos.len() < stream.get_num_read_into_requests() {
                // Let pullIntoDescriptor be ! ReadableByteStreamControllerShiftPendingPullInto(controller).
                let pull_into_descriptor = self.shift_pending_pull_into();

                // Append pullIntoDescriptor to filledPullIntos.
                filled_pull_intos.push(pull_into_descriptor);
            }

            // For each filledPullInto of filledPullIntos,
            for filled_pull_into in filled_pull_intos {
                // Perform ! ReadableByteStreamControllerCommitPullIntoDescriptor(stream, filledPullInto).
                self.commit_pull_into_descriptor(&filled_pull_into);
            }
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond-in-readable-state>
    pub(crate) fn respond_in_readable_state(
        &self,
        bytes_written: u64,
        first_descriptor: &PullIntoDescriptor,
    ) {
        // Assert: pullIntoDescriptor’s bytes filled + bytesWritten ≤ pullIntoDescriptor’s byte length.
        assert!(
            first_descriptor.bytes_filled() + bytes_written as usize <=
                first_descriptor.byte_length
        );

        // Perform ! ReadableByteStreamControllerFillHeadPullIntoDescriptor(controller, bytesWritten, pullIntoDescriptor).
        self.fill_head_pull_into_descriptor(bytes_written as usize, first_descriptor);

        // If pullIntoDescriptor’s reader type is "none",
        if first_descriptor.reader_type.is_none() {
            // Perform ? ReadableByteStreamControllerEnqueueDetachedPullIntoToQueue(controller, pullIntoDescriptor).
            self.enqueue_detached_pull_into_to_queue(first_descriptor);

            // Let filledPullIntos be the result of performing ! ReadableByteStreamControllerProcessPullIntoDescriptorsUsingQueue(controller).
            let filled_pull_intos = self.process_pull_into_descriptors_using_queue();

            // For each filledPullInto of filledPullIntos,
            for filled_pull_into in filled_pull_intos {
                // Perform ! ReadableByteStreamControllerCommitPullIntoDescriptor(controller.[[stream]], filledPullInto).
                self.commit_pull_into_descriptor(&filled_pull_into);
            }

            // Return.
            return;
        }

        // If pullIntoDescriptor’s bytes filled < pullIntoDescriptor’s minimum fill, return.
        if first_descriptor.bytes_filled() < first_descriptor.minimum_fill {
            return;
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond-with-new-view>
    pub(crate) fn respond_with_new_view(
        &self,
        view: HeapBufferSource<ArrayBufferViewU8>,
    ) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        // Assert: controller.[[pendingPullIntos]] is not empty.
        let pending_pull_intos = self.pending_pull_intos.borrow();
        assert!(!pending_pull_intos.is_empty());

        // Assert: ! IsDetachedBuffer(view.[[ViewedArrayBuffer]]) is false.
        assert!(!view.is_detached_buffer(cx));

        // Let firstDescriptor be controller.[[pendingPullIntos]][0].
        let first_descriptor = pending_pull_intos.first().unwrap();

        // Let state be controller.[[stream]].[[state]].
        let stream = self.stream.get().unwrap();

        // If state is "closed",
        if stream.is_closed() {
            // If view.[[ByteLength]] is not 0, throw a TypeError exception.
            if view.byte_length() != 0 {
                return Err(Error::Type("view byte length is not 0".to_owned()));
            }
        } else {
            // Assert: state is "readable".
            assert!(stream.is_readable());

            // If view.[[ByteLength]] is 0, throw a TypeError exception.
            if view.byte_length() == 0 {
                return Err(Error::Type("view byte length is 0".to_owned()));
            }
        }

        // If firstDescriptor’s byte offset + firstDescriptor’ bytes filled is not view.[[ByteOffset]], throw a RangeError exception.
        if first_descriptor.byte_offset + first_descriptor.bytes_filled() !=
            view.get_byte_offset(cx)
        {
            return Err(Error::Range(
                "firstDescriptor's byte offset + bytes filled is not view byte offset".to_owned(),
            ));
        }

        // If firstDescriptor’s buffer byte length is not view.[[ViewedArrayBuffer]].[[ByteLength]], throw a RangeError exception.
        if first_descriptor.buffer.borrow().byte_length() !=
            view.viewed_buffer_array_byte_length(cx)
        {
            return Err(Error::Range(
                "firstDescriptor's buffer byte length is not view viewed buffer array byte length"
                    .to_owned(),
            ));
        }

        // If firstDescriptor’s bytes filled + view.[[ByteLength]] > firstDescriptor’s byte length, throw a RangeError exception.
        if first_descriptor.bytes_filled() + view.byte_length() > first_descriptor.byte_length {
            return Err(Error::Range(
                "bytes filled + view byte length > byte length".to_owned(),
            ));
        }

        // Let viewByteLength be view.[[ByteLength]].
        let view_byte_length = view.byte_length();

        // Set firstDescriptor’s buffer to ? TransferArrayBuffer(view.[[ViewedArrayBuffer]]).
        first_descriptor.set_buffer(
            view.transfer_array_buffer(cx)
                .ok_or(Error::Type("can't transfer array buffer".to_owned()))?,
        );

        // Perform ? ReadableByteStreamControllerRespondInternal(controller, viewByteLength).
        self.respond_internal(view_byte_length as u64);

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-get-desired-size>
    pub(crate) fn get_desired_size(&self) -> Option<f64> {
        // Let state be controller.[[stream]].[[state]].
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
        Some(self.strategy_hwm - self.queue_total_size.get())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollergetbyobrequest>
    #[allow(unsafe_code)]
    pub(crate) fn get_byob_request(
        &self,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<ReadableStreamBYOBRequest>>> {
        let cx = GlobalScope::get_cx();

        // If controller.[[byobRequest]] is null and controller.[[pendingPullIntos]] is not empty,
        let pending_pull_intos = self.pending_pull_intos.borrow();
        if self.byob_request.get().is_none() && !pending_pull_intos.is_empty() {
            // Let firstDescriptor be controller.[[pendingPullIntos]][0].
            let first_descriptor = pending_pull_intos.first().unwrap();
            // Let view be ! Construct(%Uint8Array%, « firstDescriptor’s buffer,
            // firstDescriptor’s byte offset + firstDescriptor’s bytes filled,
            // firstDescriptor’s byte length − firstDescriptor’s bytes filled »).
            let buffer = first_descriptor
                .buffer
                .borrow()
                .get_typed_array()
                .map_err(|_| Error::NotFound)?;

            let bytes_filled = first_descriptor.bytes_filled();
            let byte_offset = first_descriptor.byte_offset + bytes_filled;
            let byte_length = first_descriptor.byte_length - bytes_filled;

            let uint8_array =
                create_uint8_array_with_buffer(cx, buffer, byte_offset, byte_length as i64)
                    .map_err(|_| Error::NotFound)?;

            // Let byobRequest be a new ReadableStreamBYOBRequest.
            let byob_request = ReadableStreamBYOBRequest::new(&self.global(), can_gc);

            // Set byobRequest.[[controller]] to controller.
            byob_request.set_controller(Some(&DomRoot::from_ref(self)));

            // Set byobRequest.[[view]] to view.
            byob_request.set_view(Some(Heap::boxed(unsafe {
                *uint8_array.underlying_object()
            })));

            // Set controller.[[byobRequest]] to byobRequest.
            self.byob_request.set(Some(&byob_request));
        }

        // Return controller.[[byobRequest]].
        Ok(self.byob_request.get())
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-close>
    #[allow(unsafe_code)]
    pub(crate) fn close(&self) {
        let cx = GlobalScope::get_cx();
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().unwrap();

        // If controller.[[closeRequested]] is true or stream.[[state]] is not "readable", return.
        if self.close_requested.get() || !stream.is_readable() {
            return;
        }

        // If controller.[[queueTotalSize]] > 0,
        if self.queue_total_size.get() > 0.0 {
            // Set controller.[[closeRequested]] to true.
            self.close_requested.set(true);
            // Return.
            return;
        }

        // If controller.[[pendingPullIntos]] is not empty,
        let pending_pull_intos = self.pending_pull_intos.borrow();
        if !pending_pull_intos.is_empty() {
            // Let firstPendingPullInto be controller.[[pendingPullIntos]][0].
            let first_pending_pull_into = pending_pull_intos.first().unwrap();

            // If the remainder after dividing firstPendingPullInto’s bytes filled by
            // firstPendingPullInto’s element size is not 0,
            if first_pending_pull_into.bytes_filled() % first_pending_pull_into.element_size != 0 {
                // Let e be a new TypeError exception.
                let e = Error::Type(
                    "remainder after dividing firstPendingPullInto's bytes
                    filled by firstPendingPullInto's element size is not 0"
                        .to_owned(),
                );

                // Perform ! ReadableByteStreamControllerError(controller, e).
                rooted!(in(*cx) let mut error = UndefinedValue());
                unsafe { e.to_jsval(*cx, &self.global(), error.handle_mut()) };
                self.error(error.handle());

                // Throw e.
                // TODO: how to throw exception in this context?
            }
        }

        // Perform ! ReadableByteStreamControllerClearAlgorithms(controller).
        self.clear_algorithms();

        // Perform ! ReadableStreamClose(stream).
        stream.close();
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-error>
    pub(crate) fn error(&self, e: SafeHandleValue) {
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().unwrap();

        // If stream.[[state]] is not "readable", return.
        if !stream.is_readable() {
            return;
        }

        // Perform ! ReadableByteStreamControllerClearPendingPullIntos(controller).
        self.clear_pending_pull_intos();

        // Perform ! ResetQueue(controller).
        self.reset_queue();

        // Perform ! ReadableByteStreamControllerClearAlgorithms(controller).
        self.clear_algorithms();

        // Perform ! ReadableStreamError(stream, e).
        stream.error(e);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-clear-algorithms>
    fn clear_algorithms(&self) {
        // Set controller.[[pullAlgorithm]] to undefined.
        // Set controller.[[cancelAlgorithm]] to undefined.
        self.underlying_source.set(None);
    }

    /// <https://streams.spec.whatwg.org/#reset-queue>
    pub(crate) fn reset_queue(&self) {
        // Assert: container has [[queue]] and [[queueTotalSize]] internal slots.

        // Set container.[[queue]] to a new empty list.
        self.queue.borrow_mut().clear();

        // Set container.[[queueTotalSize]] to 0.
        self.queue_total_size.set(0.0);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-clear-pending-pull-intos>
    pub(crate) fn clear_pending_pull_intos(&self) {
        // Perform ! ReadableByteStreamControllerInvalidateBYOBRequest(controller).
        self.invalidate_byob_request();

        // Set controller.[[pendingPullIntos]] to a new empty list.
        self.pending_pull_intos.borrow_mut().clear();
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-invalidate-byob-request>
    pub(crate) fn invalidate_byob_request(&self) {
        // If controller.[[byobRequest]] is null, return.
        if self.byob_request.get().is_none() {
            return;
        }

        // Set controller.[[byobRequest]].[[controller]] to undefined.
        self.byob_request.get().unwrap().set_controller(None);

        // Set controller.[[byobRequest]].[[view]] to null.
        self.byob_request.get().unwrap().set_view(None);

        // Set controller.[[byobRequest]] to null.
        self.byob_request.set(None);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-enqueue>
    #[allow(unsafe_code)]
    pub(crate) fn enqueue(
        &self,
        chunk: HeapBufferSource<ArrayBufferViewU8>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        // Let stream be controller.[[stream]].
        let stream = self.stream.get().unwrap();

        // If controller.[[closeRequested]] is true or stream.[[state]] is not "readable", return.
        if self.close_requested.get() || !stream.is_readable() {
            return Ok(());
        }

        // Let buffer be chunk.[[ViewedArrayBuffer]].
        let buffer = chunk.get_array_buffer(cx);

        // Let byteOffset be chunk.[[ByteOffset]].
        let byte_offset = chunk.get_byte_offset(cx);

        // Let byteLength be chunk.[[ByteLength]].
        let byte_length = chunk.byte_length();

        // If ! IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if buffer.is_detached_buffer(cx) {
            return Err(Error::Type("buffer is detached".to_owned()));
        }

        // Let transferredBuffer be ? TransferArrayBuffer(buffer).
        let transferred_buffer = buffer
            .transfer_array_buffer(cx)
            .ok_or(Error::Type("can't transfer array buffer".to_owned()))?;

        // If controller.[[pendingPullIntos]] is not empty,
        let pending_pull_intos = self.pending_pull_intos.borrow();
        if !pending_pull_intos.is_empty() {
            // Let firstPendingPullInto be controller.[[pendingPullIntos]][0].
            let first_descriptor = pending_pull_intos.first().unwrap();
            // If ! IsDetachedBuffer(firstPendingPullInto’s buffer) is true, throw a TypeError exception.
            if first_descriptor.buffer.borrow().is_detached_buffer(cx) {
                return Err(Error::Type("buffer is detached".to_owned()));
            }

            // Perform ! ReadableByteStreamControllerInvalidateBYOBRequest(controller).
            self.invalidate_byob_request();

            // Set firstPendingPullInto’s buffer to ! TransferArrayBuffer(firstPendingPullInto’s buffer).
            first_descriptor.set_buffer(
                first_descriptor
                    .buffer
                    .borrow()
                    .transfer_array_buffer(cx)
                    .ok_or(Error::Type("can't transfer array buffer".to_owned()))?,
            );

            // If firstPendingPullInto’s reader type is "none",
            // perform ? ReadableByteStreamControllerEnqueueDetachedPullIntoToQueue(controller, firstPendingPullInto).
            if first_descriptor.reader_type.is_none() {
                self.enqueue_detached_pull_into_to_queue(first_descriptor);
            }
        }

        // If ! ReadableStreamHasDefaultReader(stream) is true,
        if stream.has_default_reader() {
            // Perform ! ReadableByteStreamControllerProcessReadRequestsUsingQueue(controller).
            self.process_read_requests_using_queue(can_gc);

            // If ! ReadableStreamGetNumReadRequests(stream) is 0,
            if stream.get_num_read_requests() == 0 {
                // Assert: controller.[[pendingPullIntos]] is empty.
                assert!(self.pending_pull_intos.borrow().is_empty());

                // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue(
                // controller, transferredBuffer, byteOffset, byteLength).
                self.enqueue_chunk_to_queue(transferred_buffer, byte_offset, byte_length);
            } else {
                // Assert: controller.[[queue]] is empty.
                assert!(self.queue.borrow().is_empty());

                // If controller.[[pendingPullIntos]] is not empty,
                let pending_pull_intos = self.pending_pull_intos.borrow();
                if !pending_pull_intos.is_empty() {
                    // Assert: controller.[[pendingPullIntos]][0]'s reader type is "default".
                    assert!(matches!(
                        pending_pull_intos.first().unwrap().reader_type,
                        Some(ReaderType::Default)
                    ));

                    // Perform ! ReadableByteStreamControllerShiftPendingPullInto(controller).
                    self.shift_pending_pull_into();
                }

                // Let transferredView be ! Construct(%Uint8Array%, « transferredBuffer, byteOffset, byteLength »).
                let buffer = transferred_buffer
                    .get_typed_array()
                    .map_err(|_| Error::NotFound)?;

                let transferred_view =
                    create_uint8_array_with_buffer(cx, buffer, byte_offset, byte_length as i64)
                        .map_err(|_| Error::NotFound)?;

                // Perform ! ReadableStreamFulfillReadRequest(stream, transferredView, false).
                rooted!(in(*cx) let view_val =  ObjectValue(*unsafe { transferred_view.underlying_object() }));
                stream.fulfill_read_request(view_val.handle(), false);
            }
            // Otherwise, if ! ReadableStreamHasBYOBReader(stream) is true,
        } else if stream.has_byob_reader() {
            // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue(
            // controller, transferredBuffer, byteOffset, byteLength).
            self.enqueue_chunk_to_queue(transferred_buffer, byte_offset, byte_length);

            // Let filledPullIntos be the result of performing !
            // ReadableByteStreamControllerProcessPullIntoDescriptorsUsingQueue(controller).
            let filled_pull_intos = self.process_pull_into_descriptors_using_queue();

            // For each filledPullInto of filledPullIntos,
            // Perform ! ReadableByteStreamControllerCommitPullIntoDescriptor(stream, filledPullInto).
            for filled_pull_into in filled_pull_intos {
                self.commit_pull_into_descriptor(&filled_pull_into);
            }
        } else {
            // Assert: ! IsReadableStreamLocked(stream) is false.
            assert!(!stream.is_locked());

            // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue
            // (controller, transferredBuffer, byteOffset, byteLength).
            self.enqueue_chunk_to_queue(transferred_buffer, byte_offset, byte_length);
        }

        // Perform ! ReadableByteStreamControllerCallPullIfNeeded(controller).
        self.call_pull_if_needed(can_gc);

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-commit-pull-into-descriptor>
    #[allow(unsafe_code)]
    pub(crate) fn commit_pull_into_descriptor(&self, pull_into_descriptor: &PullIntoDescriptor) {
        let cx = GlobalScope::get_cx();
        // Assert: stream.[[state]] is not "errored".
        let stream = self.stream.get().unwrap();
        assert!(!stream.is_errored());

        // Assert: pullIntoDescriptor.reader type is not "none".
        assert!(pull_into_descriptor.reader_type.is_some());

        // Let done be false.
        let mut done = false;

        // If stream.[[state]] is "closed",
        if stream.is_closed() {
            // Assert: the remainder after dividing pullIntoDescriptor’s bytes filled
            // by pullIntoDescriptor’s element size is 0.
            assert!(pull_into_descriptor.bytes_filled() % pull_into_descriptor.element_size == 0);

            // Set done to true.
            done = true;
        }

        // Let filledView be ! ReadableByteStreamControllerConvertPullIntoDescriptor(pullIntoDescriptor).
        let filled_view = self.convert_pull_into_descriptor(pull_into_descriptor);
        rooted!(in(*cx) let view_val =  ObjectValue(*unsafe {
             filled_view.get_typed_array()
             .expect("can get typed array")
             .underlying_object()
        }));

        // If pullIntoDescriptor’s reader type is "default",
        if matches!(pull_into_descriptor.reader_type, Some(ReaderType::Default)) {
            // Perform ! ReadableStreamFulfillReadRequest(stream, filledView, done).

            stream.fulfill_read_request(view_val.handle(), done);
        } else {
            // Assert: pullIntoDescriptor’s reader type is "byob".
            assert!(matches!(
                pull_into_descriptor.reader_type,
                Some(ReaderType::BYOB)
            ));

            // Perform ! ReadableStreamFulfillReadIntoRequest(stream, filledView, done).
            stream.fulfill_read_into_request(view_val.handle(), done);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-convert-pull-into-descriptor>
    #[allow(unsafe_code)]
    pub(crate) fn convert_pull_into_descriptor(
        &self,
        pull_into_descriptor: &PullIntoDescriptor,
    ) -> HeapBufferSource<ArrayBufferViewU8> {
        // Let bytesFilled be pullIntoDescriptor’s bytes filled.
        let bytes_filled = pull_into_descriptor.bytes_filled();

        // Let elementSize be pullIntoDescriptor’s element size.
        let element_size = pull_into_descriptor.element_size;

        // Assert: bytesFilled ≤ pullIntoDescriptor’s byte length.
        assert!(bytes_filled <= pull_into_descriptor.byte_length);

        //Assert: the remainder after dividing bytesFilled by elementSize is 0.
        assert!(bytes_filled % element_size == 0);

        // Let buffer be ! TransferArrayBuffer(pullIntoDescriptor’s buffer).
        let buffer = pull_into_descriptor
            .buffer
            .borrow()
            .transfer_array_buffer(GlobalScope::get_cx())
            .expect("can transfer array buffer");

        // Return ! Construct(pullIntoDescriptor’s view constructor,
        // « buffer, pullIntoDescriptor’s byte offset, bytesFilled ÷ elementSize »).
        let uint8_array = create_uint8_array_with_buffer(
            GlobalScope::get_cx(),
            buffer
                .get_typed_array()
                .expect("buffer must be a typed array"),
            pull_into_descriptor.byte_offset,
            (bytes_filled / element_size) as i64,
        )
        .expect("view must be a typed array");

        HeapBufferSource::<ArrayBufferViewU8>::new(BufferSource::ArrayBufferView(Heap::boxed(
            unsafe { *uint8_array.underlying_object() },
        )))
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-process-pull-into-descriptors-using-queue>
    pub(crate) fn process_pull_into_descriptors_using_queue(&self) -> Vec<PullIntoDescriptor> {
        // Assert: controller.[[closeRequested]] is false.
        assert!(!self.close_requested.get());

        // Let filledPullIntos be a new empty list.
        let mut filled_pull_intos = Vec::new();

        // While controller.[[pendingPullIntos]] is not empty,
        let pending_pull_intos = self.pending_pull_intos.borrow();
        while !pending_pull_intos.is_empty() {
            // If controller.[[queueTotalSize]] is 0, then break.
            if self.queue_total_size.get() == 0.0 {
                break;
            }

            // Let pullIntoDescriptor be controller.[[pendingPullIntos]][0].
            let pull_into_descriptor = pending_pull_intos.first().unwrap();

            // If ! ReadableByteStreamControllerFillPullIntoDescriptorFromQueue(controller, pullIntoDescriptor) is true,
            if !self.fill_pull_into_descriptor_from_queue(pull_into_descriptor) {
                // Perform ! ReadableByteStreamControllerShiftPendingPullInto(controller).
                let pull_into_descriptor = self.shift_pending_pull_into();

                // Append pullIntoDescriptor to filledPullIntos.
                filled_pull_intos.push(pull_into_descriptor);
            }
        }

        // Return filledPullIntos.
        filled_pull_intos
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-fill-pull-into-descriptor-from-queue>
    pub(crate) fn fill_pull_into_descriptor_from_queue(
        &self,
        pull_into_descriptor: &PullIntoDescriptor,
    ) -> bool {
        let cx = GlobalScope::get_cx();
        // Let maxBytesToCopy be min(controller.[[queueTotalSize]],
        // pullIntoDescriptor’s byte length − pullIntoDescriptor’s bytes filled).
        let max_bytes_to_copy = self.queue_total_size.get().min(
            pull_into_descriptor.byte_length as f64 - pull_into_descriptor.bytes_filled() as f64,
        );

        // Let maxBytesFilled be pullIntoDescriptor’s bytes filled + maxBytesToCopy.
        let max_bytes_filled = pull_into_descriptor.bytes_filled() + max_bytes_to_copy as usize;

        // Let totalBytesToCopyRemaining be maxBytesToCopy.
        let mut total_bytes_to_copy_remaining = max_bytes_to_copy;

        // Let ready be false.
        let mut ready = false;

        // Assert: ! IsDetachedBuffer(pullIntoDescriptor’s buffer) is false.
        assert!(!pull_into_descriptor
            .buffer
            .borrow()
            .is_detached_buffer(GlobalScope::get_cx()));

        // Assert: pullIntoDescriptor’s bytes filled < pullIntoDescriptor’s minimum fill.
        assert!(pull_into_descriptor.bytes_filled() < pull_into_descriptor.minimum_fill);

        // Let remainderBytes be the remainder after dividing maxBytesFilled by pullIntoDescriptor’s element size.
        let remainder_bytes = max_bytes_filled % pull_into_descriptor.element_size;

        // Let maxAlignedBytes be maxBytesFilled − remainderBytes.
        let max_aligned_bytes = max_bytes_filled - remainder_bytes;

        // If maxAlignedBytes ≥ pullIntoDescriptor’s minimum fill,
        if max_aligned_bytes >= pull_into_descriptor.minimum_fill {
            // Set totalBytesToCopyRemaining to maxAlignedBytes − pullIntoDescriptor’s bytes filled.
            total_bytes_to_copy_remaining =
                (max_aligned_bytes - pull_into_descriptor.bytes_filled()) as f64;

            // Set ready to true.
            ready = true;
        }

        // Let queue be controller.[[queue]].
        let mut queue = self.queue.borrow_mut();

        // While totalBytesToCopyRemaining > 0,
        while total_bytes_to_copy_remaining > 0.0 {
            // Let headOfQueue be queue[0].
            let head_of_queue = queue.front().unwrap();

            // Let bytesToCopy be min(totalBytesToCopyRemaining, headOfQueue’s byte length).
            let bytes_to_copy =
                total_bytes_to_copy_remaining.min(head_of_queue.byte_length() as f64);

            // Let destStart be pullIntoDescriptor’s byte offset + pullIntoDescriptor’s bytes filled.
            let dest_start = pull_into_descriptor.byte_offset + pull_into_descriptor.bytes_filled();

            // Let descriptorBuffer be pullIntoDescriptor’s buffer.
            let descriptor_buffer = &pull_into_descriptor.buffer;

            // Let queueBuffer be headOfQueue’s buffer.
            let queue_buffer = &head_of_queue.buffer;

            // Let queueByteOffset be headOfQueue’s byte offset.
            let queue_byte_offset = head_of_queue.byte_offset();

            // Assert: ! CanCopyDataBlockBytes(descriptorBuffer, destStart,
            // queueBuffer, queueByteOffset, bytesToCopy) is true.
            assert!(descriptor_buffer.borrow().can_copy_data_block_bytes(
                cx,
                dest_start,
                queue_buffer,
                queue_byte_offset,
                bytes_to_copy
            ));

            // Perform ! CopyDataBlockBytes(descriptorBuffer.[[ArrayBufferData]], destStart,
            // queueBuffer.[[ArrayBufferData]], queueByteOffset, bytesToCopy).
            descriptor_buffer.borrow().copy_data_block_bytes(
                cx,
                dest_start,
                queue_buffer,
                queue_byte_offset,
                bytes_to_copy,
            );

            // If headOfQueue’s byte length is bytesToCopy,
            if head_of_queue.byte_length() as f64 == bytes_to_copy {
                // Remove queue[0].
                queue.pop_front().unwrap();
            } else {
                // Set headOfQueue’s byte offset to headOfQueue’s byte offset + bytesToCopy.
                head_of_queue.set_byte_offset(head_of_queue.byte_offset() + bytes_to_copy as usize);
                // Set headOfQueue’s byte length to headOfQueue’s byte length − bytesToCopy.
                head_of_queue.set_byte_length(head_of_queue.byte_length() - bytes_to_copy as usize);
            }

            // Set controller.[[queueTotalSize]] to controller.[[queueTotalSize]] − bytesToCopy.
            self.queue_total_size
                .set(self.queue_total_size.get() - bytes_to_copy);

            // Perform ! ReadableByteStreamControllerFillHeadPullIntoDescriptor(
            // controller, bytesToCopy, pullIntoDescriptor).
            self.fill_head_pull_into_descriptor(bytes_to_copy as usize, pull_into_descriptor);

            // Set totalBytesToCopyRemaining to totalBytesToCopyRemaining − bytesToCopy.
            total_bytes_to_copy_remaining -= bytes_to_copy;
        }

        // If ready is false,
        if !ready {
            // Assert: controller.[[queueTotalSize]] is 0.
            assert!(self.queue_total_size.get() == 0.0);

            // Assert: pullIntoDescriptor’s bytes filled > 0.
            assert!(pull_into_descriptor.bytes_filled() > 0);

            // Assert: pullIntoDescriptor’s bytes filled < pullIntoDescriptor’s minimum fill.
            assert!(pull_into_descriptor.bytes_filled() < pull_into_descriptor.minimum_fill);
        }

        // Return ready.
        ready
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-fill-head-pull-into-descriptor>
    pub(crate) fn fill_head_pull_into_descriptor(
        &self,
        bytes_copied: usize,
        pull_into_descriptor: &PullIntoDescriptor,
    ) {
        // Assert: either controller.[[pendingPullIntos]] is empty,
        // or controller.[[pendingPullIntos]][0] is pullIntoDescriptor.
        assert!(
            self.pending_pull_intos.borrow().is_empty() ||
                self.pending_pull_intos.borrow().first().unwrap() == pull_into_descriptor
        );

        // Assert: controller.[[byobRequest]] is null.
        assert!(self.byob_request.get().is_none());

        // Set pullIntoDescriptor’s bytes filled to bytes filled + size.
        pull_into_descriptor.set_bytes_filled(pull_into_descriptor.bytes_filled() + bytes_copied);
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerenqueuedetachedpullintotoqueue>
    pub(crate) fn enqueue_detached_pull_into_to_queue(
        &self,
        first_descriptor: &PullIntoDescriptor,
    ) {
        // Assert: pullIntoDescriptor’s reader type is "none".
        assert!(first_descriptor.reader_type.is_none());

        // If pullIntoDescriptor’s bytes filled > 0, perform ?
        // ReadableByteStreamControllerEnqueueClonedChunkToQueue(controller,
        // pullIntoDescriptor’s buffer, pullIntoDescriptor’s byte offset, pullIntoDescriptor’s bytes filled).

        if first_descriptor.bytes_filled() > 0 {
            self.enqueue_cloned_chunk_to_queue(
                &first_descriptor.buffer.borrow(),
                first_descriptor.byte_offset,
                first_descriptor.bytes_filled(),
            );
        }

        // Perform ! ReadableByteStreamControllerShiftPendingPullInto(controller).
        self.shift_pending_pull_into();
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerenqueueclonedchunktoqueue>
    #[allow(unsafe_code)]
    pub(crate) fn enqueue_cloned_chunk_to_queue(
        &self,
        buffer: &HeapBufferSource<ArrayBufferU8>,
        byte_offset: usize,
        byte_length: usize,
    ) {
        let cx = GlobalScope::get_cx();
        // Let cloneResult be CloneArrayBuffer(buffer, byteOffset, byteLength, %ArrayBuffer%).
        let clone_result =
            buffer.clone_array_buffer(GlobalScope::get_cx(), byte_offset, byte_length);

        // If cloneResult is an abrupt completion,
        if clone_result.is_none() {
            // Perform ! ReadableByteStreamControllerError(controller, cloneResult.[[Value]]).
            unsafe {
                rooted!(in(*cx) let mut rval = UndefinedValue());
                assert!(JS_GetPendingException(*cx, rval.handle_mut()));
                JS_ClearPendingException(*cx);

                self.error(rval.handle());
            };

            // Return cloneResult.
            return;
        } else {
            // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue
            // (controller, cloneResult.[[Value]], 0, byteLength).
            self.enqueue_chunk_to_queue(clone_result.unwrap(), 0, byte_length);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-enqueue-chunk-to-queue>
    pub(crate) fn enqueue_chunk_to_queue(
        &self,
        buffer: HeapBufferSource<ArrayBufferU8>,
        byte_offset: usize,
        byte_length: usize,
    ) {
        // Let entry be a new ReadableByteStreamQueueEntry object.
        let entry = QueueEntry::new(buffer, byte_offset, byte_length);

        // Append entry to controller.[[queue]].
        self.queue.borrow_mut().push_back(entry);

        // Set controller.[[queueTotalSize]] to controller.[[queueTotalSize]] + byteLength.
        self.queue_total_size
            .set(self.queue_total_size.get() + byte_length as f64);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-shift-pending-pull-into>
    pub(crate) fn shift_pending_pull_into(&self) -> PullIntoDescriptor {
        // Assert: controller.[[byobRequest]] is null.
        assert!(self.byob_request.get().is_none());

        // Let descriptor be controller.[[pendingPullIntos]][0].
        // Remove descriptor from controller.[[pendingPullIntos]].
        let descriptor = self.pending_pull_intos.borrow_mut().remove(0);

        // Return descriptor.
        descriptor
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerprocessreadrequestsusingqueue>
    pub(crate) fn process_read_requests_using_queue(&self, can_gc: CanGc) {
        // Let reader be controller.[[stream]].[[reader]].
        // Assert: reader implements ReadableStreamDefaultReader.
        let reader = self.stream.get().unwrap().get_default_reader();

        // Step 3
        reader.process_read_requests(DomRoot::from_ref(self), can_gc);
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerfillreadrequestfromqueue>
    #[allow(unsafe_code)]
    pub(crate) fn fill_read_request_from_queue(&self, read_request: &ReadRequest, can_gc: CanGc) {
        // Assert: controller.[[queueTotalSize]] > 0.
        assert!(self.queue_total_size.get() > 0.0);
        // Also assert that the queue has a non-zero length;
        assert!(!self.queue.borrow().is_empty());

        // Let entry be controller.[[queue]][0].
        // Remove entry from controller.[[queue]].
        let entry = self.remove_entry();

        // Set controller.[[queueTotalSize]] to controller.[[queueTotalSize]] − entry’s byte length.
        self.queue_total_size
            .set(self.queue_total_size.get() - entry.byte_length() as f64);

        // Perform ! ReadableByteStreamControllerHandleQueueDrain(controller).
        self.handle_queue_drain(can_gc);

        // Let view be ! Construct(%Uint8Array%, « entry’s buffer, entry’s byte offset, entry’s byte length »).

        let view = create_uint8_array_with_buffer(
            GlobalScope::get_cx(),
            entry.buffer.get_typed_array().expect("can get typed array"),
            entry.byte_offset(),
            entry.byte_length() as i64,
        )
        .expect("can create uint8 array");

        // Perform readRequest’s chunk steps, given view.
        let result = RootedTraceableBox::new(Heap::default());
        unsafe {
            result.set(ObjectValue(*view.underlying_object()));
        }

        read_request.chunk_steps(result);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-handle-queue-drain>
    pub(crate) fn handle_queue_drain(&self, can_gc: CanGc) {
        // Assert: controller.[[stream]].[[state]] is "readable".
        assert!(self.stream.get().unwrap().is_readable());

        // If controller.[[queueTotalSize]] is 0 and controller.[[closeRequested]] is true,
        if self.queue_total_size.get() == 0.0 && self.close_requested.get() {
            // Perform ! ReadableByteStreamControllerClearAlgorithms(controller).
            self.clear_algorithms();

            // Perform ! ReadableStreamClose(controller.[[stream]]).
            self.stream.get().unwrap().close();
        } else {
            // Perform ! ReadableByteStreamControllerCallPullIfNeeded(controller).
            self.call_pull_if_needed(can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
    #[allow(unsafe_code)]
    pub(crate) fn call_pull_if_needed(&self, can_gc: CanGc) {
        // Let shouldPull be ! ReadableByteStreamControllerShouldCallPull(controller).
        // If shouldPull is false, return.
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
            Controller::ReadableByteStreamController(rooted_default_controller.clone());

        let Some(underlying_source) = self.underlying_source.get() else {
            return;
        };
        let handler = PromiseNativeHandler::new(
            &global,
            Some(Box::new(PullAlgorithmFulfillmentHandler {
                controller: Dom::from_ref(&rooted_default_controller),
            })),
            Some(Box::new(PullAlgorithmRejectionHandler {
                controller: Dom::from_ref(&rooted_default_controller),
            })),
        );

        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        let result = underlying_source
            .call_pull_algorithm(controller, can_gc)
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

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-should-call-pull>
    fn should_call_pull(&self) -> bool {
        // Let stream be controller.[[stream]].
        // Note: the spec does not assert that stream is not undefined here,
        // so we return false if it is.
        let Some(stream) = self.stream.get() else {
            debug!("`should_call_pull` called on a controller without a stream.");
            return false;
        };

        // If stream.[[state]] is not "readable", return false.
        if !stream.is_readable() {
            return false;
        }

        // If controller.[[closeRequested]] is true, return false.
        if self.close_requested.get() {
            return false;
        }

        // If controller.[[started]] is false, return false.
        if !self.started.get() {
            return false;
        }

        // If ! ReadableStreamHasDefaultReader(stream) is true and ! ReadableStreamGetNumReadRequests(stream) > 0, return true.
        if stream.has_default_reader() && stream.get_num_read_requests() > 0 {
            return true;
        }

        // If ! ReadableStreamHasBYOBReader(stream) is true and ! ReadableStreamGetNumReadIntoRequests(stream) > 0, return true.
        if stream.has_byob_reader() && stream.get_num_read_into_requests() > 0 {
            return true;
        }

        // Let desiredSize be ! ReadableByteStreamControllerGetDesiredSize(controller).
        let desired_size = self.get_desired_size();

        // Assert: desiredSize is not null.
        assert!(desired_size.is_some());

        // If desiredSize > 0, return true.
        if desired_size.unwrap() > 0. {
            return true;
        }

        // Return false.
        false
    }
    /// <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller>
    #[allow(unsafe_code)]
    pub(crate) fn setup(
        &self,
        global: &GlobalScope,
        stream: DomRoot<ReadableStream>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Assert: stream.[[controller]] is undefined.
        stream.assert_no_controller();

        // If autoAllocateChunkSize is not undefined,
        if let Some(_) = self.auto_allocate_chunk_size {
            // Assert: ! IsInteger(autoAllocateChunkSize) is true. Implicit
            // Assert: autoAllocateChunkSize is positive. (Implicit by type.)
        }

        // Set controller.[[stream]] to stream.
        self.stream.set(Some(&stream));

        // Set controller.[[pullAgain]] and controller.[[pulling]] to false.
        self.pull_again.set(false);
        self.pulling.set(false);

        // Set controller.[[byobRequest]] to null.
        self.byob_request.set(None);

        // Perform ! ResetQueue(controller).
        self.reset_queue();

        // Set controller.[[closeRequested]] and controller.[[started]] to false.
        self.close_requested.set(false);
        self.started.set(false);

        // Set controller.[[strategyHWM]] to highWaterMark.
        // Set controller.[[pullAlgorithm]] to pullAlgorithm.
        // Set controller.[[cancelAlgorithm]] to cancelAlgorithm.
        // Set controller.[[autoAllocateChunkSize]] to autoAllocateChunkSize.
        // Set controller.[[pendingPullIntos]] to a new empty list.
        // Note: the above steps are done in `new`.

        // Set stream.[[controller]] to controller.
        let rooted_byte_controller = DomRoot::from_ref(self);
        stream.set_byte_controller(&rooted_byte_controller);

        if let Some(underlying_source) = rooted_byte_controller.underlying_source.get() {
            // Let startResult be the result of performing startAlgorithm. (This might throw an exception.)
            let start_result = underlying_source
                .call_start_algorithm(
                    Controller::ReadableByteStreamController(rooted_byte_controller.clone()),
                    can_gc,
                )
                .unwrap_or_else(|| {
                    let promise = Promise::new(global, can_gc);
                    promise.resolve_native(&());
                    Ok(promise)
                });

            // Let startPromise be a promise resolved with startResult.
            let start_promise = start_result?;

            // Upon fulfillment of startPromise, Upon rejection of startPromise with reason r,
            let handler = PromiseNativeHandler::new(
                global,
                Some(Box::new(StartAlgorithmFulfillmentHandler {
                    controller: Dom::from_ref(&rooted_byte_controller),
                })),
                Some(Box::new(StartAlgorithmRejectionHandler {
                    controller: Dom::from_ref(&rooted_byte_controller),
                })),
            );
            let realm = enter_realm(global);
            let comp = InRealm::Entered(&realm);
            start_promise.append_native_handler(&handler, comp, can_gc);
        };

        Ok(())
    }

    /// Setting the JS object after the heap has settled down.
    pub(crate) fn set_underlying_source_this_object(&self, this_object: HandleObject) {
        if let Some(underlying_source) = self.underlying_source.get() {
            underlying_source.set_underlying_source_this_object(this_object);
        }
    }

    pub(crate) fn remove_entry(&self) -> QueueEntry {
        self.queue
            .borrow_mut()
            .pop_front()
            .expect("Reader must have read request when remove is called into.")
    }

    pub(crate) fn get_queue_total_size(&self) -> f64 {
        self.queue_total_size.get()
    }
}

impl ReadableByteStreamControllerMethods<crate::DomTypeHolder> for ReadableByteStreamController {
    /// <https://streams.spec.whatwg.org/#rbs-controller-byob-request>
    fn GetByobRequest(
        &self,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<ReadableStreamBYOBRequest>>> {
        // Return ! ReadableByteStreamControllerGetBYOBRequest(this).
        self.get_byob_request(can_gc)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        // Return ! ReadableByteStreamControllerGetDesiredSize(this).
        self.get_desired_size()
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-close>
    fn Close(&self) -> Fallible<()> {
        // If this.[[closeRequested]] is true, throw a TypeError exception.
        if self.close_requested.get() {
            return Err(Error::Type("closeRequested is true".to_owned()));
        }

        // If this.[[stream]].[[state]] is not "readable", throw a TypeError exception.
        if !self.stream.get().unwrap().is_readable() {
            return Err(Error::Type("stream is not readable".to_owned()));
        }

        // Perform ? ReadableByteStreamControllerClose(this).
        self.close();

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-enqueue>
    #[allow(unsafe_code)]
    fn Enqueue(
        &self,
        chunk: js::gc::CustomAutoRooterGuard<js::typedarray::ArrayBufferView>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        let chunk = HeapBufferSource::<ArrayBufferViewU8>::new(BufferSource::ArrayBufferView(
            Heap::boxed(unsafe { *chunk.underlying_object() }),
        ));

        // If chunk.[[ByteLength]] is 0, throw a TypeError exception.
        if chunk.byte_length() == 0 {
            return Err(Error::Type("chunk.ByteLength is 0".to_owned()));
        }

        // If chunk.[[ViewedArrayBuffer]].[[ByteLength]] is 0, throw a TypeError exception.
        if chunk.viewed_buffer_array_byte_length(cx) == 0 {
            return Err(Error::Type(
                "chunk.ViewedArrayBuffer.ByteLength is 0".to_owned(),
            ));
        }

        // If this.[[closeRequested]] is true, throw a TypeError exception.
        if self.close_requested.get() {
            return Err(Error::Type("closeRequested is true".to_owned()));
        }

        // If this.[[stream]].[[state]] is not "readable", throw a TypeError exception.
        if !self.stream.get().unwrap().is_readable() {
            return Err(Error::Type("stream is not readable".to_owned()));
        }

        // Return ? ReadableByteStreamControllerEnqueue(this, chunk).
        self.enqueue(chunk, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-error>
    fn Error(&self, _cx: SafeJSContext, e: SafeHandleValue) -> Fallible<()> {
        // Perform ! ReadableByteStreamControllerError(this, e).
        self.error(e);
        Ok(())
    }
}
