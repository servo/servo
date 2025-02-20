/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::Cell;
use std::cmp;
use std::default::Default;
use std::str::{self, FromStr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dom_struct::dom_struct;
use encoding_rs::{Encoding, UTF_8};
use headers::{ContentLength, ContentType, HeaderMapExt};
use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use http::header::{self, HeaderMap, HeaderName, HeaderValue};
use http::Method;
use hyper_serde::Serde;
use js::jsapi::{Heap, JS_ClearPendingException};
use js::jsval::{JSVal, NullValue};
use js::rust::wrappers::JS_ParseJSON;
use js::rust::{HandleObject, MutableHandleValue};
use js::typedarray::{ArrayBuffer, ArrayBufferU8};
use mime::{self, Mime, Name};
use net_traits::http_status::HttpStatus;
use net_traits::request::{CredentialsMode, Referrer, RequestBuilder, RequestId, RequestMode};
use net_traits::{
    trim_http_whitespace, FetchMetadata, FetchResponseListener, FilteredMetadata, NetworkError,
    ReferrerPolicy, ResourceFetchTiming, ResourceTimingType,
};
use script_traits::serializable::BlobImpl;
use script_traits::DocumentActivity;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use url::Position;

use crate::body::{decode_to_utf16_with_bom_removal, BodySource, Extractable, ExtractedBody};
use crate::document_loader::DocumentLoader;
use crate::dom::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XMLHttpRequestBinding::{
    XMLHttpRequestMethods, XMLHttpRequestResponseType,
};
use crate::dom::bindings::codegen::UnionTypes::DocumentOrBlobOrArrayBufferViewOrArrayBufferOrFormDataOrStringOrURLSearchParams as DocumentOrXMLHttpRequestBodyInit;
use crate::dom::bindings::conversions::ToJSValConvertible;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{is_token, ByteString, DOMString, USVString};
use crate::dom::blob::{normalize_type_string, Blob};
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::headers::{extract_mime_type, is_forbidden_request_header};
use crate::dom::node::Node;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::progressevent::ProgressEvent;
use crate::dom::readablestream::ReadableStream;
use crate::dom::servoparser::ServoParser;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use crate::dom::xmlhttprequestupload::XMLHttpRequestUpload;
use crate::fetch::FetchCanceller;
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};
use crate::script_runtime::{CanGc, JSContext};
use crate::task_source::{SendableTaskSource, TaskSourceName};
use crate::timers::{OneshotTimerCallback, OneshotTimerHandle};

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
enum XMLHttpRequestState {
    Unsent = 0,
    Opened = 1,
    HeadersReceived = 2,
    Loading = 3,
    Done = 4,
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct GenerationId(u32);

/// Closure of required data for each async network event that comprises the
/// XHR's response.
struct XHRContext {
    xhr: TrustedXHRAddress,
    gen_id: GenerationId,
    sync_status: DomRefCell<Option<ErrorResult>>,
    resource_timing: ResourceFetchTiming,
    url: ServoUrl,
}

impl FetchResponseListener for XHRContext {
    fn process_request_body(&mut self, _: RequestId) {
        // todo
    }

    fn process_request_eof(&mut self, _: RequestId) {
        // todo
    }

    fn process_response(&mut self, _: RequestId, metadata: Result<FetchMetadata, NetworkError>) {
        let xhr = self.xhr.root();
        let rv = xhr.process_headers_available(self.gen_id, metadata, CanGc::note());
        if rv.is_err() {
            *self.sync_status.borrow_mut() = Some(rv);
        }
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        self.xhr
            .root()
            .process_data_available(self.gen_id, chunk, CanGc::note());
    }

    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let rv = self.xhr.root().process_response_complete(
            self.gen_id,
            response.map(|_| ()),
            CanGc::note(),
        );
        *self.sync_status.borrow_mut() = Some(rv);
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self, CanGc::note())
    }
}

impl ResourceTimingListener for XHRContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::XMLHttpRequest, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.xhr.root().global()
    }
}

impl PreInvoke for XHRContext {
    fn should_invoke(&self) -> bool {
        self.xhr.root().generation_id.get() == self.gen_id
    }
}

#[derive(Clone)]
pub(crate) enum XHRProgress {
    /// Notify that headers have been received
    HeadersReceived(GenerationId, Option<HeaderMap>, HttpStatus),
    /// Partial progress (after receiving headers), containing portion of the response
    Loading(GenerationId, Vec<u8>),
    /// Loading is done
    Done(GenerationId),
    /// There was an error (only Error::Abort, Error::Timeout or Error::Network is used)
    Errored(GenerationId, Error),
}

impl XHRProgress {
    fn generation_id(&self) -> GenerationId {
        match *self {
            XHRProgress::HeadersReceived(id, _, _) |
            XHRProgress::Loading(id, _) |
            XHRProgress::Done(id) |
            XHRProgress::Errored(id, _) => id,
        }
    }
}

#[dom_struct]
pub(crate) struct XMLHttpRequest {
    eventtarget: XMLHttpRequestEventTarget,
    ready_state: Cell<XMLHttpRequestState>,
    timeout: Cell<Duration>,
    with_credentials: Cell<bool>,
    upload: Dom<XMLHttpRequestUpload>,
    response_url: DomRefCell<String>,
    #[no_trace]
    status: DomRefCell<HttpStatus>,
    response: DomRefCell<Vec<u8>>,
    response_type: Cell<XMLHttpRequestResponseType>,
    response_xml: MutNullableDom<Document>,
    response_blob: MutNullableDom<Blob>,
    #[ignore_malloc_size_of = "mozjs"]
    response_arraybuffer: HeapBufferSource<ArrayBufferU8>,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    response_json: Heap<JSVal>,
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[no_trace]
    response_headers: DomRefCell<HeaderMap>,
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[no_trace]
    override_mime_type: DomRefCell<Option<Mime>>,

    // Associated concepts
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[no_trace]
    request_method: DomRefCell<Method>,
    #[no_trace]
    request_url: DomRefCell<Option<ServoUrl>>,
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[no_trace]
    request_headers: DomRefCell<HeaderMap>,
    request_body_len: Cell<usize>,
    sync: Cell<bool>,
    upload_complete: Cell<bool>,
    upload_listener: Cell<bool>,
    send_flag: Cell<bool>,

    timeout_cancel: DomRefCell<Option<OneshotTimerHandle>>,
    fetch_time: Cell<Instant>,
    generation_id: Cell<GenerationId>,
    response_status: Cell<Result<(), ()>>,
    #[no_trace]
    referrer: Referrer,
    #[no_trace]
    referrer_policy: ReferrerPolicy,
    canceller: DomRefCell<FetchCanceller>,
}

