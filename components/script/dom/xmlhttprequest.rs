/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cors::CORSResponse;
use cors::{AsyncCORSResponseListener, CORSRequest, RequestMode, allow_cross_origin_request};
use document_loader::DocumentLoader;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestMethods;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseType;
use dom::bindings::conversions::{ToJSValConvertible};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::{GlobalRef, GlobalRoot};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutHeapJSVal, MutNullableHeap};
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::{ByteString, USVString, is_token};
use dom::blob::Blob;
use dom::document::DocumentSource;
use dom::document::{Document, IsHTMLDocument};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::progressevent::ProgressEvent;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequestupload::XMLHttpRequestUpload;
use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecoderTrap, EncoderTrap, Encoding, EncodingRef};
use euclid::length::Length;
use hyper::header::Headers;
use hyper::header::{Accept, ContentLength, ContentType, qitem};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{self, Mime};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::JS_ClearPendingException;
use js::jsapi::{JSContext, JS_ParseJSON, RootedValue};
use js::jsval::{JSVal, NullValue, UndefinedValue};
use net_traits::ControlMsg::Load;
use net_traits::{AsyncResponseListener, AsyncResponseTarget, Metadata};
use net_traits::{LoadConsumer, LoadContext, LoadData, ResourceCORSData, ResourceThread};
use network_listener::{NetworkListener, PreInvoke};
use parse::html::{ParseContext, parse_html};
use parse::xml::{self, parse_xml};
use script_runtime::{ScriptChan, ScriptPort};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::default::Default;
use std::str;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use time;
use timers::{OneshotTimerCallback, OneshotTimerHandle};
use url::Url;
use url::percent_encoding::{utf8_percent_encode, USERNAME_ENCODE_SET, PASSWORD_ENCODE_SET};
use util::prefs;
use util::str::DOMString;

#[derive(JSTraceable, PartialEq, Copy, Clone, HeapSizeOf)]
enum XMLHttpRequestState {
    Unsent = 0,
    Opened = 1,
    HeadersReceived = 2,
    Loading = 3,
    Done = 4,
}

#[derive(JSTraceable, PartialEq, Clone, Copy, HeapSizeOf)]
pub struct GenerationId(u32);

/// Closure of required data for each async network event that comprises the
/// XHR's response.
struct XHRContext {
    xhr: TrustedXHRAddress,
    gen_id: GenerationId,
    cors_request: Option<CORSRequest>,
    buf: DOMRefCell<Vec<u8>>,
    sync_status: DOMRefCell<Option<ErrorResult>>,
}

#[derive(Clone)]
pub enum XHRProgress {
    /// Notify that headers have been received
    HeadersReceived(GenerationId, Option<Headers>, Option<RawStatus>),
    /// Partial progress (after receiving headers), containing portion of the response
    Loading(GenerationId, ByteString),
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
            XHRProgress::Errored(id, _) => id
        }
    }
}

#[dom_struct]
pub struct XMLHttpRequest {
    eventtarget: XMLHttpRequestEventTarget,
    ready_state: Cell<XMLHttpRequestState>,
    timeout: Cell<u32>,
    with_credentials: Cell<bool>,
    upload: JS<XMLHttpRequestUpload>,
    response_url: DOMRefCell<String>,
    status: Cell<u16>,
    status_text: DOMRefCell<ByteString>,
    response: DOMRefCell<ByteString>,
    response_type: Cell<XMLHttpRequestResponseType>,
    response_xml: MutNullableHeap<JS<Document>>,
    response_blob: MutNullableHeap<JS<Blob>>,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    response_json: MutHeapJSVal,
    #[ignore_heap_size_of = "Defined in hyper"]
    response_headers: DOMRefCell<Headers>,
    #[ignore_heap_size_of = "Defined in hyper"]
    override_mime_type: DOMRefCell<Option<Mime>>,
    #[ignore_heap_size_of = "Defined in rust-encoding"]
    override_charset: DOMRefCell<Option<EncodingRef>>,

    // Associated concepts
    #[ignore_heap_size_of = "Defined in hyper"]
    request_method: DOMRefCell<Method>,
    request_url: DOMRefCell<Option<Url>>,
    #[ignore_heap_size_of = "Defined in hyper"]
    request_headers: DOMRefCell<Headers>,
    request_body_len: Cell<usize>,
    sync: Cell<bool>,
    upload_complete: Cell<bool>,
    upload_events: Cell<bool>,
    send_flag: Cell<bool>,

    timeout_cancel: DOMRefCell<Option<OneshotTimerHandle>>,
    fetch_time: Cell<i64>,
    generation_id: Cell<GenerationId>,
    response_status: Cell<Result<(), ()>>,
}

