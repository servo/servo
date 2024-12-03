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
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, HandleValue};

use super::bindings::refcounted::Trusted;
use super::bindings::root::MutNullableDom;
use super::bindings::structuredclone;
use super::types::ReadableStreamDefaultController;
use super::underlyingsourcecontainer::TeeUnderlyingSource;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::{
    ReadableStreamDefaultReaderMethods, ReadableStreamReadResult,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::ReadableStream;
use crate::microtask::Microtask;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#read-request>
/// For now only one variant: the one matching a `read` call.
#[derive(JSTraceable)]
pub enum ReadRequest {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    Read(Rc<Promise>),
    /// <https://streams.spec.whatwg.org/#ref-for-read-request%E2%91%A2>
    Tee { tee_read_request: TeeReadRequest },
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct TeeReadRequestMicrotask {
    #[ignore_malloc_size_of = "mozjs"]
    chunk: RootedTraceableBox<Heap<JSVal>>,
    tee_read_request: Dom<TeeReadRequest>,
}

impl TeeReadRequestMicrotask {
    pub fn microtask_chunk_steps(&self) {
        self.tee_read_request.chunk_steps(&self.chunk)
    }
}

#[dom_struct]
pub struct TeeReadRequest {
    stream: Dom<ReadableStream>,
    #[ignore_malloc_size_of = "Rc"]
    branch_1: MutNullableDom<ReadableStream>,
    #[ignore_malloc_size_of = "Rc"]
    branch_2: MutNullableDom<ReadableStream>,
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
    #[ignore_malloc_size_of = "Rc"]
    tee_underlying_source: Dom<TeeUnderlyingSource>,
}

impl TeeReadRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stream: Dom<ReadableStream>,
        branch_1: MutNullableDom<ReadableStream>,
        branch_2: MutNullableDom<ReadableStream>,
        reading: Rc<Cell<bool>>,
        read_again: Rc<Cell<bool>>,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        clone_for_branch_2: Rc<Cell<bool>>,
        cancel_promise: Rc<Promise>,
        tee_underlying_source: Dom<TeeUnderlyingSource>,
    ) -> Self {
        TeeReadRequest {
            stream,
            branch_1,
            branch_2,
            reading,
            read_again,
            canceled_1,
            canceled_2,
            clone_for_branch_2,
            cancel_promise,
            tee_underlying_source,
        }
    }

    /// Call into error of the default controller of branch_1,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-error>
    pub fn branch_1_default_controller_error(&self, error: SafeHandleValue) {
        self.branch_1
            .get()
            .expect("branch_1 must be set")
            .get_default_controller()
            .error(error);
    }

    /// Call into error of the default controller of branch_2,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-error>
    pub fn branch_2_default_controller_error(&self, error: SafeHandleValue) {
        self.branch_2
            .get()
            .expect("branch_2 must be set")
            .get_default_controller()
            .error(error);
    }

    /// Call into cancel of the stream,
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>
    pub fn stream_cancel(&self, reason: SafeHandleValue, can_gc: CanGc) {
        self.stream.cancel(reason, can_gc);
    }

    /// Enqueue a microtask to perform the chunk steps
    /// <https://streams.spec.whatwg.org/#ref-for-read-request-chunk-steps%E2%91%A2>
    pub fn enqueue_chunk_steps(&self, chunk: RootedTraceableBox<Heap<JSVal>>) {
        // Queue a microtask to perform the following steps:
        let tee_read_request_chunk = TeeReadRequestMicrotask {
            chunk,
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
    pub fn chunk_steps(&self, chunk: &RootedTraceableBox<Heap<JSVal>>) {
        // Set readAgain to false.
        self.read_again.set(false);
        // Let chunk1 and chunk2 be chunk.
        let chunk1 = chunk;
        let chunk2 = chunk;

        // If canceled_2 is false and cloneForBranch2 is true,
        if !self.canceled_2.get() && self.clone_for_branch_2.get() {
            let cx = GlobalScope::get_cx();
            // Let cloneResult be StructuredClone(chunk2).
            rooted!(in(*cx) let mut clone_result = UndefinedValue());
            let data = structuredclone::write(cx, chunk2.handle(), None).unwrap();

            // If cloneResult is an abrupt completion,
            if structuredclone::read(&self.stream.global(), data, clone_result.handle_mut())
                .is_err()
            {
                // Perform ! ReadableStreamDefaultControllerError(branch_1.[[controller]], cloneResult.[[Value]]).
                self.branch_1_default_controller_error(clone_result.handle());
                // Perform ! ReadableStreamDefaultControllerError(branch_2.[[controller]], cloneResult.[[Value]]).
                self.branch_2_default_controller_error(clone_result.handle());
                // Resolve cancelPromise with ! ReadableStreamCancel(stream, cloneResult.[[Value]]).
                self.stream_cancel(clone_result.handle(), CanGc::note());

                // Return.
                return;
            } else {
                // Otherwise, set chunk2 to cloneResult.[[Value]].
                chunk2.set(*clone_result);
            }
        }

        // If canceled_1 is false, perform ! ReadableStreamDefaultControllerEnqueue(branch_1.[[controller]], chunk1).
        if !self.canceled_1.get() {
            self.branch_1_default_controller_enqueue(chunk1.handle());
        }
        // If canceled_2 is false, perform ! ReadableStreamDefaultControllerEnqueue(branch_2.[[controller]], chunk2).
        if !self.canceled_2.get() {
            self.branch_2_default_controller_enqueue(chunk2.handle());
        }
        // Set reading to false.
        self.reading.set(false);

        // If readAgain is true, perform pullAlgorithm.
        if self.read_again.get() {
            self.pull_algorithm();
        }
    }

    /// <https://streams.spec.whatwg.org/#read-request-close-steps>
    pub fn close_steps(&self) {
        // Set reading to false.
        self.reading.set(false);

        // If canceled_1 is false, perform ! ReadableStreamDefaultControllerClose(branch_1.[[controller]]).
        if !self.canceled_1.get() {
            self.branch_1_default_controller_close();
        }
        // If canceled_2 is false, perform ! ReadableStreamDefaultControllerClose(branch_2.[[controller]]).
        if !self.canceled_2.get() {
            self.branch_2_default_controller_close();
        }
        // If canceled_1 is false or canceled_2 is false, resolve cancelPromise with undefined.
        if !self.canceled_1.get() || !self.canceled_2.get() {
            self.cancel_promise.resolve_native(&());
        }
    }

    /// <https://streams.spec.whatwg.org/#read-request-error-steps>
    pub fn error_steps(&self) {
        // Set reading to false.
        self.reading.set(false);
    }

    /// Call into enqueue of the default controller of branch_1,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    pub fn branch_1_default_controller_enqueue(&self, chunk: SafeHandleValue) {
        let _ = self
            .branch_1
            .get()
            .expect("branch_1 must be set")
            .get_default_controller()
            .enqueue(GlobalScope::get_cx(), chunk, CanGc::note());
    }

    /// Call into enqueue of the default controller of branch_2,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    pub fn branch_2_default_controller_enqueue(&self, chunk: SafeHandleValue) {
        let _ = self
            .branch_2
            .get()
            .expect("branch_2 must be set")
            .get_default_controller()
            .enqueue(GlobalScope::get_cx(), chunk, CanGc::note());
    }

    /// Call into close of the default controller of branch_1,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-close>
    pub fn branch_1_default_controller_close(&self) {
        self.branch_1
            .get()
            .expect("branch_1 must be set")
            .get_default_controller()
            .close();
    }

    /// Call into close of the default controller of branch_2,
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-close>
    pub fn branch_2_default_controller_close(&self) {
        self.branch_2
            .get()
            .expect("branch_2 must be set")
            .get_default_controller()
            .close();
    }

    pub fn pull_algorithm(&self) {
        self.tee_underlying_source.pull_algorithm();
    }
}