impl XMLHttpRequest {
    fn new_inherited(global: &GlobalScope) -> XMLHttpRequest {
        XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(),
            ready_state: Cell::new(XMLHttpRequestState::Unsent),
            timeout: Cell::new(Duration::ZERO),
            with_credentials: Cell::new(false),
            upload: Dom::from_ref(&*XMLHttpRequestUpload::new(global, CanGc::note())),
            response_url: DomRefCell::new(String::new()),
            status: DomRefCell::new(HttpStatus::new_error()),
            response: DomRefCell::new(vec![]),
            response_type: Cell::new(XMLHttpRequestResponseType::_empty),
            response_xml: Default::default(),
            response_blob: Default::default(),
            response_arraybuffer: HeapBufferSource::default(),
            response_json: Heap::default(),
            response_headers: DomRefCell::new(HeaderMap::new()),
            override_mime_type: DomRefCell::new(None),

            request_method: DomRefCell::new(Method::GET),
            request_url: DomRefCell::new(None),
            request_headers: DomRefCell::new(HeaderMap::new()),
            request_body_len: Cell::new(0),
            sync: Cell::new(false),
            upload_complete: Cell::new(false),
            upload_listener: Cell::new(false),
            send_flag: Cell::new(false),

            timeout_cancel: DomRefCell::new(None),
            fetch_time: Cell::new(Instant::now()),
            generation_id: Cell::new(GenerationId(0)),
            response_status: Cell::new(Ok(())),
            referrer: global.get_referrer(),
            referrer_policy: global.get_referrer_policy(),
            canceller: DomRefCell::new(Default::default()),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<XMLHttpRequest> {
        reflect_dom_object_with_proto(
            Box::new(XMLHttpRequest::new_inherited(global)),
            global,
            proto,
            can_gc,
        )
    }

    fn sync_in_window(&self) -> bool {
        self.sync.get() && self.global().is::<Window>()
    }
}

impl XMLHttpRequestMethods<crate::DomTypeHolder> for XMLHttpRequest {
    /// <https://xhr.spec.whatwg.org/#constructors>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<XMLHttpRequest>> {
        Ok(XMLHttpRequest::new(global, proto, can_gc))
    }

    // https://xhr.spec.whatwg.org/#handler-xhr-onreadystatechange
    event_handler!(
        readystatechange,
        GetOnreadystatechange,
        SetOnreadystatechange
    );

    /// <https://xhr.spec.whatwg.org/#dom-xmlhttprequest-readystate>
    fn ReadyState(&self) -> u16 {
        self.ready_state.get() as u16
    }

    /// <https://xhr.spec.whatwg.org/#the-open()-method>
    fn Open(&self, method: ByteString, url: USVString) -> ErrorResult {
        // Step 8
        self.Open_(method, url, true, None, None)
    }

    /// <https://xhr.spec.whatwg.org/#the-open()-method>
    fn Open_(
        &self,
        method: ByteString,
        url: USVString,
        asynch: bool,
        username: Option<USVString>,
        password: Option<USVString>,
    ) -> ErrorResult {
        // Step 1
        if let Some(window) = DomRoot::downcast::<Window>(self.global()) {
            if !window.Document().is_fully_active() {
                return Err(Error::InvalidState);
            }
        }

        // Step 5
        //FIXME(seanmonstar): use a Trie instead?
        let maybe_method = method.as_str().and_then(|s| {
            // Note: hyper tests against the uppercase versions
            // Since we want to pass methods not belonging to the short list above
            // without changing capitalization, this will actually sidestep rust-http's type system
            // since methods like "patch" or "PaTcH" will be considered extension methods
            // despite the there being a rust-http method variant for them
            let upper = s.to_ascii_uppercase();
            match &*upper {
                "DELETE" | "GET" | "HEAD" | "OPTIONS" | "POST" | "PUT" | "CONNECT" | "TRACE" |
                "TRACK" => upper.parse().ok(),
                _ => s.parse().ok(),
            }
        });

        match maybe_method {
            // Step 4
            Some(Method::CONNECT) | Some(Method::TRACE) => Err(Error::Security),
            Some(ref t) if t.as_str() == "TRACK" => Err(Error::Security),
            Some(parsed_method) => {
                // Step 3
                if !is_token(&method) {
                    return Err(Error::Syntax);
                }

                // Step 2
                let base = self.global().api_base_url();
                // Step 6
                let mut parsed_url = match base.join(&url.0) {
                    Ok(parsed) => parsed,
                    // Step 7
                    Err(_) => return Err(Error::Syntax),
                };

                // Step 9
                if parsed_url.host().is_some() {
                    if let Some(user_str) = username {
                        parsed_url.set_username(&user_str.0).unwrap();
                    }
                    if let Some(pass_str) = password {
                        parsed_url.set_password(Some(&pass_str.0)).unwrap();
                    }
                }

                // Step 10
                if !asynch {
                    // FIXME: This should only happen if the global environment is a document environment
                    if !self.timeout.get().is_zero() ||
                        self.response_type.get() != XMLHttpRequestResponseType::_empty
                    {
                        return Err(Error::InvalidAccess);
                    }
                }
                // Step 11 - abort existing requests
                self.terminate_ongoing_fetch();

                // FIXME(#13767): In the WPT test: FileAPI/blob/Blob-XHR-revoke.html,
                // the xhr.open(url) is expected to hold a reference to the URL,
                // thus renders following revocations invalid. Though we won't
                // implement this for now, if ever needed, we should check blob
                // scheme and trigger corresponding actions here.

                // Step 12
                *self.request_method.borrow_mut() = parsed_method;
                *self.request_url.borrow_mut() = Some(parsed_url);
                self.sync.set(!asynch);
                *self.request_headers.borrow_mut() = HeaderMap::new();
                self.send_flag.set(false);
                self.upload_listener.set(false);
                *self.status.borrow_mut() = HttpStatus::new_error();

                // Step 13
                if self.ready_state.get() != XMLHttpRequestState::Opened {
                    self.change_ready_state(XMLHttpRequestState::Opened, CanGc::note());
                }
                Ok(())
            },
            // Step 3
            // This includes cases where as_str() returns None, and when is_token() returns false,
            // both of which indicate invalid extension method names
            _ => Err(Error::Syntax),
        }
    }

    /// <https://xhr.spec.whatwg.org/#the-setrequestheader()-method>
    fn SetRequestHeader(&self, name: ByteString, value: ByteString) -> ErrorResult {
        // Step 1: If this’s state is not opened, then throw an "InvalidStateError" DOMException.
        // Step 2: If this’s send() flag is set, then throw an "InvalidStateError" DOMException.
        if self.ready_state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(Error::InvalidState);
        }

        // Step 3: Normalize value.
        let value = trim_http_whitespace(&value);

        // Step 4: If name is not a header name or value is not a header value, then throw a
        // "SyntaxError" DOMException.
        if !is_token(&name) || !is_field_value(value) {
            return Err(Error::Syntax);
        }

        let name_str = name.as_str().ok_or(Error::Syntax)?;

        // Step 5: If (name, value) is a forbidden request-header, then return.
        if is_forbidden_request_header(name_str, value) {
            return Ok(());
        }

