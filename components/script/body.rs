/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::{ptr, str};

use encoding_rs::UTF_8;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{Heap, JSObject, JS_ClearPendingException, Value as JSValue};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::{JS_GetPendingException, JS_ParseJSON};
use js::rust::HandleValue;
use js::typedarray::{ArrayBuffer, CreateWith};
use mime::{self, Mime};
use net_traits::request::{
    BodyChunkRequest, BodyChunkResponse, BodySource as NetBodySource, RequestBody,
};
use script_traits::serializable::BlobImpl;
use url::form_urlencoded;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BlobBinding::Blob_Binding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use crate::dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::blob::{normalize_type_string, Blob};
use crate::dom::formdata::FormData;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlformelement::{encode_multipart_form_data, generate_boundary};
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::{get_read_promise_bytes, get_read_promise_done, ReadableStream};
use crate::dom::urlsearchparams::URLSearchParams;
use crate::realms::{enter_realm, AlreadyInRealm, InRealm};
use crate::script_runtime::JSContext;
use crate::task::TaskCanceller;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::{TaskSource, TaskSourceName};

/// The Dom object, or ReadableStream, that is the source of a body.
/// <https://fetch.spec.whatwg.org/#concept-body-source>
#[derive(Clone, PartialEq)]
pub enum BodySource {
    /// A ReadableStream comes with a null-source.
    Null,
    /// Another Dom object as source,
    /// TODO: store the actual object
    /// and re-extract a stream on re-direct.
    Object,
}

/// The reason to stop reading from the body.
enum StopReading {
    /// The stream has errored.
    Error,
    /// The stream is done.
    Done,
}

/// The IPC route handler
/// for <https://fetch.spec.whatwg.org/#concept-request-transmit-body>.
/// This route runs in the script process,
/// and will queue tasks to perform operations
/// on the stream and transmit body chunks over IPC.
#[derive(Clone)]
struct TransmitBodyConnectHandler {
    stream: Trusted<ReadableStream>,
    task_source: NetworkingTaskSource,
    canceller: TaskCanceller,
    bytes_sender: Option<IpcSender<BodyChunkResponse>>,
    control_sender: IpcSender<BodyChunkRequest>,
    in_memory: Option<Vec<u8>>,
    in_memory_done: bool,
    source: BodySource,
}

impl TransmitBodyConnectHandler {
    pub fn new(
        stream: Trusted<ReadableStream>,
        task_source: NetworkingTaskSource,
        canceller: TaskCanceller,
        control_sender: IpcSender<BodyChunkRequest>,
        in_memory: Option<Vec<u8>>,
        source: BodySource,
    ) -> TransmitBodyConnectHandler {
        TransmitBodyConnectHandler {
            stream,
            task_source,
            canceller,
            bytes_sender: None,
            control_sender,
            in_memory,
            in_memory_done: false,
            source,
        }
    }

    /// Reset `in_memory_done`, called when a stream is
    /// re-extracted from the source to support a re-direct.
    pub fn reset_in_memory_done(&mut self) {
        self.in_memory_done = false;
    }

    /// Re-extract the source to support streaming it again for a re-direct.
    /// TODO: actually re-extract the source, instead of just cloning data, to support Blob.
    fn re_extract(&mut self, chunk_request_receiver: IpcReceiver<BodyChunkRequest>) {
        let mut body_handler = self.clone();
        body_handler.reset_in_memory_done();

        ROUTER.add_route(
            chunk_request_receiver.to_opaque(),
            Box::new(move |message| {
                let request = message.to().unwrap();
                match request {
                    BodyChunkRequest::Connect(sender) => {
                        body_handler.start_reading(sender);
                    },
                    BodyChunkRequest::Extract(receiver) => {
                        body_handler.re_extract(receiver);
                    },
                    BodyChunkRequest::Chunk => body_handler.transmit_source(),
                    // Note: this is actually sent from this process
                    // by the TransmitBodyPromiseHandler when reading stops.
                    BodyChunkRequest::Done => {
                        body_handler.stop_reading(StopReading::Done);
                    },
                    // Note: this is actually sent from this process
                    // by the TransmitBodyPromiseHandler when the stream errors.
                    BodyChunkRequest::Error => {
                        body_handler.stop_reading(StopReading::Error);
                    },
                }
            }),
        );
    }

