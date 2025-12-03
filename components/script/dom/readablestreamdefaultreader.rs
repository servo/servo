/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::MutNullableDom;
use super::byteteereadrequest::ByteTeeReadRequest;
use super::readablebytestreamcontroller::ReadableByteStreamController;
use super::types::ReadableStreamDefaultController;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::{
    ReadableStreamDefaultReaderMethods, ReadableStreamReadResult,
};
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::defaultteereadrequest::DefaultTeeReadRequest;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::{ReadableStream, bytes_from_chunk_jsval};
use crate::dom::readablestreamgenericreader::ReadableStreamGenericReader;
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

type ReadAllBytesSuccessSteps = dyn Fn(&[u8]);
type ReadAllBytesFailureSteps = dyn Fn(SafeJSContext, SafeHandleValue);

impl js::gc::Rootable for ContinueReadMicrotask {}

/// Microtask handler to continue the read loop without recursion.
/// Spec note: "This recursion could potentially cause a stack overflow
/// if implemented directly. Implementations will need to mitigate this,
/// e.g. by using a non-recursive variant of this algorithm, or queuing
/// a microtask…"
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ContinueReadMicrotask {
    reader: Dom<ReadableStreamDefaultReader>,
    request: ReadRequest,
}

impl Callback for ContinueReadMicrotask {
    fn callback(&self, cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // https://streams.spec.whatwg.org/#ref-for-read-loop%E2%91%A0
        // Note: continuing the read-loop from inside a micro-task to break recursion.
        self.reader.read(cx, &self.request, can_gc);
    }
}

/// <https://streams.spec.whatwg.org/#read-loop>
fn read_loop(
    reader: &ReadableStreamDefaultReader,
    cx: SafeJSContext,
    success_steps: Rc<ReadAllBytesSuccessSteps>,
    failure_steps: Rc<ReadAllBytesFailureSteps>,
    can_gc: CanGc,
) {
    // For the purposes of the above algorithm, to read-loop given reader,
    // bytes, successSteps, and failureSteps:

    // Step 1 .Let readRequest be a new read request with the following items:
    let req = ReadRequest::ReadLoop {
        success_steps,
        failure_steps,
        reader: Dom::from_ref(reader),
        bytes: Rc::new(DomRefCell::new(Vec::new())),
    };
    // Step 2 .Perform ! ReadableStreamDefaultReaderRead(reader, readRequest).
    reader.read(cx, &req, can_gc);
}

