/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::{ptr, slice, str};

use base::generic_channel::GenericSharedMemory;
use constellation_traits::BlobImpl;
use encoding_rs::{Encoding, UTF_8};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{Heap, JS_ClearPendingException, JSObject, Value as JSValue};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleValue;
use js::rust::wrappers::{JS_GetPendingException, JS_ParseJSON};
use js::typedarray::{ArrayBufferU8, Uint8};
use mime::{self, Mime};
use net_traits::request::{
    BodyChunkRequest, BodyChunkResponse, BodySource as NetBodySource, RequestBody,
};
use url::form_urlencoded;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::BlobBinding::Blob_Binding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use crate::dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::blob::{Blob, normalize_type_string};
use crate::dom::formdata::FormData;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlformelement::{encode_multipart_form_data, generate_boundary};
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::readablestream::{ReadableStream, get_read_promise_bytes, get_read_promise_done};
use crate::dom::urlsearchparams::URLSearchParams;
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext};
use crate::task_source::SendableTaskSource;

/// The Dom object, or ReadableStream, that is the source of a body.
/// <https://fetch.spec.whatwg.org/#concept-body-source>
#[derive(Clone, PartialEq)]
pub(crate) enum BodySource {
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
    task_source: SendableTaskSource,
    bytes_sender: Option<IpcSender<BodyChunkResponse>>,
    control_sender: IpcSender<BodyChunkRequest>,
    in_memory: Option<GenericSharedMemory>,
    in_memory_done: bool,
    source: BodySource,
}

impl TransmitBodyConnectHandler {
    pub(crate) fn new(
        stream: Trusted<ReadableStream>,
        task_source: SendableTaskSource,
        control_sender: IpcSender<BodyChunkRequest>,
        in_memory: Option<GenericSharedMemory>,
        source: BodySource,
    ) -> TransmitBodyConnectHandler {
        TransmitBodyConnectHandler {
            stream,
            task_source,
            bytes_sender: None,
            control_sender,
            in_memory,
            in_memory_done: false,
            source,
        }
    }

    /// Reset `in_memory_done`, called when a stream is
    /// re-extracted from the source to support a re-direct.
    pub(crate) fn reset_in_memory_done(&mut self) {
        self.in_memory_done = false;
    }