impl XMLHttpRequest {
    fn new_inherited(global: GlobalRef) -> XMLHttpRequest {
        XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(),
            ready_state: Cell::new(XMLHttpRequestState::Unsent),
            timeout: Cell::new(0u32),
            with_credentials: Cell::new(false),
            upload: JS::from_rooted(&XMLHttpRequestUpload::new(global)),
            response_url: DOMRefCell::new(String::from("")),
            status: Cell::new(0),
            status_text: DOMRefCell::new(ByteString::new(vec!())),
            response: DOMRefCell::new(ByteString::new(vec!())),
            response_type: Cell::new(XMLHttpRequestResponseType::_empty),
            response_xml: Default::default(),
            response_blob: Default::default(),
            response_json: MutHeapJSVal::new(),
            response_headers: DOMRefCell::new(Headers::new()),
            override_mime_type: DOMRefCell::new(None),
            override_charset: DOMRefCell::new(None),

            request_method: DOMRefCell::new(Method::Get),
            request_url: DOMRefCell::new(None),
            request_headers: DOMRefCell::new(Headers::new()),
            request_body_len: Cell::new(0),
            sync: Cell::new(false),
            upload_complete: Cell::new(false),
            upload_events: Cell::new(false),
            send_flag: Cell::new(false),

            timeout_cancel: DOMRefCell::new(None),
            fetch_time: Cell::new(0),
            generation_id: Cell::new(GenerationId(0)),
            response_status: Cell::new(Ok(())),
        }
    }
    pub fn new(global: GlobalRef) -> Root<XMLHttpRequest> {
        reflect_dom_object(box XMLHttpRequest::new_inherited(global),
                           global,
                           XMLHttpRequestBinding::Wrap)
    }

    // https://xhr.spec.whatwg.org/#constructors
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<XMLHttpRequest>> {
        Ok(XMLHttpRequest::new(global))
    }

    fn sync_in_window(&self) -> bool {
        match self.global() {
            GlobalRoot::Window(_) if self.sync.get() => true,
            _ => false
        }
    }

    fn check_cors(context: Arc<Mutex<XHRContext>>,
                  load_data: LoadData,
                  req: CORSRequest,
                  script_chan: Box<ScriptChan + Send>,
                  resource_thread: ResourceThread) {
        struct CORSContext {
            xhr: Arc<Mutex<XHRContext>>,
            load_data: RefCell<Option<LoadData>>,
            req: CORSRequest,
            script_chan: Box<ScriptChan + Send>,
            resource_thread: ResourceThread,
        }

        impl AsyncCORSResponseListener for CORSContext {
            fn response_available(&self, response: CORSResponse) {
                if response.network_error {
                    let mut context = self.xhr.lock().unwrap();
                    let xhr = context.xhr.root();
                    xhr.process_partial_response(XHRProgress::Errored(context.gen_id, Error::Network));
                    *context.sync_status.borrow_mut() = Some(Err(Error::Network));
                    return;
                }

                let mut load_data = self.load_data.borrow_mut().take().unwrap();
                load_data.cors = Some(ResourceCORSData {
                    preflight: self.req.preflight_flag,
                    origin: self.req.origin.clone()
                });

                XMLHttpRequest::initiate_async_xhr(self.xhr.clone(), self.script_chan.clone(),
                                                   self.resource_thread.clone(), load_data);
            }
        }

        let cors_context = CORSContext {
            xhr: context,
            load_data: RefCell::new(Some(load_data)),
            req: req.clone(),
            script_chan: script_chan.clone(),
            resource_thread: resource_thread,
        };

        req.http_fetch_async(box cors_context, script_chan);
    }

    fn initiate_async_xhr(context: Arc<Mutex<XHRContext>>,
                          script_chan: Box<ScriptChan + Send>,
                          resource_thread: ResourceThread,
                          load_data: LoadData) {
        impl AsyncResponseListener for XHRContext {
            fn headers_available(&mut self, metadata: Metadata) {
                let xhr = self.xhr.root();
                let rv = xhr.process_headers_available(self.cors_request.clone(),
                                                       self.gen_id,
                                                       metadata);
                if rv.is_err() {
                    *self.sync_status.borrow_mut() = Some(rv);
                }
            }

            fn data_available(&mut self, payload: Vec<u8>) {
                self.buf.borrow_mut().extend_from_slice(&payload);
                self.xhr.root().process_data_available(self.gen_id, self.buf.borrow().clone());
            }

            fn response_complete(&mut self, status: Result<(), String>) {
                let rv = self.xhr.root().process_response_complete(self.gen_id, status);
                *self.sync_status.borrow_mut() = Some(rv);
            }
        }

        impl PreInvoke for XHRContext {
            fn should_invoke(&self) -> bool {
                self.xhr.root().generation_id.get() == self.gen_id
            }
        }

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = NetworkListener {
            context: context,
            script_chan: script_chan,
        };
        let response_target = AsyncResponseTarget {
            sender: action_sender,
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify(message.to().unwrap());
        });
        resource_thread.send(Load(load_data, LoadConsumer::Listener(response_target), None)).unwrap();
    }
}

impl XMLHttpRequestMethods for XMLHttpRequest {
    // https://xhr.spec.whatwg.org/#handler-xhr-onreadystatechange
    event_handler!(readystatechange, GetOnreadystatechange, SetOnreadystatechange);

    // https://xhr.spec.whatwg.org/#dom-xmlhttprequest-readystate
    fn ReadyState(&self) -> u16 {
        self.ready_state.get() as u16
    }

    // https://xhr.spec.whatwg.org/#the-open()-method
    fn Open(&self, method: ByteString, url: USVString) -> ErrorResult {
        // Step 8
        self.Open_(method, url, true, None, None)
    }

