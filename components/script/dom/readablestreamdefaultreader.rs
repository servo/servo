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

use super::bindings::root::MutNullableDom;
use super::types::ReadableStreamDefaultController;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::{
    ReadableStreamDefaultReaderMethods, ReadableStreamReadResult,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::defaultteereadrequest::DefaultTeeReadRequest;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::ReadableStream;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#read-request>
#[derive(JSTraceable)]
#[crown::unrooted_must_root_lint::must_root]
pub enum ReadRequest {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    Read(Rc<Promise>),
    /// <https://streams.spec.whatwg.org/#ref-for-read-request%E2%91%A2>
    DefaultTee {
        tee_read_request: Dom<DefaultTeeReadRequest>,
    },
}

impl ReadRequest {
    /// <https://streams.spec.whatwg.org/#read-request-chunk-steps>
    pub fn chunk_steps(&self, chunk: RootedTraceableBox<Heap<JSVal>>) {
        match self {
            ReadRequest::Read(promise) => {
                promise.resolve_native(&ReadableStreamReadResult {
                    done: Some(false),
                    value: chunk,
                });
            },
            ReadRequest::DefaultTee { tee_read_request } => {
                tee_read_request.enqueue_chunk_steps(chunk);
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#read-request-close-steps>
    pub fn close_steps(&self) {
        match self {
            ReadRequest::Read(promise) => {
                let result = RootedTraceableBox::new(Heap::default());
                result.set(UndefinedValue());
                promise.resolve_native(&ReadableStreamReadResult {
                    done: Some(true),
                    value: result,
                });
            },
            ReadRequest::DefaultTee { tee_read_request } => {
                tee_read_request.close_steps();
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#read-request-error-steps>
    pub fn error_steps(&self, e: SafeHandleValue) {
        match self {
            ReadRequest::Read(promise) => promise.reject_native(&e),
            ReadRequest::DefaultTee { tee_read_request } => {
                tee_read_request.error_steps();
            },
        }
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-stream-tee>
#[derive(Clone, JSTraceable, MallocSizeOf)]
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
    /// Upon rejection of reader.[[closedPromise]] with reason r,
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, _can_gc: CanGc) {
        let branch_1_controller = &self.branch_1_controller;
        let branch_2_controller = &self.branch_2_controller;

        // Perform ! ReadableStreamDefaultControllerError(branch_1.[[controller]], r).
        branch_1_controller.error(v);
        // Perform ! ReadableStreamDefaultControllerError(branch_2.[[controller]], r).
        branch_2_controller.error(v);

        // If canceled_1 is false or canceled_2 is false, resolve cancelPromise with undefined.
        if !self.canceled_1.get() || !self.canceled_2.get() {
            self.cancel_promise.resolve_native(&());
        }
    }
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
#[dom_struct]
pub struct ReadableStreamDefaultReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
    stream: MutNullableDom<ReadableStream>,

    #[ignore_malloc_size_of = "no VecDeque support"]
    read_requests: DomRefCell<VecDeque<ReadRequest>>,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-closedpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    closed_promise: DomRefCell<Rc<Promise>>,
}

impl ReadableStreamDefaultReader {
    /// <https://streams.spec.whatwg.org/#default-reader-constructor>
    #[allow(non_snake_case)]
    pub fn Constructor(
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

    pub fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> ReadableStreamDefaultReader {
        ReadableStreamDefaultReader {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            read_requests: DomRefCell::new(Default::default()),
            closed_promise: DomRefCell::new(Promise::new(global, can_gc)),
        }
    }

    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-reader>
    pub fn set_up(
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

        self.generic_initialize(global, stream, can_gc)?;

        // Set reader.[[readRequests]] to a new empty list.
        self.read_requests.borrow_mut().clear();

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-initialize>
    pub fn generic_initialize(
        &self,
        global: &GlobalScope,
        stream: &ReadableStream,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Set reader.[[stream]] to stream.
        self.stream.set(Some(stream));

        // Set stream.[[reader]] to reader.
        stream.set_reader(Some(self));

        if stream.is_readable() {
            // If stream.[[state]] is "readable
            // Set reader.[[closedPromise]] to a new promise.
            *self.closed_promise.borrow_mut() = Promise::new(global, can_gc);
        } else if stream.is_closed() {
            // Otherwise, if stream.[[state]] is "closed",
            // Set reader.[[closedPromise]] to a promise resolved with undefined.
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            *self.closed_promise.borrow_mut() = Promise::new_resolved(global, cx, rval.handle())?;
        } else {
            // Assert: stream.[[state]] is "errored"
            assert!(stream.is_errored());

            // Set reader.[[closedPromise]] to a promise rejected with stream.[[storedError]].
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            *self.closed_promise.borrow_mut() = Promise::new_rejected(global, cx, error.handle())?;

            // Set reader.[[closedPromise]].[[PromiseIsHandled]] to true
            self.closed_promise.borrow().set_promise_is_handled();
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub fn close(&self) {
        // Resolve reader.[[closedPromise]] with undefined.
        self.closed_promise.borrow().resolve_native(&());
        // If reader implements ReadableStreamDefaultReader,
        // Let readRequests be reader.[[readRequests]].
        let mut read_requests = self.take_read_requests();
        // Set reader.[[readRequests]] to an empty list.
        // For each readRequest of readRequests,
        for request in read_requests.drain(0..) {
            // Perform readRequest’s close steps.
            request.close_steps();
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub fn add_read_request(&self, read_request: ReadRequest) {
        self.read_requests.borrow_mut().push_back(read_request);
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub fn get_num_read_requests(&self) -> usize {
        self.read_requests.borrow().len()
    }

    /// steps 6, 7, 8 of <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error(&self, e: SafeHandleValue) {
        // step 6
        self.closed_promise.borrow().reject_native(&e);

        // step 7
        self.closed_promise.borrow().set_promise_is_handled();

        // step 8
        self.error_read_requests(e);
    }

    /// The removal steps of <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    pub fn remove_read_request(&self) -> ReadRequest {
        self.read_requests
            .borrow_mut()
            .pop_front()
            .expect("Reader must have read request when remove is called into.")
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-release>
    #[allow(unsafe_code)]
    pub fn generic_release(&self) {
        // Let stream be reader.[[stream]].

        // Assert: stream is not undefined.
        assert!(self.stream.get().is_some());

        if let Some(stream) = self.stream.get() {
            // Assert: stream.[[reader]] is reader.
            assert!(stream.has_default_reader());

            if stream.is_readable() {
                // If stream.[[state]] is "readable", reject reader.[[closedPromise]] with a TypeError exception.
                self.closed_promise
                    .borrow()
                    .reject_error(Error::Type("stream state is not readable".to_owned()));
            } else {
                // Otherwise, set reader.[[closedPromise]] to a promise rejected with a TypeError exception.
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut error = UndefinedValue());
                unsafe {
                    Error::Type("Cannot release lock due to stream state.".to_owned()).to_jsval(
                        *cx,
                        &self.global(),
                        error.handle_mut(),
                    )
                };

                *self.closed_promise.borrow_mut() =
                    Promise::new_rejected(&self.global(), cx, error.handle()).unwrap();
            }
            // Set reader.[[closedPromise]].[[PromiseIsHandled]] to true.
            self.closed_promise.borrow().set_promise_is_handled();

            // Perform ! stream.[[controller]].[[ReleaseSteps]]().
            stream.perform_release_steps();

            // Set stream.[[reader]] to undefined.
            stream.set_reader(None);
            // Set reader.[[stream]] to undefined.
            self.stream.set(None);
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease>
    #[allow(unsafe_code)]
    pub fn release(&self) {
        // Perform ! ReadableStreamReaderGenericRelease(reader).
        self.generic_release();
        // Let e be a new TypeError exception.
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error = UndefinedValue());
        unsafe {
            Error::Type("Reader is released".to_owned()).to_jsval(
                *cx,
                &self.global(),
                error.handle_mut(),
            )
        };

        // Perform ! ReadableStreamDefaultReaderErrorReadRequests(reader, e).
        self.error_read_requests(error.handle());
    }

    fn take_read_requests(&self) -> VecDeque<ReadRequest> {
        mem::take(&mut *self.read_requests.borrow_mut())
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-cancel>
    fn generic_cancel(&self, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        // Let stream be reader.[[stream]].
        let stream = self.stream.get();

        // Assert: stream is not undefined.
        let stream =
            stream.expect("Reader should have a stream when generic cancel is called into.");

        // Return ! ReadableStreamCancel(stream, reason).
        stream.cancel(reason, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreadererrorreadrequests>
    fn error_read_requests(&self, rval: SafeHandleValue) {
        // step 1
        let mut read_requests = self.take_read_requests();

        // step 2 & 3
        for request in read_requests.drain(0..) {
            request.error_steps(rval);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub fn read(&self, read_request: ReadRequest) {
        // Let stream be reader.[[stream]].

        // Assert: stream is not undefined.
        assert!(self.stream.get().is_some());

        let stream = self.stream.get().unwrap();

        // Set stream.[[disturbed]] to true.
        stream.set_is_disturbed(true);
        // If stream.[[state]] is "closed", perform readRequest’s close steps.
        if stream.is_closed() {
            read_request.close_steps();
        } else if stream.is_errored() {
            // Otherwise, if stream.[[state]] is "errored",
            // perform readRequest’s error steps given stream.[[storedError]].
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            read_request.error_steps(error.handle());
        } else {
            // Otherwise
            // Assert: stream.[[state]] is "readable".
            assert!(stream.is_readable());
            // Perform ! stream.[[controller]].[[PullSteps]](readRequest).
            stream.perform_pull_steps(read_request);
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-readablestreamgenericreader-closedpromise%E2%91%A1>
    pub fn append_native_handler_to_closed_promise(
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
        let rejection_handler = Box::new(ClosedPromiseRejectionHandler {
            branch_1_controller: Dom::from_ref(&branch_1_controller),
            branch_2_controller: Dom::from_ref(&branch_2_controller),
            canceled_1,
            canceled_2,
            cancel_promise,
        });
        let handler = PromiseNativeHandler::new(&global, None, Some(rejection_handler));

        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);

        self.closed_promise
            .borrow()
            .append_native_handler(&handler, comp, can_gc);
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
        ReadableStreamDefaultReader::Constructor(global, proto, can_gc, stream)
    }

    /// <https://streams.spec.whatwg.org/#default-reader-read>
    #[allow(unsafe_code)]
    fn Read(&self, can_gc: CanGc) -> Rc<Promise> {
        // If this.[[stream]] is undefined, return a promise rejected with a TypeError exception.
        if self.stream.get().is_none() {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            unsafe {
                Error::Type("stream is undefined".to_owned()).to_jsval(
                    *cx,
                    &self.global(),
                    error.handle_mut(),
                )
            };
            return Promise::new_rejected(&self.global(), cx, error.handle()).unwrap();
        }
        // Let promise be a new promise.
        let promise = Promise::new(&self.reflector_.global(), can_gc);

        // Let readRequest be a new read request with the following items:
        // chunk steps, given chunk
        // Resolve promise with «[ "value" → chunk, "done" → false ]».
        //
        // close steps
        // Resolve promise with «[ "value" → undefined, "done" → true ]».
        //
        // error steps, given e
        // Reject promise with e.
        let read_request = ReadRequest::Read(promise.clone());

        // Perform ! ReadableStreamDefaultReaderRead(this, readRequest).
        self.read(read_request);

        // Return promise.
        promise
    }

    /// <https://streams.spec.whatwg.org/#default-reader-release-lock>
    fn ReleaseLock(&self) {
        if self.stream.get().is_some() {
            // step 2 - Perform ! ReadableStreamDefaultReaderRelease(this).
            self.release();
        }
        // step 1 - If this.[[stream]] is undefined, return.
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        self.closed_promise.borrow().clone()
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        if self.stream.get().is_none() {
            // If this.[[stream]] is undefined,
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&self.reflector_.global(), can_gc);
            promise.reject_error(Error::Type("stream is undefined".to_owned()));
            promise
        } else {
            // Return ! ReadableStreamReaderGenericCancel(this, reason).
            self.generic_cancel(reason, can_gc)
        }
    }
}