    /// In case of re-direct, and of a source available in memory,
    /// send it all in one chunk.
    ///
    /// TODO: this method should be deprecated
    /// in favor of making `re_extract` actually re-extract a stream from the source.
    /// See #26686
    fn transmit_source(&mut self) {
        if self.in_memory_done {
            // Step 5.1.3
            self.stop_reading(StopReading::Done);
            return;
        }

        if let BodySource::Null = self.source {
            panic!("ReadableStream(Null) sources should not re-direct.");
        }

        if let Some(bytes) = self.in_memory.clone() {
            // The memoized bytes are sent so we mark it as done again
            self.in_memory_done = true;
            let _ = self
                .bytes_sender
                .as_ref()
                .expect("No bytes sender to transmit source.")
                .send(BodyChunkResponse::Chunk(bytes.clone()));
            return;
        }
        warn!("Re-directs for file-based Blobs not supported yet.");
    }

    /// Take the IPC sender sent by `net`, so we can send body chunks with it.
    /// Also the entry point to <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
    fn start_reading(&mut self, sender: IpcSender<BodyChunkResponse>) {
        self.bytes_sender = Some(sender);

        // If we're using an actual ReadableStream, acquire a reader for it.
        if self.source == BodySource::Null {
            let stream = self.stream.clone();
            let _ = self.task_source.queue_with_canceller(
                task!(start_reading_request_body_stream: move || {
                    // Step 1, Let body be request’s body.
                    let rooted_stream = stream.root();

                    // TODO: Step 2, If body is null.

                    // Step 3, get a reader for stream.
                    rooted_stream.start_reading().expect("Couldn't acquire a reader for the body stream.");

                    // Note: this algorithm continues when the first chunk is requested by `net`.
                }),
                &self.canceller,
            );
        }
    }

    /// Drop the IPC sender sent by `net`
    fn stop_reading(&mut self, reason: StopReading) {
        let bytes_sender = self
            .bytes_sender
            .take()
            .expect("Stop reading called multiple times on TransmitBodyConnectHandler.");
        match reason {
            StopReading::Error => {
                let _ = bytes_sender.send(BodyChunkResponse::Error);
            },
            StopReading::Done => {
                let _ = bytes_sender.send(BodyChunkResponse::Done);
            },
        }
    }

    /// Step 4 and following of <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
    fn transmit_body_chunk(&mut self) {
        if self.in_memory_done {
            // Step 5.1.3
            self.stop_reading(StopReading::Done);
            return;
        }

        let stream = self.stream.clone();
        let control_sender = self.control_sender.clone();
        let bytes_sender = self
            .bytes_sender
            .clone()
            .expect("No bytes sender to transmit chunk.");

        // In case of the data being in-memory, send everything in one chunk, by-passing SpiderMonkey.
        if let Some(bytes) = self.in_memory.clone() {
            let _ = bytes_sender.send(BodyChunkResponse::Chunk(bytes));
            // Mark this body as `done` so that we can stop reading in the next tick,
            // matching the behavior of the promise-based flow
            self.in_memory_done = true;
            return;
        }

        let _ = self.task_source.queue_with_canceller(
            task!(setup_native_body_promise_handler: move || {
                let rooted_stream = stream.root();
                let global = rooted_stream.global();

                // Step 4, the result of reading a chunk from body’s stream with reader.
                let promise = rooted_stream.read_a_chunk();

                // Step 5, the parallel steps waiting for and handling the result of the read promise,
                // are a combination of the promise native handler here,
                // and the corresponding IPC route in `component::net::http_loader`.
                let promise_handler = Box::new(TransmitBodyPromiseHandler {
                    bytes_sender: bytes_sender.clone(),
                    stream: rooted_stream.clone(),
                    control_sender: control_sender.clone(),
                });

                let rejection_handler = Box::new(TransmitBodyPromiseRejectionHandler {
                    bytes_sender,
                    stream: rooted_stream,
                    control_sender,
                });

                let handler =
                    PromiseNativeHandler::new(&global, Some(promise_handler), Some(rejection_handler));

                let realm = enter_realm(&*global);
                let comp = InRealm::Entered(&realm);
                promise.append_native_handler(&handler, comp);
            }),
            &self.canceller,
        );
    }
}

