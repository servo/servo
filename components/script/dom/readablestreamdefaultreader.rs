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
use crate::script_runtime::JSContext as SafeJSContext;

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
        _global: &GlobalScope,
        _proto: Option<SafeHandleObject>,
        _stream: DomRoot<ReadableStream>,
    ) -> Fallible<DomRoot<Self>> {
        // TODO
        Err(Error::NotFound)
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

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-initialize>
    pub fn new(
        global: &GlobalScope,
        stream: &ReadableStream,
    ) -> DomRoot<ReadableStreamDefaultReader> {
        let promise = Promise::new(global);
        if stream.is_closed() {
            promise.resolve_native(&());
        }
        if stream.is_errored() {
            promise.reject_native(&());
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

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error(&self, e: SafeHandleValue) {
        self.closed_promise.borrow().reject_native(&e);
        let pending_requests = self.take_read_requests();
        for request in pending_requests {
            request.error_steps(e);
        }
    }

    /// The removal steps of <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    pub fn remove_read_request(&self) -> ReadRequest {
        self.read_requests
            .borrow_mut()
            .pop_front()
            .expect("Reader must have read request when remove is called into.")
    }

    /// https://streams.spec.whatwg.org/#readable-stream-reader-generic-release
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
                        .to_jsval(*cx, &*self.global(), rval.handle_mut())
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
                .to_jsval(*cx, &*self.global(), rval.handle_mut())
        };

        // step 3
        self.error_read_requests(rval.handle());
    }

    fn take_read_requests(&self) -> VecDeque<ReadRequest> {
        mem::take(&mut *self.read_requests.borrow_mut())
    }

    /// https://streams.spec.whatwg.org/#readable-stream-reader-generic-cancel
    fn generic_cancel(&self, promise: &Rc<Promise>, reason: SafeHandleValue) {
        // step 1
        if let Some(stream) = self.stream.get() {
            // step 2
            assert!(self.stream.get().is_some());

            // step 3
            stream.cancel(promise, reason);
        }
    }

    /// https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreadererrorreadrequests
    fn error_read_requests(&self, rval: SafeHandleValue) {
        // step 1
        let mut read_requests = self.take_read_requests();

        // step 2 & 3
        for request in read_requests.drain(0..) {
            request.error_steps(rval);
        }
    }

    /// https://streams.spec.whatwg.org/#readable-stream-default-reader-read
    fn read(&self, read_request: ReadRequest) {
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
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    #[allow(unsafe_code)]
    fn Read(&self) -> Rc<Promise> {
        // step 1
        if self.stream.get().is_none() {
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut rval = UndefinedValue());
            unsafe {
                Error::Type("stream is undefined".to_owned())
                    .clone()
                    .to_jsval(*cx, &*self.global(), rval.handle_mut())
            };
            return Promise::new_rejected(&self.global(), cx, rval.handle()).unwrap();
        }
        // step 2
        let promise = Promise::new(&self.reflector_.global());

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
        if self.stream.get().is_none() {
            // step 1
            return;
        } else {
            // step 2
            self.release();
        }
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        self.closed_promise.borrow().clone()
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue) -> Rc<Promise> {
        let promise = Promise::new(&self.reflector_.global());

        if self.stream.get().is_none() {
            promise.reject_error(Error::Type("stream is undefined".to_owned()));
        } else {
            self.generic_cancel(&promise, reason);
        }

        promise
    }
}