    // https://xhr.spec.whatwg.org/#the-open()-method
    fn Open_(&self, method: ByteString, url: USVString, async: bool,
             username: Option<USVString>, password: Option<USVString>) -> ErrorResult {
        // Step 1
        match self.global() {
            GlobalRoot::Window(ref window) => {
                if !window.Document().r().is_fully_active() { return Err(Error::InvalidState); }
            }
            _ => {}
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
                "DELETE" | "GET" | "HEAD" | "OPTIONS" |
                "POST" | "PUT" | "CONNECT" | "TRACE" |
                "TRACK" => upper.parse().ok(),
                _ => s.parse().ok()
            }
        });

        match maybe_method {
            // Step 4
            Some(Method::Connect) | Some(Method::Trace) => Err(Error::Security),
            Some(Method::Extension(ref t)) if &**t == "TRACK" => Err(Error::Security),
            Some(parsed_method) => {
                // Step 3
                if !is_token(&method) {
                    return Err(Error::Syntax)
                }

                // Step 2
                let base = self.global().r().api_base_url();
                // Step 6
                let mut parsed_url = match base.join(&url.0) {
                    Ok(parsed) => parsed,
                    // Step 7
                    Err(_) => return Err(Error::Syntax)
                };

                // Step 9
                if parsed_url.host().is_some() {
                    if let Some(scheme_data) = parsed_url.relative_scheme_data_mut() {
                        if let Some(user_str) = username {
                            scheme_data.username = utf8_percent_encode(&user_str.0, USERNAME_ENCODE_SET);

                            // ensure that the password is mutated when a username is provided
                            scheme_data.password = password.map(|pass_str| {
                                utf8_percent_encode(&pass_str.0, PASSWORD_ENCODE_SET)
                            });
                        }
                    }
                }

                // Step 10
                if !async {
                    // FIXME: This should only happen if the global environment is a document environment
                    if self.timeout.get() != 0 || self.with_credentials.get() ||
                       self.response_type.get() != XMLHttpRequestResponseType::_empty {
                        return Err(Error::InvalidAccess)
                    }
                }
                // Step 11 - abort existing requests
                self.terminate_ongoing_fetch();

                // Step 12
                *self.request_method.borrow_mut() = parsed_method;
                *self.request_url.borrow_mut() = Some(parsed_url);
                self.sync.set(!async);
                *self.request_headers.borrow_mut() = Headers::new();
                self.send_flag.set(false);
                *self.status_text.borrow_mut() = ByteString::new(vec!());
                self.status.set(0);

                // Step 13
                if self.ready_state.get() != XMLHttpRequestState::Opened {
                    self.change_ready_state(XMLHttpRequestState::Opened);
                }
                Ok(())
            },
            // Step 3
            // This includes cases where as_str() returns None, and when is_token() returns false,
            // both of which indicate invalid extension method names
            _ => Err(Error::Syntax)
        }
    }

    // https://xhr.spec.whatwg.org/#the-setrequestheader()-method
    fn SetRequestHeader(&self, name: ByteString, value: ByteString) -> ErrorResult {
        // Step 1, 2
        if self.ready_state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(Error::InvalidState);
        }

        // Step 3
        let value = trim_http_whitespace(&value);

        // Step 4
        if !is_token(&name) || !is_field_value(&value) {
            return Err(Error::Syntax);
        }
        let name_lower = name.to_lower();
        let name_str = match name_lower.as_str() {
            Some(s) => {
                // Step 5
                // Disallowed headers and header prefixes:
                // https://fetch.spec.whatwg.org/#forbidden-header-name
                let disallowedHeaders =
                    ["accept-charset", "accept-encoding",
                    "access-control-request-headers",
                    "access-control-request-method",
                    "connection", "content-length",
                    "cookie", "cookie2", "date", "dnt",
                    "expect", "host", "keep-alive", "origin",
                    "referer", "te", "trailer", "transfer-encoding",
                    "upgrade", "via"];

                let disallowedHeaderPrefixes = ["sec-", "proxy-"];

                if disallowedHeaders.iter().any(|header| *header == s) ||
                   disallowedHeaderPrefixes.iter().any(|prefix| s.starts_with(prefix)) {
                    return Ok(())
                } else {
                    s
                }
            },
            None => unreachable!()
        };

        debug!("SetRequestHeader: name={:?}, value={:?}", name.as_str(), str::from_utf8(value).ok());
        let mut headers = self.request_headers.borrow_mut();


        // Step 6
        let value = match headers.get_raw(name_str) {
            Some(raw) => {
                debug!("SetRequestHeader: old value = {:?}", raw[0]);
                let mut buf = raw[0].clone();
                buf.extend_from_slice(b", ");
                buf.extend_from_slice(value);
                debug!("SetRequestHeader: new value = {:?}", buf);
                buf
            },
            None => value.into(),
        };

        headers.set_raw(name_str.to_owned(), vec![value]);
        Ok(())
    }

    // https://xhr.spec.whatwg.org/#the-timeout-attribute
    fn Timeout(&self) -> u32 {
        self.timeout.get()
    }

    // https://xhr.spec.whatwg.org/#the-timeout-attribute
    fn SetTimeout(&self, timeout: u32) -> ErrorResult {
        // Step 1
        if self.sync_in_window() {
            return Err(Error::InvalidAccess);
        }
        // Step 2
        self.timeout.set(timeout);

        if self.send_flag.get() {
            if timeout == 0 {
                self.cancel_timeout();
                return Ok(());
            }
            let progress = time::now().to_timespec().sec - self.fetch_time.get();
            if timeout > (progress * 1000) as u32 {
                self.set_timeout(timeout - (progress * 1000) as u32);
            } else {
                // Immediately execute the timeout steps
                self.set_timeout(0);
            }
        }
        Ok(())
    }

    // https://xhr.spec.whatwg.org/#the-withcredentials-attribute
    fn WithCredentials(&self) -> bool {
        self.with_credentials.get()
    }

    // https://xhr.spec.whatwg.org/#dom-xmlhttprequest-withcredentials
    fn SetWithCredentials(&self, with_credentials: bool) -> ErrorResult {
        match self.ready_state.get() {
            // Step 1
            XMLHttpRequestState::HeadersReceived |
            XMLHttpRequestState::Loading |
            XMLHttpRequestState::Done => Err(Error::InvalidState),
            // Step 2
            _ if self.send_flag.get() => Err(Error::InvalidState),
            // Step 3
            _ if self.sync_in_window() => Err(Error::InvalidAccess),
            // Step 4
            _ => {
                self.with_credentials.set(with_credentials);
                Ok(())
            }
        }
    }

    // https://xhr.spec.whatwg.org/#the-upload-attribute
    fn Upload(&self) -> Root<XMLHttpRequestUpload> {
        Root::from_ref(&*self.upload)
    }

    // https://xhr.spec.whatwg.org/#the-send()-method
    fn Send(&self, data: Option<BodyInit>) -> ErrorResult {
        // Step 1, 2
        if self.ready_state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(Error::InvalidState);
        }

        // Step 3
        let data = match *self.request_method.borrow() {
            Method::Get | Method::Head => None,
            _ => data
        };
        // Step 4
        let extracted = data.as_ref().map(|d| d.extract());
        self.request_body_len.set(extracted.as_ref().map_or(0, |e| e.0.len()));

        // Step 6
        self.upload_events.set(false);
        // Step 7
        self.upload_complete.set(match extracted {
            None => true,
            Some (ref e) if e.0.is_empty() => true,
            _ => false
        });
        // Step 8
        self.send_flag.set(true);

        // Step 9
        if !self.sync.get() {
            let event_target = self.upload.upcast::<EventTarget>();
            if event_target.has_handlers() {
                self.upload_events.set(true);
            }

            // If one of the event handlers below aborts the fetch by calling
            // abort or open we will need the current generation id to detect it.
            // Substep 1
            let gen_id = self.generation_id.get();
            self.dispatch_response_progress_event(atom!("loadstart"));
            if self.generation_id.get() != gen_id {
                return Ok(());
            }
            // Substep 2
            if !self.upload_complete.get() {
                self.dispatch_upload_progress_event(atom!("loadstart"), Some(0));
                if self.generation_id.get() != gen_id {
                    return Ok(());
                }
            }

        }

        // Step 5
        let global = self.global();
        let pipeline_id = global.r().pipeline();
        let mut load_data =
            LoadData::new(LoadContext::Browsing,
                          self.request_url.borrow().clone().unwrap(),
                          Some(pipeline_id));
        if load_data.url.origin().ne(&global.r().get_url().origin()) {
            load_data.credentials_flag = self.WithCredentials();
        }
        load_data.data = extracted.as_ref().map(|e| e.0.clone());

        // XHR spec differs from http, and says UTF-8 should be in capitals,
        // instead of "utf-8", which is what Hyper defaults to. So not
        // using content types provided by Hyper.
        let n = "content-type";
        match extracted {
            Some((_, Some(ref content_type))) =>
                load_data.headers.set_raw(n.to_owned(), vec![content_type.bytes().collect()]),
            _ => (),
        }

        load_data.preserved_headers = (*self.request_headers.borrow()).clone();

        if !load_data.preserved_headers.has::<Accept>() {
            let mime = Mime(mime::TopLevel::Star, mime::SubLevel::Star, vec![]);
            load_data.preserved_headers.set(Accept(vec![qitem(mime)]));
        }

        load_data.method = (*self.request_method.borrow()).clone();

        // CORS stuff
        let global = self.global();
        let referer_url = self.global().r().get_url();
        let mode = if self.upload_events.get() {
            RequestMode::ForcedPreflight
        } else {
            RequestMode::CORS
        };
        let mut combined_headers = load_data.headers.clone();
        combined_headers.extend(load_data.preserved_headers.iter());
        let cors_request = CORSRequest::maybe_new(referer_url.clone(),
                                                  load_data.url.clone(),
                                                  mode,
                                                  load_data.method.clone(),
                                                  combined_headers,
                                                  true);
        match cors_request {
            Ok(None) => {
                let mut buf = String::new();
                buf.push_str(&referer_url.scheme);
                buf.push_str("://");

                if let Some(ref h) = referer_url.serialize_host() {
                    buf.push_str(h);
                }

                if let Some(ref p) = referer_url.port().as_ref() {
                    buf.push_str(":");
                    buf.push_str(&p.to_string());
                }

                if let Some(ref h) = referer_url.serialize_path() {
                    buf.push_str(h);
                }

                self.request_headers.borrow_mut().set_raw("Referer".to_owned(), vec![buf.into_bytes()]);
            },
            Ok(Some(ref req)) => self.insert_trusted_header("origin".to_owned(),
                                                            req.origin.to_string()),
            _ => {}
        }

        debug!("request_headers = {:?}", *self.request_headers.borrow());

        self.fetch_time.set(time::now().to_timespec().sec);
        let rv = self.fetch(load_data, cors_request, global.r());
        // Step 10
        if self.sync.get() {
            return rv;
        }

        let timeout = self.timeout.get();
        if timeout > 0 {
            self.set_timeout(timeout);
        }
        Ok(())
    }

    // https://xhr.spec.whatwg.org/#the-abort()-method
    fn Abort(&self) {
        // Step 1
        self.terminate_ongoing_fetch();
        // Step 2
        let state = self.ready_state.get();
        if (state == XMLHttpRequestState::Opened && self.send_flag.get()) ||
           state == XMLHttpRequestState::HeadersReceived ||
           state == XMLHttpRequestState::Loading {
            let gen_id = self.generation_id.get();
            self.process_partial_response(XHRProgress::Errored(gen_id, Error::Abort));
            // If open was called in one of the handlers invoked by the
            // above call then we should terminate the abort sequence
            if self.generation_id.get() != gen_id {
                return
            }
        }
        // Step 3
        self.ready_state.set(XMLHttpRequestState::Unsent);
    }

    // https://xhr.spec.whatwg.org/#the-responseurl-attribute
    fn ResponseURL(&self) -> USVString {
        USVString(self.response_url.borrow().clone())
    }

    // https://xhr.spec.whatwg.org/#the-status-attribute
    fn Status(&self) -> u16 {
        self.status.get()
    }

    // https://xhr.spec.whatwg.org/#the-statustext-attribute
    fn StatusText(&self) -> ByteString {
        self.status_text.borrow().clone()
    }

    // https://xhr.spec.whatwg.org/#the-getresponseheader()-method
    fn GetResponseHeader(&self, name: ByteString) -> Option<ByteString> {
        self.filter_response_headers().iter().find(|h| {
            name.eq_ignore_case(&h.name().parse().unwrap())
        }).map(|h| {
            ByteString::new(h.value_string().into_bytes())
        })
    }

    // https://xhr.spec.whatwg.org/#the-getallresponseheaders()-method
    fn GetAllResponseHeaders(&self) -> ByteString {
        ByteString::new(self.filter_response_headers().to_string().into_bytes())
    }

    // https://xhr.spec.whatwg.org/#the-overridemimetype()-method
    fn OverrideMimeType(&self, mime: DOMString) -> ErrorResult {
        // Step 1
        match self.ready_state.get() {
            XMLHttpRequestState::Loading | XMLHttpRequestState::Done => return Err(Error::InvalidState),
            _ => {},
        }
        // Step 2
        let override_mime = try!(mime.parse::<Mime>().map_err(|_| Error::Syntax));
        // Step 3
        *self.override_mime_type.borrow_mut() = Some(override_mime.clone());
        // Step 4
        let value = override_mime.get_param(mime::Attr::Charset);
        *self.override_charset.borrow_mut() = value.and_then(|value| {
            encoding_from_whatwg_label(value)
        });
        Ok(())
    }

    // https://xhr.spec.whatwg.org/#the-responsetype-attribute
    fn ResponseType(&self) -> XMLHttpRequestResponseType {
        self.response_type.get()
    }

    // https://xhr.spec.whatwg.org/#the-responsetype-attribute
    fn SetResponseType(&self, response_type: XMLHttpRequestResponseType) -> ErrorResult {
        // Step 1
        match self.global() {
            GlobalRoot::Worker(_) if response_type == XMLHttpRequestResponseType::Document => return Ok(()),
            _ => {}
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
            }
        }
    }

    #[allow(unsafe_code)]
    // https://xhr.spec.whatwg.org/#the-response-attribute
    fn Response(&self, cx: *mut JSContext) -> JSVal {
        unsafe {
            let mut rval = RootedValue::new(cx, UndefinedValue());
            match self.response_type.get() {
                XMLHttpRequestResponseType::_empty | XMLHttpRequestResponseType::Text => {
                    let ready_state = self.ready_state.get();
                    // Step 2
                    if ready_state == XMLHttpRequestState::Done || ready_state == XMLHttpRequestState::Loading {
                        self.text_response().to_jsval(cx, rval.handle_mut());
                    } else {
                    // Step 1
                        "".to_jsval(cx, rval.handle_mut());
                    }
                },
                // Step 1
                _ if self.ready_state.get() != XMLHttpRequestState::Done => {
                    return NullValue();
                },
                // Step 2
                XMLHttpRequestResponseType::Document => {
                    let op_doc = self.document_response();
                    if let Some(doc) = op_doc {
                        doc.to_jsval(cx, rval.handle_mut());
                    } else {
                    // Substep 1
                        return NullValue();
                    }
                },
                XMLHttpRequestResponseType::Json => {
                    self.json_response(cx).to_jsval(cx, rval.handle_mut());
                },
                XMLHttpRequestResponseType::Blob => {
                    self.blob_response().to_jsval(cx, rval.handle_mut());
                },
                _ => {
                    // XXXManishearth handle other response types
                    self.response.borrow().to_jsval(cx, rval.handle_mut());
                }
            }
            rval.ptr
        }
    }

    // https://xhr.spec.whatwg.org/#the-responsetext-attribute
    fn GetResponseText(&self) -> Fallible<USVString> {
        match self.response_type.get() {
            XMLHttpRequestResponseType::_empty | XMLHttpRequestResponseType::Text => {
                Ok(USVString(String::from(match self.ready_state.get() {
                    // Step 3
                    XMLHttpRequestState::Loading | XMLHttpRequestState::Done => self.text_response(),
                    // Step 2
                    _ => "".to_owned()
                })))
            },
            // Step 1
            _ => Err(Error::InvalidState)
        }
    }

    // https://xhr.spec.whatwg.org/#the-responsexml-attribute
    fn GetResponseXML(&self) -> Fallible<Option<Root<Document>>> {
        match self.response_type.get() {
            XMLHttpRequestResponseType::_empty | XMLHttpRequestResponseType::Document => {
                // Step 3
                if let XMLHttpRequestState::Done = self.ready_state.get() {
                    Ok(self.response_xml.get().or_else(|| {
                        let response = self.document_response();
                        self.response_xml.set(response.r());
                        response
                    }))
                } else {
                // Step 2
                    Ok(None)
                }
            }
            // Step 1
            _ => { Err(Error::InvalidState) }
        }
    }
}