/// The handler of read promises of body streams used in
/// <https://fetch.spec.whatwg.org/#concept-request-transmit-body>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct TransmitBodyPromiseHandler {
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    bytes_sender: IpcSender<BodyChunkResponse>,
    stream: DomRoot<ReadableStream>,
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    control_sender: IpcSender<BodyChunkRequest>,
}

impl Callback for TransmitBodyPromiseHandler {
    /// Step 5 of <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
    fn callback(&self, cx: JSContext, v: HandleValue, _realm: InRealm) {
        let is_done = match get_read_promise_done(cx.clone(), &v) {
            Ok(is_done) => is_done,
            Err(_) => {
                // Step 5.5, the "otherwise" steps.
                // TODO: terminate fetch.
                let _ = self.control_sender.send(BodyChunkRequest::Done);
                return self.stream.stop_reading();
            },
        };

        if is_done {
            // Step 5.3, the "done" steps.
            // TODO: queue a fetch task on request to process request end-of-body.
            let _ = self.control_sender.send(BodyChunkRequest::Done);
            return self.stream.stop_reading();
        }

        let chunk = match get_read_promise_bytes(cx.clone(), &v) {
            Ok(chunk) => chunk,
            Err(_) => {
                // Step 5.5, the "otherwise" steps.
                let _ = self.control_sender.send(BodyChunkRequest::Error);
                return self.stream.stop_reading();
            },
        };

        // Step 5.1 and 5.2, transmit chunk.
        // Send the chunk to the body transmitter in net::http_loader::obtain_response.
        // TODO: queue a fetch task on request to process request body for request.
        let _ = self.bytes_sender.send(BodyChunkResponse::Chunk(chunk));
    }
}

/// The handler of read promises rejection of body streams used in
/// <https://fetch.spec.whatwg.org/#concept-request-transmit-body>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct TransmitBodyPromiseRejectionHandler {
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    bytes_sender: IpcSender<BodyChunkResponse>,
    stream: DomRoot<ReadableStream>,
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    control_sender: IpcSender<BodyChunkRequest>,
}

impl Callback for TransmitBodyPromiseRejectionHandler {
    /// <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm) {
        // Step 5.4, the "rejection" steps.
        let _ = self.control_sender.send(BodyChunkRequest::Error);
        return self.stream.stop_reading();
    }
}

/// The result of <https://fetch.spec.whatwg.org/#concept-bodyinit-extract>
pub struct ExtractedBody {
    pub stream: DomRoot<ReadableStream>,
    pub source: BodySource,
    pub total_bytes: Option<usize>,
    pub content_type: Option<DOMString>,
}

impl ExtractedBody {
    /// Build a request body from the extracted body,
    /// to be sent over IPC to net to use with `concept-request-transmit-body`,
    /// see <https://fetch.spec.whatwg.org/#concept-request-transmit-body>.
    ///
    /// Also returning the corresponding readable stream,
    /// to be stored on the request in script,
    /// and potentially used as part of `consume_body`,
    /// see <https://fetch.spec.whatwg.org/#concept-body-consume-body>
    ///
    /// Transmitting a body over fetch, and consuming it in script,
    /// are mutually exclusive operations, since each will lock the stream to a reader.
    pub fn into_net_request_body(self) -> (RequestBody, DomRoot<ReadableStream>) {
        let ExtractedBody {
            stream,
            total_bytes,
            content_type: _,
            source,
        } = self;

        // First, setup some infra to be used to transmit body
        //  from `components::script` to `components::net`.
        let (chunk_request_sender, chunk_request_receiver) = ipc::channel().unwrap();

        let trusted_stream = Trusted::new(&*stream);

        let global = stream.global();
        let task_source = global.networking_task_source();
        let canceller = global.task_canceller(TaskSourceName::Networking);

        // In case of the data being in-memory, send everything in one chunk, by-passing SM.
        let in_memory = stream.get_in_memory_bytes();

        let net_source = match source {
            BodySource::Null => NetBodySource::Null,
            _ => NetBodySource::Object,
        };

        let mut body_handler = TransmitBodyConnectHandler::new(
            trusted_stream,
            task_source,
            canceller,
            chunk_request_sender.clone(),
            in_memory,
            source,
        );

        ROUTER.add_route(
            chunk_request_receiver.to_opaque(),
            Box::new(move |message| {
                let request = message.to().unwrap();
                match request {
                    BodyChunkRequest::Connect(sender) => {
                        body_handler.start_reading(sender);
                    },
                    BodyChunkRequest::Extract(receiver) => {
                        body_handler.re_extract(receiver);
                    },
                    BodyChunkRequest::Chunk => body_handler.transmit_body_chunk(),
                    // Note: this is actually sent from this process
                    // by the TransmitBodyPromiseHandler when reading stops.
                    BodyChunkRequest::Done => {
                        body_handler.stop_reading(StopReading::Done);
                    },
                    // Note: this is actually sent from this process
                    // by the TransmitBodyPromiseHandler when the stream errors.
                    BodyChunkRequest::Error => {
                        body_handler.stop_reading(StopReading::Error);
                    },
                }
            }),
        );

        // Return `components::net` view into this request body,
        // which can be used by `net` to transmit it over the network.
        let request_body = RequestBody::new(chunk_request_sender, net_source, total_bytes);

        // Also return the stream for this body, which can be used by script to consume it.
        (request_body, stream)
    }