        debug!(
            "SetRequestHeader: name={:?}, value={:?}",
            name_str,
            str::from_utf8(value).ok()
        );
        let mut headers = self.request_headers.borrow_mut();

        // Step 6: Combine (name, value) in this’s author request headers.
        // https://fetch.spec.whatwg.org/#concept-header-list-combine
        let value = match headers.get(name_str).map(HeaderValue::as_bytes) {
            Some(raw) => {
                let mut buf = raw.to_vec();
                buf.extend_from_slice(b", ");
                buf.extend_from_slice(value);
                buf
            },
            None => value.into(),
        };

        headers.insert(
            HeaderName::from_str(name_str).unwrap(),
            HeaderValue::from_bytes(&value).unwrap(),
        );
        Ok(())
    }

    /// <https://xhr.spec.whatwg.org/#the-timeout-attribute>
    fn Timeout(&self) -> u32 {
        self.timeout.get().as_millis() as u32
    }

    /// <https://xhr.spec.whatwg.org/#the-timeout-attribute>
    fn SetTimeout(&self, timeout: u32) -> ErrorResult {
        // Step 1
        if self.sync_in_window() {
            return Err(Error::InvalidAccess);
        }

        // Step 2
        let timeout = Duration::from_millis(timeout as u64);
        self.timeout.set(timeout);

        if self.send_flag.get() {
            if timeout.is_zero() {
                self.cancel_timeout();
                return Ok(());
            }
            let progress = Instant::now() - self.fetch_time.get();
            if timeout > progress {
                self.set_timeout(timeout - progress);
            } else {
                // Immediately execute the timeout steps
                self.set_timeout(Duration::ZERO);
            }
        }
        Ok(())
    }

    /// <https://xhr.spec.whatwg.org/#the-withcredentials-attribute>
    fn WithCredentials(&self) -> bool {
        self.with_credentials.get()
    }

    /// <https://xhr.spec.whatwg.org/#dom-xmlhttprequest-withcredentials>
    fn SetWithCredentials(&self, with_credentials: bool) -> ErrorResult {
        match self.ready_state.get() {
            // Step 1
            XMLHttpRequestState::HeadersReceived |
            XMLHttpRequestState::Loading |
            XMLHttpRequestState::Done => Err(Error::InvalidState),
            // Step 2
            _ if self.send_flag.get() => Err(Error::InvalidState),
            // Step 3
            _ => {
                self.with_credentials.set(with_credentials);
                Ok(())
            },
        }
    }

    /// <https://xhr.spec.whatwg.org/#the-upload-attribute>
    fn Upload(&self) -> DomRoot<XMLHttpRequestUpload> {
        DomRoot::from_ref(&*self.upload)
    }

    /// <https://xhr.spec.whatwg.org/#the-send()-method>
    fn Send(&self, data: Option<DocumentOrXMLHttpRequestBodyInit>, can_gc: CanGc) -> ErrorResult {
        // Step 1, 2
        if self.ready_state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(Error::InvalidState);
        }

        // Step 3
        let data = match *self.request_method.borrow() {
            Method::GET | Method::HEAD => None,
            _ => data,
        };
        // Step 4 (first half)
        let mut extracted_or_serialized = match data {
            Some(DocumentOrXMLHttpRequestBodyInit::Document(ref doc)) => {
                let bytes = Vec::from(serialize_document(doc)?.as_ref());
                let content_type = if doc.is_html_document() {
                    "text/html;charset=UTF-8"
                } else {
                    "application/xml;charset=UTF-8"
                };
                let total_bytes = bytes.len();
                let global = self.global();
                let stream = ReadableStream::new_from_bytes(&global, bytes, can_gc)?;
                Some(ExtractedBody {
                    stream,
                    total_bytes: Some(total_bytes),
                    content_type: Some(DOMString::from(content_type)),
                    source: BodySource::Object,
                })
            },
            Some(DocumentOrXMLHttpRequestBodyInit::Blob(ref b)) => {
                let extracted_body = b
                    .extract(&self.global(), can_gc)
                    .expect("Couldn't extract body.");
                if !extracted_body.in_memory() && self.sync.get() {
                    warn!("Sync XHR with not in-memory Blob as body not supported");
                    None
                } else {
                    Some(extracted_body)
                }
            },
            Some(DocumentOrXMLHttpRequestBodyInit::FormData(ref formdata)) => Some(
                formdata
                    .extract(&self.global(), can_gc)
                    .expect("Couldn't extract body."),
            ),
            Some(DocumentOrXMLHttpRequestBodyInit::String(ref str)) => Some(
                str.extract(&self.global(), can_gc)
                    .expect("Couldn't extract body."),
            ),
            Some(DocumentOrXMLHttpRequestBodyInit::URLSearchParams(ref urlsp)) => Some(
                urlsp
                    .extract(&self.global(), can_gc)
                    .expect("Couldn't extract body."),
            ),
            Some(DocumentOrXMLHttpRequestBodyInit::ArrayBuffer(ref typedarray)) => {
                let bytes = typedarray.to_vec();
                let total_bytes = bytes.len();
                let global = self.global();
                let stream = ReadableStream::new_from_bytes(&global, bytes, can_gc)?;
                Some(ExtractedBody {
                    stream,
                    total_bytes: Some(total_bytes),
                    content_type: None,
                    source: BodySource::Object,
                })
            },
            Some(DocumentOrXMLHttpRequestBodyInit::ArrayBufferView(ref typedarray)) => {
                let bytes = typedarray.to_vec();
                let total_bytes = bytes.len();
                let global = self.global();
                let stream = ReadableStream::new_from_bytes(&global, bytes, can_gc)?;
                Some(ExtractedBody {
                    stream,
                    total_bytes: Some(total_bytes),
                    content_type: None,
                    source: BodySource::Object,
                })
            },
            None => None,
        };

        self.request_body_len.set(
            extracted_or_serialized
                .as_ref()
                .map_or(0, |e| e.total_bytes.unwrap_or(0)),
        );

        // Step 5
        // If we dont have data to upload, we dont want to emit events
        let has_handlers = self.upload.upcast::<EventTarget>().has_handlers();
        self.upload_listener.set(has_handlers && data.is_some());

        // todo preserved headers?

        // Step 7
        self.upload_complete.set(false);
        // Step 8
        // FIXME handle the 'timed out flag'
        // Step 9
        self.upload_complete.set(extracted_or_serialized.is_none());
        // Step 10
        self.send_flag.set(true);

        // Step 11
        if !self.sync.get() {
            // If one of the event handlers below aborts the fetch by calling
            // abort or open we will need the current generation id to detect it.
            // Substep 1
            let gen_id = self.generation_id.get();
            self.dispatch_response_progress_event(atom!("loadstart"), can_gc);
            if self.generation_id.get() != gen_id {
                return Ok(());
            }
            // Substep 2
            if !self.upload_complete.get() && self.upload_listener.get() {
                self.dispatch_upload_progress_event(atom!("loadstart"), Ok(Some(0)), can_gc);
                if self.generation_id.get() != gen_id {
                    return Ok(());
                }
            }
        }

        // Step 6
        //TODO - set referrer_policy/referrer_url in request
        let credentials_mode = if self.with_credentials.get() {
            CredentialsMode::Include
        } else {
            CredentialsMode::CredentialsSameOrigin
        };
        let use_url_credentials = if let Some(ref url) = *self.request_url.borrow() {
            !url.username().is_empty() || url.password().is_some()
        } else {
            unreachable!()
        };

        let content_type = match extracted_or_serialized.as_mut() {
            Some(body) => body.content_type.take(),
            None => None,
        };

        let mut request = RequestBuilder::new(
            self.global().webview_id(),
            self.request_url.borrow().clone().unwrap(),
            self.referrer.clone(),
        )
        .method(self.request_method.borrow().clone())
        .headers((*self.request_headers.borrow()).clone())
        .unsafe_request(true)
        // XXXManishearth figure out how to avoid this clone
        .body(extracted_or_serialized.map(|e| e.into_net_request_body().0))
        .synchronous(self.sync.get())
        .mode(RequestMode::CorsMode)
        .use_cors_preflight(self.upload_listener.get())
        .credentials_mode(credentials_mode)
        .use_url_credentials(use_url_credentials)
        .origin(self.global().origin().immutable().clone())
        .referrer_policy(self.referrer_policy)
        .insecure_requests_policy(self.global().insecure_requests_policy())
        .pipeline_id(Some(self.global().pipeline_id()));

        // step 4 (second half)
        if let Some(content_type) = content_type {
            let encoding = match data {
                Some(DocumentOrXMLHttpRequestBodyInit::String(_)) |
                Some(DocumentOrXMLHttpRequestBodyInit::Document(_)) =>
                // XHR spec differs from http, and says UTF-8 should be in capitals,
                // instead of "utf-8", which is what Hyper defaults to. So not
                // using content types provided by Hyper.
                {
                    Some("UTF-8")
                },
                _ => None,
            };

            let mut content_type_set = false;
            if !request.headers.contains_key(header::CONTENT_TYPE) {
                request.headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(&content_type).unwrap(),
                );
                content_type_set = true;
            }

            if !content_type_set {
                let ct = request.headers.typed_get::<ContentType>();
                if let Some(ct) = ct {
                    if let Some(encoding) = encoding {
                        let mime: Mime = ct.into();
                        for param in mime.params() {
                            if param.0 == mime::CHARSET &&
                                !param.1.as_ref().eq_ignore_ascii_case(encoding)
                            {
                                let new_params: Vec<(Name, Name)> = mime
                                    .params()
                                    .filter(|p| p.0 != mime::CHARSET)
                                    .map(|p| (p.0, p.1))
                                    .collect();

                                let new_mime = format!(
                                    "{}/{}; charset={}{}{}",
                                    mime.type_().as_ref(),
                                    mime.subtype().as_ref(),
                                    encoding,
                                    if new_params.is_empty() { "" } else { "; " },
                                    new_params
                                        .iter()
                                        .map(|p| format!("{}={}", p.0, p.1))
                                        .collect::<Vec<String>>()
                                        .join("; ")
                                );
                                let new_mime: Mime = new_mime.parse().unwrap();
                                request.headers.typed_insert(ContentType::from(new_mime))
                            }
                        }
                    }
                }
            }
        }

        self.fetch_time.set(Instant::now());

        let rv = self.fetch(request, &self.global());
        // Step 10
        if self.sync.get() {
            return rv;
        }

        let timeout = self.timeout.get();
        if timeout > Duration::ZERO {
            self.set_timeout(timeout);
        }
        Ok(())
    }

    /// <https://xhr.spec.whatwg.org/#the-abort()-method>
    fn Abort(&self, can_gc: CanGc) {
        // Step 1
        self.terminate_ongoing_fetch();
        // Step 2
        let state = self.ready_state.get();
        if (state == XMLHttpRequestState::Opened && self.send_flag.get()) ||
            state == XMLHttpRequestState::HeadersReceived ||
            state == XMLHttpRequestState::Loading
        {
            let gen_id = self.generation_id.get();
            self.process_partial_response(XHRProgress::Errored(gen_id, Error::Abort), can_gc);
            // If open was called in one of the handlers invoked by the
            // above call then we should terminate the abort sequence
            if self.generation_id.get() != gen_id {
                return;
            }
        }
        // Step 3
        if self.ready_state.get() == XMLHttpRequestState::Done {
            self.change_ready_state(XMLHttpRequestState::Unsent, can_gc);
            self.response_status.set(Err(()));
            self.response.borrow_mut().clear();
            self.response_headers.borrow_mut().clear();
        }
    }

    /// <https://xhr.spec.whatwg.org/#the-responseurl-attribute>
    fn ResponseURL(&self) -> USVString {
        USVString(self.response_url.borrow().clone())
    }

    /// <https://xhr.spec.whatwg.org/#the-status-attribute>
    fn Status(&self) -> u16 {
        self.status.borrow().raw_code()
    }

    /// <https://xhr.spec.whatwg.org/#the-statustext-attribute>
    fn StatusText(&self) -> ByteString {
        ByteString::new(self.status.borrow().message().to_vec())
    }

    /// <https://xhr.spec.whatwg.org/#the-getresponseheader()-method>
    fn GetResponseHeader(&self, name: ByteString) -> Option<ByteString> {
        let headers = self.filter_response_headers();
        let headers = headers.get_all(HeaderName::from_str(&name.as_str()?.to_lowercase()).ok()?);
        let mut first = true;
        let s = headers.iter().fold(Vec::new(), |mut vec, value| {
            if !first {
                vec.extend(", ".as_bytes());
            }
            if let Ok(v) = str::from_utf8(value.as_bytes()).map(|s| s.trim().as_bytes()) {
                vec.extend(v);
                first = false;
            }
            vec
        });

        // There was no header with that name so we never got to change that value
        if first {
            None
        } else {
            Some(ByteString::new(s))
        }
    }

    /// <https://xhr.spec.whatwg.org/#the-getallresponseheaders()-method>
    fn GetAllResponseHeaders(&self) -> ByteString {
        let headers = self.filter_response_headers();
        let keys = headers.keys();
        let v = keys.fold(Vec::new(), |mut vec, k| {
            let values = headers.get_all(k);
            vec.extend(k.as_str().as_bytes());
            vec.extend(": ".as_bytes());
            let mut first = true;
            for value in values {
                if !first {
                    vec.extend(", ".as_bytes());
                    first = false;
                }
                vec.extend(value.as_bytes());
            }
            vec.extend("\r\n".as_bytes());
            vec
        });

        ByteString::new(v)
    }

    /// <https://xhr.spec.whatwg.org/#the-overridemimetype()-method>
    fn OverrideMimeType(&self, mime: DOMString) -> ErrorResult {
        // 1. If this’s state is loading or done, then throw an "InvalidStateError"
        //   DOMException.
        match self.ready_state.get() {
            XMLHttpRequestState::Loading | XMLHttpRequestState::Done => {
                return Err(Error::InvalidState);
            },
            _ => {},
        }

        // 2. Set this’s override MIME type to the result of parsing mime.
        // 3. If this’s override MIME type is failure, then set this’s override MIME type
        //    to application/octet-stream.
        let override_mime = match mime.parse::<Mime>() {
            Ok(mime) => mime,
            Err(_) => "application/octet-stream"
                .parse::<Mime>()
                .map_err(|_| Error::Syntax)?,
        };

        *self.override_mime_type.borrow_mut() = Some(override_mime);
        Ok(())
    }

    /// <https://xhr.spec.whatwg.org/#the-responsetype-attribute>
    fn ResponseType(&self) -> XMLHttpRequestResponseType {
        self.response_type.get()
    }

    /// <https://xhr.spec.whatwg.org/#the-responsetype-attribute>
    fn SetResponseType(&self, response_type: XMLHttpRequestResponseType) -> ErrorResult {
        // Step 1
        if self.global().is::<WorkerGlobalScope>() &&
            response_type == XMLHttpRequestResponseType::Document
        {
            return Ok(());
        }
        match self.ready_state.get() {
            // Step 2
            XMLHttpRequestState::Loading | XMLHttpRequestState::Done => Err(Error::InvalidState),
            _ => {
                if self.sync_in_window() {
                    // Step 3
                    Err(Error::InvalidAccess)
                } else {
                    // Step 4
                    self.response_type.set(response_type);
                    Ok(())
                }
            },
        }
    }

    #[allow(unsafe_code)]
    /// <https://xhr.spec.whatwg.org/#the-response-attribute>
    fn Response(&self, cx: JSContext, can_gc: CanGc, mut rval: MutableHandleValue) {
        match self.response_type.get() {
            XMLHttpRequestResponseType::_empty | XMLHttpRequestResponseType::Text => unsafe {
                let ready_state = self.ready_state.get();
                // Step 2
                if ready_state == XMLHttpRequestState::Done ||
                    ready_state == XMLHttpRequestState::Loading
                {
                    self.text_response().to_jsval(*cx, rval);
                } else {
                    // Step 1
                    "".to_jsval(*cx, rval);
                }
            },
            // Step 1
            _ if self.ready_state.get() != XMLHttpRequestState::Done => {
                rval.set(NullValue());
            },
            // Step 2
            XMLHttpRequestResponseType::Document => unsafe {
                self.document_response(can_gc).to_jsval(*cx, rval);
            },
            XMLHttpRequestResponseType::Json => self.json_response(cx, rval),
            XMLHttpRequestResponseType::Blob => unsafe {
                self.blob_response(can_gc).to_jsval(*cx, rval);
            },
            XMLHttpRequestResponseType::Arraybuffer => match self.arraybuffer_response(cx) {
                Some(array_buffer) => unsafe { array_buffer.to_jsval(*cx, rval) },
                None => rval.set(NullValue()),
            },
        }
    }

    /// <https://xhr.spec.whatwg.org/#the-responsetext-attribute>
    fn GetResponseText(&self) -> Fallible<USVString> {
        match self.response_type.get() {
            XMLHttpRequestResponseType::_empty | XMLHttpRequestResponseType::Text => {
                Ok(USVString(match self.ready_state.get() {
                    // Step 3
                    XMLHttpRequestState::Loading | XMLHttpRequestState::Done => {
                        self.text_response()
                    },
                    // Step 2
                    _ => "".to_owned(),
                }))
            },
            // Step 1
            _ => Err(Error::InvalidState),
        }
    }

    /// <https://xhr.spec.whatwg.org/#the-responsexml-attribute>
    fn GetResponseXML(&self, can_gc: CanGc) -> Fallible<Option<DomRoot<Document>>> {
        match self.response_type.get() {
            XMLHttpRequestResponseType::_empty | XMLHttpRequestResponseType::Document => {
                // Step 3
                if let XMLHttpRequestState::Done = self.ready_state.get() {
                    Ok(self.document_response(can_gc))
                } else {
                    // Step 2
                    Ok(None)
                }
            },
            // Step 1
            _ => Err(Error::InvalidState),
        }
    }
}

