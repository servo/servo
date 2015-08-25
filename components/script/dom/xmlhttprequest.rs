/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestMethods;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseType;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseType::{_empty, Json, Text};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast, XMLHttpRequestDerived};
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::Error::{InvalidState, InvalidAccess};
use dom::bindings::error::Error::{Network, Syntax, Security, Abort, Timeout};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::{GlobalField, GlobalRef, GlobalRoot};
use dom::bindings::js::Root;
use dom::bindings::js::{JS, MutNullableHeap};
use dom::bindings::refcounted::Trusted;
use dom::bindings::str::ByteString;
use dom::bindings::utils::{Reflectable, reflect_dom_object};
use dom::document::Document;
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::progressevent::ProgressEvent;
use dom::urlsearchparams::URLSearchParamsHelpers;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTargetTypeId;
use dom::xmlhttprequestupload::XMLHttpRequestUpload;
use network_listener::{NetworkListener, PreInvoke};
use script_task::{ScriptChan, Runnable, ScriptPort, CommonScriptMsg};

use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecoderTrap, Encoding, EncodingRef, EncoderTrap};

use hyper::header::Headers;
use hyper::header::{Accept, ContentLength, ContentType, qitem};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{self, Mime};

use js::jsapi::JS_ClearPendingException;
use js::jsapi::{JS_ParseJSON, JSContext, RootedValue};
use js::jsval::{JSVal, NullValue, UndefinedValue};

use cors::CORSResponse;
use cors::{allow_cross_origin_request, CORSRequest, RequestMode, AsyncCORSResponseListener};
use net_traits::ControlMsg::Load;
use net_traits::{AsyncResponseListener, AsyncResponseTarget, Metadata};
use net_traits::{ResourceTask, ResourceCORSData, LoadData, LoadConsumer};
use util::mem::HeapSizeOf;
use util::str::DOMString;
use util::task::spawn_named;

use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::{RefCell, Cell};
use std::default::Default;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::{Mutex, Arc};
use std::thread::sleep_ms;
use time;
use url::{Url, UrlParser};

use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams;
use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams::{eString, eURLSearchParams};

pub type SendParam = StringOrURLSearchParams;

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
    /// There was an error (only Abort, Timeout or Network is used)
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
    response_url: DOMString,
    status: Cell<u16>,
    status_text: DOMRefCell<ByteString>,
    response: DOMRefCell<ByteString>,
    response_type: Cell<XMLHttpRequestResponseType>,
    response_xml: MutNullableHeap<JS<Document>>,
    #[ignore_heap_size_of = "Defined in hyper"]
    response_headers: DOMRefCell<Headers>,

    // Associated concepts
    request_method: DOMRefCell<Method>,
    request_url: DOMRefCell<Option<Url>>,
    #[ignore_heap_size_of = "Defined in hyper"]
    request_headers: DOMRefCell<Headers>,
    request_body_len: Cell<usize>,
    sync: Cell<bool>,
    upload_complete: Cell<bool>,
    upload_events: Cell<bool>,
    send_flag: Cell<bool>,

    global: GlobalField,
    #[ignore_heap_size_of = "Defined in std"]
    timeout_cancel: DOMRefCell<Option<Sender<()>>>,
    fetch_time: Cell<i64>,
    #[ignore_heap_size_of = "Cannot calculate Heap size"]
    timeout_target: DOMRefCell<Option<Box<ScriptChan + Send>>>,
    generation_id: Cell<GenerationId>,
    response_status: Cell<Result<(), ()>>,
}

impl XMLHttpRequest {
    fn new_inherited(global: GlobalRef) -> XMLHttpRequest {
        XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestEventTargetTypeId::XMLHttpRequest),
            ready_state: Cell::new(XMLHttpRequestState::Unsent),
            timeout: Cell::new(0u32),
            with_credentials: Cell::new(false),
            upload: JS::from_rooted(&XMLHttpRequestUpload::new(global)),
            response_url: "".to_owned(),
            status: Cell::new(0),
            status_text: DOMRefCell::new(ByteString::new(vec!())),
            response: DOMRefCell::new(ByteString::new(vec!())),
            response_type: Cell::new(_empty),
            response_xml: Default::default(),
            response_headers: DOMRefCell::new(Headers::new()),

