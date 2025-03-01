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
use super::types::ReadableStreamDefaultController;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::{
    ReadableStreamDefaultReaderMethods, ReadableStreamReadResult,
};
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::defaultteereadrequest::DefaultTeeReadRequest;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::{get_read_promise_bytes, get_read_promise_done, ReadableStream};
use crate::dom::readablestreamgenericreader::ReadableStreamGenericReader;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

type ReadAllBytesSuccessSteps = dyn Fn(&[u8]);
type ReadAllBytesFailureSteps = dyn Fn(SafeJSContext, SafeHandleValue);

impl js::gc::Rootable for ReadLoopFulFillmentHandler {}

/// <https://streams.spec.whatwg.org/#read-loop>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ReadLoopFulFillmentHandler {
    #[ignore_malloc_size_of = "Rc is hard"]
    #[no_trace]
    success_steps: Rc<ReadAllBytesSuccessSteps>,

    #[ignore_malloc_size_of = "Rc is hard"]
    #[no_trace]
    failure_steps: Rc<ReadAllBytesFailureSteps>,

    reader: Dom<ReadableStreamDefaultReader>,

    #[ignore_malloc_size_of = "Rc is hard"]
    bytes: Rc<DomRefCell<Vec<u8>>>,
}

impl Callback for ReadLoopFulFillmentHandler {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        let global = self.reader.global();
        let is_done = match get_read_promise_done(cx, &v) {
            Ok(is_done) => is_done,
            Err(err) => {
                self.reader
                    .release(can_gc)
                    .expect("Releasing the reader should succeed");
                rooted!(in(*cx) let mut v = UndefinedValue());
                err.to_jsval(cx, &global, v.handle_mut());
                (self.failure_steps)(cx, v.handle());
                return;
            },
        };

