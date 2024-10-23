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
    closed_promise: Rc<Promise>,
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
            closed_promise,
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
        self.closed_promise.resolve_native(&());
        let pending_requests = self.get_read_requests();
        for request in pending_requests {
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
        self.closed_promise.reject_native(&e);
        let pending_requests = self.get_read_requests();
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

    fn get_read_requests(&self) -> VecDeque<ReadRequest> {
        mem::take(&mut *self.read_requests.borrow_mut())
    }
}

impl ReadableStreamDefaultReaderMethods for ReadableStreamDefaultReader {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    fn Read(&self) -> Rc<Promise> {
        let promise = Promise::new(&self.reflector_.global());

        if let Some(stream) = self.stream.get() {
            stream.perform_pull_steps(ReadRequest::Read(promise.clone()));
        } else {
            promise.reject_error(Error::Type("stream is undefined".to_owned()));
        }

        promise
    }

    /// <https://streams.spec.whatwg.org/#default-reader-release-lock>
    #[allow(unsafe_code)]
    fn ReleaseLock(&self) {
        // step 2
        // https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease
        if let Some(stream) = self.stream.get() {
            // step 2.1
            // https://streams.spec.whatwg.org/#readable-stream-reader-generic-release
            stream.release_lock(&self.closed_promise);
            self.stream.set(None);

            // step 2.2
            let error =
                Error::Type("No chunks are available because the stream is errored".to_owned());

            // step 2.3
            // <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreadererrorreadrequests>

            // step 2.3.1
            let mut read_requests = self.get_read_requests();
            // step 2.3.2 & 2.3.3
            for request in read_requests.drain(0..) {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                unsafe {
                    error
                        .clone()
                        .to_jsval(*cx, &*self.global(), rval.handle_mut())
                };
                request.error_steps(rval.handle());
            }
        } else {
            // step 1
            return;
        }
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        self.closed_promise.clone()
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, reason: SafeHandleValue) -> Rc<Promise> {
        let promise = Promise::new(&self.reflector_.global());

        if let Some(stream) = self.stream.get() {
            // step 2.1 & 2.2
            assert!(self.stream.get().is_some());
            // 2.3
            stream.cancel(&promise, reason);
        } else {
            promise.reject_error(Error::Type("stream is undefined".to_owned()));
        }

        promise
    }
}