            request_method: DOMRefCell::new(Method::Get),
            request_url: DOMRefCell::new(None),
            request_headers: DOMRefCell::new(Headers::new()),
            request_body_len: Cell::new(0),
            sync: Cell::new(false),
            send_flag: Cell::new(false),

            upload_complete: Cell::new(false),
            upload_events: Cell::new(false),

            global: GlobalField::from_rooted(&global),
            timeout_cancel: DOMRefCell::new(None),
            fetch_time: Cell::new(0),
            timeout_target: DOMRefCell::new(None),
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

    fn check_cors(context: Arc<Mutex<XHRContext>>,
                  load_data: LoadData,
                  req: CORSRequest,
                  script_chan: Box<ScriptChan + Send>,
                  resource_task: ResourceTask) {
        struct CORSContext {
            xhr: Arc<Mutex<XHRContext>>,
            load_data: RefCell<Option<LoadData>>,
            req: CORSRequest,
            script_chan: Box<ScriptChan + Send>,
            resource_task: ResourceTask,
        }

        impl AsyncCORSResponseListener for CORSContext {
            fn response_available(&self, response: CORSResponse) {
                if response.network_error {
                    let mut context = self.xhr.lock().unwrap();
                    let xhr = context.xhr.root();
                    xhr.r().process_partial_response(XHRProgress::Errored(context.gen_id, Network));
                    *context.sync_status.borrow_mut() = Some(Err(Network));
                    return;
                }

                let mut load_data = self.load_data.borrow_mut().take().unwrap();
                load_data.cors = Some(ResourceCORSData {
                    preflight: self.req.preflight_flag,
                    origin: self.req.origin.clone()
                });

                XMLHttpRequest::initiate_async_xhr(self.xhr.clone(), self.script_chan.clone(),
                                                   self.resource_task.clone(), load_data);
            }
        }

        let cors_context = CORSContext {
            xhr: context,
            load_data: RefCell::new(Some(load_data)),
            req: req.clone(),
            script_chan: script_chan.clone(),
            resource_task: resource_task,
        };

        req.http_fetch_async(box cors_context, script_chan);
    }

    fn initiate_async_xhr(context: Arc<Mutex<XHRContext>>,
                          script_chan: Box<ScriptChan + Send>,
                          resource_task: ResourceTask,
                          load_data: LoadData) {
        impl AsyncResponseListener for XHRContext {
            fn headers_available(&self, metadata: Metadata) {
                let xhr = self.xhr.root();
                let rv = xhr.r().process_headers_available(self.cors_request.clone(),
                                                           self.gen_id,
                                                           metadata);
                if rv.is_err() {
                    *self.sync_status.borrow_mut() = Some(rv);
                }
            }

            fn data_available(&self, payload: Vec<u8>) {
                self.buf.borrow_mut().push_all(&payload);
                let xhr = self.xhr.root();
                xhr.r().process_data_available(self.gen_id, self.buf.borrow().clone());
            }

            fn response_complete(&self, status: Result<(), String>) {
                let xhr = self.xhr.root();
                let rv = xhr.r().process_response_complete(self.gen_id, status);
                *self.sync_status.borrow_mut() = Some(rv);
            }
        }

        impl PreInvoke for XHRContext {
            fn should_invoke(&self) -> bool {
                let xhr = self.xhr.root();
                xhr.r().generation_id.get() == self.gen_id
            }
        }

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = box NetworkListener {
            context: context,
            script_chan: script_chan,
        };
        let response_target = AsyncResponseTarget {
            sender: action_sender,
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify(message.to().unwrap());
        });
        resource_task.send(Load(load_data, LoadConsumer::Listener(response_target))).unwrap();
    }
}

impl<'a> XMLHttpRequestMethods for &'a XMLHttpRequest {
    event_handler!(readystatechange, GetOnreadystatechange, SetOnreadystatechange);