impl ReadRequest {
    /// <https://streams.spec.whatwg.org/#read-request-chunk-steps>
    #[allow(unsafe_code)]
    pub fn chunk_steps(&self, chunk: RootedTraceableBox<Heap<JSVal>>) {
        match self {
            ReadRequest::Read(promise) => {
                promise.resolve_native(&ReadableStreamReadResult {
                    done: Some(false),
                    value: chunk,
                });
            },
            ReadRequest::Tee { tee_read_request } => {
                tee_read_request.enqueue_chunk_steps(chunk);
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-close-step>
    pub fn close_steps(&self) {
        match self {
            ReadRequest::Read(promise) => {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                let result = RootedTraceableBox::new(Heap::default());
                result.set(*rval);
                promise.resolve_native(&ReadableStreamReadResult {
                    done: Some(true),
                    value: result,
                });
            },
            ReadRequest::Tee { tee_read_request } => {
                tee_read_request.close_steps();
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-close-step>
    pub fn error_steps(&self, e: SafeHandleValue) {
        match self {
            ReadRequest::Read(promise) => promise.reject_native(&e),
            ReadRequest::Tee { tee_read_request } => {
                tee_read_request.error_steps();
            },
        }
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-stream-tee>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(crown::unrooted_must_root)]
struct ClosedPromiseRejectionHandler {
    #[ignore_malloc_size_of = "Trusted are hard"]
    branch_1_controller: Trusted<ReadableStreamDefaultController>,
    #[ignore_malloc_size_of = "Trusted are hard"]
    branch_2_controller: Trusted<ReadableStreamDefaultController>,
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
    fn callback(&self, _cx: SafeJSContext, v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
        let branch_1_controller = self.branch_1_controller.root();
        let branch_2_controller = self.branch_1_controller.root();

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

    #[ignore_malloc_size_of = "Rc is hard"]
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
        _proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        stream: &ReadableStream,
    ) -> Fallible<DomRoot<Self>> {
        // step 1
        Self::set_up(global, stream, can_gc)
    }

    fn new_inherited(
        stream: &ReadableStream,
        closed_promise: Rc<Promise>,
    ) -> ReadableStreamDefaultReader {
        ReadableStreamDefaultReader {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(Some(stream)),
            read_requests: DomRefCell::new(Default::default()),
            closed_promise: DomRefCell::new(closed_promise),
        }
    }

    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-reader>
    pub fn set_up(
        global: &GlobalScope,
        stream: &ReadableStream,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStreamDefaultReader>> {
        // step 1
        if stream.is_locked() {
            return Err(Error::Type("stream is locked".to_owned()));
        }
        // step 2 & 3
        Ok(Self::generic_initialize(global, stream, can_gc))
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-initialize>
    pub fn generic_initialize(
        global: &GlobalScope,
        stream: &ReadableStream,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStreamDefaultReader> {
        let promise;
        if stream.is_readable() {
            // If stream.[[state]] is "readable
            // Set reader.[[closedPromise]] to a new promise.
            promise = Promise::new(global, can_gc);
        } else if stream.is_closed() {
            // Otherwise, if stream.[[state]] is "closed",
            // Set reader.[[closedPromise]] to a promise resolved with undefined.
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            promise = Promise::new_resolved(global, cx, rval.handle()).unwrap();
        } else {
            // Assert: stream.[[state]] is "errored"
            assert!(stream.is_errored());

            // Set reader.[[closedPromise]] to a promise rejected with stream.[[storedError]].
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            stream.get_stored_error(rval.handle_mut());
            promise = Promise::new_rejected(global, cx, rval.handle()).unwrap();

            // Set reader.[[closedPromise]].[[PromiseIsHandled]] to true
            promise.set_promise_is_handled();
        }
        reflect_dom_object(
            Box::new(ReadableStreamDefaultReader::new_inherited(stream, promise)),
            global,
        )
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub fn close(&self) {
        // step 5
        self.closed_promise.borrow().resolve_native(&());
        // step 6
        let mut read_requests = self.take_read_requests();
        for request in read_requests.drain(0..) {
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
        // step 1 & 2
        assert!(self.stream.get().is_some());

        if let Some(stream) = self.stream.get() {
            // step 3
            assert!(stream.has_default_reader());

            if stream.is_readable() {
                // step 4
                self.closed_promise
                    .borrow()
                    .reject_error(Error::Type("stream state is not readable".to_owned()));
            } else {
                // step 5
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                unsafe {
                    Error::Type("Cannot release lock due to stream state.".to_owned())
                        .clone()
                        .to_jsval(*cx, &self.global(), rval.handle_mut())
                };

                *self.closed_promise.borrow_mut() =
                    Promise::new_rejected(&self.global(), cx, rval.handle()).unwrap();
            }
            // step 6
            self.closed_promise.borrow().set_promise_is_handled();

            // step 7
            stream.perform_release_steps();

            // step 8
            stream.set_reader(None);
            // step 9
            self.stream.set(None);
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease>
    #[allow(unsafe_code)]
    pub fn release(&self) {
        // step 1
        self.generic_release();
        // step 2
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        unsafe {
            Error::Type("Reader is released".to_owned())
                .clone()
                .to_jsval(*cx, &self.global(), rval.handle_mut())
        };

        // step 3
        self.error_read_requests(rval.handle());
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
        // step 1 & 2
        assert!(self.stream.get().is_some());

        if let Some(stream) = self.stream.get() {
            // step 3
            stream.set_is_disturbed(true);
            if stream.is_closed() {
                // step 4
                read_request.close_steps();
            } else if stream.is_errored() {
                // step 5
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                stream.get_stored_error(rval.handle_mut());
                read_request.error_steps(rval.handle());
            } else {
                // step 6
                assert!(stream.is_readable());
                stream.perform_pull_steps(read_request);
            }
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-readablestreamgenericreader-closedpromise%E2%91%A1>
    pub fn append_native_handler_to_closed_promise(
        &self,
        branch_1: MutNullableDom<ReadableStream>,
        branch_2: MutNullableDom<ReadableStream>,
        canceled_1: Rc<Cell<bool>>,
        canceled_2: Rc<Cell<bool>>,
        cancel_promise: Rc<Promise>,
    ) {
        let branch_1_controller = branch_1
            .get()
            .expect("branch_1 must be set")
            .get_default_controller();

        let branch_2_controller = branch_2
            .get()
            .expect("branch_2 must be set")
            .get_default_controller();

        let global = self.global();
        let rejection_handler = Box::new(ClosedPromiseRejectionHandler {
            branch_1_controller: Trusted::new(&branch_1_controller),
            branch_2_controller: Trusted::new(&branch_2_controller),
            canceled_1,
            canceled_2,
            cancel_promise,
        });
        let handler = PromiseNativeHandler::new(&global, None, Some(rejection_handler));

        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);

        self.closed_promise
            .borrow()
            .append_native_handler(&handler, comp, CanGc::note());
    }
}

impl ReadableStreamDefaultReaderMethods for ReadableStreamDefaultReader {
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
        // step 1
        if self.stream.get().is_none() {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            unsafe {
                Error::Type("stream is undefined".to_owned())
                    .clone()
                    .to_jsval(*cx, &self.global(), rval.handle_mut())
            };
            return Promise::new_rejected(&self.global(), cx, rval.handle()).unwrap();
        }
        // step 2
        let promise = Promise::new(&self.reflector_.global(), can_gc);

        // step 3
        let read_request = ReadRequest::Read(promise.clone());

        // step 4
        self.read(read_request);

        // step 5
        promise
    }

    /// <https://streams.spec.whatwg.org/#default-reader-release-lock>
    #[allow(unsafe_code)]
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
