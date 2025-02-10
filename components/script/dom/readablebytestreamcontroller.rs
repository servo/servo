/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use dom_struct::dom_struct;
use image::buffer;
use js::jsapi::{Heap, JS_ClearPendingException};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::wrappers::JS_GetPendingException;
use js::rust::HandleValue as SafeHandleValue;
use js::typedarray::{ArrayBufferU8, ArrayBufferViewU8};

use super::bindings::buffer_source::HeapBufferSource;
use super::bindings::cell::DomRefCell;
use super::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderReadOptions;
use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::Dom;
use super::readablestream::ReaderType;
use super::readablestreambyobreader::ReadIntoRequest;
use super::readablestreamdefaultreader::ReadRequest;
use crate::dom::bindings::buffer_source::{create_uint8_array_with_buffer, BufferSource};
use crate::dom::bindings::codegen::Bindings::ReadableByteStreamControllerBinding::ReadableByteStreamControllerMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::readablestream::ReadableStream;
use crate::dom::readablestreambyobrequest::ReadableStreamBYOBRequest;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry>
#[derive(JSTraceable, MallocSizeOf)]
struct QueueEntry {
    /// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry-buffer>
    #[ignore_malloc_size_of = "HeapBufferSource"]
    buffer: HeapBufferSource<ArrayBufferU8>,
    /// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry-byte-offset>
    byte_offset: usize,
    /// <https://streams.spec.whatwg.org/#readable-byte-stream-queue-entry-byte-length>
    byte_length: usize,
}

/// <https://streams.spec.whatwg.org/#pull-into-descriptor>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct PullIntoDescriptor {
    #[ignore_malloc_size_of = "HeapBufferSource"]
    /// An ArrayBuffer
    buffer: DomRefCell<HeapBufferSource<ArrayBufferU8>>,
    /// A positive integer representing the initial byte length of buffer
    buffer_byte_length: usize,
    /// A nonnegative integer byte offset into the buffer where the underlying byte source will start writing
    byte_offset: usize,
    /// A positive integer number of bytes which can be written into the buffer
    byte_length: usize,
    /// A nonnegative integer number of bytes that have been written into the buffer so far
    bytes_filled: usize,
    /// A positive integer representing the minimum number of bytes that must be written into the buffer
    /// before the associated read() request may be fulfilled. By default, this equals the element size.
    minimun_fill: usize,
    /// A positive integer representing the number of bytes that can be written into the buffer at a time,
    /// using views of the type described by the view constructor
    element_size: usize,
    #[ignore_malloc_size_of = "HeapBufferSource"]
    /// A typed array constructor or %DataView%, which will be used for constructing a view with which to write into the buffer
    view_constructor: DomRefCell<HeapBufferSource<ArrayBufferViewU8>>,
    /// Either "default" or "byob", indicating what type of readable stream reader initiated this request, or "none" if the initiating reader was released
    reader_type: Option<ReaderType>,
}

impl PullIntoDescriptor {
    pub(crate) fn set_buffer(&self, buffer: HeapBufferSource<ArrayBufferU8>) {
        *self.buffer.borrow_mut() = buffer;
    }
}

/// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
#[dom_struct]
pub(crate) struct ReadableByteStreamController {
    reflector_: Reflector,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-stream>
    stream: MutNullableDom<ReadableStream>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-strategyhwm>
    strategy_hwm: Cell<f64>,
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-queue>
    queue: DomRefCell<VecDeque<QueueEntry>>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-queuetotalsize>
    queue_total_size: Cell<f64>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-byobrequest>
    byob_request: MutNullableDom<ReadableStreamBYOBRequest>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-pendingpullintos>
    pending_pull_intos: DomRefCell<Vec<PullIntoDescriptor>>,
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller-closerequested>
    close_requested: Cell<bool>,
}