    // https://xhr.spec.whatwg.org/#dom-xmlhttprequest-readystate
    fn ReadyState(self) -> u16 {
        self.ready_state.get() as u16
    }

    // https://xhr.spec.whatwg.org/#the-open()-method
    fn Open(self, method: ByteString, url: DOMString) -> ErrorResult {
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
        // Step 2
        match maybe_method {
            // Step 4
            Some(Method::Connect) | Some(Method::Trace) => Err(Security),
            Some(Method::Extension(ref t)) if &**t == "TRACK" => Err(Security),
            Some(parsed_method) => {
                // Step 3
                if !method.is_token() {
                    return Err(Syntax)
                }

                *self.request_method.borrow_mut() = parsed_method;

                // Step 6
                let base = self.global.root().r().get_url();
                let parsed_url = match UrlParser::new().base_url(&base).parse(&url) {
                    Ok(parsed) => parsed,
                    Err(_) => return Err(Syntax) // Step 7
                };
                // XXXManishearth Do some handling of username/passwords
                if self.sync.get() {
                    // FIXME: This should only happen if the global environment is a document environment
                    if self.timeout.get() != 0 || self.with_credentials.get() || self.response_type.get() != _empty {
                        return Err(InvalidAccess)
                    }
                }
                // abort existing requests
                self.terminate_ongoing_fetch();

                // Step 12
                *self.request_url.borrow_mut() = Some(parsed_url);
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
            // This includes cases where as_str() returns None, and when is_token() returns false,
            // both of which indicate invalid extension method names
            _ => Err(Syntax), // Step 3
        }
    }

    // https://xhr.spec.whatwg.org/#the-open()-method
    fn Open_(self, method: ByteString, url: DOMString, async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult {
        self.sync.set(!async);
        self.Open(method, url)
    }

    // https://xhr.spec.whatwg.org/#the-setrequestheader()-method
    fn SetRequestHeader(self, name: ByteString, mut value: ByteString) -> ErrorResult {
        if self.ready_state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(InvalidState); // Step 1, 2
        }
        if !name.is_token() || !value.is_field_value() {
            return Err(Syntax); // Step 3, 4
        }
        let name_lower = name.to_lower();
        let name_str = match name_lower.as_str() {
            Some(s) => {
                match s {
                    // Disallowed headers
                    "accept-charset" | "accept-encoding" |
                    "access-control-request-headers" |
                    "access-control-request-method" |
                    "connection" | "content-length" |
                    "cookie" | "cookie2" | "date" |"dnt" |
                    "expect" | "host" | "keep-alive" | "origin" |
                    "referer" | "te" | "trailer" | "transfer-encoding" |
                    "upgrade" | "user-agent" | "via" => {
                        return Ok(()); // Step 5
                    },
                    _ => s
                }
            },
            None => return Err(Syntax)
        };

        debug!("SetRequestHeader: name={:?}, value={:?}", name.as_str(), value.as_str());
        let mut headers = self.request_headers.borrow_mut();


        // Steps 6,7
        match headers.get_raw(name_str) {
            Some(raw) => {
                debug!("SetRequestHeader: old value = {:?}", raw[0]);
                let mut buf = raw[0].clone();
                buf.push_all(b", ");
                buf.push_all(&value);
                debug!("SetRequestHeader: new value = {:?}", buf);
                value = ByteString::new(buf);
            },
            None => {}
        }

        headers.set_raw(name_str.to_owned(), vec![value.to_vec()]);
        Ok(())
    }

    // https://xhr.spec.whatwg.org/#the-timeout-attribute
    fn Timeout(self) -> u32 {
        self.timeout.get()
    }

    // https://xhr.spec.whatwg.org/#the-timeout-attribute
    fn SetTimeout(self, timeout: u32) -> ErrorResult {
        if self.sync.get() {
            // FIXME: Not valid for a worker environment
            Err(InvalidAccess)
        } else {
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
    }

    // https://xhr.spec.whatwg.org/#the-withcredentials-attribute
    fn WithCredentials(self) -> bool {
        self.with_credentials.get()
    }

    // https://xhr.spec.whatwg.org/#dom-xmlhttprequest-withcredentials
    fn SetWithCredentials(self, with_credentials: bool) -> ErrorResult {
        match self.ready_state.get() {
            XMLHttpRequestState::HeadersReceived |
            XMLHttpRequestState::Loading |
            XMLHttpRequestState::Done => Err(InvalidState),
            _ if self.send_flag.get() => Err(InvalidState),
            _ => match self.global.root() {
                GlobalRoot::Window(_) if self.sync.get() => Err(InvalidAccess),
                _ => {
                    self.with_credentials.set(with_credentials);
                    Ok(())
                },
            },
        }
    }

    // https://xhr.spec.whatwg.org/#the-upload-attribute
    fn Upload(self) -> Root<XMLHttpRequestUpload> {
        self.upload.root()
    }

    // https://xhr.spec.whatwg.org/#the-send()-method
    fn Send(self, data: Option<SendParam>) -> ErrorResult {
        if self.ready_state.get() != XMLHttpRequestState::Opened || self.send_flag.get() {
            return Err(InvalidState); // Step 1, 2
        }

        let data = match *self.request_method.borrow() {
            Method::Get | Method::Head => None, // Step 3
            _ => data
        };
        let extracted = data.as_ref().map(|d| d.extract());
        self.request_body_len.set(extracted.as_ref().map(|e| e.len()).unwrap_or(0));

        // Step 6
        self.upload_events.set(false);
        // Step 7
        self.upload_complete.set(match extracted {
            None => true,
            Some (ref v) if v.is_empty() => true,
            _ => false
        });

        if !self.sync.get() {
            // Step 8
            let upload_target = self.upload.root();
            let event_target = EventTargetCast::from_ref(upload_target.r());
            if event_target.has_handlers() {
                self.upload_events.set(true);
            }

            // Step 9
            self.send_flag.set(true);
            // If one of the event handlers below aborts the fetch by calling
            // abort or open we will need the current generation id to detect it.
            let gen_id = self.generation_id.get();
            self.dispatch_response_progress_event("loadstart".to_owned());
            if self.generation_id.get() != gen_id {
                return Ok(());
            }
            if !self.upload_complete.get() {
                self.dispatch_upload_progress_event("loadstart".to_owned(), Some(0));
                if self.generation_id.get() != gen_id {
                    return Ok(());
                }
            }

        }

        let global = self.global.root();
        let pipeline_id = global.r().pipeline();
        let mut load_data = LoadData::new(self.request_url.borrow().clone().unwrap(), Some(pipeline_id));
        load_data.data = extracted;

        #[inline]
        fn join_raw(a: &str, b: &str) -> Vec<u8> {
            let len = a.len() + b.len();
            let mut vec = Vec::with_capacity(len);
            vec.push_all(a.as_bytes());
            vec.push_all(b.as_bytes());
            vec
        }

        // XHR spec differs from http, and says UTF-8 should be in capitals,
        // instead of "utf-8", which is what Hyper defaults to.
        let params = ";charset=UTF-8";
        let n = "content-type";
        match data {
            Some(eString(_)) =>
                load_data.headers.set_raw(n.to_owned(), vec![join_raw("text/plain", params)]),
            Some(eURLSearchParams(_)) =>
                load_data.headers.set_raw(
                    n.to_owned(), vec![join_raw("application/x-www-form-urlencoded", params)]),
            None => ()
        }

        load_data.preserved_headers = (*self.request_headers.borrow()).clone();

        if !load_data.preserved_headers.has::<Accept>() {
            let mime = Mime(mime::TopLevel::Star, mime::SubLevel::Star, vec![]);
            load_data.preserved_headers.set(Accept(vec![qitem(mime)]));
        }

        load_data.method = (*self.request_method.borrow()).clone();

        // CORS stuff
        let global = self.global.root();
        let referer_url = self.global.root().r().get_url();
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
                                                  combined_headers);
        match cors_request {
            Ok(None) => {
                let mut buf = String::new();
                buf.push_str(&referer_url.scheme);
                buf.push_str("://");
                referer_url.serialize_host().map(|ref h| buf.push_str(h));
                referer_url.port().as_ref().map(|&p| {
                    buf.push_str(":");
                    buf.push_str(&p.to_string());
                });
                referer_url.serialize_path().map(|ref h| buf.push_str(h));
                self.request_headers.borrow_mut().set_raw("Referer".to_owned(), vec![buf.into_bytes()]);
            },
            Ok(Some(ref req)) => self.insert_trusted_header("origin".to_owned(),
                                                            req.origin.to_string()),
            _ => {}
        }

        debug!("request_headers = {:?}", *self.request_headers.borrow());

        self.fetch_time.set(time::now().to_timespec().sec);
        let rv = self.fetch(load_data, cors_request, global.r());
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
    fn Abort(self) {
        self.terminate_ongoing_fetch();
        let state = self.ready_state.get();
        if (state == XMLHttpRequestState::Opened && self.send_flag.get()) ||
           state == XMLHttpRequestState::HeadersReceived ||
           state == XMLHttpRequestState::Loading {
            let gen_id = self.generation_id.get();
            self.process_partial_response(XHRProgress::Errored(gen_id, Abort));
            // If open was called in one of the handlers invoked by the
            // above call then we should terminate the abort sequence
            if self.generation_id.get() != gen_id {
                return
            }
        }
        self.ready_state.set(XMLHttpRequestState::Unsent);
    }

    // https://xhr.spec.whatwg.org/#the-responseurl-attribute
    fn ResponseURL(self) -> DOMString {
        self.response_url.clone()
    }

    // https://xhr.spec.whatwg.org/#the-status-attribute
    fn Status(self) -> u16 {
        self.status.get()
    }

    // https://xhr.spec.whatwg.org/#the-statustext-attribute
    fn StatusText(self) -> ByteString {
        self.status_text.borrow().clone()
    }

    // https://xhr.spec.whatwg.org/#the-getresponseheader()-method
    fn GetResponseHeader(self, name: ByteString) -> Option<ByteString> {
        self.filter_response_headers().iter().find(|h| {
            name.eq_ignore_case(&h.name().parse().unwrap())
        }).map(|h| {
            ByteString::new(h.value_string().into_bytes())
        })
    }

    // https://xhr.spec.whatwg.org/#the-getallresponseheaders()-method
    fn GetAllResponseHeaders(self) -> ByteString {
        ByteString::new(self.filter_response_headers().to_string().into_bytes())
    }

    // https://xhr.spec.whatwg.org/#the-responsetype-attribute
    fn ResponseType(self) -> XMLHttpRequestResponseType {
        self.response_type.get()
    }

    // https://xhr.spec.whatwg.org/#the-responsetype-attribute
    fn SetResponseType(self, response_type: XMLHttpRequestResponseType) -> ErrorResult {
        match self.global.root() {
            GlobalRoot::Worker(_) if response_type == XMLHttpRequestResponseType::Document
            => return Ok(()),
            _ => {}
        }
        match self.ready_state.get() {
            XMLHttpRequestState::Loading | XMLHttpRequestState::Done => Err(InvalidState),
            _ if self.sync.get() => Err(InvalidAccess),
            _ => {
                self.response_type.set(response_type);
                Ok(())
            }
        }
    }

    #[allow(unsafe_code)]
    // https://xhr.spec.whatwg.org/#the-response-attribute
    fn Response(self, cx: *mut JSContext) -> JSVal {
         let mut rval = RootedValue::new(cx, UndefinedValue());
         match self.response_type.get() {
            _empty | Text => {
                let ready_state = self.ready_state.get();
                if ready_state == XMLHttpRequestState::Done || ready_state == XMLHttpRequestState::Loading {
                    self.text_response().to_jsval(cx, rval.handle_mut());
                } else {
                    "".to_jsval(cx, rval.handle_mut());
                }
            },
            _ if self.ready_state.get() != XMLHttpRequestState::Done => {
                return NullValue()
            },
            Json => {
                let decoded = UTF_8.decode(&self.response.borrow(), DecoderTrap::Replace).unwrap().to_owned();
                let decoded: Vec<u16> = decoded.utf16_units().collect();
                unsafe {
                    if JS_ParseJSON(cx,
                                    decoded.as_ptr() as *const i16,
                                    decoded.len() as u32,
                                    rval.handle_mut()) == 0 {
                        JS_ClearPendingException(cx);
                        return NullValue();
                    }
                }
            }
            _ => {
                // XXXManishearth handle other response types
                self.response.borrow().to_jsval(cx, rval.handle_mut());
            }
        }
        rval.ptr
    }

    // https://xhr.spec.whatwg.org/#the-responsetext-attribute
    fn GetResponseText(self) -> Fallible<DOMString> {
        match self.response_type.get() {
            _empty | Text => {
                match self.ready_state.get() {
                    XMLHttpRequestState::Loading | XMLHttpRequestState::Done => Ok(self.text_response()),
                    _ => Ok("".to_owned())
                }
            },
            _ => Err(InvalidState)
        }
    }

    // https://xhr.spec.whatwg.org/#the-responsexml-attribute
    fn GetResponseXML(self) -> Option<Root<Document>> {
        self.response_xml.get().map(Root::from_rooted)
    }
}


impl XMLHttpRequestDerived for EventTarget {
    fn is_xmlhttprequest(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::XMLHttpRequestEventTarget(XMLHttpRequestEventTargetTypeId::XMLHttpRequest) => true,
            _ => false
        }
    }
}

pub type TrustedXHRAddress = Trusted<XMLHttpRequest>;

trait PrivateXMLHttpRequestHelpers {
    fn change_ready_state(self, XMLHttpRequestState);
    fn process_headers_available(self, cors_request: Option<CORSRequest>,
                                 gen_id: GenerationId, metadata: Metadata) -> Result<(), Error>;
    fn process_data_available(self, gen_id: GenerationId, payload: Vec<u8>);
    fn process_response_complete(self, gen_id: GenerationId, status: Result<(), String>) -> ErrorResult;
    fn process_partial_response(self, progress: XHRProgress);
    fn terminate_ongoing_fetch(self);
    fn insert_trusted_header(self, name: String, value: String);
    fn dispatch_progress_event(self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>);
    fn dispatch_upload_progress_event(self, type_: DOMString, partial_load: Option<u64>);
    fn dispatch_response_progress_event(self, type_: DOMString);
    fn text_response(self) -> DOMString;
    fn set_timeout(self, timeout: u32);
    fn cancel_timeout(self);
    fn filter_response_headers(self) -> Headers;
    fn discard_subsequent_responses(self);
    fn fetch(self, load_data: LoadData, cors_request: Result<Option<CORSRequest>,()>,
             global: GlobalRef) -> ErrorResult;
}

impl<'a> PrivateXMLHttpRequestHelpers for &'a XMLHttpRequest {
    fn change_ready_state(self, rs: XMLHttpRequestState) {
        assert!(self.ready_state.get() != rs);
        self.ready_state.set(rs);
        let global = self.global.root();
        let event = Event::new(global.r(),
                               "readystatechange".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::Cancelable);
        let target = EventTargetCast::from_ref(self);
        event.r().fire(target);
    }