pub type TrustedXHRAddress = Trusted<XMLHttpRequest>;


impl XMLHttpRequest {
    fn change_ready_state(&self, rs: XMLHttpRequestState) {
        assert!(self.ready_state.get() != rs);
        self.ready_state.set(rs);
        let global = self.global();
        let event = Event::new(global.r(),
                               atom!("readystatechange"),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::Cancelable);
        event.fire(self.upcast());
    }

    fn process_headers_available(&self, cors_request: Option<CORSRequest>,
                                 gen_id: GenerationId, metadata: Metadata) -> Result<(), Error> {

        let bypass_cross_origin_check = {
            // We want to be able to do cross-origin requests in browser.html.
            // If the XHR happens in a top level window and the mozbrowser
            // preference is enabled, we allow bypassing the CORS check.
            // This is a temporary measure until we figure out Servo privilege
            // story. See https://github.com/servo/servo/issues/9582
            if let GlobalRoot::Window(win) = self.global() {
                let is_root_pipeline = win.parent_info().is_none();
                let is_mozbrowser_enabled = prefs::get_pref("dom.mozbrowser.enabled").as_boolean().unwrap_or(false);
                is_root_pipeline && is_mozbrowser_enabled
            } else {
                false
            }
        };

        if !bypass_cross_origin_check {
            if let Some(ref req) = cors_request {
                match metadata.headers {
                    Some(ref h) if allow_cross_origin_request(req, h) => {},
                    _ => {
                        self.process_partial_response(XHRProgress::Errored(gen_id, Error::Network));
                        return Err(Error::Network);
                    }
                }
            }
        } else {
            debug!("Bypassing cross origin check");
        }

        *self.response_url.borrow_mut() = metadata.final_url.serialize_no_fragment();

        // XXXManishearth Clear cache entries in case of a network error
        self.process_partial_response(XHRProgress::HeadersReceived(gen_id,
                                                                   metadata.headers,
                                                                   metadata.status));
        Ok(())
    }

