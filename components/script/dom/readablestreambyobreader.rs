/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::gc::CustomAutoRooterGuard;
use js::jsapi::Heap;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use js::typedarray::ArrayBufferView;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBReaderBinding::ReadableStreamBYOBReaderMethods;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamReadResult;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#read-request>
/// For now only one variant: the one matching a `read` call.
#[derive(JSTraceable)]
pub enum ReadIntoRequest {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    Read(Rc<Promise>),
}

impl ReadIntoRequest {
    /// <https://streams.spec.whatwg.org/#read-into-request-chunk-steps>
    pub fn chunk_steps(&self, chunk: Vec<u8>) {
        self.resolve_request_promise(Some(chunk), false);
    }

    /// <https://streams.spec.whatwg.org/#read-into-request-close-steps>
    pub fn close_steps(&self, chunk: Option<Vec<u8>>) {
        self.resolve_request_promise(chunk, true);
    }

    /// <https://streams.spec.whatwg.org/#read-into-request-error-steps>
    pub fn error_steps(&self) {
        match self {
            // TODO: pass error type.
            ReadIntoRequest::Read(promise) => promise.reject_native(&()),
        }
    }

    #[allow(unsafe_code)]
    fn resolve_request_promise(&self, chunk: Option<Vec<u8>>, done: bool) {
        match self {
            ReadIntoRequest::Read(promise) => {
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                let result = RootedTraceableBox::new(Heap::default());
                if let Some(chunk) = chunk {
                    unsafe {
                        chunk.to_jsval(*cx, rval.handle_mut());
                        result.set(*rval);
                    }
                }
                promise.resolve_native(&ReadableStreamReadResult {
                    done: Some(done),
                    value: result,
                });
            },
        }
    }
}

/// <https://streams.spec.whatwg.org/#readablestreambyobreader>
#[dom_struct]
pub struct ReadableStreamBYOBReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
    /// TODO: use MutNullableDom
    stream: Dom<ReadableStream>,

    #[ignore_malloc_size_of = "Rc is hard"]
    read_into_requests: DomRefCell<VecDeque<ReadIntoRequest>>,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-closedpromise>
    #[ignore_malloc_size_of = "Rc is hard"]
    closed_promise: Rc<Promise>,
}

impl ReadableStreamBYOBReader {
    /// <https://streams.spec.whatwg.org/#byob-reader-constructor>
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
    ) -> ReadableStreamBYOBReader {
        ReadableStreamBYOBReader {
            reflector_: Reflector::new(),
            read_into_requests: DomRefCell::new(Default::default()),
            stream: Dom::from_ref(stream),
            closed_promise,
        }
    }

    pub fn new(global: &GlobalScope, stream: &ReadableStream) -> DomRoot<ReadableStreamBYOBReader> {
        let promise = Promise::new(global);
        if stream.is_closed() {
            promise.resolve_native(&());
        }
        if stream.is_errored() {
            promise.reject_native(&());
        }
        reflect_dom_object(
            Box::new(ReadableStreamBYOBReader::new_inherited(stream, promise)),
            global,
        )
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub fn close(&self) {
        // step 5
        self.closed_promise.resolve_native(&());
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-into-request>
    pub fn add_read_into_request(&self, read_into_request: ReadIntoRequest) {
        self.read_into_requests
            .borrow_mut()
            .push_back(read_into_request);
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-into-requests>
    pub fn get_num_read_into_requests(&self) -> usize {
        self.read_into_requests.borrow().len()
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error(&self, _error: Error) {
        self.closed_promise.reject_native(&());
        for request in self.read_into_requests.borrow_mut().drain(0..) {
            request.error_steps();
        }
    }

    /// The removal steps of <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-into-request>
    pub fn remove_read_into_request(&self) -> ReadIntoRequest {
        self.read_into_requests
            .borrow_mut()
            .pop_front()
            .expect("Reader must have read request when remove is called into.")
    }
}

impl ReadableStreamBYOBReaderMethods for ReadableStreamBYOBReader {
    /// <https://streams.spec.whatwg.org/#byob-reader-read>
    fn Read(&self, _view: CustomAutoRooterGuard<ArrayBufferView>) -> Rc<Promise> {
        // TODO
        Promise::new(&self.reflector_.global())
    }

    /// <https://streams.spec.whatwg.org/#byob-reader-release-lock>
    fn ReleaseLock(&self) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
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