    /// Is the data of the stream of this extracted body available in memory?
    pub fn in_memory(&self) -> bool {
        self.stream.in_memory()
    }
}

/// <https://fetch.spec.whatwg.org/#concept-bodyinit-extract>
pub trait Extractable {
    fn extract(&self, global: &GlobalScope) -> Fallible<ExtractedBody>;
}

impl Extractable for BodyInit {
    // https://fetch.spec.whatwg.org/#concept-bodyinit-extract
    fn extract(&self, global: &GlobalScope) -> Fallible<ExtractedBody> {
        match self {
            BodyInit::String(ref s) => s.extract(global),
            BodyInit::URLSearchParams(ref usp) => usp.extract(global),
            BodyInit::Blob(ref b) => b.extract(global),
            BodyInit::FormData(ref formdata) => formdata.extract(global),
            BodyInit::ArrayBuffer(ref typedarray) => {
                let bytes = typedarray.to_vec();
                let total_bytes = bytes.len();
                let stream = ReadableStream::new_from_bytes(&global, bytes);
                Ok(ExtractedBody {
                    stream,
                    total_bytes: Some(total_bytes),
                    content_type: None,
                    source: BodySource::Object,
                })
            },
            BodyInit::ArrayBufferView(ref typedarray) => {
                let bytes = typedarray.to_vec();
                let total_bytes = bytes.len();
                let stream = ReadableStream::new_from_bytes(&global, bytes);
                Ok(ExtractedBody {
                    stream,
                    total_bytes: Some(total_bytes),
                    content_type: None,
                    source: BodySource::Object,
                })
            },
            BodyInit::ReadableStream(stream) => {
                // TODO:
                // 1. If the keepalive flag is set, then throw a TypeError.

                if stream.is_locked() || stream.is_disturbed() {
                    return Err(Error::Type(
                        "The body's stream is disturbed or locked".to_string(),
                    ));
                }

                Ok(ExtractedBody {
                    stream: stream.clone(),
                    total_bytes: None,
                    content_type: None,
                    source: BodySource::Null,
                })
            },
        }
    }
}

impl Extractable for Vec<u8> {
    fn extract(&self, global: &GlobalScope) -> Fallible<ExtractedBody> {
        let bytes = self.clone();
        let total_bytes = self.len();
        let stream = ReadableStream::new_from_bytes(&global, bytes);
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type: None,
            // A vec is used only in `submit_entity_body`.
            source: BodySource::Object,
        })
    }
}