    fn process_data_available(&self, gen_id: GenerationId, payload: Vec<u8>) {
        self.process_partial_response(XHRProgress::Loading(gen_id, ByteString::new(payload)));
    }

    fn process_response_complete(&self, gen_id: GenerationId, status: Result<(), String>)
                                 -> ErrorResult {
        match status {
            Ok(()) => {
                self.process_partial_response(XHRProgress::Done(gen_id));
                Ok(())
            },
            Err(_) => {
                self.process_partial_response(XHRProgress::Errored(gen_id, Error::Network));
                Err(Error::Network)
            }
        }
    }

    fn process_partial_response(&self, progress: XHRProgress) {
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
                if !self.sync.get() {
                    self.dispatch_upload_progress_event(atom!("progress"), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event(atom!("load"), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event(atom!("loadend"), None);
                    return_if_fetch_was_terminated!();
                }
                // Part of step 13, send() (processing response)
                // XXXManishearth handle errors, if any (substep 1)
                // Substep 2
                status.map(|RawStatus(code, reason)| {
                    self.status.set(code);
                    *self.status_text.borrow_mut() = ByteString::new(reason.into_owned().into_bytes());
                });
                headers.as_ref().map(|h| *self.response_headers.borrow_mut() = h.clone());

                // Substep 3
                if !self.sync.get() {
                    self.change_ready_state(XMLHttpRequestState::HeadersReceived);
                }
            },
            XHRProgress::Loading(_, partial_response) => {
                // For synchronous requests, this should not fire any events, and just store data
                // Part of step 11, send() (processing response body)
                // XXXManishearth handle errors, if any (substep 2)

                *self.response.borrow_mut() = partial_response;
                if !self.sync.get() {
                    if self.ready_state.get() == XMLHttpRequestState::HeadersReceived {
                        self.change_ready_state(XMLHttpRequestState::Loading);
                        return_if_fetch_was_terminated!();
                    }
                    self.dispatch_response_progress_event(atom!("progress"));
                }
            },
            XHRProgress::Done(_) => {
                assert!(self.ready_state.get() == XMLHttpRequestState::HeadersReceived ||
                        self.ready_state.get() == XMLHttpRequestState::Loading ||
                        self.sync.get());

                self.cancel_timeout();

                // Part of step 11, send() (processing response end of file)
                // XXXManishearth handle errors, if any (substep 2)

                // Subsubsteps 5-7
                self.send_flag.set(false);

                self.change_ready_state(XMLHttpRequestState::Done);
                return_if_fetch_was_terminated!();
                // Subsubsteps 10-12
                self.dispatch_response_progress_event(atom!("progress"));
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event(atom!("load"));
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event(atom!("loadend"));
            },
            XHRProgress::Errored(_, e) => {
                self.cancel_timeout();

                self.discard_subsequent_responses();
                self.send_flag.set(false);
                // XXXManishearth set response to NetworkError
                self.change_ready_state(XMLHttpRequestState::Done);
                return_if_fetch_was_terminated!();

                let errormsg = match e {
                    Error::Abort => "abort",
                    Error::Timeout => "timeout",
                    _ => "error",
                };

                let upload_complete = &self.upload_complete;
                if !upload_complete.get() {
                    upload_complete.set(true);
                    self.dispatch_upload_progress_event(atom!("progress"), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event(Atom::from(errormsg), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event(atom!("loadend"), None);
                    return_if_fetch_was_terminated!();
                }
                self.dispatch_response_progress_event(atom!("progress"));
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event(Atom::from(errormsg));
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event(atom!("loadend"));
            }
        }
    }

    fn terminate_ongoing_fetch(&self) {
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
        self.response_status.set(Ok(()));
    }

    fn insert_trusted_header(&self, name: String, value: String) {
        // Insert a header without checking spec-compliance
        // Use for hardcoded headers
        self.request_headers.borrow_mut().set_raw(name, vec![value.into_bytes()]);
    }

    fn dispatch_progress_event(&self, upload: bool, type_: Atom, loaded: u64, total: Option<u64>) {
        let global = self.global();
        let progressevent = ProgressEvent::new(global.r(),
                                               type_,
                                               EventBubbles::DoesNotBubble,
                                               EventCancelable::NotCancelable,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0));
        let target = if upload {
            self.upload.upcast()
        } else {
            self.upcast()
        };
        progressevent.upcast::<Event>().fire(target);
    }

    fn dispatch_upload_progress_event(&self, type_: Atom, partial_load: Option<u64>) {
        // If partial_load is None, loading has completed and we can just use the value from the request body

        let total = self.request_body_len.get() as u64;
        self.dispatch_progress_event(true, type_, partial_load.unwrap_or(total), Some(total));
    }

    fn dispatch_response_progress_event(&self, type_: Atom) {
        let len = self.response.borrow().len() as u64;
        let total = self.response_headers.borrow().get::<ContentLength>().map(|x| { **x as u64 });
        self.dispatch_progress_event(false, type_, len, total);
    }
    fn set_timeout(&self, duration_ms: u32) {
        // Sets up the object to timeout in a given number of milliseconds
        // This will cancel all previous timeouts
        let global = self.global();
        let callback = OneshotTimerCallback::XhrTimeout(XHRTimeoutCallback {
            xhr: Trusted::new(self, global.r().networking_task_source()),
            generation_id: self.generation_id.get(),
        });
        let duration = Length::new(duration_ms as u64);
        *self.timeout_cancel.borrow_mut() = Some(global.r().schedule_callback(callback, duration));
    }

    fn cancel_timeout(&self) {
        if let Some(handle) = self.timeout_cancel.borrow_mut().take() {
            let global = self.global();
            global.r().unschedule_callback(handle);
        }
    }

    // https://xhr.spec.whatwg.org/#text-response
    fn text_response(&self) -> String {
        // Step 3, 5
        let charset = self.final_charset().unwrap_or(UTF_8);
        // TODO: Step 4 - add support for XML encoding guess stuff using XML spec

        // According to Simon, decode() should never return an error, so unwrap()ing
        // the result should be fine. XXXManishearth have a closer look at this later
        // Step 1, 2, 6
        charset.decode(&self.response.borrow(), DecoderTrap::Replace).unwrap()
    }

    // https://xhr.spec.whatwg.org/#blob-response
    fn blob_response(&self) -> Root<Blob> {
        // Step 1
        if let Some(response) = self.response_blob.get() {
            return response;
        }
        // Step 2
        let mime = self.final_mime_type().as_ref().map(Mime::to_string).unwrap_or("".to_owned());

        // Step 3, 4
        let blob = Blob::new(self.global().r(), self.response.borrow().to_vec(), &mime);
        self.response_blob.set(Some(blob.r()));
        blob
    }

    // https://xhr.spec.whatwg.org/#document-response
    fn document_response(&self) -> Option<Root<Document>> {
        // Step 1
        let response = self.response_xml.get();
        if response.is_some() {
            return self.response_xml.get();
        }

        let mime_type = self.final_mime_type();
        // TODO: prescan the response to determine encoding if final charset is null
        let charset = self.final_charset().unwrap_or(UTF_8);
        let temp_doc: Root<Document>;
        match mime_type {
            Some(Mime(mime::TopLevel::Text, mime::SubLevel::Html, _)) => {
                // Step 5
                if self.response_type.get() == XMLHttpRequestResponseType::_empty {
                    return None;
                }
                // Step 6
                else {
                    temp_doc = self.document_text_html();
                }
            },
            // Step 7
            Some(Mime(mime::TopLevel::Text, mime::SubLevel::Xml, _)) |
            Some(Mime(mime::TopLevel::Application, mime::SubLevel::Xml, _)) |
            None => {
                temp_doc = self.handle_xml();
            },
            Some(Mime(_, mime::SubLevel::Ext(sub), _)) => {
                if sub.ends_with("+xml") {
                    temp_doc = self.handle_xml();
                }
                else {
                    return None;
                }
            },
            // Step 4
            _ => { return None; }
        }
        // Step 9
        temp_doc.set_encoding(charset);
        // Step 13
        self.response_xml.set(Some(temp_doc.r()));
        return self.response_xml.get();
    }

    #[allow(unsafe_code)]
    // https://xhr.spec.whatwg.org/#json-response
    fn json_response(&self, cx: *mut JSContext) -> JSVal {
        // Step 1
        let response_json = self.response_json.get();
        if !response_json.is_null_or_undefined() {
            return response_json;
        }
        // Step 2
        let bytes = self.response.borrow();
        // Step 3
        if bytes.len() == 0 {
            return NullValue();
        }
        // Step 4
        let json_text = UTF_8.decode(&bytes, DecoderTrap::Replace).unwrap();
        let json_text: Vec<u16> = json_text.encode_utf16().collect();
        // Step 5
        let mut rval = RootedValue::new(cx, UndefinedValue());
        unsafe {
            if !JS_ParseJSON(cx,
                             json_text.as_ptr(),
                             json_text.len() as u32,
                             rval.handle_mut()) {
                JS_ClearPendingException(cx);
                return NullValue();
            }
        }
        // Step 6
        self.response_json.set(rval.ptr);
        self.response_json.get()
    }

    fn document_text_html(&self) -> Root<Document>{
        let charset = self.final_charset().unwrap_or(UTF_8);
        let wr = self.global();
        let wr = wr.r();
        let decoded = charset.decode(&self.response.borrow(), DecoderTrap::Replace).unwrap();
        let document = self.new_doc(IsHTMLDocument::HTMLDocument);
        // TODO: Disable scripting while parsing
        parse_html(document.r(), DOMString::from(decoded), wr.get_url(), ParseContext::Owner(Some(wr.pipeline())));
        document
    }

    fn handle_xml(&self) -> Root<Document> {
        let charset = self.final_charset().unwrap_or(UTF_8);
        let wr = self.global();
        let wr = wr.r();
        let decoded = charset.decode(&self.response.borrow(), DecoderTrap::Replace).unwrap();
        let document = self.new_doc(IsHTMLDocument::NonHTMLDocument);
        // TODO: Disable scripting while parsing
        parse_xml(document.r(), DOMString::from(decoded), wr.get_url(), xml::ParseContext::Owner(Some(wr.pipeline())));
        document
    }

    fn new_doc(&self, is_html_document: IsHTMLDocument) -> Root<Document> {
        let wr = self.global();
        let wr = wr.r();
        let win = wr.as_window();
        let doc = win.Document();
        let doc = doc.r();
        let docloader = DocumentLoader::new(&*doc.loader());
        let base = self.global().r().get_url();
        let parsed_url = match base.join(&self.ResponseURL().0) {
            Ok(parsed) => Some(parsed),
            Err(_) => None // Step 7
        };
        let mime_type = self.final_mime_type();
        let content_type = mime_type.map(|mime|{
            DOMString::from(format!("{}", mime))
        });
        Document::new(win,
                      None,
                      parsed_url,
                      is_html_document,
                      content_type,
                      None,
                      DocumentSource::FromParser, docloader)
    }

    fn filter_response_headers(&self) -> Headers {
        // https://fetch.spec.whatwg.org/#concept-response-header-list
        use hyper::error::Result;
        use hyper::header::SetCookie;
        use hyper::header::{Header, HeaderFormat};
        use std::fmt;

        // a dummy header so we can use headers.remove::<SetCookie2>()
        #[derive(Clone, Debug, HeapSizeOf)]
        struct SetCookie2;
        impl Header for SetCookie2 {
            fn header_name() -> &'static str {
                "set-cookie2"
            }

            fn parse_header(_: &[Vec<u8>]) -> Result<SetCookie2> {
                unimplemented!()
            }
        }
        impl HeaderFormat for SetCookie2 {
            fn fmt_header(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                unimplemented!()
            }
        }

        let mut headers = self.response_headers.borrow().clone();
        headers.remove::<SetCookie>();
        headers.remove::<SetCookie2>();
        // XXXManishearth additional CORS filtering goes here
        headers
    }

