/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleValue as SafeHandleValue;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::readablestream::ReadableStream;
use crate::dom::readablestreamdefaultreader::ReadRequest;
use crate::script_runtime::JSContext as SafeJSContext;

#[derive(JSTraceable)]
pub enum UnderlyingSource {
    /// Facilitate partial integration with sources
    /// that are currently read into memory.
    Memory(usize),
    /// A blob as underlying source, with a known total size.
    Blob(usize),
    /// A fetch response as underlying source.
    FetchResponse,

    Js(JsUnderlyingSource),
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
#[dom_struct]
pub struct ReadableStreamDefaultController {
    reflector_: Reflector,

    /// Loosely matches the underlying queue,
    /// <https://streams.spec.whatwg.org/#internal-queues>
    buffer: RefCell<Vec<u8>>,

    #[ignore_malloc_size_of = "Rc is hard"]
    underlying_source: Rc<UnderlyingSource>,

    stream: MutNullableDom<ReadableStream>,
}

impl ReadableStreamDefaultController {
    fn new_inherited(underlying_source: Rc<UnderlyingSource>) -> ReadableStreamDefaultController {
        ReadableStreamDefaultController {
            reflector_: Reflector::new(),
            buffer: RefCell::new(vec![]),
            stream: MutNullableDom::new(None),
            underlying_source,
        }
    }
    pub fn new(
        global: &GlobalScope,
        underlying_source: Rc<UnderlyingSource>,
    ) -> DomRoot<ReadableStreamDefaultController> {
        reflect_dom_object(
            Box::new(ReadableStreamDefaultController::new_inherited(
                underlying_source,
            )),
            global,
        )
    }

    pub fn set_stream(&self, stream: &ReadableStream) {
        self.stream.set(Some(stream))
    }

    /// <https://streams.spec.whatwg.org/#ref-for-abstract-opdef-readablestreamcontroller-pullsteps>
    pub fn perform_pull_steps(&self, read_request: ReadRequest) {
        // if buffer contains bytes, perform chunk steps.
        // <https://streams.spec.whatwg.org/#read-request-chunk-steps>

        // Call into underlying source if necessary.

        // else, append read request to reader.
        self.stream.get().unwrap().add_read_request(read_request);
    }

    pub fn enqueue_chunk(&self, mut chunk: Vec<u8>) {
        let mut buffer = self.buffer.borrow_mut();
        chunk.append(&mut buffer);
        *buffer = chunk;
    }
}

impl ReadableStreamDefaultControllerMethods for ReadableStreamDefaultController {
    /// <https://streams.spec.whatwg.org/#rs-default-controller-desired-size>
    fn GetDesiredSize(&self) -> Option<f64> {
        // TODO
        None
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-close>
    fn Close(&self) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-enqueue>
    fn Enqueue(&self, _cx: SafeJSContext, _chunk: SafeHandleValue) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rs-default-controller-error>
    fn Error(&self, _cx: SafeJSContext, _e: SafeHandleValue) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }
}