impl ReadableByteStreamController {
    fn new_inherited(_global: &GlobalScope, _can_gc: CanGc) -> ReadableByteStreamController {
        ReadableByteStreamController {
            reflector_: Reflector::new(),
            byob_request: MutNullableDom::new(None),
            stream: MutNullableDom::new(None),
            pending_pull_intos: DomRefCell::new(Vec::new()),
            strategy_hwm: Default::default(),
            close_requested: Default::default(),
            queue: DomRefCell::new(Default::default()),
            queue_total_size: Default::default(),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<ReadableByteStreamController> {
        reflect_dom_object(
            Box::new(ReadableByteStreamController::new_inherited(global, can_gc)),
            global,
            can_gc,
        )
    }

    pub(crate) fn set_stream(&self, stream: &ReadableStream) {
        self.stream.set(Some(stream))
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-pull-into>
    pub(crate) fn perform_pull_into(
        &self,
        _read_into_request: &ReadIntoRequest,
        _view: HeapBufferSource<ArrayBufferViewU8>,
        _options: &ReadableStreamBYOBReaderReadOptions,
        _can_gc: CanGc,
    ) {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond>
    pub(crate) fn respond(&self, _bytes_written: u64) -> Fallible<()> {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-respond-with-new-view>
    pub(crate) fn respond_with_new_view(
        &self,
        _view: HeapBufferSource<ArrayBufferViewU8>,
    ) -> Fallible<()> {
        todo!()
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
        Some(self.strategy_hwm.get() - self.queue_total_size.get())
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
            // Let view be ! Construct(%Uint8Array%, « firstDescriptor’s buffer, firstDescriptor’s byte offset + firstDescriptor’s bytes filled,
            // firstDescriptor’s byte length − firstDescriptor’s bytes filled »).
            let buffer = first_descriptor
                .buffer
                .borrow()
                .get_typed_array()
                .map_err(|_| Error::NotFound)?;

            let byte_offset = first_descriptor.byte_offset + first_descriptor.bytes_filled;
            let byte_length = first_descriptor.byte_length - first_descriptor.bytes_filled;

            let uint8_array = create_uint8_array_with_buffer(
                cx,
                Heap::boxed(unsafe { *buffer.underlying_object() }),
                byte_offset,
                byte_length as i64,
            )
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

            // If the remainder after dividing firstPendingPullInto’s bytes filled by firstPendingPullInto’s element size is not 0,
            if first_pending_pull_into.bytes_filled % first_pending_pull_into.element_size != 0 {
                // Let e be a new TypeError exception.
                let e = Error::Type("remainder after dividing firstPendingPullInto's bytes filled by firstPendingPullInto's element size is not 0".to_owned());

                // Perform ! ReadableByteStreamControllerError(controller, e).
                rooted!(in(*cx) let mut error = UndefinedValue());
                unsafe { e.to_jsval(*cx, &self.global(), error.handle_mut()) };
                self.error(error.handle());

                // Throw e.
                todo!()
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
        todo!()
    }

    /// https://streams.spec.whatwg.org/#reset-queue
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
    pub(crate) fn enqueue(&self, chunk: HeapBufferSource<ArrayBufferViewU8>) -> Fallible<()> {
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
        let transferred_buffer = buffer.transfer_array_buffer(cx);

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
            first_descriptor.set_buffer(first_descriptor.buffer.borrow().transfer_array_buffer(cx));

            // If firstPendingPullInto’s reader type is "none", perform ? ReadableByteStreamControllerEnqueueDetachedPullIntoToQueue(controller, firstPendingPullInto).
            if first_descriptor.reader_type.is_none() {
                self.enqueue_detached_pull_into_to_queue(first_descriptor);
            }
        }

        // If ! ReadableStreamHasDefaultReader(stream) is true,
        if stream.has_default_reader() {
            // Perform ! ReadableByteStreamControllerProcessReadRequestsUsingQueue(controller).
            self.process_read_requests_using_queue();

            // If ! ReadableStreamGetNumReadRequests(stream) is 0,
            if stream.get_num_read_requests() == 0 {
                // Assert: controller.[[pendingPullIntos]] is empty.
                assert!(self.pending_pull_intos.borrow().is_empty());

                // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue(controller, transferredBuffer, byteOffset, byteLength).
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
                        Some(ReaderType::Default(_))
                    ));

                    // Perform ! ReadableByteStreamControllerShiftPendingPullInto(controller).
                    self.shift_pending_pull_into();
                }

                // Let transferredView be ! Construct(%Uint8Array%, « transferredBuffer, byteOffset, byteLength »).
                let buffer = transferred_buffer
                    .get_typed_array()
                    .expect("buffer must be a typed array");
                let view = create_uint8_array_with_buffer(
                    cx,
                    Heap::boxed(unsafe { *buffer.underlying_object() }),
                    byte_offset,
                    byte_length as i64,
                )
                .expect("view must be a typed array");

                // Perform ! ReadableStreamFulfillReadRequest(stream, transferredView, false).
                rooted!(in(*cx) let view_val =  ObjectValue(*unsafe { view.underlying_object() }));
                stream.fulfill_read_request(view_val.handle(), false);
            }
            // Otherwise, if ! ReadableStreamHasBYOBReader(stream) is true,
        } else if stream.has_byob_reader() {
            // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue(controller, transferredBuffer, byteOffset, byteLength).
            self.enqueue_chunk_to_queue(transferred_buffer, byte_offset, byte_length);

            // Let filledPullIntos be the result of performing ! ReadableByteStreamControllerProcessPullIntoDescriptorsUsingQueue(controller).
            let filled_pull_intos = self.process_pull_into_descriptors_using_queue();

            // For each filledPullInto of filledPullIntos,
            // Perform ! ReadableByteStreamControllerCommitPullIntoDescriptor(stream, filledPullInto).
            for filled_pull_into in filled_pull_intos {
                self.commit_pull_into_descriptor(filled_pull_into);
            }
        } else {
            // Assert: ! IsReadableStreamLocked(stream) is false.
            assert!(!stream.is_locked());

            // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue(controller, transferredBuffer, byteOffset, byteLength).
            self.enqueue_chunk_to_queue(transferred_buffer, byte_offset, byte_length);
        }

        // Perform ! ReadableByteStreamControllerCallPullIfNeeded(controller).
        self.call_pull_if_needed();
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-process-pull-into-descriptors-using-queue>
    pub(crate) fn process_pull_into_descriptors_using_queue(&self) {
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
            if self.fill_pull_into_descriptor_from_queue(pull_into_descriptor) {}
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-fill-pull-into-descriptor-from-queue>
    pub(crate) fn fill_pull_into_descriptor_from_queue(
        &self,
        pull_into_descriptor: &PullIntoDescriptor,
    ) -> bool {
        let cx = GlobalScope::get_cx();
        // Let maxBytesToCopy be min(controller.[[queueTotalSize]], pullIntoDescriptor’s byte length − pullIntoDescriptor’s bytes filled).
        let max_bytes_to_copy = self.queue_total_size.get().min(
            pull_into_descriptor.byte_length as f64 - pull_into_descriptor.bytes_filled as f64,
        );

        // Let maxBytesFilled be pullIntoDescriptor’s bytes filled + maxBytesToCopy.
        let max_bytes_filled = pull_into_descriptor.bytes_filled + max_bytes_to_copy as usize;

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
        assert!(pull_into_descriptor.bytes_filled < pull_into_descriptor.minimun_fill);

        // Let remainderBytes be the remainder after dividing maxBytesFilled by pullIntoDescriptor’s element size.
        let remainder_bytes = max_bytes_filled % pull_into_descriptor.element_size;

        // Let maxAlignedBytes be maxBytesFilled − remainderBytes.
        let max_aligned_bytes = max_bytes_filled - remainder_bytes;

        // If maxAlignedBytes ≥ pullIntoDescriptor’s minimum fill,
        if max_aligned_bytes >= pull_into_descriptor.minimun_fill {
            // Set totalBytesToCopyRemaining to maxAlignedBytes − pullIntoDescriptor’s bytes filled.
            total_bytes_to_copy_remaining =
                (max_aligned_bytes - pull_into_descriptor.bytes_filled) as f64;

            // Set ready to true.
            ready = true;
        }

        // Let queue be controller.[[queue]].
        let queue = self.queue.borrow();

        // While totalBytesToCopyRemaining > 0,
        while total_bytes_to_copy_remaining > 0.0 {
            // Let headOfQueue be queue[0].
            let head_of_queue = queue.front().unwrap();

            // Let bytesToCopy be min(totalBytesToCopyRemaining, headOfQueue’s byte length).
            let bytes_to_copy = total_bytes_to_copy_remaining.min(head_of_queue.byte_length as f64);

            // Let destStart be pullIntoDescriptor’s byte offset + pullIntoDescriptor’s bytes filled.
            let dest_start = pull_into_descriptor.byte_offset + pull_into_descriptor.bytes_filled;

            // Let descriptorBuffer be pullIntoDescriptor’s buffer.
            let descriptor_buffer = pull_into_descriptor.buffer.borrow();

            // Let queueBuffer be headOfQueue’s buffer.
            let queue_buffer = head_of_queue.buffer;

            // Let queueByteOffset be headOfQueue’s byte offset.
            let queue_byte_offset = head_of_queue.byte_offset;

            // Assert: ! CanCopyDataBlockBytes(descriptorBuffer, destStart, queueBuffer, queueByteOffset, bytesToCopy) is true.
            assert!(descriptor_buffer.can_copy_data_block_bytes(
                cx,
                dest_start,
                &queue_buffer,
                queue_byte_offset,
                bytes_to_copy
            ));

            todo!()
        }

        todo!()
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerenqueuedetachedpullintotoqueue>
    pub(crate) fn enqueue_detached_pull_into_to_queue(
        &self,
        first_descriptor: &PullIntoDescriptor,
    ) {
        // Assert: pullIntoDescriptor’s reader type is "none".
        assert!(first_descriptor.reader_type.is_none());

        // If pullIntoDescriptor’s bytes filled > 0, perform ? ReadableByteStreamControllerEnqueueClonedChunkToQueue(controller,
        // pullIntoDescriptor’s buffer, pullIntoDescriptor’s byte offset, pullIntoDescriptor’s bytes filled).

        if first_descriptor.bytes_filled > 0 {
            self.enqueue_cloned_chunk_to_queue(
                &first_descriptor.buffer.borrow(),
                first_descriptor.byte_offset,
                first_descriptor.bytes_filled,
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
        // TODO: how to detect abrupt completion?
        if false {
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
            // Perform ! ReadableByteStreamControllerEnqueueChunkToQueue(controller, cloneResult.[[Value]], 0, byteLength).
            self.enqueue_chunk_to_queue(clone_result, 0, byte_length);
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
        let entry = QueueEntry {
            buffer,
            byte_offset,
            byte_length,
        };

        // Append entry to controller.[[queue]].
        self.queue.borrow_mut().push_back(entry);

        // Set controller.[[queueTotalSize]] to controller.[[queueTotalSize]] + byteLength.
        self.queue_total_size
            .set(self.queue_total_size.get() + byte_length as f64);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-shift-pending-pull-into>
    pub(crate) fn shift_pending_pull_into(&self) -> &PullIntoDescriptor {
        // Assert: controller.[[byobRequest]] is null.
        assert!(self.byob_request.get().is_none());

        // Let descriptor be controller.[[pendingPullIntos]][0].
        let descriptor = self.pending_pull_intos.borrow().first().unwrap();

        // Remove descriptor from controller.[[pendingPullIntos]].
        self.pending_pull_intos.borrow_mut().remove(0);

        // Return descriptor.
        // descriptor
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerprocessreadrequestsusingqueue>
    pub(crate) fn process_read_requests_using_queue(&self) {
        // Let reader be controller.[[stream]].[[reader]].
        // Assert: reader implements ReadableStreamDefaultReader.
        let reader = self.stream.get().unwrap().get_default_reader();

        // Step 3
        reader.process_read_requests(DomRoot::from_ref(self));
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerfillreadrequestfromqueue>
    #[allow(unsafe_code)]
    pub(crate) fn fill_read_request_from_queue(&self, read_request: &ReadRequest) {
        // Assert: controller.[[queueTotalSize]] > 0.
        assert!(self.queue_total_size.get() > 0.0);
        // Also assert that the queue has a non-zero length;
        assert!(!self.queue.borrow().is_empty());

        // Let entry be controller.[[queue]][0].
        // Remove entry from controller.[[queue]].
        let entry = self.remove_entry();

        // Set controller.[[queueTotalSize]] to controller.[[queueTotalSize]] − entry’s byte length.
        self.queue_total_size
            .set(self.queue_total_size.get() - entry.byte_length as f64);

        // Perform ! ReadableByteStreamControllerHandleQueueDrain(controller).
        self.handle_queue_drain();

        // Let view be ! Construct(%Uint8Array%, « entry’s buffer, entry’s byte offset, entry’s byte length »).
        let buffer = entry
            .buffer
            .get_typed_array()
            .expect("buffer must be a typed array");

        let view = create_uint8_array_with_buffer(
            GlobalScope::get_cx(),
            Heap::boxed(unsafe { *buffer.underlying_object() }),
            entry.byte_offset,
            entry.byte_length as i64,
        )
        .expect("view must be a typed array");

        // Perform readRequest’s chunk steps, given view.
        let result = RootedTraceableBox::new(Heap::default());
        unsafe {
            result.set(ObjectValue(*view.underlying_object()));
        }

        read_request.chunk_steps(result);
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-handle-queue-drain>
    pub(crate) fn handle_queue_drain(&self) {
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
            self.call_pull_if_needed();
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-byte-stream-controller-call-pull-if-needed>
    pub(crate) fn call_pull_if_needed(&self) {
        todo!()
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
        self.enqueue(chunk)
    }

    /// <https://streams.spec.whatwg.org/#rbs-controller-error>
    fn Error(&self, _cx: SafeJSContext, e: SafeHandleValue) -> Fallible<()> {
        // Perform ! ReadableByteStreamControllerError(this, e).
        self.error(e);
        Ok(())
    }
}
