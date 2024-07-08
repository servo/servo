/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamDefaultReaderMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestream::ReadableStream;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#read-request>
#[derive(JSTraceable)]
pub struct ReadRequest {
    promise: Rc<Promise>,
}

impl ReadRequest {
    /// <https://streams.spec.whatwg.org/#read-request-chunk-steps>
    pub fn chunk_steps(&self, chunk: Vec<u8>) {
        self.promise.resolve_native(&chunk);
    }
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
#[dom_struct]
pub struct ReadableStreamDefaultReader {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamgenericreader-stream>
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

    fn new(global: &GlobalScope, stream: &ReadableStream) -> DomRoot<ReadableStreamDefaultReader> {
        reflect_dom_object(
            Box::new(ReadableStreamDefaultReader::new_inherited(
                stream,
                Promise::new(global),
            )),
            global,
        )
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub fn add_read_request(&self, read_request: ReadRequest) {
        self.read_requests.borrow_mut().push_back(read_request);
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub fn error(&self, _error: Error) {
        self.closed_promise.reject_native(&());
        for _request in self.read_requests.borrow_mut().drain(0..) {
            // https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreadererrorreadrequests
        }
    }
}

impl ReadableStreamDefaultReaderMethods for ReadableStreamDefaultReader {
    /// <https://streams.spec.whatwg.org/#default-reader-read>
    fn Read(&self) -> Rc<Promise> {
        // TODO
        let promise = Promise::new(&self.reflector_.global());

        self.stream.perform_pull_steps(ReadRequest {
            promise: promise.clone(),
        });

        promise
    }

    /// <https://streams.spec.whatwg.org/#default-reader-release-lock>
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
