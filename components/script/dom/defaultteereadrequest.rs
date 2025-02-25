/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleValue as SafeHandleValue;

use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::DomRoot;
use super::bindings::structuredclone;
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::defaultteeunderlyingsource::DefaultTeeUnderlyingSource;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::microtask::Microtask;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) struct DefaultTeeReadRequestMicrotask {
    #[ignore_malloc_size_of = "mozjs"]
    chunk: Box<Heap<JSVal>>,
    tee_read_request: Dom<DefaultTeeReadRequest>,
}

impl DefaultTeeReadRequestMicrotask {
    pub(crate) fn microtask_chunk_steps(&self, can_gc: CanGc) {
        self.tee_read_request.chunk_steps(&self.chunk, can_gc)
    }
}

#[dom_struct]
/// <https://streams.spec.whatwg.org/#ref-for-read-request%E2%91%A2>
pub(crate) struct DefaultTeeReadRequest {
    reflector_: Reflector,
    stream: Dom<ReadableStream>,
    branch_1: Dom<ReadableStream>,
    branch_2: Dom<ReadableStream>,
    #[ignore_malloc_size_of = "Rc"]
    reading: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    read_again: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_1: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    clone_for_branch_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    cancel_promise: Rc<Promise>,
    tee_underlying_source: Dom<DefaultTeeUnderlyingSource>,
}
impl DefaultTeeReadRequest {
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        stream: &ReadableStream,
        branch_1: &ReadableStream,
        branch_2: &ReadableStream,
        reading: Rc<Cell<bool>>,
        read_again: Rc<Cell<bool>>,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        clone_for_branch_2: Rc<Cell<bool>>,
        cancel_promise: Rc<Promise>,
        tee_underlying_source: &DefaultTeeUnderlyingSource,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(DefaultTeeReadRequest {
                reflector_: Reflector::new(),
                stream: Dom::from_ref(stream),
                branch_1: Dom::from_ref(branch_1),
                branch_2: Dom::from_ref(branch_2),
                reading,
                read_again,
                canceled_1,
                canceled_2,
                clone_for_branch_2,
                cancel_promise,
                tee_underlying_source: Dom::from_ref(tee_underlying_source),
            }),
            &*stream.global(),
            can_gc,
        )
    }
    /// Call into cancel of the stream,
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>
    pub(crate) fn stream_cancel(&self, reason: SafeHandleValue, can_gc: CanGc) {
        self.stream.cancel(reason, can_gc);
    }
    /// Enqueue a microtask to perform the chunk steps
    /// <https://streams.spec.whatwg.org/#ref-for-read-request-chunk-steps%E2%91%A2>
    pub(crate) fn enqueue_chunk_steps(&self, chunk: RootedTraceableBox<Heap<JSVal>>) {
        // Queue a microtask to perform the following steps:
        let tee_read_request_chunk = DefaultTeeReadRequestMicrotask {
            chunk: Heap::boxed(*chunk.handle()),
            tee_read_request: Dom::from_ref(self),
        };
        let global = self.stream.global();
        let microtask_queue = global.microtask_queue();
        let cx = GlobalScope::get_cx();
        microtask_queue.enqueue(
            Microtask::ReadableStreamTeeReadRequest(tee_read_request_chunk),
            cx,
        );
    }
    /// <https://streams.spec.whatwg.org/#ref-for-read-request-chunk-steps%E2%91%A2>
    #[allow(clippy::borrowed_box)]
    pub(crate) fn chunk_steps(&self, chunk: &Box<Heap<JSVal>>, can_gc: CanGc) {
        // Set readAgain to false.
        self.read_again.set(false);
        // Let chunk1 and chunk2 be chunk.
        let chunk1 = chunk;
        let chunk2 = chunk;
        let cx = GlobalScope::get_cx();

        rooted!(in(*cx) let chunk1_value = chunk1.get());
        rooted!(in(*cx) let chunk2_value = chunk2.get());
        // If canceled_2 is false and cloneForBranch2 is true,
        if !self.canceled_2.get() && self.clone_for_branch_2.get() {
            // Let cloneResult be StructuredClone(chunk2).
            rooted!(in(*cx) let mut clone_result = UndefinedValue());
            let data = structuredclone::write(cx, chunk2_value.handle(), None).unwrap();
            // If cloneResult is an abrupt completion,
            if structuredclone::read(&self.stream.global(), data, clone_result.handle_mut())
                .is_err()
            {
                // Perform ! ReadableStreamDefaultControllerError(branch_1.[[controller]], cloneResult.[[Value]]).
                self.readable_stream_default_controller_error(
                    &self.branch_1,
                    clone_result.handle(),
                    can_gc,
                );

                // Perform ! ReadableStreamDefaultControllerError(branch_2.[[controller]], cloneResult.[[Value]]).
                self.readable_stream_default_controller_error(
                    &self.branch_2,
                    clone_result.handle(),
                    can_gc,
                );
                // Resolve cancelPromise with ! ReadableStreamCancel(stream, cloneResult.[[Value]]).
                self.stream_cancel(clone_result.handle(), can_gc);
                // Return.
                return;
            } else {
                // Otherwise, set chunk2 to cloneResult.[[Value]].
                chunk2.set(*clone_result);
            }
        }
        // If canceled_1 is false, perform ! ReadableStreamDefaultControllerEnqueue(branch_1.[[controller]], chunk1).
        if !self.canceled_1.get() {
            self.readable_stream_default_controller_enqueue(
                &self.branch_1,
                chunk1_value.handle(),
                can_gc,
            );
        }
        // If canceled_2 is false, perform ! ReadableStreamDefaultControllerEnqueue(branch_2.[[controller]], chunk2).
        if !self.canceled_2.get() {
            self.readable_stream_default_controller_enqueue(
                &self.branch_2,
                chunk2_value.handle(),
                can_gc,
            );
        }
        // Set reading to false.
        self.reading.set(false);
        // If readAgain is true, perform pullAlgorithm.
        if self.read_again.get() {
            self.pull_algorithm(can_gc);
        }
    }
    /// <https://streams.spec.whatwg.org/#read-request-close-steps>
    pub(crate) fn close_steps(&self, can_gc: CanGc) {
        // Set reading to false.
        self.reading.set(false);
        // If canceled_1 is false, perform ! ReadableStreamDefaultControllerClose(branch_1.[[controller]]).
        if !self.canceled_1.get() {
            self.readable_stream_default_controller_close(&self.branch_1, can_gc);
        }
        // If canceled_2 is false, perform ! ReadableStreamDefaultControllerClose(branch_2.[[controller]]).
        if !self.canceled_2.get() {
            self.readable_stream_default_controller_close(&self.branch_2, can_gc);
        }
        // If canceled_1 is false or canceled_2 is false, resolve cancelPromise with undefined.
        if !self.canceled_1.get() || !self.canceled_2.get() {
            self.cancel_promise.resolve_native(&(), can_gc);
        }
    }
    /// <https://streams.spec.whatwg.org/#read-request-error-steps>
    pub(crate) fn error_steps(&self) {
        // Set reading to false.
        self.reading.set(false);
    }
    /// Call into enqueue of the default controller of a stream,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    fn readable_stream_default_controller_enqueue(
        &self,
        stream: &ReadableStream,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) {
        stream
            .get_default_controller()
            .enqueue(GlobalScope::get_cx(), chunk, can_gc)
            .expect("enqueue failed for stream controller in DefaultTeeReadRequest");
    }

    /// Call into close of the default controller of a stream,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-close>
    fn readable_stream_default_controller_close(&self, stream: &ReadableStream, can_gc: CanGc) {
        stream.get_default_controller().close(can_gc);
    }

    /// Call into error of the default controller of stream,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-error>
    fn readable_stream_default_controller_error(
        &self,
        stream: &ReadableStream,
        error: SafeHandleValue,
        can_gc: CanGc,
    ) {
        stream.get_default_controller().error(error, can_gc);
    }

    pub(crate) fn pull_algorithm(&self, can_gc: CanGc) {
        self.tee_underlying_source.pull_algorithm(can_gc);
    }
}
