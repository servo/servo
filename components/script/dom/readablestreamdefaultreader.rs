/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::{
    ReadableStreamDefaultReaderMethods, ReadableStreamReadResult,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::set_dictionary_property;
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
    #[allow(crown::unrooted_must_root)]
    pub fn chunk_steps(&self, chunk: Vec<u8>) {
        match self {
            ReadRequest::Read(promise) => {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                let result = RootedTraceableBox::new(Heap::default());
                unsafe {
                    chunk.to_jsval(*cx, rval.handle_mut());
                    result.set(*rval);
                }
                let result = ReadableStreamReadResult {
                    done: Some(false),
                    value: result,
                };
                promise.resolve_native(&result);
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-close-step>
    #[allow(crown::unrooted_must_root)]
    pub fn close_steps(&self) {
        match self {
            ReadRequest::Read(promise) => {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                let result = RootedTraceableBox::new(Heap::default());
                result.set(*rval);
                let result = ReadableStreamReadResult {
                    done: Some(true),
                    value: result,
                };
                promise.resolve_native(&result);
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-read-request-close-step>
    pub fn error_steps(&self) {
        match self {
            // TODO: pass error type.
            ReadRequest::Read(promise) => promise.reject_native(&()),
        }
    }
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
#[dom_struct]
pub struct ReadableStreamDefaultReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
    /// TODO: use MutNullableDom
    stream: Dom<ReadableStream>,

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
            stream: Dom::from_ref(stream),
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

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub fn add_read_request(&self, read_request: ReadRequest) {
        self.read_requests.borrow_mut().push_back(read_request);
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub fn get_num_read_requests(&self) -> usize {
        self.read_requests.borrow().len()
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error(&self, _error: Error) {
        self.closed_promise.reject_native(&());
        for request in self.read_requests.borrow_mut().drain(0..) {
            request.error_steps();
        }
    }

    /// The removal steps of <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    pub fn remove_read_request(&self) -> ReadRequest {
        self.read_requests
            .borrow_mut()
            .pop_front()
            .expect("Reader must have read request when remove is called into.")
    }
}

impl ReadableStreamDefaultReaderMethods for ReadableStreamDefaultReader {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    fn Read(&self) -> Rc<Promise> {
        let promise = Promise::new(&self.reflector_.global());

        self.stream
            .perform_pull_steps(ReadRequest::Read(promise.clone()));

        promise
    }

    /// <https://streams.spec.whatwg.org/#default-reader-release-lock>
    fn ReleaseLock(&self) {
        if self.stream.is_readable() {
            self.closed_promise.reject_native(&());
        }

        // TODO: https://streams.spec.whatwg.org/#readable-stream-reader-generic-release

        // TODO: use TypeError.
        // <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreadererrorreadrequests>
        for request in self.read_requests.borrow_mut().drain(0..) {
            request.error_steps();
        }
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn Closed(&self) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global())
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn Cancel(&self, _cx: SafeJSContext, _reason: SafeHandleValue) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global())
    }
}
