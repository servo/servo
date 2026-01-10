/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use js::typedarray::ArrayBufferViewU8;

use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::DomRoot;
use super::byteteeunderlyingsource::ByteTeePullAlgorithm;
use crate::dom::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::error::{ErrorToJsval, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::root::Dom;
use crate::dom::byteteeunderlyingsource::ByteTeeUnderlyingSource;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::microtask::Microtask;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, expect(crown::unrooted_must_root))]
pub(crate) struct ByteTeeReadIntoRequestMicrotask {
    #[ignore_malloc_size_of = "mozjs"]
    chunk: HeapBufferSource<ArrayBufferViewU8>,
    tee_read_request: Dom<ByteTeeReadIntoRequest>,
}

impl ByteTeeReadIntoRequestMicrotask {
    pub(crate) fn microtask_chunk_steps(&self, can_gc: CanGc) {
        self.tee_read_request
            .chunk_steps(self.chunk.clone(), can_gc)
            .expect("Failed to enqueue chunk");
    }
}

#[dom_struct]
pub(crate) struct ByteTeeReadIntoRequest {
    reflector_: Reflector,
    for_branch2: bool,
    byob_branch: Dom<ReadableStream>,
    other_branch: Dom<ReadableStream>,
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
impl ByteTeeReadIntoRequest {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        for_branch2: bool,
        byob_branch: &ReadableStream,
        other_branch: &ReadableStream,
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
            Box::new(ByteTeeReadIntoRequest {
                reflector_: Reflector::new(),
                for_branch2,
                byob_branch: Dom::from_ref(byob_branch),
                other_branch: Dom::from_ref(other_branch),
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

    pub(crate) fn enqueue_chunk_steps(&self, chunk: HeapBufferSource<ArrayBufferViewU8>) {
        // Queue a microtask to perform the following steps:
        let byte_tee_read_request_chunk = ByteTeeReadIntoRequestMicrotask {
            chunk,
            tee_read_request: Dom::from_ref(self),
        };

        self.global()
            .enqueue_microtask(Microtask::ReadableStreamByteTeeReadIntoRequest(
                byte_tee_read_request_chunk,
            ));
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-into-request-chunk-steps%E2%91%A0>
    #[allow(clippy::borrowed_box)]
    pub(crate) fn chunk_steps(
        &self,
        chunk: HeapBufferSource<ArrayBufferViewU8>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        let cx = GlobalScope::get_cx();

        // Set readAgainForBranch1 to false.
        self.read_again_for_branch_1.set(false);

        // Set readAgainForBranch2 to false.
        self.read_again_for_branch_2.set(false);

        // Let byobCanceled be canceled2 if forBranch2 is true, and canceled1 otherwise.
        let byob_canceled = if self.for_branch2 {
            self.canceled_2.get()
        } else {
            self.canceled_1.get()
        };

        // Let otherCanceled be canceled2 if forBranch2 is false, and canceled1 otherwise.
        let other_canceled = if self.for_branch2 {
            self.canceled_1.get()
        } else {
            self.canceled_2.get()
        };

        // If otherCanceled is false,
        if !other_canceled {
            // Let cloneResult be CloneAsUint8Array(chunk).
            let clone_result = chunk.clone_as_uint8_array(cx);

            // If cloneResult is an abrupt completion,
            if let Err(error) = clone_result {
                rooted!(in(*cx) let mut error_value = UndefinedValue());
                error
                    .clone()
                    .to_jsval(cx, &self.global(), error_value.handle_mut(), can_gc);

                // Perform ! ReadableByteStreamControllerError(byobBranch.[[controller]], cloneResult.[[Value]]).
                let byob_branch_controller = self.byob_branch.get_byte_controller();
                byob_branch_controller.error(error_value.handle(), can_gc);

                // Perform ! ReadableByteStreamControllerError(otherBranch.[[controller]], cloneResult.[[Value]]).
                let other_branch_controller = self.other_branch.get_byte_controller();
                other_branch_controller.error(error_value.handle(), can_gc);

                // Resolve cancelPromise with ! ReadableStreamCancel(stream, cloneResult.[[Value]]).
                let cancel_result =
                    self.stream
                        .cancel(cx, &self.stream.global(), error_value.handle(), can_gc);
                self.cancel_promise.resolve_native(&cancel_result, can_gc);

                // Return.
                return Ok(());
            } else {
                // Otherwise, let clonedChunk be cloneResult.[[Value]].
                let cloned_chunk = clone_result.unwrap();

                // If byobCanceled is false, perform !
                // ReadableByteStreamControllerRespondWithNewView(byobBranch.[[controller]], chunk).
                if !byob_canceled {
                    let byob_branch_controller = self.byob_branch.get_byte_controller();
                    byob_branch_controller.respond_with_new_view(cx, chunk, can_gc)?;
                }

                // Perform ! ReadableByteStreamControllerEnqueue(otherBranch.[[controller]], clonedChunk).
                let other_branch_controller = self.other_branch.get_byte_controller();
                other_branch_controller.enqueue(cx, cloned_chunk, can_gc)?;
            }
        } else if !byob_canceled {
            // Otherwise, if byobCanceled is false, perform
            // ! ReadableByteStreamControllerRespondWithNewView(byobBranch.[[controller]], chunk).

            let byob_branch_controller = self.byob_branch.get_byte_controller();
            byob_branch_controller.respond_with_new_view(cx, chunk, can_gc)?;
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

    /// <https://streams.spec.whatwg.org/#ref-for-read-into-request-close-steps%E2%91%A1>
    pub(crate) fn close_steps(
        &self,
        chunk: Option<HeapBufferSource<ArrayBufferViewU8>>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        let cx = GlobalScope::get_cx();

        // Set reading to false.
        self.reading.set(false);

        // Let byobCanceled be canceled2 if forBranch2 is true, and canceled1 otherwise.
        let byob_canceled = if self.for_branch2 {
            self.canceled_2.get()
        } else {
            self.canceled_1.get()
        };

        // Let otherCanceled be canceled2 if forBranch2 is false, and canceled1 otherwise.
        let other_canceled = if self.for_branch2 {
            self.canceled_1.get()
        } else {
            self.canceled_2.get()
        };

        // If byobCanceled is false, perform ! ReadableByteStreamControllerClose(byobBranch.[[controller]]).
        if !byob_canceled {
            let byob_branch_controller = self.byob_branch.get_byte_controller();
            byob_branch_controller.close(cx, can_gc)?;
        }

        // If otherCanceled is false, perform ! ReadableByteStreamControllerClose(otherBranch.[[controller]]).
        if !other_canceled {
            let other_branch_controller = self.other_branch.get_byte_controller();
            other_branch_controller.close(cx, can_gc)?;
        }

        // If chunk is not undefined,
        if let Some(chunk_value) = chunk {
            if chunk_value.is_undefined() {
                // Nothing to respond with if the provided chunk is undefined.
                // Continue with the remaining close steps.
            } else {
                let chunk = chunk_value;
                // Assert: chunk.[[ByteLength]] is 0.
                assert_eq!(chunk.byte_length(), 0);

                // If byobCanceled is false, perform !
                // ReadableByteStreamControllerRespondWithNewView(byobBranch.[[controller]], chunk).
                if !byob_canceled {
                    let byob_branch_controller = self.byob_branch.get_byte_controller();
                    byob_branch_controller.respond_with_new_view(cx, chunk, can_gc)?;
                }

                // If otherCanceled is false and otherBranch.[[controller]].[[pendingPullIntos]] is not empty,
                // perform ! ReadableByteStreamControllerRespond(otherBranch.[[controller]], 0).
                if !other_canceled {
                    let other_branch_controller = self.other_branch.get_byte_controller();
                    if other_branch_controller.get_pending_pull_intos_size() > 0 {
                        other_branch_controller.respond(cx, 0, can_gc)?;
                    }
                }
            }
        }

        // If byobCanceled is false or otherCanceled is false, resolve cancelPromise with undefined.
        if !byob_canceled || !other_canceled {
            self.cancel_promise.resolve_native(&(), can_gc);
        }

        Ok(())
    }
    /// <https://streams.spec.whatwg.org/#ref-for-read-into-request-error-steps%E2%91%A0>
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