    /// Re-extract the source to support streaming it again for a re-direct.
    /// TODO: actually re-extract the source, instead of just cloning data, to support Blob.
    fn re_extract(&mut self, chunk_request_receiver: IpcReceiver<BodyChunkRequest>) {
        let mut body_handler = self.clone();
        body_handler.reset_in_memory_done();

        ROUTER.add_typed_route(
            chunk_request_receiver,
            Box::new(move |message| {
                let request = message.unwrap();
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
                .send(BodyChunkResponse::Chunk(bytes));
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
            self.task_source
                .queue(task!(start_reading_request_body_stream: move || {
                    // Step 1, Let body be request’s body.
                    let rooted_stream = stream.root();

                    // TODO: Step 2, If body is null.

                    // Step 3, get a reader for stream.
                    rooted_stream.acquire_default_reader(CanGc::note())
                        .expect("Couldn't acquire a reader for the body stream.");

                    // Note: this algorithm continues when the first chunk is requested by `net`.
                }));
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

        self.task_source.queue(
            task!(setup_native_body_promise_handler: move || {
                let rooted_stream = stream.root();
                let global = rooted_stream.global();
                let cx = GlobalScope::get_cx();

                // Step 4, the result of reading a chunk from body’s stream with reader.
                let promise = rooted_stream.read_a_chunk(CanGc::note());

                // Step 5, the parallel steps waiting for and handling the result of the read promise,
                // are a combination of the promise native handler here,
                // and the corresponding IPC route in `component::net::http_loader`.
                rooted!(in(*cx) let mut promise_handler = Some(TransmitBodyPromiseHandler {
                    bytes_sender: bytes_sender.clone(),
                    stream: Dom::from_ref(&rooted_stream.clone()),
                    control_sender: control_sender.clone(),
                }));

                rooted!(in(*cx) let mut rejection_handler = Some(TransmitBodyPromiseRejectionHandler {
                    bytes_sender,
                    stream: Dom::from_ref(&rooted_stream.clone()),
                    control_sender,
                }));

                let handler =
                    PromiseNativeHandler::new(&global, promise_handler.take().map(|h| Box::new(h) as Box<_>), rejection_handler.take().map(|h| Box::new(h) as Box<_>), CanGc::note());

                let realm = enter_realm(&*global);
                let comp = InRealm::Entered(&realm);
                promise.append_native_handler(&handler, comp, CanGc::note());
            })
        );
    }
}

/// The handler of read promises of body streams used in
/// <https://fetch.spec.whatwg.org/#concept-request-transmit-body>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct TransmitBodyPromiseHandler {
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    bytes_sender: IpcSender<BodyChunkResponse>,
    stream: Dom<ReadableStream>,
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    control_sender: IpcSender<BodyChunkRequest>,
}

impl js::gc::Rootable for TransmitBodyPromiseHandler {}

impl Callback for TransmitBodyPromiseHandler {
    /// Step 5 of <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
    fn callback(&self, cx: JSContext, v: HandleValue, _realm: InRealm, can_gc: CanGc) {
        let is_done = match get_read_promise_done(cx, &v, can_gc) {
            Ok(is_done) => is_done,
            Err(_) => {
                // Step 5.5, the "otherwise" steps.
                // TODO: terminate fetch.
                let _ = self.control_sender.send(BodyChunkRequest::Done);
                return self.stream.stop_reading(can_gc);
            },
        };

        if is_done {
            // Step 5.3, the "done" steps.
            // TODO: queue a fetch task on request to process request end-of-body.
            let _ = self.control_sender.send(BodyChunkRequest::Done);
            return self.stream.stop_reading(can_gc);
        }

        let chunk = match get_read_promise_bytes(cx, &v, can_gc) {
            Ok(chunk) => chunk,
            Err(_) => {
                // Step 5.5, the "otherwise" steps.
                let _ = self.control_sender.send(BodyChunkRequest::Error);
                return self.stream.stop_reading(can_gc);
            },
        };

        // Step 5.1 and 5.2, transmit chunk.
        // Send the chunk to the body transmitter in net::http_loader::obtain_response.
        // TODO: queue a fetch task on request to process request body for request.
        let _ = self
            .bytes_sender
            .send(BodyChunkResponse::Chunk(GenericSharedMemory::from_bytes(
                &chunk,
            )));
    }
}

/// The handler of read promises rejection of body streams used in
/// <https://fetch.spec.whatwg.org/#concept-request-transmit-body>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct TransmitBodyPromiseRejectionHandler {
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    bytes_sender: IpcSender<BodyChunkResponse>,
    stream: Dom<ReadableStream>,
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    control_sender: IpcSender<BodyChunkRequest>,
}

impl js::gc::Rootable for TransmitBodyPromiseRejectionHandler {}

impl Callback for TransmitBodyPromiseRejectionHandler {
    /// <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
    fn callback(&self, _cx: JSContext, _v: HandleValue, _realm: InRealm, can_gc: CanGc) {
        // Step 5.4, the "rejection" steps.
        let _ = self.control_sender.send(BodyChunkRequest::Error);
        self.stream.stop_reading(can_gc);
    }
}

/// <https://fetch.spec.whatwg.org/#body-with-type>
pub(crate) struct ExtractedBody {
    /// <https://fetch.spec.whatwg.org/#concept-body-stream>
    pub(crate) stream: DomRoot<ReadableStream>,
    /// <https://fetch.spec.whatwg.org/#concept-body-source>
    pub(crate) source: BodySource,
    /// <https://fetch.spec.whatwg.org/#concept-body-total-bytes>
    pub(crate) total_bytes: Option<usize>,
    /// <https://fetch.spec.whatwg.org/#body-with-type-type>
    pub(crate) content_type: Option<DOMString>,
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
    pub(crate) fn into_net_request_body(self) -> (RequestBody, DomRoot<ReadableStream>) {
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
        let task_source = global.task_manager().networking_task_source();

        // In case of the data being in-memory, send everything in one chunk, by-passing SM.
        let in_memory = stream.get_in_memory_bytes();

        let net_source = match source {
            BodySource::Null => NetBodySource::Null,
            _ => NetBodySource::Object,
        };

        let mut body_handler = TransmitBodyConnectHandler::new(
            trusted_stream,
            task_source.into(),
            chunk_request_sender.clone(),
            in_memory,
            source,
        );

        ROUTER.add_typed_route(
            chunk_request_receiver,
            Box::new(move |message| {
                match message.unwrap() {
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
    pub(crate) fn in_memory(&self) -> bool {
        self.stream.in_memory()
    }
}

/// <https://fetch.spec.whatwg.org/#concept-bodyinit-extract>
pub(crate) trait Extractable {
    fn extract(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<ExtractedBody>;
}

impl Extractable for BodyInit {
    /// <https://fetch.spec.whatwg.org/#concept-bodyinit-extract>
    fn extract(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<ExtractedBody> {
        match self {
            BodyInit::String(s) => s.extract(global, can_gc),
            BodyInit::URLSearchParams(usp) => usp.extract(global, can_gc),
            BodyInit::Blob(b) => b.extract(global, can_gc),
            BodyInit::FormData(formdata) => formdata.extract(global, can_gc),
            BodyInit::ArrayBuffer(typedarray) => {
                let bytes = typedarray.to_vec();
                let total_bytes = bytes.len();
                let stream = ReadableStream::new_from_bytes(global, bytes, can_gc)?;
                Ok(ExtractedBody {
                    stream,
                    total_bytes: Some(total_bytes),
                    content_type: None,
                    source: BodySource::Object,
                })
            },
            BodyInit::ArrayBufferView(typedarray) => {
                let bytes = typedarray.to_vec();
                let total_bytes = bytes.len();
                let stream = ReadableStream::new_from_bytes(global, bytes, can_gc)?;
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
    fn extract(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<ExtractedBody> {
        let bytes = self.clone();
        let total_bytes = self.len();
        let stream = ReadableStream::new_from_bytes(global, bytes, can_gc)?;
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
    fn extract(&self, _global: &GlobalScope, can_gc: CanGc) -> Fallible<ExtractedBody> {
        let blob_type = self.Type();
        let content_type = if blob_type.is_empty() {
            None
        } else {
            Some(blob_type)
        };
        let total_bytes = self.Size() as usize;
        let stream = self.get_stream(can_gc)?;
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

impl Extractable for DOMString {
    fn extract(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<ExtractedBody> {
        let bytes = self.as_bytes().to_owned();
        let total_bytes = bytes.len();
        let content_type = Some(DOMString::from("text/plain;charset=UTF-8"));
        let stream = ReadableStream::new_from_bytes(global, bytes, can_gc)?;
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

impl Extractable for FormData {
    fn extract(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<ExtractedBody> {
        let boundary = generate_boundary();
        let bytes = encode_multipart_form_data(&mut self.datums(), boundary.clone(), UTF_8);
        let total_bytes = bytes.len();
        let content_type = Some(DOMString::from(format!(
            "multipart/form-data; boundary={}",
            boundary
        )));
        let stream = ReadableStream::new_from_bytes(global, bytes, can_gc)?;
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

impl Extractable for URLSearchParams {
    fn extract(&self, global: &GlobalScope, can_gc: CanGc) -> Fallible<ExtractedBody> {
        let bytes = self.serialize_utf8().into_bytes();
        let total_bytes = bytes.len();
        let content_type = Some(DOMString::from(
            "application/x-www-form-urlencoded;charset=UTF-8",
        ));
        let stream = ReadableStream::new_from_bytes(global, bytes, can_gc)?;
        Ok(ExtractedBody {
            stream,
            total_bytes: Some(total_bytes),
            content_type,
            source: BodySource::Object,
        })
    }
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
pub(crate) enum BodyType {
    Blob,
    Bytes,
    FormData,
    Json,
    Text,
    ArrayBuffer,
}

pub(crate) enum FetchedData {
    Text(String),
    Json(RootedTraceableBox<Heap<JSValue>>),
    BlobData(DomRoot<Blob>),
    Bytes(RootedTraceableBox<Heap<*mut JSObject>>),
    FormData(DomRoot<FormData>),
    ArrayBuffer(RootedTraceableBox<Heap<*mut JSObject>>),
    JSException(RootedTraceableBox<Heap<JSVal>>),
}

/// <https://fetch.spec.whatwg.org/#concept-body-consume-body>
/// <https://fetch.spec.whatwg.org/#body-fully-read>
/// A combination of parts of both algorithms,
/// `body-fully-read` can be fully implemented, and separated, later,
/// see #36049.
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) fn consume_body<T: BodyMixin + DomObject>(
    object: &T,
    body_type: BodyType,
    can_gc: CanGc,
) -> Rc<Promise> {
    let global = object.global();
    let cx = GlobalScope::get_cx();

    // Enter the realm of the object whose body is being consumed.
    let realm = enter_realm(&*global);
    let comp = InRealm::Entered(&realm);

    // Let promise be a new promise.
    // Note: re-ordered so we can return the promise below.
    let promise = Promise::new_in_current_realm(comp, can_gc);

    // If object is unusable, then return a promise rejected with a TypeError.
    if object.is_unusable() {
        promise.reject_error(
            Error::Type("The body's stream is disturbed or locked".to_string()),
            can_gc,
        );
        return promise;
    }

    let stream = match object.body() {
        Some(stream) => stream,
        None => {
            // If object’s body is null, then run successSteps with an empty byte sequence.
            resolve_result_promise(
                body_type,
                &promise,
                object.get_mime_type(can_gc),
                Vec::with_capacity(0),
                cx,
                can_gc,
            );
            return promise;
        },
    };

    // Note: from `fully_read`.
    // Let reader be the result of getting a reader for body’s stream.
    // If that threw an exception,
    // then run errorSteps with that exception and return.
    let reader = match stream.acquire_default_reader(can_gc) {
        Ok(r) => r,
        Err(e) => {
            promise.reject_error(e, can_gc);
            return promise;
        },
    };

    // Let errorSteps given error be to reject promise with error.
    let error_promise = promise.clone();

    // Let successSteps given a byte sequence data be to resolve promise
    // with the result of running convertBytesToJSValue with data.
    // If that threw an exception, then run errorSteps with that exception.
    let mime_type = object.get_mime_type(can_gc);
    let success_promise = promise.clone();

    // Read all bytes from reader, given successSteps and errorSteps.
    // Note: spec uses an intermediary concept of `fully_read`,
    // which seems useful when invoking fetch from other places.
    // TODO: #36049
    reader.read_all_bytes(
        cx,
        Rc::new(move |bytes: &[u8]| {
            resolve_result_promise(
                body_type,
                &success_promise,
                mime_type.clone(),
                bytes.to_vec(),
                cx,
                can_gc,
            );
        }),
        Rc::new(move |cx, v| {
            error_promise.reject(cx, v, can_gc);
        }),
        can_gc,
    );

    promise
}

/// The success steps of
/// <https://fetch.spec.whatwg.org/#concept-body-consume-body>.
fn resolve_result_promise(
    body_type: BodyType,
    promise: &Promise,
    mime_type: Vec<u8>,
    body: Vec<u8>,
    cx: JSContext,
    can_gc: CanGc,
) {
    let pkg_data_results = run_package_data_algorithm(cx, body, body_type, mime_type, can_gc);

    match pkg_data_results {
        Ok(results) => {
            match results {
                FetchedData::Text(s) => promise.resolve_native(&USVString(s), can_gc),
                FetchedData::Json(j) => promise.resolve_native(&j, can_gc),
                FetchedData::BlobData(b) => promise.resolve_native(&b, can_gc),
                FetchedData::FormData(f) => promise.resolve_native(&f, can_gc),
                FetchedData::Bytes(b) => promise.resolve_native(&b, can_gc),
                FetchedData::ArrayBuffer(a) => promise.resolve_native(&a, can_gc),
                FetchedData::JSException(e) => promise.reject_native(&e.handle(), can_gc),
            };
        },
        Err(err) => promise.reject_error(err, can_gc),
    }
}

/// The algorithm that takes a byte sequence
/// and returns a JavaScript value or throws an exception of
/// <https://fetch.spec.whatwg.org/#concept-body-consume-body>.
fn run_package_data_algorithm(
    cx: JSContext,
    bytes: Vec<u8>,
    body_type: BodyType,
    mime_type: Vec<u8>,
    can_gc: CanGc,
) -> Fallible<FetchedData> {
    let mime = &*mime_type;
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    let global = GlobalScope::from_safe_context(cx, InRealm::Already(&in_realm_proof));
    match body_type {
        BodyType::Text => run_text_data_algorithm(bytes),
        BodyType::Json => run_json_data_algorithm(cx, bytes),
        BodyType::Blob => run_blob_data_algorithm(&global, bytes, mime, can_gc),
        BodyType::FormData => run_form_data_algorithm(&global, bytes, mime, can_gc),
        BodyType::ArrayBuffer => run_array_buffer_data_algorithm(cx, bytes, can_gc),
        BodyType::Bytes => run_bytes_data_algorithm(cx, bytes, can_gc),
    }
}

/// <https://fetch.spec.whatwg.org/#ref-for-concept-body-consume-body%E2%91%A4>
fn run_text_data_algorithm(bytes: Vec<u8>) -> Fallible<FetchedData> {
    // This implements the Encoding standard's "decode UTF-8", which removes the
    // BOM if present.
    let no_bom_bytes = if bytes.starts_with(b"\xEF\xBB\xBF") {
        &bytes[3..]
    } else {
        &bytes
    };
    Ok(FetchedData::Text(
        String::from_utf8_lossy(no_bom_bytes).into_owned(),
    ))
}

#[expect(unsafe_code)]
/// <https://fetch.spec.whatwg.org/#ref-for-concept-body-consume-body%E2%91%A3>
fn run_json_data_algorithm(cx: JSContext, bytes: Vec<u8>) -> Fallible<FetchedData> {
    // The JSON spec allows implementations to either ignore UTF-8 BOM or treat it as an error.
    // `JS_ParseJSON` treats this as an error, so it is necessary for us to strip it if present.
    //
    // https://datatracker.ietf.org/doc/html/rfc8259#section-8.1
    let json_text = decode_to_utf16_with_bom_removal(&bytes, UTF_8);
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

/// <https://fetch.spec.whatwg.org/#ref-for-concept-body-consume-body%E2%91%A0>
fn run_blob_data_algorithm(
    root: &GlobalScope,
    bytes: Vec<u8>,
    mime: &[u8],
    can_gc: CanGc,
) -> Fallible<FetchedData> {
    let mime_string = if let Ok(s) = String::from_utf8(mime.to_vec()) {
        s
    } else {
        "".to_string()
    };
    let blob = Blob::new(
        root,
        BlobImpl::new_from_bytes(bytes, normalize_type_string(&mime_string)),
        can_gc,
    );
    Ok(FetchedData::BlobData(blob))
}

/// <https://fetch.spec.whatwg.org/#ref-for-concept-body-consume-body%E2%91%A2>
fn run_form_data_algorithm(
    root: &GlobalScope,
    bytes: Vec<u8>,
    mime: &[u8],
    can_gc: CanGc,
) -> Fallible<FetchedData> {
    let mime_str = str::from_utf8(mime).unwrap_or_default();
    let mime: Mime = mime_str
        .parse()
        .map_err(|_| Error::Type("Inappropriate MIME-type for Body".to_string()))?;

    // TODO
    // ... Parser for Mime(TopLevel::Multipart, SubLevel::FormData, _)
    // ... is not fully determined yet.
    if mime.type_() == mime::APPLICATION && mime.subtype() == mime::WWW_FORM_URLENCODED {
        let entries = form_urlencoded::parse(&bytes);
        let formdata = FormData::new(None, root, can_gc);
        for (k, e) in entries {
            formdata.Append(USVString(k.into_owned()), USVString(e.into_owned()));
        }
        return Ok(FetchedData::FormData(formdata));
    }

    Err(Error::Type("Inappropriate MIME-type for Body".to_string()))
}

/// <https://fetch.spec.whatwg.org/#ref-for-concept-body-consume-body%E2%91%A1>
fn run_bytes_data_algorithm(cx: JSContext, bytes: Vec<u8>, can_gc: CanGc) -> Fallible<FetchedData> {
    rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

    create_buffer_source::<Uint8>(cx, &bytes, array_buffer_ptr.handle_mut(), can_gc)
        .map_err(|_| Error::JSFailed)?;

    let rooted_heap = RootedTraceableBox::from_box(Heap::boxed(array_buffer_ptr.get()));
    Ok(FetchedData::Bytes(rooted_heap))
}

/// <https://fetch.spec.whatwg.org/#ref-for-concept-body-consume-body>
pub(crate) fn run_array_buffer_data_algorithm(
    cx: JSContext,
    bytes: Vec<u8>,
    can_gc: CanGc,
) -> Fallible<FetchedData> {
    rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

    create_buffer_source::<ArrayBufferU8>(cx, &bytes, array_buffer_ptr.handle_mut(), can_gc)
        .map_err(|_| Error::JSFailed)?;

    let rooted_heap = RootedTraceableBox::from_box(Heap::boxed(array_buffer_ptr.get()));
    Ok(FetchedData::ArrayBuffer(rooted_heap))
}

#[expect(unsafe_code)]
pub(crate) fn decode_to_utf16_with_bom_removal(
    bytes: &[u8],
    encoding: &'static Encoding,
) -> Vec<u16> {
    let mut decoder = encoding.new_decoder_with_bom_removal();
    let capacity = decoder
        .max_utf16_buffer_length(bytes.len())
        .expect("Overflow");
    let mut utf16 = Vec::with_capacity(capacity);
    let extra = unsafe { slice::from_raw_parts_mut(utf16.as_mut_ptr(), capacity) };
    let (_, read, written, _) = decoder.decode_to_utf16(bytes, extra, true);
    assert_eq!(read, bytes.len());
    unsafe { utf16.set_len(written) }
    utf16
}

/// <https://fetch.spec.whatwg.org/#body>
pub(crate) trait BodyMixin {
    /// <https://fetch.spec.whatwg.org/#dom-body-bodyused>
    fn is_body_used(&self) -> bool;
    /// <https://fetch.spec.whatwg.org/#body-unusable>
    fn is_unusable(&self) -> bool;
    /// <https://fetch.spec.whatwg.org/#dom-body-body>
    fn body(&self) -> Option<DomRoot<ReadableStream>>;
    /// <https://fetch.spec.whatwg.org/#concept-body-mime-type>
    fn get_mime_type(&self, can_gc: CanGc) -> Vec<u8>;
}
