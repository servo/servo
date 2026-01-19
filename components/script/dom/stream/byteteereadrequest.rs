/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::typedarray::ArrayBufferViewU8;
use script_bindings::error::Fallible;

use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::DomRoot;
use super::byteteeunderlyingsource::ByteTeePullAlgorithm;
use crate::dom::bindings::buffer_source::{BufferSource, HeapBufferSource};
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::stream::byteteeunderlyingsource::ByteTeeUnderlyingSource;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::stream::readablestream::ReadableStream;
use crate::microtask::Microtask;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, expect(crown::unrooted_must_root))]
pub(crate) struct ByteTeeReadRequestMicrotask {
    #[ignore_malloc_size_of = "mozjs"]
    chunk: Box<Heap<JSVal>>,
    tee_read_request: Dom<ByteTeeReadRequest>,
}

impl ByteTeeReadRequestMicrotask {
    pub(crate) fn microtask_chunk_steps(&self, can_gc: CanGc) {
        self.tee_read_request
            .chunk_steps(&self.chunk, can_gc)
            .expect("ByteTeeReadRequestMicrotask::microtask_chunk_steps failed");
    }
}

#[dom_struct]
/// <https://streams.spec.whatwg.org/#ref-for-read-request%E2%91%A2>
pub(crate) struct ByteTeeReadRequest {
    reflector_: Reflector,
    branch_1: Dom<ReadableStream>,
    branch_2: Dom<ReadableStream>,
    stream: Dom<ReadableStream>,
    #[ignore_malloc_size_of = "Rc"]
    read_again_for_branch_1: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    read_again_for_branch_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    reading: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_1: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    cancel_promise: Rc<Promise>,
    tee_underlying_source: Dom<ByteTeeUnderlyingSource>,
}
impl ByteTeeReadRequest {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        branch_1: &ReadableStream,
        branch_2: &ReadableStream,
        stream: &ReadableStream,
        read_again_for_branch_1: Rc<Cell<bool>>,
        read_again_for_branch_2: Rc<Cell<bool>>,
        reading: Rc<Cell<bool>>,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        cancel_promise: Rc<Promise>,
        tee_underlying_source: &ByteTeeUnderlyingSource,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(ByteTeeReadRequest {
                reflector_: Reflector::new(),
                branch_1: Dom::from_ref(branch_1),
                branch_2: Dom::from_ref(branch_2),
                stream: Dom::from_ref(stream),
                read_again_for_branch_1,
                read_again_for_branch_2,
                reading,
                canceled_1,
                canceled_2,
                cancel_promise,
                tee_underlying_source: Dom::from_ref(tee_underlying_source),
            }),
            global,
            can_gc,
        )
    }

    /// Enqueue a microtask to perform the chunk steps
    /// <https://streams.spec.whatwg.org/#ref-for-read-request-chunk-steps%E2%91%A2>
    pub(crate) fn enqueue_chunk_steps(
        &self,
        global: &GlobalScope,
        chunk: RootedTraceableBox<Heap<JSVal>>,
    ) {
        // Queue a microtask to perform the following steps:
        let byte_tee_read_request_chunk = ByteTeeReadRequestMicrotask {
            chunk: Heap::boxed(*chunk.handle()),
            tee_read_request: Dom::from_ref(self),
        };
        global.enqueue_microtask(Microtask::ReadableStreamByteTeeReadRequest(
            byte_tee_read_request_chunk,
        ));
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-chunk-steps%E2%91%A3>
    #[allow(clippy::borrowed_box)]
    pub(crate) fn chunk_steps(&self, chunk: &Box<Heap<JSVal>>, can_gc: CanGc) -> Fallible<()> {
        let cx = GlobalScope::get_cx();

        // Set readAgainForBranch1 to false.
        self.read_again_for_branch_1.set(false);

        // Set readAgainForBranch2 to false.
        self.read_again_for_branch_2.set(false);

        // Let chunk1 and chunk2 be chunk.
        let chunk1 = chunk;
        let chunk2 = chunk;

        // Helper to surface clone failures exactly once
        let handle_clone_error = |error: Error| {
            rooted!(in(*cx) let mut error_value = UndefinedValue());
            error
                .clone()
                .to_jsval(cx, &self.global(), error_value.handle_mut(), can_gc);

            let branch_1_controller = self.branch_1.get_byte_controller();
            let branch_2_controller = self.branch_2.get_byte_controller();

            branch_1_controller.error(error_value.handle(), can_gc);
            branch_2_controller.error(error_value.handle(), can_gc);

            let cancel_result =
                self.stream
                    .cancel(cx, &self.stream.global(), error_value.handle(), can_gc);
            self.cancel_promise.resolve_native(&cancel_result, can_gc);
        };

        // Prepare per branch chunks ahead of the spec enqueue steps.
        let chunk1_view = if !self.canceled_1.get() {
            Some(HeapBufferSource::<ArrayBufferViewU8>::new(
                BufferSource::ArrayBufferView(RootedTraceableBox::from_box(Heap::boxed(
                    chunk1.get().to_object(),
                ))),
            ))
        } else {
            None
        };

        let mut chunk2_view = None;

        // If canceled1 is false and canceled2 is false,
        if !self.canceled_1.get() && !self.canceled_2.get() {
            // Let cloneResult be CloneAsUint8Array(chunk).
            let chunk2_source =
                HeapBufferSource::<ArrayBufferViewU8>::new(BufferSource::ArrayBufferView(
                    RootedTraceableBox::from_box(Heap::boxed(chunk2.get().to_object())),
                ));
            let clone_result = chunk2_source.clone_as_uint8_array(cx);

            // If cloneResult is an abrupt completion,
            if let Err(error) = clone_result {
                handle_clone_error(error);
                return Ok(());
            } else {
                // Otherwise, set chunk2 to cloneResult.[[Value]].
                chunk2_view = clone_result.ok();
            }
        } else if !self.canceled_2.get() {
            // Only branch2 needs data; clone once for it.
            let chunk2_source =
                HeapBufferSource::<ArrayBufferViewU8>::new(BufferSource::ArrayBufferView(
                    RootedTraceableBox::from_box(Heap::boxed(chunk2.get().to_object())),
                ));
            match chunk2_source.clone_as_uint8_array(cx) {
                Ok(clone) => chunk2_view = Some(clone),
                Err(error) => {
                    handle_clone_error(error);
                    return Ok(());
                },
            }
        }

        // If canceled1 is false, perform ! ReadableByteStreamControllerEnqueue(branch1.[[controller]], chunk1).
        if let Some(chunk1_view) = chunk1_view {
            let branch_1_controller = self.branch_1.get_byte_controller();
            branch_1_controller.enqueue(cx, chunk1_view, can_gc)?;
        }

        // If canceled2 is false, perform ! ReadableByteStreamControllerEnqueue(branch2.[[controller]], chunk2).
        if let Some(chunk2_view) = chunk2_view {
            let branch_2_controller = self.branch_2.get_byte_controller();
            branch_2_controller.enqueue(cx, chunk2_view, can_gc)?;
        }

        // Set reading to false.
        self.reading.set(false);

        // If readAgainForBranch1 is true, perform pull1Algorithm.
        if self.read_again_for_branch_1.get() {
            self.pull_algorithm(Some(ByteTeePullAlgorithm::Pull1Algorithm), can_gc);
        } else if self.read_again_for_branch_2.get() {
            // Otherwise, if readAgainForBranch2 is true, perform pull2Algorithm.
            self.pull_algorithm(Some(ByteTeePullAlgorithm::Pull2Algorithm), can_gc);
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-close-steps%E2%91%A2>
    pub(crate) fn close_steps(&self, can_gc: CanGc) -> Fallible<()> {
        let cx = GlobalScope::get_cx();
        let branch_1_controller = self.branch_1.get_byte_controller();
        let branch_2_controller = self.branch_2.get_byte_controller();

        // Set reading to false.
        self.reading.set(false);

        // If canceled1 is false, perform ! ReadableByteStreamControllerClose(branch1.[[controller]]).
        if !self.canceled_1.get() {
            branch_1_controller.close(cx, can_gc)?;
        }

        // If canceled2 is false, perform ! ReadableByteStreamControllerClose(branch2.[[controller]]).
        if !self.canceled_2.get() {
            branch_2_controller.close(cx, can_gc)?;
        }

        // If branch1.[[controller]].[[pendingPullIntos]] is not empty,
        // perform ! ReadableByteStreamControllerRespond(branch1.[[controller]], 0).
        if branch_1_controller.get_pending_pull_intos_size() > 0 {
            branch_1_controller.respond(cx, 0, can_gc)?;
        }

        // If branch2.[[controller]].[[pendingPullIntos]] is not empty,
        // perform ! ReadableByteStreamControllerRespond(branch2.[[controller]], 0).
        if branch_2_controller.get_pending_pull_intos_size() > 0 {
            branch_2_controller.respond(cx, 0, can_gc)?;
        }

        // If canceled1 is false or canceled2 is false, resolve cancelPromise with undefined.
        if !self.canceled_1.get() || !self.canceled_2.get() {
            self.cancel_promise.resolve_native(&(), can_gc);
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-error-steps%E2%91%A3>
    pub(crate) fn error_steps(&self) {
        // Set reading to false.
        self.reading.set(false);
    }

    pub(crate) fn pull_algorithm(
        &self,
        byte_tee_pull_algorithm: Option<ByteTeePullAlgorithm>,
        can_gc: CanGc,
    ) {
        self.tee_underlying_source
            .pull_algorithm(byte_tee_pull_algorithm, can_gc);
    }
}
