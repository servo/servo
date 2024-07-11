/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::rust::{HandleValue as SafeHandleValue, HandleValue};

use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::ReadableStream;
use crate::dom::readablestreamdefaultreader::ReadRequest;
use crate::dom::underlyingsourcecontainer::{UnderlyingSourceContainer, UnderlyingSourceType};
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{JSContext, JSContext as SafeJSContext};

/// The fulfillment handler for
/// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct PullAlgorithmFulfillmentHandler {
    controller: DomRoot<ReadableStreamDefaultController>,
}

impl Callback for PullAlgorithmFulfillmentHandler {
    /// Handle fufillment of pull algo promise.
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        todo!();
    }
}

/// The rejection handler for
/// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct PullAlgorithmRejectionHandler {
    controller: DomRoot<ReadableStreamDefaultController>,
}

impl Callback for PullAlgorithmRejectionHandler {
    /// Handle rejection of pull algo promise.
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        todo!();
    }
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
#[dom_struct]
pub struct ReadableStreamDefaultController {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-queue>
    buffer: RefCell<Vec<u8>>,

    underlying_source: Dom<UnderlyingSourceContainer>,

    stream: MutNullableDom<ReadableStream>,
}

impl ReadableStreamDefaultController {
    fn new_inherited(
        global: &GlobalScope,
        underlying_source_type: UnderlyingSourceType,
    ) -> ReadableStreamDefaultController {
        ReadableStreamDefaultController {
            reflector_: Reflector::new(),
            buffer: RefCell::new(vec![]),
            stream: MutNullableDom::new(None),
            underlying_source: Dom::from_ref(&*UnderlyingSourceContainer::new(
                global,
                underlying_source_type,
            )),
        }
    }
    pub fn new(
        global: &GlobalScope,
        underlying_source: UnderlyingSourceType,
    ) -> DomRoot<ReadableStreamDefaultController> {
        reflect_dom_object(
            Box::new(ReadableStreamDefaultController::new_inherited(
                global,
                underlying_source,
            )),
            global,
        )
    }

    pub fn set_stream(&self, stream: &ReadableStream) {
        self.stream.set(Some(stream))
    }

    fn get_chunk_with_length(&self, length: usize) -> Vec<u8> {
        let mut buffer = self.buffer.borrow_mut();
        let buffer_len = buffer.len();
        assert!(buffer_len >= length);
        buffer.split_off(buffer_len - length)
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-should-call-pull>
    fn should_pull(&self) -> bool {
        // TODO: implement the algo.
        true
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-call-pull-if-needed>
    fn call_pull_if_needed(&self) {
        if !self.should_pull() {
            return;
        }

        let global = self.global();
        if let Some(promise) = self.underlying_source.call_pull_algorithm(self) {
            let fulfillment_handler = Box::new(PullAlgorithmFulfillmentHandler {
                controller: DomRoot::from_ref(self),
            });
            let rejection_handler = Box::new(PullAlgorithmRejectionHandler {
                controller: DomRoot::from_ref(self),
            });
            let handler = PromiseNativeHandler::new(
                &global,
                Some(fulfillment_handler),
                Some(rejection_handler),
            );

            let realm = enter_realm(&*global);
            let comp = InRealm::Entered(&realm);
            promise.append_native_handler(&handler, comp);
        }
    }

    /// <https://streams.spec.whatwg.org/#ref-for-abstract-opdef-readablestreamcontroller-pullsteps>
    pub fn perform_pull_steps(&self, read_request: ReadRequest) {
        // if buffer contains bytes, perform chunk steps.
        if !self.buffer.borrow().is_empty() {
            // TODO: use <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-strategysizealgorithm>
            let chunk = self.get_chunk_with_length(self.buffer.borrow().len());

            // TODO: handle close requested.

            self.call_pull_if_needed();

            read_request.chunk_steps(chunk);
        }

        // else, append read request to reader.
        self.stream
            .get()
            .expect("Controller must have a stream when pull steps are called into.")
            .add_read_request(read_request);

        self.call_pull_if_needed();
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    pub fn enqueue_chunk(&self, mut chunk: Vec<u8>) {
        let stream = self
            .stream
            .get()
            .expect("Controller must have a stream when a chunk is enqueued.");
        if stream.is_locked() && stream.get_num_read_requests() > 0 {
            stream.fulfill_read_request(chunk, false);
            return;
        }

        // TODO: strategy size algo.

        // <https://streams.spec.whatwg.org/#enqueue-value-with-size>
        let mut buffer = self.buffer.borrow_mut();
        chunk.append(&mut buffer);
        *buffer = chunk;

        self.call_pull_if_needed();
    }

    /// Does the stream have all data in memory?
    pub fn in_memory(&self) -> bool {
        self.underlying_source.in_memory()
    }

    /// Return bytes synchronously if the stream has all data in memory.
    pub fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        if self.underlying_source.in_memory() {
            return Some(self.buffer.borrow().clone());
        }
        None
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-close>
    pub fn close(&self) {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ref-for-readable-stream-error>
    pub fn error(&self) {
        todo!()
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