    fn process_headers_available(self, cors_request: Option<CORSRequest>,
                                 gen_id: GenerationId, metadata: Metadata) -> Result<(), Error> {

        if let Some(ref req) = cors_request {
            match metadata.headers {
                Some(ref h) if allow_cross_origin_request(req, h) => {},
                _ => {
                    self.process_partial_response(XHRProgress::Errored(gen_id, Network));
                    return Err(Network);
                }
            }
        }

        // XXXManishearth Clear cache entries in case of a network error
        self.process_partial_response(XHRProgress::HeadersReceived(gen_id,
                                                                   metadata.headers,
                                                                   metadata.status));
        Ok(())
    }

    fn process_data_available(self, gen_id: GenerationId, payload: Vec<u8>) {
        self.process_partial_response(XHRProgress::Loading(gen_id, ByteString::new(payload)));
    }

    fn process_response_complete(self, gen_id: GenerationId, status: Result<(), String>)
                                 -> ErrorResult {
        match status {
            Ok(()) => {
                self.process_partial_response(XHRProgress::Done(gen_id));
                Ok(())
            },
            Err(_) => {
                self.process_partial_response(XHRProgress::Errored(gen_id, Network));
                Err(Network)
            }
        }
    }

    fn process_partial_response(self, progress: XHRProgress) {
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
                    self.dispatch_upload_progress_event("progress".to_owned(), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event("load".to_owned(), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event("loadend".to_owned(), None);
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
                    self.dispatch_response_progress_event("progress".to_owned());
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
                self.dispatch_response_progress_event("progress".to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event("load".to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event("loadend".to_owned());
            },
            XHRProgress::Errored(_, e) => {
                self.cancel_timeout();

                self.discard_subsequent_responses();
                self.send_flag.set(false);
                // XXXManishearth set response to NetworkError
                self.change_ready_state(XMLHttpRequestState::Done);
                return_if_fetch_was_terminated!();

                let errormsg = match e {
                    Abort => "abort",
                    Timeout => "timeout",
                    _ => "error",
                };

                let upload_complete: &Cell<bool> = &self.upload_complete;
                if !upload_complete.get() {
                    upload_complete.set(true);
                    self.dispatch_upload_progress_event("progress".to_owned(), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event(errormsg.to_owned(), None);
                    return_if_fetch_was_terminated!();
                    self.dispatch_upload_progress_event("loadend".to_owned(), None);
                    return_if_fetch_was_terminated!();
                }
                self.dispatch_response_progress_event("progress".to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event(errormsg.to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event("loadend".to_owned());
            }
        }
    }

    fn terminate_ongoing_fetch(self) {
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
        *self.timeout_target.borrow_mut() = None;
        self.response_status.set(Ok(()));
    }

    fn insert_trusted_header(self, name: String, value: String) {
        // Insert a header without checking spec-compliance
        // Use for hardcoded headers
        self.request_headers.borrow_mut().set_raw(name, vec![value.into_bytes()]);
    }

    fn dispatch_progress_event(self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>) {
        let global = self.global.root();
        let upload_target = self.upload.root();
        let progressevent = ProgressEvent::new(global.r(),
                                               type_, EventBubbles::DoesNotBubble, EventCancelable::NotCancelable,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0));
        let target = if upload {
            EventTargetCast::from_ref(upload_target.r())
        } else {
            EventTargetCast::from_ref(self)
        };
        let event = EventCast::from_ref(progressevent.r());
        event.fire(target);
    }

    fn dispatch_upload_progress_event(self, type_: DOMString, partial_load: Option<u64>) {
        // If partial_load is None, loading has completed and we can just use the value from the request body

        let total = self.request_body_len.get() as u64;
        self.dispatch_progress_event(true, type_, partial_load.unwrap_or(total), Some(total));
    }

    fn dispatch_response_progress_event(self, type_: DOMString) {
        let len = self.response.borrow().len() as u64;
        let total = self.response_headers.borrow().get::<ContentLength>().map(|x| {**x as u64});
        self.dispatch_progress_event(false, type_, len, total);
    }
    fn set_timeout(self, duration_ms: u32) {
        struct XHRTimeout {
            xhr: TrustedXHRAddress,
            gen_id: GenerationId,
        }

        impl Runnable for XHRTimeout {
            fn handler(self: Box<XHRTimeout>) {
                let this = *self;
                let xhr = this.xhr.root();
                if xhr.r().ready_state.get() != XMLHttpRequestState::Done {
                    xhr.r().process_partial_response(XHRProgress::Errored(this.gen_id, Timeout));
                }
            }
        }

        // Sets up the object to timeout in a given number of milliseconds
        // This will cancel all previous timeouts
        let timeout_target = (*self.timeout_target.borrow().as_ref().unwrap()).clone();
        let global = self.global.root();
        let xhr = Trusted::new(global.r().get_cx(), self, global.r().script_chan());
        let gen_id = self.generation_id.get();
        let (cancel_tx, cancel_rx) = channel();
        *self.timeout_cancel.borrow_mut() = Some(cancel_tx);
        spawn_named("XHR:Timer".to_owned(), move || {
            sleep_ms(duration_ms);
            match cancel_rx.try_recv() {
                Err(TryRecvError::Empty) => {
                    timeout_target.send(CommonScriptMsg::RunnableMsg(box XHRTimeout {
                        xhr: xhr,
                        gen_id: gen_id,
                    })).unwrap();
                },
                Err(TryRecvError::Disconnected) | Ok(()) => {
                    // This occurs if xhr.timeout_cancel (the sender) goes out of scope (i.e, xhr went out of scope)
                    // or if the oneshot timer was overwritten. The former case should not happen due to pinning.
                    debug!("XHR timeout was overwritten or canceled")
                }
            }
        }
    );
    }

    fn cancel_timeout(self) {
        if let Some(cancel_tx) = self.timeout_cancel.borrow_mut().take() {
            let _ = cancel_tx.send(());
        }
    }

    fn text_response(self) -> DOMString {
        let mut encoding = UTF_8 as EncodingRef;
        match self.response_headers.borrow().get() {
            Some(&ContentType(mime::Mime(_, _, ref params))) => {
                for &(ref name, ref value) in params {
                    if name == &mime::Attr::Charset {
                        encoding = encoding_from_whatwg_label(&value.to_string()).unwrap_or(encoding);
                    }
                }
            },
            None => {}
        }


        // According to Simon, decode() should never return an error, so unwrap()ing
        // the result should be fine. XXXManishearth have a closer look at this later
        encoding.decode(&self.response.borrow(), DecoderTrap::Replace).unwrap().to_owned()
    }
    fn filter_response_headers(self) -> Headers {
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

    fn discard_subsequent_responses(self) {
        self.response_status.set(Err(()));
    }

    fn fetch(self,
              load_data: LoadData,
              cors_request: Result<Option<CORSRequest>,()>,
              global: GlobalRef) -> ErrorResult {
        let cors_request = match cors_request {
            Err(_) => {
                // Happens in case of cross-origin non-http URIs
                self.process_partial_response(XHRProgress::Errored(
                    self.generation_id.get(), Network));
                return Err(Network);
            }
            Ok(req) => req,
        };

        let xhr = Trusted::new(global.get_cx(), self, global.script_chan());

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
            (global.script_chan(), None)
        };
        *self.timeout_target.borrow_mut() = Some(script_chan.clone());

        let resource_task = global.resource_task();
        if let Some(req) = cors_request {
            XMLHttpRequest::check_cors(context.clone(), load_data, req.clone(),
                                       script_chan.clone(), resource_task);
        } else {
            XMLHttpRequest::initiate_async_xhr(context.clone(), script_chan,
                                               resource_task, load_data);
        }

        if let Some(script_port) = script_port {
            loop {
                global.process_event(script_port.recv());
                let context = context.lock().unwrap();
                let sync_status = context.sync_status.borrow();
                if let Some(ref status) = *sync_status {
                    return status.clone();
                }
            }
        }
        Ok(())
    }
}

trait Extractable {
    fn extract(&self) -> Vec<u8>;
}
impl Extractable for SendParam {
    // https://fetch.spec.whatwg.org/#concept-bodyinit-extract
    fn extract(&self) -> Vec<u8> {
        match *self {
            eString(ref s) => {
                let encoding = UTF_8 as EncodingRef;
                encoding.encode(s, EncoderTrap::Replace).unwrap()
            },
            eURLSearchParams(ref usp) => {
                // Default encoding is UTF-8.
                usp.r().serialize(None).as_bytes().to_owned()
            },
        }
    }
}