    fn discard_subsequent_responses(&self) {
        self.response_status.set(Err(()));
    }

    fn fetch(&self,
              load_data: LoadData,
              cors_request: Result<Option<CORSRequest>, ()>,
              global: GlobalRef) -> ErrorResult {
        let cors_request = match cors_request {
            Err(_) => {
                // Happens in case of unsupported cross-origin URI schemes.
                // Supported schemes are http, https, data, and about.
                self.process_partial_response(XHRProgress::Errored(
                    self.generation_id.get(), Error::Network));
                return Err(Error::Network);
            }
            Ok(req) => req,
        };

        let xhr = Trusted::new(self, global.networking_task_source());

        let context = Arc::new(Mutex::new(XHRContext {
            xhr: xhr,
            cors_request: cors_request.clone(),
            gen_id: self.generation_id.get(),
            buf: DOMRefCell::new(vec!()),
            sync_status: DOMRefCell::new(None),
        }));

        let (script_chan, script_port) = if self.sync.get() {
            let (tx, rx) = global.new_script_pair();
            (tx, Some(rx))
        } else {
            (global.networking_task_source(), None)
        };

        let resource_thread = global.resource_thread();
        if let Some(req) = cors_request {
            XMLHttpRequest::check_cors(context.clone(), load_data, req.clone(),
                                       script_chan.clone(), resource_thread);
        } else {
            XMLHttpRequest::initiate_async_xhr(context.clone(), script_chan,
                                               resource_thread, load_data);
        }

        if let Some(script_port) = script_port {
            loop {
                global.process_event(script_port.recv().unwrap());
                let context = context.lock().unwrap();
                let sync_status = context.sync_status.borrow();
                if let Some(ref status) = *sync_status {
                    return status.clone();
                }
            }
        }
        Ok(())
    }