        if is_done {
            // <https://streams.spec.whatwg.org/#ref-for-read-request-close-steps%E2%91%A6>
            // Call successSteps with bytes.
            (self.success_steps)(&self.bytes.borrow());
            self.reader
                .release(can_gc)
                .expect("Releasing the reader should succeed");
        } else {
            // <https://streams.spec.whatwg.org/#ref-for-read-request-chunk-steps%E2%91%A6>
            let chunk = match get_read_promise_bytes(cx, &v) {
                Ok(chunk) => chunk,
                Err(err) => {
                    //  If chunk is not a Uint8Array object, call failureSteps with a TypeError and abort these steps.
                    rooted!(in(*cx) let mut v = UndefinedValue());
                    err.to_jsval(cx, &global, v.handle_mut());
                    (self.failure_steps)(cx, v.handle());
                    self.reader
                        .release(can_gc)
                        .expect("Releasing the reader should succeed");
                    return;
                },
            };

            // Append the bytes represented by chunk to bytes.
            let mut bytes = self.bytes.borrow_mut();
            bytes.extend_from_slice(&chunk);

            // Read-loop given reader, bytes, successSteps, and failureSteps.
            read_loop(
                &global,
                &mut Some(self.clone()),
                Box::new(ReadLoopRejectionHandler {
                    failure_steps: self.failure_steps.clone(),
                }),
                realm,
                can_gc,
            );
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
/// <https://streams.spec.whatwg.org/#readablestreamdefaultreader-read-all-bytes>
struct ReadLoopRejectionHandler {
    #[ignore_malloc_size_of = "Rc is hard"]
    #[no_trace]
    failure_steps: Rc<ReadAllBytesFailureSteps>,
}

impl Callback for ReadLoopRejectionHandler {
    /// <https://streams.spec.whatwg.org/#ref-for-read-request-error-steps%E2%91%A6>
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, _can_gc: CanGc) {
        // Call failureSteps with e.
        (self.failure_steps)(cx, v);
    }
}

/// <https://streams.spec.whatwg.org/#read-loop>
fn read_loop(
    global: &GlobalScope,
    fulfillment_handler: &mut Option<ReadLoopFulFillmentHandler>,
    rejection_handler: Box<ReadLoopRejectionHandler>,
    realm: InRealm,
    can_gc: CanGc,
) {
    // Let readRequest be a new read request with the following items:
    // Note: the custom read request logic is implemented
    // using a native promise handler attached to the promise returned by `Read`
    // (which internally uses a default read request).

    // Perform ! ReadableStreamDefaultReaderRead(reader, readRequest).
    let read_promise = fulfillment_handler
        .as_ref()
        .expect("Fulfillment handler should be some.")
        .reader
        .Read(can_gc);

    let handler = PromiseNativeHandler::new(
        global,
        fulfillment_handler.take().map(|h| Box::new(h) as Box<_>),
        Some(rejection_handler),
        can_gc,
    );
    read_promise.append_native_handler(&handler, realm, can_gc);
}

/// <https://streams.spec.whatwg.org/#read-request>
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum ReadRequest {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    Read(#[ignore_malloc_size_of = "Rc is hard"] Rc<Promise>),
    /// <https://streams.spec.whatwg.org/#ref-for-read-request%E2%91%A2>
    DefaultTee {
        tee_read_request: Dom<DefaultTeeReadRequest>,
    },
}

impl ReadRequest {
    /// <https://streams.spec.whatwg.org/#read-request-chunk-steps>
    pub(crate) fn chunk_steps(&self, chunk: RootedTraceableBox<Heap<JSVal>>, can_gc: CanGc) {
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
        }
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-stream-tee>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ClosedPromiseRejectionHandler {
    branch_1_controller: Dom<ReadableStreamDefaultController>,
    branch_2_controller: Dom<ReadableStreamDefaultController>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_1: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    canceled_2: Rc<Cell<bool>>,
    #[ignore_malloc_size_of = "Rc"]
    cancel_promise: Rc<Promise>,
}

impl Callback for ClosedPromiseRejectionHandler {
    /// Continuation of <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
    /// Upon rejection of `reader.closedPromise` with reason `r``,
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        let branch_1_controller = &self.branch_1_controller;
        let branch_2_controller = &self.branch_2_controller;

        // Perform ! ReadableStreamDefaultControllerError(branch_1.[[controller]], r).
        branch_1_controller.error(v, can_gc);
        // Perform ! ReadableStreamDefaultControllerError(branch_2.[[controller]], r).
        branch_2_controller.error(v, can_gc);

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
    #[ignore_malloc_size_of = "Rc is hard"]
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
        self.generic_release(can_gc)?;
        // Let e be a new TypeError exception.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error = UndefinedValue());
        Error::Type("Reader is released".to_owned()).to_jsval(
            cx,
            &self.global(),
            error.handle_mut(),
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
    pub(crate) fn read(&self, read_request: &ReadRequest, can_gc: CanGc) {
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
            stream.perform_pull_steps(read_request, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-readablestreamgenericreader-closedpromise%E2%91%A1>
    pub(crate) fn append_native_handler_to_closed_promise(
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
            Some(Box::new(ClosedPromiseRejectionHandler {
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
        global: &GlobalScope,
        success_steps: Rc<ReadAllBytesSuccessSteps>,
        failure_steps: Rc<ReadAllBytesFailureSteps>,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // To read all bytes from a ReadableStreamDefaultReader reader,
        // given successSteps, which is an algorithm accepting a byte sequence,
        // and failureSteps, which is an algorithm accepting a JavaScript value:
        // read-loop given reader, a new byte sequence, successSteps, and failureSteps.
        // Note: read-loop done using native promise handlers.
        rooted!(in(*cx) let mut fulfillment_handler = Some(ReadLoopFulFillmentHandler {
            success_steps,
            failure_steps: failure_steps.clone(),
            reader: Dom::from_ref(self),
            bytes: Rc::new(DomRefCell::new(Vec::new())),
        }));
        let rejection_handler = Box::new(ReadLoopRejectionHandler { failure_steps });
        read_loop(
            global,
            &mut fulfillment_handler,
            rejection_handler,
            realm,
            can_gc,
        );
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
        // If this.[[stream]] is undefined, return a promise rejected with a TypeError exception.
        if self.stream.get().is_none() {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            Error::Type("stream is undefined".to_owned()).to_jsval(
                cx,
                &self.global(),
                error.handle_mut(),
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
        self.read(&read_request, can_gc);

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
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        self.cancel(&self.global(), reason, can_gc)
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