pub(crate) type TrustedXHRAddress = Trusted<XMLHttpRequest>;

impl XMLHttpRequest {
    fn change_ready_state(&self, rs: XMLHttpRequestState, can_gc: CanGc) {
        assert_ne!(self.ready_state.get(), rs);
        self.ready_state.set(rs);
        if rs != XMLHttpRequestState::Unsent {
            let event = Event::new(
                &self.global(),
                atom!("readystatechange"),
                EventBubbles::DoesNotBubble,
                EventCancelable::Cancelable,
                can_gc,
            );
            event.fire(self.upcast(), can_gc);
        }
    }

    fn process_headers_available(
        &self,
        gen_id: GenerationId,
        metadata: Result<FetchMetadata, NetworkError>,
        can_gc: CanGc,
    ) -> Result<(), Error> {
        let metadata = match metadata {
            Ok(meta) => match meta {
                FetchMetadata::Unfiltered(m) => m,
                FetchMetadata::Filtered { filtered, .. } => match filtered {
                    FilteredMetadata::Basic(m) => m,
                    FilteredMetadata::Cors(m) => m,
                    FilteredMetadata::Opaque => return Err(Error::Network),
                    FilteredMetadata::OpaqueRedirect(_) => return Err(Error::Network),
                },
            },
            Err(_) => {
                self.process_partial_response(XHRProgress::Errored(gen_id, Error::Network), can_gc);
                return Err(Error::Network);
            },
        };

        metadata.final_url[..Position::AfterQuery].clone_into(&mut self.response_url.borrow_mut());

        // XXXManishearth Clear cache entries in case of a network error
        self.process_partial_response(
            XHRProgress::HeadersReceived(
                gen_id,
                metadata.headers.map(Serde::into_inner),
                metadata.status,
            ),
            can_gc,
        );
        Ok(())
    }