impl Extractable for Blob {
    fn extract(&self, _global: &GlobalScope) -> Fallible<ExtractedBody> {
        let blob_type = self.Type();
        let content_type = if blob_type.as_ref().is_empty() {
            None
        } else {
            Some(blob_type)
        };
        let total_bytes = self.Size() as usize;
        Ok(ExtractedBody {
            stream: self.get_stream(),
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

impl Extractable for DOMString {
    fn extract(&self, global: &GlobalScope) -> Fallible<ExtractedBody> {
        let bytes = self.as_bytes().to_owned();
        let total_bytes = bytes.len();
        let content_type = Some(DOMString::from("text/plain;charset=UTF-8"));
        let stream = ReadableStream::new_from_bytes(&global, bytes);
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

impl Extractable for FormData {
    fn extract(&self, global: &GlobalScope) -> Fallible<ExtractedBody> {
        let boundary = generate_boundary();
        let bytes = encode_multipart_form_data(&mut self.datums(), boundary.clone(), UTF_8);
        let total_bytes = bytes.len();
        let content_type = Some(DOMString::from(format!(
            "multipart/form-data;boundary={}",
            boundary
        )));
        let stream = ReadableStream::new_from_bytes(&global, bytes);
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

impl Extractable for URLSearchParams {
    fn extract(&self, global: &GlobalScope) -> Fallible<ExtractedBody> {
        let bytes = self.serialize_utf8().into_bytes();
        let total_bytes = bytes.len();
        let content_type = Some(DOMString::from(
            "application/x-www-form-urlencoded;charset=UTF-8",
        ));
        let stream = ReadableStream::new_from_bytes(&global, bytes);
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
pub enum BodyType {
    Blob,
    FormData,
    Json,
    Text,
    ArrayBuffer,
}

pub enum FetchedData {
    Text(String),
    Json(RootedTraceableBox<Heap<JSValue>>),
    BlobData(DomRoot<Blob>),
    FormData(DomRoot<FormData>),
    ArrayBuffer(RootedTraceableBox<Heap<*mut JSObject>>),
    JSException(RootedTraceableBox<Heap<JSVal>>),
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
struct ConsumeBodyPromiseRejectionHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    result_promise: Rc<Promise>,
}

impl Callback for ConsumeBodyPromiseRejectionHandler {
    /// Continuing Step 4 of <https://fetch.spec.whatwg.org/#concept-body-consume-body>
    /// Step 3 of <https://fetch.spec.whatwg.org/#concept-read-all-bytes-from-readablestream>,
    // the rejection steps.
    fn callback(&self, cx: JSContext, v: HandleValue, _realm: InRealm) {
        self.result_promise.reject(cx, v);
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
/// The promise handler used to consume the body,
/// <https://fetch.spec.whatwg.org/#concept-body-consume-body>
struct ConsumeBodyPromiseHandler {
    #[ignore_malloc_size_of = "Rc are hard"]
    result_promise: Rc<Promise>,
    stream: Option<DomRoot<ReadableStream>>,
    body_type: DomRefCell<Option<BodyType>>,
    mime_type: DomRefCell<Option<Vec<u8>>>,
    bytes: DomRefCell<Option<Vec<u8>>>,
}

impl ConsumeBodyPromiseHandler {
    /// Step 5 of <https://fetch.spec.whatwg.org/#concept-body-consume-body>
    fn resolve_result_promise(&self, cx: JSContext) {
        let body_type = self.body_type.borrow_mut().take().unwrap();
        let mime_type = self.mime_type.borrow_mut().take().unwrap();
        let body = self.bytes.borrow_mut().take().unwrap();

        let pkg_data_results = run_package_data_algorithm(cx, body, body_type, mime_type);

        match pkg_data_results {
            Ok(results) => {
                match results {
                    FetchedData::Text(s) => self.result_promise.resolve_native(&USVString(s)),
                    FetchedData::Json(j) => self.result_promise.resolve_native(&j),
                    FetchedData::BlobData(b) => self.result_promise.resolve_native(&b),
                    FetchedData::FormData(f) => self.result_promise.resolve_native(&f),
                    FetchedData::ArrayBuffer(a) => self.result_promise.resolve_native(&a),
                    FetchedData::JSException(e) => self.result_promise.reject_native(&e.handle()),
                };
            },
            Err(err) => self.result_promise.reject_error(err),
        }
    }
}

impl Callback for ConsumeBodyPromiseHandler {
    /// Continuing Step 4 of <https://fetch.spec.whatwg.org/#concept-body-consume-body>
    /// Step 3 of <https://fetch.spec.whatwg.org/#concept-read-all-bytes-from-readablestream>.
    fn callback(&self, cx: JSContext, v: HandleValue, _realm: InRealm) {
        let stream = self
            .stream
            .as_ref()
            .expect("ConsumeBodyPromiseHandler has no stream in callback.");

        let is_done = match get_read_promise_done(cx.clone(), &v) {
            Ok(is_done) => is_done,
            Err(err) => {
                stream.stop_reading();
                // When read is fulfilled with a value that doesn't matches with neither of the above patterns.
                return self.result_promise.reject_error(err);
            },
        };

        if is_done {
            // When read is fulfilled with an object whose done property is true.
            self.resolve_result_promise(cx.clone());
        } else {
            let chunk = match get_read_promise_bytes(cx.clone(), &v) {
                Ok(chunk) => chunk,
                Err(err) => {
                    stream.stop_reading();
                    // When read is fulfilled with a value that matches with neither of the above patterns
                    return self.result_promise.reject_error(err);
                },
            };

            let mut bytes = self
                .bytes
                .borrow_mut()
                .take()
                .expect("No bytes for ConsumeBodyPromiseHandler.");

            // Append the value property to bytes.
            bytes.extend_from_slice(&*chunk);

            let global = stream.global();

            // Run the above step again.
            let read_promise = stream.read_a_chunk();

            let promise_handler = Box::new(ConsumeBodyPromiseHandler {
                result_promise: self.result_promise.clone(),
                stream: self.stream.clone(),
                body_type: DomRefCell::new(self.body_type.borrow_mut().take()),
                mime_type: DomRefCell::new(self.mime_type.borrow_mut().take()),
                bytes: DomRefCell::new(Some(bytes)),
            });

            let rejection_handler = Box::new(ConsumeBodyPromiseRejectionHandler {
                result_promise: self.result_promise.clone(),
            });

            let handler =
                PromiseNativeHandler::new(&global, Some(promise_handler), Some(rejection_handler));

            let realm = enter_realm(&*global);
            let comp = InRealm::Entered(&realm);
            read_promise.append_native_handler(&handler, comp);
        }
    }
}

// https://fetch.spec.whatwg.org/#concept-body-consume-body
#[allow(crown::unrooted_must_root)]
pub fn consume_body<T: BodyMixin + DomObject>(object: &T, body_type: BodyType) -> Rc<Promise> {
    let in_realm_proof = AlreadyInRealm::assert();
    let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof));

    // Step 1
    if object.is_disturbed() || object.is_locked() {
        promise.reject_error(Error::Type(
            "The body's stream is disturbed or locked".to_string(),
        ));
        return promise;
    }

    consume_body_with_promise(
        object,
        body_type,
        promise.clone(),
        InRealm::Already(&in_realm_proof),
    );

    promise
}

// https://fetch.spec.whatwg.org/#concept-body-consume-body
#[allow(crown::unrooted_must_root)]
fn consume_body_with_promise<T: BodyMixin + DomObject>(
    object: &T,
    body_type: BodyType,
    promise: Rc<Promise>,
    comp: InRealm,
) {
    let global = object.global();

    // Step 2.
    let stream = match object.body() {
        Some(stream) => stream,
        None => {
            let stream = ReadableStream::new_from_bytes(&global, Vec::with_capacity(0));
            stream
        },
    };

    // Step 3.
    if stream.start_reading().is_err() {
        return promise.reject_error(Error::Type(
            "The response's stream is disturbed or locked".to_string(),
        ));
    }

    // Step 4, read all the bytes.
    // Starts here, continues in the promise handler.

    // Step 1 of
    // https://fetch.spec.whatwg.org/#concept-read-all-bytes-from-readablestream
    let read_promise = stream.read_a_chunk();

    let promise_handler = Box::new(ConsumeBodyPromiseHandler {
        result_promise: promise.clone(),
        stream: Some(stream),
        body_type: DomRefCell::new(Some(body_type)),
        mime_type: DomRefCell::new(Some(object.get_mime_type())),
        // Step 2.
        bytes: DomRefCell::new(Some(vec![])),
    });

    let rejection_handler = Box::new(ConsumeBodyPromiseRejectionHandler {
        result_promise: promise,
    });

    let handler = PromiseNativeHandler::new(
        &object.global(),
        Some(promise_handler),
        Some(rejection_handler),
    );
    // We are already in a realm and a script.
    read_promise.append_native_handler(&handler, comp);
}

// https://fetch.spec.whatwg.org/#concept-body-package-data
fn run_package_data_algorithm(
    cx: JSContext,
    bytes: Vec<u8>,
    body_type: BodyType,
    mime_type: Vec<u8>,
) -> Fallible<FetchedData> {
    let mime = &*mime_type;
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    let global = GlobalScope::from_safe_context(cx, InRealm::Already(&in_realm_proof));
    match body_type {
        BodyType::Text => run_text_data_algorithm(bytes),
        BodyType::Json => run_json_data_algorithm(cx, bytes),
        BodyType::Blob => run_blob_data_algorithm(&global, bytes, mime),
        BodyType::FormData => run_form_data_algorithm(&global, bytes, mime),
        BodyType::ArrayBuffer => run_array_buffer_data_algorithm(cx, bytes),
    }
}

fn run_text_data_algorithm(bytes: Vec<u8>) -> Fallible<FetchedData> {
    Ok(FetchedData::Text(
        String::from_utf8_lossy(&bytes).into_owned(),
    ))
}

#[allow(unsafe_code)]
fn run_json_data_algorithm(cx: JSContext, bytes: Vec<u8>) -> Fallible<FetchedData> {
    let json_text = String::from_utf8_lossy(&bytes);
    let json_text: Vec<u16> = json_text.encode_utf16().collect();
    rooted!(in(*cx) let mut rval = UndefinedValue());
    unsafe {
        if !JS_ParseJSON(
            *cx,
            json_text.as_ptr(),
            json_text.len() as u32,
            rval.handle_mut(),
        ) {
            rooted!(in(*cx) let mut exception = UndefinedValue());
            assert!(JS_GetPendingException(*cx, exception.handle_mut()));
            JS_ClearPendingException(*cx);
            return Ok(FetchedData::JSException(RootedTraceableBox::from_box(
                Heap::boxed(exception.get()),
            )));
        }
        let rooted_heap = RootedTraceableBox::from_box(Heap::boxed(rval.get()));
        Ok(FetchedData::Json(rooted_heap))
    }
}

fn run_blob_data_algorithm(
    root: &GlobalScope,
    bytes: Vec<u8>,
    mime: &[u8],
) -> Fallible<FetchedData> {
    let mime_string = if let Ok(s) = String::from_utf8(mime.to_vec()) {
        s
    } else {
        "".to_string()
    };
    let blob = Blob::new(
        root,
        BlobImpl::new_from_bytes(bytes, normalize_type_string(&mime_string)),
    );
    Ok(FetchedData::BlobData(blob))
}

fn run_form_data_algorithm(
    root: &GlobalScope,
    bytes: Vec<u8>,
    mime: &[u8],
) -> Fallible<FetchedData> {
    let mime_str = if let Ok(s) = str::from_utf8(mime) {
        s
    } else {
        ""
    };
    let mime: Mime = mime_str
        .parse()
        .map_err(|_| Error::Type("Inappropriate MIME-type for Body".to_string()))?;

    // TODO
    // ... Parser for Mime(TopLevel::Multipart, SubLevel::FormData, _)
    // ... is not fully determined yet.
    if mime.type_() == mime::APPLICATION && mime.subtype() == mime::WWW_FORM_URLENCODED {
        let entries = form_urlencoded::parse(&bytes);
        let formdata = FormData::new(None, root);
        for (k, e) in entries {
            formdata.Append(USVString(k.into_owned()), USVString(e.into_owned()));
        }
        return Ok(FetchedData::FormData(formdata));
    }

    Err(Error::Type("Inappropriate MIME-type for Body".to_string()))
}

#[allow(unsafe_code)]
pub fn run_array_buffer_data_algorithm(cx: JSContext, bytes: Vec<u8>) -> Fallible<FetchedData> {
    rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
    let arraybuffer = unsafe {
        ArrayBuffer::create(
            *cx,
            CreateWith::Slice(&bytes),
            array_buffer_ptr.handle_mut(),
        )
    };
    if arraybuffer.is_err() {
        return Err(Error::JSFailed);
    }
    let rooted_heap = RootedTraceableBox::from_box(Heap::boxed(array_buffer_ptr.get()));
    Ok(FetchedData::ArrayBuffer(rooted_heap))
}

/// <https://fetch.spec.whatwg.org/#body>
pub trait BodyMixin {
    /// <https://fetch.spec.whatwg.org/#concept-body-disturbed>
    fn is_disturbed(&self) -> bool;
    /// <https://fetch.spec.whatwg.org/#dom-body-body>
    fn body(&self) -> Option<DomRoot<ReadableStream>>;
    /// <https://fetch.spec.whatwg.org/#concept-body-locked>
    fn is_locked(&self) -> bool;
    /// <https://fetch.spec.whatwg.org/#concept-body-mime-type>
    fn get_mime_type(&self) -> Vec<u8>;
}