    fn final_charset(&self) -> Option<EncodingRef> {
        if self.override_charset.borrow().is_some() {
            self.override_charset.borrow().clone()
        } else {
            match self.response_headers.borrow().get() {
                Some(&ContentType(ref mime)) => {
                    let value = mime.get_param(mime::Attr::Charset);
                    value.and_then(|value|{
                        encoding_from_whatwg_label(value)
                    })
                }
                None => { None }
            }
        }
    }

    fn final_mime_type(&self) -> Option<Mime> {
        if self.override_mime_type.borrow().is_some() {
            self.override_mime_type.borrow().clone()
        } else {
            match self.response_headers.borrow().get() {
                Some(&ContentType(ref mime)) => { Some(mime.clone()) },
                None => { None }
            }
        }
    }
}

#[derive(JSTraceable, HeapSizeOf)]
pub struct XHRTimeoutCallback {
    #[ignore_heap_size_of = "Because it is non-owning"]
    xhr: Trusted<XMLHttpRequest>,
    generation_id: GenerationId,
}

impl XHRTimeoutCallback {
    pub fn invoke(self) {
        let xhr = self.xhr.root();
        if xhr.ready_state.get() != XMLHttpRequestState::Done {
            xhr.process_partial_response(XHRProgress::Errored(self.generation_id, Error::Timeout));
        }
    }
}