    fn process_data_available(&self, gen_id: GenerationId, payload: Vec<u8>, can_gc: CanGc) {
        self.process_partial_response(XHRProgress::Loading(gen_id, payload), can_gc);
    }

    fn process_response_complete(
        &self,
        gen_id: GenerationId,
        status: Result<(), NetworkError>,
        can_gc: CanGc,
    ) -> ErrorResult {
        match status {
            Ok(()) => {
                self.process_partial_response(XHRProgress::Done(gen_id), can_gc);
                Ok(())
            },
            Err(_) => {
                self.process_partial_response(XHRProgress::Errored(gen_id, Error::Network), can_gc);
                Err(Error::Network)
            },
        }
    }

    fn process_partial_response(&self, progress: XHRProgress, can_gc: CanGc) {
        let msg_id = progress.generation_id();

        // Aborts processing if abort() or open() was called
        // (including from one of the event handlers called below)
        macro_rules! return_if_fetch_was_terminated(
            () => (
                if msg_id != self.generation_id.get() {
                    return
                }
            );
        );

        // Ignore message if it belongs to a terminated fetch
        return_if_fetch_was_terminated!();

        // Ignore messages coming from previously-errored responses or requests that have timed out
        if self.response_status.get().is_err() {
            return;
        }

        match progress {
            XHRProgress::HeadersReceived(_, headers, status) => {
                assert!(self.ready_state.get() == XMLHttpRequestState::Opened);
                // For synchronous requests, this should not fire any events, and just store data
                // XXXManishearth Find a way to track partial progress of the send (onprogresss for XHRUpload)

                // Part of step 13, send() (processing request end of file)
                // Substep 1
                self.upload_complete.set(true);
                // Substeps 2-4
                if !self.sync.get() && self.upload_listener.get() {
                    self.dispatch_upload_progress_event(atom!("progress"), Ok(None), can_gc);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event(atom!("load"), Ok(None), can_gc);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event(atom!("loadend"), Ok(None), can_gc);
                    return_if_fetch_was_terminated!();
                }
                // Part of step 13, send() (processing response)
                // XXXManishearth handle errors, if any (substep 1)
                // Substep 2
                if !status.is_error() {
                    *self.status.borrow_mut() = status.clone();
                }
                if let Some(h) = headers.as_ref() {
                    *self.response_headers.borrow_mut() = h.clone();
                }
                {
                    let len = headers.and_then(|h| h.typed_get::<ContentLength>());
                    let mut response = self.response.borrow_mut();
                    response.clear();
                    if let Some(len) = len {
                        // don't attempt to prereserve more than 4 MB of memory,
                        // to avoid giving servers the ability to DOS the client by
                        // providing arbitrarily large content-lengths.
                        //
                        // this number is arbitrary, it's basically big enough that most
                        // XHR requests won't hit it, but not so big that it allows for DOS
                        let size = cmp::min(0b100_0000000000_0000000000, len.0 as usize);

                        // preallocate the buffer
                        response.reserve(size);
                    }
                }
                // Substep 3
                if !self.sync.get() {
                    self.change_ready_state(XMLHttpRequestState::HeadersReceived, can_gc);
                }
            },
            XHRProgress::Loading(_, mut partial_response) => {
                // For synchronous requests, this should not fire any events, and just store data
                // Part of step 11, send() (processing response body)
                // XXXManishearth handle errors, if any (substep 2)

                self.response.borrow_mut().append(&mut partial_response);
                if !self.sync.get() {
                    if self.ready_state.get() == XMLHttpRequestState::HeadersReceived {
                        self.ready_state.set(XMLHttpRequestState::Loading);
                    }
                    let event = Event::new(
                        &self.global(),
                        atom!("readystatechange"),
                        EventBubbles::DoesNotBubble,
                        EventCancelable::Cancelable,
                        can_gc,
                    );
                    event.fire(self.upcast(), can_gc);
                    return_if_fetch_was_terminated!();
                    self.dispatch_response_progress_event(atom!("progress"), can_gc);
                }
            },
            XHRProgress::Done(_) => {
                assert!(
                    self.ready_state.get() == XMLHttpRequestState::HeadersReceived ||
                        self.ready_state.get() == XMLHttpRequestState::Loading ||
                        self.sync.get()
                );

                self.cancel_timeout();
                self.canceller.borrow_mut().ignore();

                // Part of step 11, send() (processing response end of file)
                // XXXManishearth handle errors, if any (substep 2)

                // Subsubsteps 6-8
                self.send_flag.set(false);

                self.change_ready_state(XMLHttpRequestState::Done, can_gc);
                return_if_fetch_was_terminated!();
                // Subsubsteps 11-12
                self.dispatch_response_progress_event(atom!("load"), can_gc);
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event(atom!("loadend"), can_gc);
            },
            XHRProgress::Errored(_, e) => {
                self.cancel_timeout();
                self.canceller.borrow_mut().ignore();

                self.discard_subsequent_responses();
                self.send_flag.set(false);
                // XXXManishearth set response to NetworkError
                self.change_ready_state(XMLHttpRequestState::Done, can_gc);
                return_if_fetch_was_terminated!();

                let errormsg = match e {
                    Error::Abort => "abort",
                    Error::Timeout => "timeout",
                    _ => "error",
                };

                let upload_complete = &self.upload_complete;
                if !upload_complete.get() {
                    upload_complete.set(true);
                    if self.upload_listener.get() {
                        self.dispatch_upload_progress_event(Atom::from(errormsg), Err(()), can_gc);
                        return_if_fetch_was_terminated!();
                        self.dispatch_upload_progress_event(atom!("loadend"), Err(()), can_gc);
                        return_if_fetch_was_terminated!();
                    }
                }
                self.dispatch_response_progress_event(Atom::from(errormsg), can_gc);
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event(atom!("loadend"), can_gc);
            },
        }
    }