/// <https://streams.spec.whatwg.org/#read-request>
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum ReadRequest {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    Read(#[conditional_malloc_size_of] Rc<Promise>),
    /// <https://streams.spec.whatwg.org/#ref-for-read-request%E2%91%A2>
    DefaultTee {
        tee_read_request: Dom<DefaultTeeReadRequest>,
    },
    /// Spec read loop variant, driven by read-request steps (no Promise).
    /// <https://streams.spec.whatwg.org/#read-loop>
    ReadLoop {
        #[ignore_malloc_size_of = "Rc is hard"]
        #[no_trace]
        success_steps: Rc<ReadAllBytesSuccessSteps>,
        #[ignore_malloc_size_of = "Rc is hard"]
        #[no_trace]
        failure_steps: Rc<ReadAllBytesFailureSteps>,
        reader: Dom<ReadableStreamDefaultReader>,
        #[conditional_malloc_size_of]
        bytes: Rc<DomRefCell<Vec<u8>>>,
    },
    ByteTee {
        byte_tee_read_request: Dom<ByteTeeReadRequest>,
    },
}

impl ReadRequest {
    /// <https://streams.spec.whatwg.org/#read-request-chunk-steps>
    pub(crate) fn chunk_steps(
        &self,
        chunk: RootedTraceableBox<Heap<JSVal>>,
        global: &GlobalScope,
        can_gc: CanGc,
    ) {
        match self {
            ReadRequest::Read(promise) => {
                // chunk steps, given chunk
                // Resolve promise with «[ "value" → chunk, "done" → false ]».
                promise.resolve_native(
                    &ReadableStreamReadResult {
                        done: Some(false),
                        value: chunk,
                    },
                    can_gc,
                );
            },
            ReadRequest::DefaultTee { tee_read_request } => {
                tee_read_request.enqueue_chunk_steps(chunk);
            },
            ReadRequest::ByteTee {
                byte_tee_read_request,
            } => {
                byte_tee_read_request.enqueue_chunk_steps(global, chunk);
            },
            ReadRequest::ReadLoop {
                success_steps: _,
                failure_steps,
                reader,
                bytes,
            } => {
                // Spec: chunk steps, given chunk
                let cx = GlobalScope::get_cx();
                let global = reader.global();

                match bytes_from_chunk_jsval(cx, &chunk, can_gc) {
                    Ok(vec) => {
                        // Step 2. Append the bytes represented by chunk to bytes.
                        bytes.borrow_mut().extend_from_slice(&vec);

                        // Step 3. Read-loop given reader, bytes, successSteps, and failureSteps.
                        // Spec note: Avoid direct recursion; queue into a microtask.
                        // Resolving the promise will queue a microtask to call into the native handler.
                        let tick = Promise::new(&global, can_gc);
                        tick.resolve_native(&(), can_gc);

                        let handler = PromiseNativeHandler::new(
                            &global,
                            Some(Box::new(ContinueReadMicrotask {
                                reader: Dom::from_ref(reader),
                                request: self.clone(),
                            })),
                            None,
                            can_gc,
                        );

                        let realm = enter_realm(&*global);
                        let comp = InRealm::Entered(&realm);
                        tick.append_native_handler(&handler, comp, can_gc);
                    },
                    Err(err) => {
                        // Step 1. If chunk is not a Uint8Array object, call failureSteps with a TypeError and abort.
                        rooted!(in(*cx) let mut v = UndefinedValue());
                        err.to_jsval(cx, &global, v.handle_mut(), can_gc);
                        (failure_steps)(cx, v.handle());
                    },
                }
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#read-request-close-steps>
    pub(crate) fn close_steps(&self, can_gc: CanGc) {
        match self {
            ReadRequest::Read(promise) => {
                // close steps
                // Resolve promise with «[ "value" → undefined, "done" → true ]».
                let result = RootedTraceableBox::new(Heap::default());
                result.set(UndefinedValue());
                promise.resolve_native(
                    &ReadableStreamReadResult {
                        done: Some(true),
                        value: result,
                    },
                    can_gc,
                );
            },
            ReadRequest::DefaultTee { tee_read_request } => {
                tee_read_request.close_steps(can_gc);
            },
            ReadRequest::ByteTee {
                byte_tee_read_request,
            } => {
                byte_tee_read_request
                    .close_steps(can_gc)
                    .expect("ByteTeeReadRequest close steps should not fail");
            },
            ReadRequest::ReadLoop {
                success_steps,
                reader,
                bytes,
                ..
            } => {
                // Step 1. Call successSteps with bytes.
                (success_steps)(&bytes.borrow());

                reader
                    .release(can_gc)
                    .expect("Releasing the read-all-bytes reader should succeed");
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#read-request-error-steps>
    pub(crate) fn error_steps(&self, e: SafeHandleValue, can_gc: CanGc) {
        match self {
            ReadRequest::Read(promise) => {
                // error steps, given e
                // Reject promise with e.
                promise.reject_native(&e, can_gc)
            },
            ReadRequest::DefaultTee { tee_read_request } => {
                tee_read_request.error_steps();
            },
            ReadRequest::ByteTee {
                byte_tee_read_request,
            } => {
                byte_tee_read_request.error_steps();
            },
            ReadRequest::ReadLoop {
                failure_steps,
                reader,
                ..
            } => {
                // Step 1. Call failureSteps with e.
                let cx = GlobalScope::get_cx();
                (failure_steps)(cx, e);

                reader
                    .release(can_gc)
                    .expect("Releasing the read-all-bytes reader should succeed");
            },
        }
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamtee>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ByteTeeClosedPromiseRejectionHandler {
    branch_1_controller: Dom<ReadableByteStreamController>,
    branch_2_controller: Dom<ReadableByteStreamController>,
    #[conditional_malloc_size_of]
    canceled_1: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    canceled_2: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    cancel_promise: Rc<Promise>,
    #[conditional_malloc_size_of]
    reader_version: Rc<Cell<u64>>,
    expected_version: u64,
}

impl Callback for ByteTeeClosedPromiseRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamtee>
    /// Upon rejection of reader.[[closedPromise]] with reason r,
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // If thisReader is not the current `reader`, return.
        if self.reader_version.get() != self.expected_version {
            return;
        }

        // Perform ! ReadableByteStreamControllerError(branch1.[[controller]], r).
        self.branch_1_controller.error(v, can_gc);

        // Perform ! ReadableByteStreamControllerError(branch2.[[controller]], r).
        self.branch_2_controller.error(v, can_gc);

        // If canceled1 is false or canceled2 is false, resolve cancelPromise with undefined.
        if !self.canceled_1.get() || !self.canceled_2.get() {
            self.cancel_promise.resolve_native(&(), can_gc);
        }
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-stream-tee>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct DefaultTeeClosedPromiseRejectionHandler {
    branch_1_controller: Dom<ReadableStreamDefaultController>,
    branch_2_controller: Dom<ReadableStreamDefaultController>,
    #[conditional_malloc_size_of]
    canceled_1: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    canceled_2: Rc<Cell<bool>>,
    #[conditional_malloc_size_of]
    cancel_promise: Rc<Promise>,
}

impl Callback for DefaultTeeClosedPromiseRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    /// Upon rejection of reader.[[closedPromise]] with reason r,
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // Perform ! ReadableStreamDefaultControllerError(branch_1.[[controller]], r).
        self.branch_1_controller.error(v, can_gc);
        // Perform ! ReadableStreamDefaultControllerError(branch_2.[[controller]], r).
        self.branch_2_controller.error(v, can_gc);

        // If canceled_1 is false or canceled_2 is false, resolve cancelPromise with undefined.
        if !self.canceled_1.get() || !self.canceled_2.get() {
            self.cancel_promise.resolve_native(&(), can_gc);
        }
    }
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
#[dom_struct]
pub(crate) struct ReadableStreamDefaultReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
    stream: MutNullableDom<ReadableStream>,

    read_requests: DomRefCell<VecDeque<ReadRequest>>,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-closedpromise>
    #[conditional_malloc_size_of]
    closed_promise: DomRefCell<Rc<Promise>>,
}

impl ReadableStreamDefaultReader {
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStreamDefaultReader> {
        reflect_dom_object_with_proto(
            Box::new(ReadableStreamDefaultReader::new_inherited(global, can_gc)),
            global,
            proto,
            can_gc,
        )
    }

    fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> ReadableStreamDefaultReader {
        ReadableStreamDefaultReader {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            read_requests: DomRefCell::new(Default::default()),
            closed_promise: DomRefCell::new(Promise::new(global, can_gc)),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<ReadableStreamDefaultReader> {
        reflect_dom_object(
            Box::new(Self::new_inherited(global, can_gc)),
            global,
            can_gc,
        )
    }

    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-reader>
    pub(crate) fn set_up(
        &self,
        stream: &ReadableStream,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // If ! IsReadableStreamLocked(stream) is true, throw a TypeError exception.
        if stream.is_locked() {
            return Err(Error::Type("stream is locked".to_owned()));
        }
        // Perform ! ReadableStreamReaderGenericInitialize(reader, stream).

        self.generic_initialize(global, stream, can_gc);

        // Set reader.[[readRequests]] to a new empty list.
        self.read_requests.borrow_mut().clear();

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub(crate) fn close(&self, can_gc: CanGc) {
        // Resolve reader.[[closedPromise]] with undefined.
        self.closed_promise.borrow().resolve_native(&(), can_gc);
        // If reader implements ReadableStreamDefaultReader,
        // Let readRequests be reader.[[readRequests]].
        let mut read_requests = self.take_read_requests();
        // Set reader.[[readRequests]] to an empty list.
        // For each readRequest of readRequests,
        for request in read_requests.drain(0..) {
            // Perform readRequest’s close steps.
            request.close_steps(can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub(crate) fn add_read_request(&self, read_request: &ReadRequest) {
        self.read_requests
            .borrow_mut()
            .push_back(read_request.clone());
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub(crate) fn get_num_read_requests(&self) -> usize {
        self.read_requests.borrow().len()
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub(crate) fn error(&self, e: SafeHandleValue, can_gc: CanGc) {
        // Reject reader.[[closedPromise]] with e.
        self.closed_promise.borrow().reject_native(&e, can_gc);

        // Set reader.[[closedPromise]].[[PromiseIsHandled]] to true.
        self.closed_promise.borrow().set_promise_is_handled();

        // Perform ! ReadableStreamDefaultReaderErrorReadRequests(reader, e).
        self.error_read_requests(e, can_gc);
    }

    /// The removal steps of <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    pub(crate) fn remove_read_request(&self) -> ReadRequest {
        self.read_requests
            .borrow_mut()
            .pop_front()
            .expect("Reader must have read request when remove is called into.")
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease>
    pub(crate) fn release(&self, can_gc: CanGc) -> Fallible<()> {
        // Perform ! ReadableStreamReaderGenericRelease(reader).
        self.generic_release(can_gc)
            .expect("Generic release failed");
        // Let e be a new TypeError exception.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error = UndefinedValue());
        Error::Type("Reader is released".to_owned()).to_jsval(
            cx,
            &self.global(),
            error.handle_mut(),
            can_gc,
        );

        // Perform ! ReadableStreamDefaultReaderErrorReadRequests(reader, e).
        self.error_read_requests(error.handle(), can_gc);
        Ok(())
    }

    fn take_read_requests(&self) -> VecDeque<ReadRequest> {
        mem::take(&mut *self.read_requests.borrow_mut())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreadererrorreadrequests>
    fn error_read_requests(&self, rval: SafeHandleValue, can_gc: CanGc) {
        // step 1
        let mut read_requests = self.take_read_requests();

        // step 2 & 3
        for request in read_requests.drain(0..) {
            request.error_steps(rval, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub(crate) fn read(&self, cx: SafeJSContext, read_request: &ReadRequest, can_gc: CanGc) {
        // Let stream be reader.[[stream]].

        // Assert: stream is not undefined.
        assert!(self.stream.get().is_some());

        let stream = self.stream.get().unwrap();

        // Set stream.[[disturbed]] to true.
        stream.set_is_disturbed(true);
        // If stream.[[state]] is "closed", perform readRequest’s close steps.
        if stream.is_closed() {
            read_request.close_steps(can_gc);
        } else if stream.is_errored() {
            // Otherwise, if stream.[[state]] is "errored",
            // perform readRequest’s error steps given stream.[[storedError]].
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            read_request.error_steps(error.handle(), can_gc);
        } else {
            // Otherwise
            // Assert: stream.[[state]] is "readable".
            assert!(stream.is_readable());
            // Perform ! stream.[[controller]].[[PullSteps]](readRequest).
            stream.perform_pull_steps(cx, read_request, can_gc);
        }
    }

    /// Attach the byte-tee error handler to this reader's closedPromise.
    /// Used by ReadableByteStreamTee.
    pub(crate) fn byte_tee_append_native_handler_to_closed_promise(
        &self,
        branch_1: &ReadableStream,
        branch_2: &ReadableStream,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        cancel_promise: Rc<Promise>,
        reader_version: Rc<Cell<u64>>,
        expected_version: u64,
        can_gc: CanGc,
    ) {
        // Note: for byte tee we always operate on *byte controllers*.
        let branch_1_controller = branch_1.get_byte_controller();
        let branch_2_controller = branch_2.get_byte_controller();

        let global = self.global();
        let handler = PromiseNativeHandler::new(
            &global,
            None,
            Some(Box::new(ByteTeeClosedPromiseRejectionHandler {
                branch_1_controller: Dom::from_ref(&branch_1_controller),
                branch_2_controller: Dom::from_ref(&branch_2_controller),
                canceled_1,
                canceled_2,
                cancel_promise,
                reader_version,
                expected_version,
            })),
            can_gc,
        );

        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);

        self.closed_promise
            .borrow()
            .append_native_handler(&handler, comp, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#ref-for-readablestreamgenericreader-closedpromise%E2%91%A1>
    pub(crate) fn default_tee_append_native_handler_to_closed_promise(
        &self,
        branch_1: &ReadableStream,
        branch_2: &ReadableStream,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        cancel_promise: Rc<Promise>,
        can_gc: CanGc,
    ) {
        let branch_1_controller = branch_1.get_default_controller();

        let branch_2_controller = branch_2.get_default_controller();

        let global = self.global();
        let handler = PromiseNativeHandler::new(
            &global,
            None,
            Some(Box::new(DefaultTeeClosedPromiseRejectionHandler {
                branch_1_controller: Dom::from_ref(&branch_1_controller),
                branch_2_controller: Dom::from_ref(&branch_2_controller),
                canceled_1,
                canceled_2,
                cancel_promise,
            })),
            can_gc,
        );

        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);

        self.closed_promise
            .borrow()
            .append_native_handler(&handler, comp, can_gc);
    }

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultreader-read-all-bytes>
    pub(crate) fn read_all_bytes(
        &self,
        cx: SafeJSContext,
        success_steps: Rc<ReadAllBytesSuccessSteps>,
        failure_steps: Rc<ReadAllBytesFailureSteps>,
        can_gc: CanGc,
    ) {
        // To read all bytes from a ReadableStreamDefaultReader reader,
        // given successSteps, which is an algorithm accepting a byte sequence,
        // and failureSteps, which is an algorithm accepting a JavaScript value:
        // read-loop given reader, a new byte sequence, successSteps, and failureSteps.
        read_loop(self, cx, success_steps, failure_steps, can_gc);
    }

    /// step 3 of <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamcontrollerprocessreadrequestsusingqueue>
    pub(crate) fn process_read_requests(
        &self,
        cx: SafeJSContext,
        controller: DomRoot<ReadableByteStreamController>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // While reader.[[readRequests]] is not empty,
        while !self.read_requests.borrow().is_empty() {
            // If controller.[[queueTotalSize]] is 0, return.
            if controller.get_queue_total_size() == 0.0 {
                return Ok(());
            }

            // Let readRequest be reader.[[readRequests]][0].
            // Remove entry from controller.[[queue]].
            let read_request = self.remove_read_request();

            // Perform ! ReadableByteStreamControllerFillReadRequestFromQueue(controller, readRequest).
            controller
                .fill_read_request_from_queue(cx, &read_request, can_gc)
                .expect("Fill read request from queue failed");
        }
        Ok(())
    }
}

impl ReadableStreamDefaultReaderMethods<crate::DomTypeHolder> for ReadableStreamDefaultReader {
    /// <https://streams.spec.whatwg.org/#default-reader-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        stream: &ReadableStream,
    ) -> Fallible<DomRoot<Self>> {
        let reader = Self::new_with_proto(global, proto, can_gc);

        // Perform ? SetUpReadableStreamDefaultReader(this, stream).
        Self::set_up(&reader, stream, global, can_gc)?;

        Ok(reader)
    }

    /// <https://streams.spec.whatwg.org/#default-reader-read>
    fn Read(&self, can_gc: CanGc) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        // If this.[[stream]] is undefined, return a promise rejected with a TypeError exception.
        if self.stream.get().is_none() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            Error::Type("stream is undefined".to_owned()).to_jsval(
                cx,
                &self.global(),
                error.handle_mut(),
                can_gc,
            );
            return Promise::new_rejected(&self.global(), cx, error.handle(), can_gc);
        }
        // Let promise be a new promise.
        let promise = Promise::new(&self.global(), can_gc);

        // Let readRequest be a new read request with the following items:
        // chunk steps, given chunk
        // Resolve promise with «[ "value" → chunk, "done" → false ]».
        //
        // close steps
        // Resolve promise with «[ "value" → undefined, "done" → true ]».
        //
        // error steps, given e
        // Reject promise with e.

        // Rooting(unrooted_must_root): the read request contains only a promise,
        // which does not need to be rooted,
        // as it is safely managed natively via an Rc.
        let read_request = ReadRequest::Read(promise.clone());

        // Perform ! ReadableStreamDefaultReaderRead(this, readRequest).
        self.read(cx, &read_request, can_gc);

        // Return promise.
        promise
    }

    /// <https://streams.spec.whatwg.org/#default-reader-release-lock>
    fn ReleaseLock(&self, can_gc: CanGc) -> Fallible<()> {
        if self.stream.get().is_none() {
            // Step 1: If this.[[stream]] is undefined, return.
            return Ok(());
        }

        // Step 2: Perform !ReadableStreamDefaultReaderRelease(this).
        self.release(can_gc)
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        self.closed()
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        self.generic_cancel(cx, &self.global(), reason, can_gc)
    }
}

impl ReadableStreamGenericReader for ReadableStreamDefaultReader {
    fn get_closed_promise(&self) -> Rc<Promise> {
        self.closed_promise.borrow().clone()
    }

    fn set_closed_promise(&self, promise: Rc<Promise>) {
        *self.closed_promise.borrow_mut() = promise;
    }

    fn set_stream(&self, stream: Option<&ReadableStream>) {
        self.stream.set(stream);
    }

    fn get_stream(&self) -> Option<DomRoot<ReadableStream>> {
        self.stream.get()
    }

    fn as_default_reader(&self) -> Option<&ReadableStreamDefaultReader> {
        Some(self)
    }
}