trait Extractable {
    fn extract(&self) -> (Vec<u8>, Option<DOMString>);
}
impl Extractable for BodyInit {
    // https://fetch.spec.whatwg.org/#concept-bodyinit-extract
    fn extract(&self) -> (Vec<u8>, Option<DOMString>) {
        match *self {
            BodyInit::String(ref s) => {
                let encoding = UTF_8 as EncodingRef;
                (encoding.encode(s, EncoderTrap::Replace).unwrap(),
                    Some(DOMString::from("text/plain;charset=UTF-8")))
            },
            BodyInit::URLSearchParams(ref usp) => {
                // Default encoding is UTF-8.
                (usp.serialize(None).into_bytes(),
                    Some(DOMString::from("application/x-www-form-urlencoded;charset=UTF-8")))
            },
            BodyInit::Blob(ref b) => {
                let data = b.get_data();
                let content_type = if b.Type().as_ref().is_empty() {
                    None
                } else {
                    Some(b.Type())
                };
                (data.get_bytes().to_vec(), content_type)
            },
        }
    }
}

/// Returns whether `bs` is a `field-value`, as defined by
/// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-32).
pub fn is_field_value(slice: &[u8]) -> bool {
    // Classifications of characters necessary for the [CRLF] (SP|HT) rule
    #[derive(PartialEq)]
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
            13  => { // CR
                if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::CR;
                    true
                } else {
                    false
                }
            },
            10 => { // LF
                if prev == PreviousCharacter::CR {
                    prev = PreviousCharacter::LF;
                    true
                } else {
                    false
                }
            },
            32 => { // SP
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
            9 => { // HT
                if prev == PreviousCharacter::LF || prev == PreviousCharacter::SPHT {
                    prev = PreviousCharacter::SPHT;
                    true
                } else {
                    false
                }
            },
            0...31 | 127 => false, // CTLs
            x if x > 127 => false, // non ASCII
            _ if prev == PreviousCharacter::Other || prev == PreviousCharacter::SPHT => {
                prev = PreviousCharacter::Other;
                true
            },
            _ => false // Previous character was a CR/LF but not part of the [CRLF] (SP|HT) rule
        }
    })
}

/// Normalize `self`, as defined by
/// [the Fetch Spec](https://fetch.spec.whatwg.org/#concept-header-value-normalize).
pub fn trim_http_whitespace(mut slice: &[u8]) -> &[u8] {
    const HTTP_WS_BYTES: &'static [u8] = b"\x09\x0A\x0D\x20";

    loop {
        match slice.split_first() {
            Some((first, remainder)) if HTTP_WS_BYTES.contains(first) =>
                slice = remainder,
            _ => break,
        }
    }

    loop {
        match slice.split_last() {
            Some((last, remainder)) if HTTP_WS_BYTES.contains(last) =>
                slice = remainder,
            _ => break,
        }
    }

    slice
}