    fn terminate_ongoing_fetch(&self) {
        self.canceller.borrow_mut().cancel();
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
        self.response_status.set(Ok(()));
    }

    fn dispatch_progress_event(
        &self,
        upload: bool,
        type_: Atom,
        loaded: u64,
        total: Option<u64>,
        can_gc: CanGc,
    ) {
        let (total_length, length_computable) = if self
            .response_headers
            .borrow()
            .contains_key(header::CONTENT_ENCODING)
        {
            (0, false)
        } else {
            (total.unwrap_or(0), total.is_some())
        };
        let progressevent = ProgressEvent::new(
            &self.global(),
            type_,
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            length_computable,
            loaded,
            total_length,
            can_gc,
        );
        let target = if upload {
            self.upload.upcast()
        } else {
            self.upcast()
        };
        progressevent.upcast::<Event>().fire(target, can_gc);
    }

    fn dispatch_upload_progress_event(
        &self,
        type_: Atom,
        partial_load: Result<Option<u64>, ()>,
        can_gc: CanGc,
    ) {
        // If partial_load is Ok(None), loading has completed and we can just use the value from the request body
        // If an error occured, we pass 0 for both loaded and total

        let request_body_len = self.request_body_len.get() as u64;
        let (loaded, total) = match partial_load {
            Ok(l) => match l {
                Some(loaded) => (loaded, Some(request_body_len)),
                None => (request_body_len, Some(request_body_len)),
            },
            Err(()) => (0, None),
        };
        self.dispatch_progress_event(true, type_, loaded, total, can_gc);
    }

    fn dispatch_response_progress_event(&self, type_: Atom, can_gc: CanGc) {
        let len = self.response.borrow().len() as u64;
        let total = self
            .response_headers
            .borrow()
            .typed_get::<ContentLength>()
            .map(|v| v.0);
        self.dispatch_progress_event(false, type_, len, total, can_gc);
    }

