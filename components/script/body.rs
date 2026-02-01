/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::Cursor;
use std::rc::Rc;
use std::{fs, ptr, slice, str};

use base::generic_channel::GenericSharedMemory;
use constellation_traits::BlobImpl;
use encoding_rs::{Encoding, UTF_8};
use http::HeaderMap;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{Heap, JS_ClearPendingException, JSObject, Value as JSValue};
use js::jsval::{JSVal, UndefinedValue};
use js::realm::CurrentRealm;
use js::rust::HandleValue;
use js::rust::wrappers::{JS_GetPendingException, JS_ParseJSON};
use js::typedarray::{ArrayBufferU8, Uint8};
use mime::{self, Mime};
use mime_multipart_hyper1::{Node, read_multipart_body};
use net_traits::request::{
    BodyChunkRequest, BodyChunkResponse, BodySource as NetBodySource, RequestBody,
};
use url::form_urlencoded;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::BlobBinding::Blob_Binding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use crate::dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::blob::{Blob, normalize_type_string};
use crate::dom::file::File;
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

/// <https://fetch.spec.whatwg.org/#concept-body-clone>
pub(crate) fn clone_body_stream_for_dom_body(
    original_body_stream: &MutNullableDom<ReadableStream>,
    cloned_body_stream: &MutNullableDom<ReadableStream>,
    can_gc: CanGc,
) -> Fallible<()> {
    // To clone a body *body*, run these steps:

    let Some(stream) = original_body_stream.get() else {
        return Ok(());
    };

    // step 1. Let « out1, out2 » be the result of teeing body’s stream.
    let branches = stream.tee(true, can_gc)?;
    let out1 = &*branches[0];
    let out2 = &*branches[1];

    // step 2. Set body’s stream to out1.
    // step 3. Return a body whose stream is out2 and other members are copied from body.
    original_body_stream.set(Some(out1));
    cloned_body_stream.set(Some(out2));

    Ok(())
}

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
    control_sender: Option<IpcSender<BodyChunkRequest>>,
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
            control_sender: Some(control_sender),
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
    /// It is important to drop the control_sender as this will allow us to clean ourselves up.
    /// Otherwise, the following cycle will happen: The control sender is owned by us which keeps the control receiver
    /// alive in the router which keeps us alive.
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
        let _ = self.control_sender.take();
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
                    control_sender: control_sender.clone().unwrap(),
                }));

                rooted!(in(*cx) let mut rejection_handler = Some(TransmitBodyPromiseRejectionHandler {
                    bytes_sender,
                    stream: Dom::from_ref(&rooted_stream.clone()),
                    control_sender: control_sender.unwrap(),
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
    fn callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        let can_gc = CanGc::from_cx(cx);
        let _realm = InRealm::Already(&cx.into());
        let cx = cx.into();
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
    fn callback(&self, cx: &mut CurrentRealm, _v: HandleValue) {
        // Step 5.4, the "rejection" steps.
        let _ = self.control_sender.send(BodyChunkRequest::Error);
        self.stream.stop_reading(CanGc::from_cx(cx));
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
    fn extract(
        &self,
        global: &GlobalScope,
        keep_alive: bool,
        can_gc: CanGc,
    ) -> Fallible<ExtractedBody>;
}

impl Extractable for BodyInit {
    /// <https://fetch.spec.whatwg.org/#concept-bodyinit-extract>
    fn extract(
        &self,
        global: &GlobalScope,
        keep_alive: bool,
        can_gc: CanGc,
    ) -> Fallible<ExtractedBody> {
        match self {
            BodyInit::String(s) => s.extract(global, keep_alive, can_gc),
            BodyInit::URLSearchParams(usp) => usp.extract(global, keep_alive, can_gc),
            BodyInit::Blob(b) => b.extract(global, keep_alive, can_gc),
            BodyInit::FormData(formdata) => formdata.extract(global, keep_alive, can_gc),
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
                // If keepalive is true, then throw a TypeError.
                if keep_alive {
                    return Err(Error::Type(
                        "The body's stream is for a keepalive request".to_string(),
                    ));
                }
                // If object is disturbed or locked, then throw a TypeError.
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
    fn extract(
        &self,
        global: &GlobalScope,
        _keep_alive: bool,
        can_gc: CanGc,
    ) -> Fallible<ExtractedBody> {
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
    fn extract(
        &self,
        _global: &GlobalScope,
        _keep_alive: bool,
        can_gc: CanGc,
    ) -> Fallible<ExtractedBody> {
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
    fn extract(
        &self,
        global: &GlobalScope,
        _keep_alive: bool,
        can_gc: CanGc,
    ) -> Fallible<ExtractedBody> {
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
    fn extract(
        &self,
        global: &GlobalScope,
        _keep_alive: bool,
        can_gc: CanGc,
    ) -> Fallible<ExtractedBody> {
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
    fn extract(
        &self,
        global: &GlobalScope,
        _keep_alive: bool,
        can_gc: CanGc,
    ) -> Fallible<ExtractedBody> {
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

    // <https://fetch.spec.whatwg.org/#concept-body-consume-body>
    // Otherwise, fully read object’s body given successSteps, errorSteps, and object’s relevant global object.
    //
    // <https://fetch.spec.whatwg.org/#body-fully-read>
    // Let reader be the result of getting a reader for body’s stream.
    // Read all bytes from reader, given successSteps and errorSteps.
    //
    // <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    // Set stream.[[disturbed]] to true.
    // Otherwise, if stream.[[state]] is "errored", perform readRequest’s error steps given stream.[[storedError]].
    //
    // If the body stream is already errored (for example, the fetch was aborted after the Response exists),
    // the normal fully read path would reject with [[storedError]] but would also mark the stream disturbed.
    // Once the stream is disturbed, later calls reject with TypeError ("disturbed or locked") instead of the
    // original AbortError. This early return rejects with the same [[storedError]] without disturbing the
    // stream, so repeated calls (for example, calling text() twice) keep rejecting with AbortError.
    if stream.is_errored() {
        rooted!(in(*cx) let mut stored_error = UndefinedValue());
        stream.get_stored_error(stored_error.handle_mut());
        promise.reject(cx, stored_error.handle(), can_gc);
        return promise;
    }

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

fn extract_name_from_content_disposition(headers: &HeaderMap) -> Option<String> {
    let cd = headers.get(CONTENT_DISPOSITION)?.to_str().ok()?;

    for part in cd.split(';').map(|s| s.trim()) {
        if let Some(rest) = part.strip_prefix("name=") {
            let v = rest.trim();
            let v = v.strip_prefix('"').unwrap_or(v);
            let v = v.strip_suffix('"').unwrap_or(v);
            return Some(v.to_string());
        }
    }
    None
}

fn extract_filename_from_content_disposition(headers: &HeaderMap) -> Option<String> {
    let cd = headers.get(CONTENT_DISPOSITION)?.to_str().ok()?;
    if let Some(index) = cd.find("filename=") {
        let start = index + "filename=".len();
        return Some(
            cd.get(start..)
                .unwrap_or_default()
                .trim_matches('"')
                .to_owned(),
        );
    }
    if let Some(index) = cd.find("filename*=UTF-8''") {
        let start = index + "filename*=UTF-8''".len();
        return Some(
            cd.get(start..)
                .unwrap_or_default()
                .trim_matches('"')
                .to_owned(),
        );
    }
    None
}

fn content_type_from_headers(headers: &HeaderMap) -> Result<String, Error> {
    match headers.get(CONTENT_TYPE) {
        Some(value) => Ok(value
            .to_str()
            .map_err(|_| Error::Type("Inappropriate MIME-type for Body".to_string()))?
            .to_string()),
        None => Ok("text/plain".to_string()),
    }
}

fn append_form_data_entry_from_part(
    root: &GlobalScope,
    formdata: &FormData,
    headers: &HeaderMap,
    body: Vec<u8>,
    can_gc: CanGc,
) -> Fallible<()> {
    let Some(name) = extract_name_from_content_disposition(headers) else {
        return Ok(());
    };
    // A part whose `Content-Disposition` header contains a `name` parameter whose value is `_charset_` is parsed like any other part. It does not change the encoding.
    let filename = extract_filename_from_content_disposition(headers);
    if let Some(filename) = filename {
        // Each part whose `Content-Disposition` header contains a `filename` parameter must be parsed into an entry whose value is a File object whose contents are the contents of the part.
        //
        // The name attribute of the File object must have the value of the `filename` parameter of the part.
        //
        // The type attribute of the File object must have the value of the `Content-Type` header of the part if the part has such header, and `text/plain` (the default defined by [RFC7578] section 4.4) otherwise.
        let content_type = content_type_from_headers(headers)?;
        let file = File::new(
            root,
            BlobImpl::new_from_bytes(body, normalize_type_string(&content_type)),
            DOMString::from(filename),
            None,
            can_gc,
        );
        let blob = file.upcast::<Blob>();
        formdata.Append_(USVString(name), blob, None);
    } else {
        // Each part whose `Content-Disposition` header does not contain a `filename` parameter must be parsed into an entry whose value is the UTF-8 decoded without BOM content of the part. This is done regardless of the presence or the value of a `Content-Type` header and regardless of the presence or the value of a `charset` parameter.

        let (value, _) = UTF_8.decode_without_bom_handling(&body);
        formdata.Append(USVString(name), USVString(value.to_string()));
    }
    Ok(())
}

fn append_multipart_nodes(
    root: &GlobalScope,
    formdata: &FormData,
    nodes: Vec<Node>,
    can_gc: CanGc,
) -> Fallible<()> {
    for node in nodes {
        match node {
            Node::Part(part) => {
                append_form_data_entry_from_part(root, formdata, &part.headers, part.body, can_gc)?;
            },
            Node::File(file_part) => {
                let body = fs::read(&file_part.path)
                    .map_err(|_| Error::Type("file part could not be read".to_string()))?;
                append_form_data_entry_from_part(root, formdata, &file_part.headers, body, can_gc)?;
            },
            Node::Multipart((_, inner)) => {
                append_multipart_nodes(root, formdata, inner, can_gc)?;
            },
        }
    }
    Ok(())
}

/// <https://fetch.spec.whatwg.org/#ref-for-concept-body-consume-body%E2%91%A2>
fn run_form_data_algorithm(
    root: &GlobalScope,
    bytes: Vec<u8>,
    mime: &[u8],
    can_gc: CanGc,
) -> Fallible<FetchedData> {
    // The formData() method steps are to return the result of running consume body
    // with this and the following steps given a byte sequence bytes:
    let mime_str = str::from_utf8(mime).unwrap_or_default();
    let mime: Mime = mime_str
        .parse()
        .map_err(|_| Error::Type("Inappropriate MIME-type for Body".to_string()))?;

    // Let mimeType be the result of get the MIME type with this.
    //
    // If mimeType is non-null, then switch on mimeType’s essence and run the corresponding steps:
    if mime.type_() == mime::MULTIPART && mime.subtype() == mime::FORM_DATA {
        // "multipart/form-data"
        // Parse bytes, using the value of the `boundary` parameter from mimeType,
        // per the rules set forth in Returning Values from Forms: multipart/form-data. [RFC7578]
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            mime_str
                .parse()
                .map_err(|_| Error::Type("Inappropriate MIME-type for Body".to_string()))?,
        );

        if let Some(boundary) = mime.get_param(mime::BOUNDARY) {
            let closing_boundary = format!("--{}--", boundary.as_str()).into_bytes();
            let trimmed_bytes = bytes.strip_suffix(b"\r\n").unwrap_or(&bytes);
            if trimmed_bytes == closing_boundary {
                let formdata = FormData::new(None, root, can_gc);
                return Ok(FetchedData::FormData(formdata));
            }
        }

        let mut cursor = Cursor::new(bytes);
        // If that fails for some reason, then throw a TypeError.
        let nodes = read_multipart_body(&mut cursor, &headers, false)
            .map_err(|_| Error::Type("Inappropriate MIME-type for Body".to_string()))?;
        // The above is a rough approximation of what is needed for `multipart/form-data`,
        // a more detailed parsing specification is to be written. Volunteers welcome.

        // Return a new FormData object, appending each entry, resulting from the parsing operation, to its entry list.
        let formdata = FormData::new(None, root, can_gc);

        append_multipart_nodes(root, &formdata, nodes, can_gc)?;

        return Ok(FetchedData::FormData(formdata));
    }

    if mime.type_() == mime::APPLICATION && mime.subtype() == mime::WWW_FORM_URLENCODED {
        // "application/x-www-form-urlencoded"
        // Let entries be the result of parsing bytes.
        //
        // Return a new FormData object whose entry list is entries.
        let entries = form_urlencoded::parse(&bytes);
        let formdata = FormData::new(None, root, can_gc);
        for (k, e) in entries {
            formdata.Append(USVString(k.into_owned()), USVString(e.into_owned()));
        }
        return Ok(FetchedData::FormData(formdata));
    }

    // Throw a TypeError.
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
