/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use super::bindings::root::MutNullableDom;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::{
    ReadableStreamDefaultReaderMethods, ReadableStreamReadResult,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#read-request>
/// For now only one variant: the one matching a `read` call.
#[derive(JSTraceable)]
pub enum ReadRequest {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    Read(Rc<Promise>),
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
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-close-step>
    pub fn error_steps(&self, e: SafeHandleValue) {
        match self {
            ReadRequest::Read(promise) => promise.reject_native(&e),
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
        error!("ReadableStreamDefaultReader::Constructor");
        let reader = ReadableStreamDefaultReader::new_inherited(global, can_gc);

        // Perform ? SetUpReadableStreamDefaultReader(this, stream).
        Self::set_up(&reader, stream, global, can_gc)?;

        Ok(reflect_dom_object(Box::new(reader), global))
    }

    pub fn new_inherited(global: &GlobalScope, can_gc: CanGc) -> ReadableStreamDefaultReader {
        error!("ReadableStreamDefaultReader::new_inherited");
        ReadableStreamDefaultReader {
            reflector_: Reflector::new(),
            stream: MutNullableDom::new(None),
            read_requests: DomRefCell::new(VecDeque::new()),
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
        error!("ReadableStreamDefaultReader::set_up");
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
        error!("ReadableStreamDefaultReader::generic_initialize");
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
            rooted!(in(*cx) let mut rval = UndefinedValue());
            stream.get_stored_error(rval.handle_mut());

            *self.closed_promise.borrow_mut() = Promise::new_rejected(global, cx, rval.handle())?;

            // Set reader.[[closedPromise]].[[PromiseIsHandled]] to true

            self.closed_promise.borrow().set_promise_is_handled();
        }

        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub fn close(&self) {
        error!("ReadableStreamDefaultReader::close");
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
        error!("ReadableStreamDefaultReader::add_read_request");
        self.read_requests.borrow_mut().push_back(read_request);
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub fn get_num_read_requests(&self) -> usize {
        error!("ReadableStreamDefaultReader::get_num_read_requests");
        self.read_requests.borrow().len()
    }

    /// steps 6, 7, 8 of <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error(&self, e: SafeHandleValue) {
        error!("ReadableStreamDefaultReader::error");
        // step 6
        self.closed_promise.borrow().reject_native(&e);

        // step 7
        self.closed_promise.borrow().set_promise_is_handled();

        // step 8
        self.error_read_requests(e);
    }

    /// The removal steps of <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    pub fn remove_read_request(&self) -> ReadRequest {
        error!("ReadableStreamDefaultReader::remove_read_request");
        self.read_requests
            .borrow_mut()
            .pop_front()
            .expect("Reader must have read request when remove is called into.")
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-release>
    #[allow(unsafe_code)]
    pub fn generic_release(&self) {
        error!("ReadableStreamDefaultReader::generic_release");
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
        error!("ReadableStreamDefaultReader::release");
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
        error!("ReadableStreamDefaultReader::generic_cancel");
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
        error!("ReadableStreamDefaultReader::error_read_requests");
        // step 1
        let mut read_requests = self.take_read_requests();

        // step 2 & 3
        for request in read_requests.drain(0..) {
            request.error_steps(rval);
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    fn read(&self, read_request: ReadRequest) {
        error!("ReadableStreamDefaultReader::read");
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
        error!("ReadableStreamDefaultReader::Read");
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
