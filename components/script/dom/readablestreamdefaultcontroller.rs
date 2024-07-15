/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::rust::{HandleValue as SafeHandleValue, HandleValue};

use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::{
    ReadableStreamDefaultControllerMethods, ValueWithSize,
};
use crate::dom::bindings::import::module::UnionTypes::ReadableStreamDefaultControllerOrReadableByteStreamController as Controller;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::refcounted::Trusted;
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
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmFulfillmentHandler {
    controller: Dom<ReadableStreamDefaultController>,
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
#[allow(crown::unrooted_must_root)]
struct PullAlgorithmRejectionHandler {
    controller: Dom<ReadableStreamDefaultController>,
}

impl Callback for PullAlgorithmRejectionHandler {
    /// Handle rejection of pull algo promise.
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        todo!();
    }
}

/// <https://streams.spec.whatwg.org/#value-with-size>
#[derive(JSTraceable)]
#[allow(crown::unrooted_must_root)]
pub enum EnqueuedValue {
    /// A value enqueued from Rust.
    Native(Vec<u8>),
    /// A Js value.
    Js(ValueWithSize),
}

/// <https://streams.spec.whatwg.org/#queue-with-sizes>
#[derive(Default, JSTraceable, MallocSizeOf)]
pub struct QueueWithSizes {
    total_size: usize,
    #[ignore_malloc_size_of = "Rc is hard"]
    queue: VecDeque<EnqueuedValue>,
}

impl QueueWithSizes {
    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&mut self) -> EnqueuedValue {
        self.queue
            .pop_front()
            .expect("Buffer cannot be empty when dequeue value is called into.")
    }

    /// <https://streams.spec.whatwg.org/#enqueue-value-with-size>
    fn enqueue_value_with_size(&mut self, value: EnqueuedValue) {
        self.queue.push_back(value);
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Only used with native sources.
    fn get_in_memory_bytes(&self) -> Vec<u8> {
        self.queue
            .iter()
            .flat_map(|value| {
                let EnqueuedValue::Native(chunk) = value else {
                    unreachable!(
                        "`get_in_memory_bytes` can only be called on a queue with native values."
                    )
                };
                chunk.clone()
            })
            .collect()
    }
}

/// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
#[dom_struct]
pub struct ReadableStreamDefaultController {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-queue>
    queue: RefCell<QueueWithSizes>,

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
            queue: RefCell::new(Default::default()),
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

    /// <https://streams.spec.whatwg.org/#dequeue-value>
    fn dequeue_value(&self) -> EnqueuedValue {
        let mut queue = self.queue.borrow_mut();
        queue.dequeue_value()
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
        let controller = Controller::ReadableStreamDefaultController(DomRoot::from_ref(self));
        if let Some(promise) = self.underlying_source.call_pull_algorithm(controller) {
            let fulfillment_handler = Box::new(PullAlgorithmFulfillmentHandler {
                controller: Dom::from_ref(self),
            });
            let rejection_handler = Box::new(PullAlgorithmRejectionHandler {
                controller: Dom::from_ref(self),
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
        // if queue contains bytes, perform chunk steps.
        if !self.queue.borrow().is_empty() {
            // TODO: use <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller-strategysizealgorithm>
            let chunk = self.dequeue_value();

            // TODO: handle close requested.

            self.call_pull_if_needed();

            if let EnqueuedValue::Native(chunk) = chunk {
                read_request.chunk_steps(chunk);
            }
        }

        // else, append read request to reader.
        self.stream
            .get()
            .expect("Controller must have a stream when pull steps are called into.")
            .add_read_request(read_request);

        self.call_pull_if_needed();
    }

    /// Native call to
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-enqueue>
    pub fn enqueue_native(&self, chunk: Vec<u8>) {
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
        let mut queue = self.queue.borrow_mut();
        queue.enqueue_value_with_size(EnqueuedValue::Native(chunk));

        self.call_pull_if_needed();
    }

    /// Does the stream have all data in memory?
    pub fn in_memory(&self) -> bool {
        self.underlying_source.in_memory()
    }

    /// Return bytes synchronously if the stream has all data in memory.
    pub fn get_in_memory_bytes(&self) -> Option<Vec<u8>> {
        if self.underlying_source.in_memory() {
            return Some(self.queue.borrow().get_in_memory_bytes());
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