    fn set_timeout(&self, duration: Duration) {
        // Sets up the object to timeout in a given number of milliseconds
        // This will cancel all previous timeouts
        let callback = OneshotTimerCallback::XhrTimeout(XHRTimeoutCallback {
            xhr: Trusted::new(self),
            generation_id: self.generation_id.get(),
        });
        *self.timeout_cancel.borrow_mut() =
            Some(self.global().schedule_callback(callback, duration));
    }

    fn cancel_timeout(&self) {
        if let Some(handle) = self.timeout_cancel.borrow_mut().take() {
            self.global().unschedule_callback(handle);
        }
    }

    /// <https://xhr.spec.whatwg.org/#text-response>
    fn text_response(&self) -> String {
        // Step 3, 5
        let charset = self.final_charset().unwrap_or(UTF_8);
        // TODO: Step 4 - add support for XML encoding guess stuff using XML spec

        // According to Simon, decode() should never return an error, so unwrap()ing
        // the result should be fine. XXXManishearth have a closer look at this later
        // Step 1, 2, 6
        let response = self.response.borrow();
        let (text, _, _) = charset.decode(&response);
        text.into_owned()
    }

    /// <https://xhr.spec.whatwg.org/#blob-response>
    fn blob_response(&self, can_gc: CanGc) -> DomRoot<Blob> {
        // Step 1
        if let Some(response) = self.response_blob.get() {
            return response;
        }
        // Step 2
        let mime = self
            .final_mime_type()
            .as_ref()
            .map(|m| normalize_type_string(m.as_ref()))
            .unwrap_or("".to_owned());

        // Step 3, 4
        let bytes = self.response.borrow().to_vec();
        let blob = Blob::new(
            &self.global(),
            BlobImpl::new_from_bytes(bytes, mime),
            can_gc,
        );
        self.response_blob.set(Some(&blob));
        blob
    }

    /// <https://xhr.spec.whatwg.org/#arraybuffer-response>
    fn arraybuffer_response(&self, cx: JSContext) -> Option<ArrayBuffer> {
        // Step 5: Set the response object to a new ArrayBuffer with the received bytes
        // For caching purposes, skip this step if the response is already created
        if !self.response_arraybuffer.is_initialized() {
            let bytes = self.response.borrow();

            // If this is not successful, the response won't be set and the function will return None
            self.response_arraybuffer.set_data(cx, &bytes).ok()?;
        }

        // Return the correct ArrayBuffer
        self.response_arraybuffer.get_buffer().ok()
    }

    /// <https://xhr.spec.whatwg.org/#document-response>
    fn document_response(&self, can_gc: CanGc) -> Option<DomRoot<Document>> {
        // Caching: if we have existing response xml, redirect it directly
        let response = self.response_xml.get();
        if response.is_some() {
            return response;
        }

        // Step 1
        if self.response_status.get().is_err() {
            return None;
        }

        // Step 2
        let mime_type = self.final_mime_type();
        // Step 5.3, 7
        let charset = self.final_charset().unwrap_or(UTF_8);
        let temp_doc: DomRoot<Document>;
        match mime_type {
            Some(ref mime) if mime.type_() == mime::TEXT && mime.subtype() == mime::HTML => {
                // Step 4
                if self.response_type.get() == XMLHttpRequestResponseType::_empty {
                    return None;
                } else {
                    // TODO Step 5.2 "If charset is null, prescan the first 1024 bytes of xhr’s received bytes"
                    // Step 5
                    temp_doc = self.document_text_html(can_gc);
                }
            },
            // Step 7
            None => {
                temp_doc = self.handle_xml(can_gc);
                // Not sure it the parser should throw an error for this case
                // The specification does not indicates this test,
                // but for now we check the document has no child nodes
                let has_no_child_nodes = temp_doc.upcast::<Node>().children().next().is_none();
                if has_no_child_nodes {
                    return None;
                }
            },
            Some(ref mime)
                if (mime.type_() == mime::TEXT && mime.subtype() == mime::XML) ||
                    (mime.type_() == mime::APPLICATION && mime.subtype() == mime::XML) ||
                    mime.suffix() == Some(mime::XML) =>
            {
                temp_doc = self.handle_xml(can_gc);
                // Not sure it the parser should throw an error for this case
                // The specification does not indicates this test,
                // but for now we check the document has no child nodes
                let has_no_child_nodes = temp_doc.upcast::<Node>().children().next().is_none();
                if has_no_child_nodes {
                    return None;
                }
            },
            // Step 3
            _ => {
                return None;
            },
        }
        // Step 8
        temp_doc.set_encoding(charset);

        // Step 9 to 11
        // Done by handle_text_html and handle_xml

        // Step 12
        self.response_xml.set(Some(&temp_doc));
        self.response_xml.get()
    }

    #[allow(unsafe_code)]
    /// <https://xhr.spec.whatwg.org/#json-response>
    fn json_response(&self, cx: JSContext, mut rval: MutableHandleValue) {
        // Step 1
        let response_json = self.response_json.get();
        if !response_json.is_null_or_undefined() {
            return rval.set(response_json);
        }
        // Step 2
        let bytes = self.response.borrow();
        // Step 3
        if bytes.len() == 0 {
            return rval.set(NullValue());
        }
        // Step 4
        // https://xhr.spec.whatwg.org/#json-response refers to
        // https://infra.spec.whatwg.org/#parse-json-from-bytes which refers to
        // https://encoding.spec.whatwg.org/#utf-8-decode which means
        // that the encoding is always UTF-8 and the UTF-8 BOM is removed,
        // if present, but UTF-16BE/LE BOM must not be honored.
        let json_text = decode_to_utf16_with_bom_removal(&bytes, UTF_8);
        // Step 5
        unsafe {
            if !JS_ParseJSON(*cx, json_text.as_ptr(), json_text.len() as u32, rval) {
                JS_ClearPendingException(*cx);
                return rval.set(NullValue());
            }
        }
        // Step 6
        self.response_json.set(rval.get());
    }

    fn document_text_html(&self, can_gc: CanGc) -> DomRoot<Document> {
        let charset = self.final_charset().unwrap_or(UTF_8);
        let wr = self.global();
        let response = self.response.borrow();
        let (decoded, _, _) = charset.decode(&response);
        let document = self.new_doc(IsHTMLDocument::HTMLDocument, can_gc);
        // TODO: Disable scripting while parsing
        ServoParser::parse_html_document(
            &document,
            Some(DOMString::from(decoded)),
            wr.get_url(),
            can_gc,
        );
        document
    }

    fn handle_xml(&self, can_gc: CanGc) -> DomRoot<Document> {
        let charset = self.final_charset().unwrap_or(UTF_8);
        let wr = self.global();
        let response = self.response.borrow();
        let (decoded, _, _) = charset.decode(&response);
        let document = self.new_doc(IsHTMLDocument::NonHTMLDocument, can_gc);
        // TODO: Disable scripting while parsing
        ServoParser::parse_xml_document(
            &document,
            Some(DOMString::from(decoded)),
            wr.get_url(),
            can_gc,
        );
        document
    }

    fn new_doc(&self, is_html_document: IsHTMLDocument, can_gc: CanGc) -> DomRoot<Document> {
        let wr = self.global();
        let win = wr.as_window();
        let doc = win.Document();
        let docloader = DocumentLoader::new(&doc.loader());
        let base = wr.get_url();
        let parsed_url = match base.join(&self.ResponseURL().0) {
            Ok(parsed) => Some(parsed),
            Err(_) => None, // Step 7
        };
        let content_type = self.final_mime_type();
        Document::new(
            win,
            HasBrowsingContext::No,
            parsed_url,
            doc.origin().clone(),
            is_html_document,
            content_type,
            None,
            DocumentActivity::Inactive,
            DocumentSource::FromParser,
            docloader,
            None,
            None,
            Default::default(),
            false,
            Some(doc.insecure_requests_policy()),
            can_gc,
        )
    }

    fn filter_response_headers(&self) -> HeaderMap {
        // https://fetch.spec.whatwg.org/#concept-response-header-list
        let mut headers = self.response_headers.borrow().clone();
        headers.remove(header::SET_COOKIE);
        headers.remove(HeaderName::from_static("set-cookie2"));
        // XXXManishearth additional CORS filtering goes here
        headers
    }

    fn discard_subsequent_responses(&self) {
        self.response_status.set(Err(()));
    }

    fn fetch(&self, request_builder: RequestBuilder, global: &GlobalScope) -> ErrorResult {
        let xhr = Trusted::new(self);

        let context = Arc::new(Mutex::new(XHRContext {
            xhr,
            gen_id: self.generation_id.get(),
            sync_status: DomRefCell::new(None),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url: request_builder.url.clone(),
        }));

        let (task_source, script_port) = if self.sync.get() {
            let (sender, receiver) = global.new_script_pair();
            (
                SendableTaskSource {
                    sender,
                    pipeline_id: global.pipeline_id(),
                    name: TaskSourceName::Networking,
                    canceller: Default::default(),
                },
                Some(receiver),
            )
        } else {
            (
                global.task_manager().networking_task_source().to_sendable(),
                None,
            )
        };

        *self.canceller.borrow_mut() = FetchCanceller::new(request_builder.id);
        global.fetch(request_builder, context.clone(), task_source);

        if let Some(script_port) = script_port {
            loop {
                if !global.process_event(script_port.recv().unwrap()) {
                    // We're exiting.
                    return Err(Error::Abort);
                }
                let context = context.lock().unwrap();
                let sync_status = context.sync_status.borrow();
                if let Some(ref status) = *sync_status {
                    return status.clone();
                }
            }
        }
        Ok(())
    }

    /// <https://xhr.spec.whatwg.org/#final-charset>
    fn final_charset(&self) -> Option<&'static Encoding> {
        // 1. Let label be null.
        // 2. Let responseMIME be the result of get a response MIME type for xhr.
        // 3. If responseMIME’s parameters["charset"] exists, then set label to it.
        let response_charset = self
            .response_mime_type()
            .and_then(|mime| mime.get_param(mime::CHARSET).map(|c| c.to_string()));

        // 4. If xhr’s override MIME type’s parameters["charset"] exists, then set label to it.
        let override_charset = self
            .override_mime_type
            .borrow()
            .as_ref()
            .and_then(|mime| mime.get_param(mime::CHARSET).map(|c| c.to_string()));

        // 5. If label is null, then return null.
        // 6. Let encoding be the result of getting an encoding from label.
        // 7. If encoding is failure, then return null.
        // 8. Return encoding.
        override_charset
            .or(response_charset)
            .and_then(|charset| Encoding::for_label(charset.as_bytes()))
    }

    /// <https://xhr.spec.whatwg.org/#response-mime-type>
    fn response_mime_type(&self) -> Option<Mime> {
        return extract_mime_type(&self.response_headers.borrow())
            .and_then(|mime_as_bytes| {
                String::from_utf8(mime_as_bytes)
                    .unwrap_or_default()
                    .parse()
                    .ok()
            })
            .or(Some(mime::TEXT_XML));
    }

    /// <https://xhr.spec.whatwg.org/#final-mime-type>
    fn final_mime_type(&self) -> Option<Mime> {
        if self.override_mime_type.borrow().is_some() {
            self.override_mime_type.borrow().clone()
        } else {
            self.response_mime_type()
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct XHRTimeoutCallback {
    #[ignore_malloc_size_of = "Because it is non-owning"]
    xhr: Trusted<XMLHttpRequest>,
    generation_id: GenerationId,
}

impl XHRTimeoutCallback {
    pub(crate) fn invoke(self, can_gc: CanGc) {
        let xhr = self.xhr.root();
        if xhr.ready_state.get() != XMLHttpRequestState::Done {
            xhr.process_partial_response(
                XHRProgress::Errored(self.generation_id, Error::Timeout),
                can_gc,
            );
        }
    }
}

fn serialize_document(doc: &Document) -> Fallible<DOMString> {
    let mut writer = vec![];
    match serialize(&mut writer, &doc.upcast::<Node>(), SerializeOpts::default()) {
        Ok(_) => Ok(DOMString::from(String::from_utf8(writer).unwrap())),
        Err(_) => Err(Error::InvalidState),
    }
}

/// Returns whether `bs` is a `field-value`, as defined by
/// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-32).
pub(crate) fn is_field_value(slice: &[u8]) -> bool {
    // Classifications of characters necessary for the [CRLF] (SP|HT) rule
    #[derive(PartialEq)]
    #[allow(clippy::upper_case_acronyms)]
    enum PreviousCharacter {
        Other,
        CR,
        LF,
        SPHT, // SP or HT
    }
    let mut prev = PreviousCharacter::Other; // The previous character
    slice.iter().all(|&x| {
        // http://tools.ietf.org/html/rfc2616#section-2.2
        match x {
            13 => {
                // CR
                if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::CR;
                    true
                } else {
                    false
                }
            },
            10 => {
                // LF
                if prev == PreviousCharacter::CR {
                    prev = PreviousCharacter::LF;
                    true
                } else {
                    false
                }
            },
            32 => {
                // SP
                if prev == PreviousCharacter::LF || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::SPHT;
                    true
                } else if prev == PreviousCharacter::Other {
                    // Counts as an Other here, since it's not preceded by a CRLF
                    // SP is not a CTL, so it can be used anywhere
                    // though if used immediately after a CR the CR is invalid
                    // We don't change prev since it's already Other
                    true
                } else {
                    false
                }
            },
            9 => {
                // HT
                if prev == PreviousCharacter::LF || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::SPHT;
                    true
                } else {
                    false
                }
            },
            0..=31 | 127 => false, // CTLs
            x if x > 127 => false, // non ASCII
            _ if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT => {
                prev = PreviousCharacter::Other;
                true
            },
            _ => false, // Previous character was a CR/LF but not part of the [CRLF] (SP|HT) rule
        }
    })
}
