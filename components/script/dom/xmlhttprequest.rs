/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestMethods;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseType;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseTypeValues;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseTypeValues::{_empty, Json, Text};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast, XMLHttpRequestDerived};
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::{Error, ErrorResult, Fallible, InvalidState, InvalidAccess};
use dom::bindings::error::{Network, Syntax, Security, Abort, Timeout};
use dom::bindings::global::{GlobalField, GlobalRef, WorkerField};
use dom::bindings::js::{MutNullableJS, JS, JSRef, Temporary, OptionalRootedRootable};
use dom::bindings::str::ByteString;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::Document;
use dom::event::Event;
use dom::eventtarget::{EventTarget, EventTargetHelpers, XMLHttpRequestTargetTypeId};
use dom::progressevent::ProgressEvent;
use dom::urlsearchparams::URLSearchParamsHelpers;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequestupload::XMLHttpRequestUpload;

use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecodeReplace, Encoding, EncodingRef, EncodeReplace};

use http::headers::response::HeaderCollection as ResponseHeaderCollection;
use http::headers::request::HeaderCollection as RequestHeaderCollection;
use http::headers::content_type::MediaType;
use http::headers::{HeaderEnum, HeaderValueByteIterator};
use http::headers::request::Header;
use http::method::{Method, Get, Head, Connect, Trace, ExtensionMethod};
use http::status::Status;

use js::jsapi::{JS_AddObjectRoot, JS_ParseJSON, JS_RemoveObjectRoot, JSContext};
use js::jsapi::JS_ClearPendingException;
use js::jsval::{JSVal, NullValue, UndefinedValue};

use libc;
use libc::c_void;

use net::resource_task::{ResourceTask, ResourceCORSData, Load, LoadData, Payload, Done};
use cors::{allow_cross_origin_request, CORSRequest, CORSMode, ForcedPreflightMode};
use script_task::{ScriptChan, XHRProgressMsg};
use servo_util::str::DOMString;
use servo_util::task::spawn_named;

use std::ascii::StrAsciiExt;
use std::cell::{Cell, RefCell};
use std::comm::{Sender, Receiver, channel};
use std::default::Default;
use std::io::{BufReader, MemWriter, Timer};
use std::from_str::FromStr;
use std::path::BytesContainer;
use std::task::TaskBuilder;
use std::time::duration::Duration;
use std::num::Zero;
use time;
use url::{Url, UrlParser};

use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams::{eString, eURLSearchParams, StringOrURLSearchParams};
pub type SendParam = StringOrURLSearchParams;


#[deriving(PartialEq)]
#[jstraceable]
pub enum XMLHttpRequestId {
    XMLHttpRequestTypeId,
    XMLHttpRequestUploadTypeId
}

#[deriving(PartialEq)]
#[jstraceable]
enum XMLHttpRequestState {
    Unsent = 0,
    Opened = 1,
    HeadersReceived = 2,
    Loading = 3,
    XHRDone = 4, // So as not to conflict with the ProgressMsg `Done`
}

pub enum XHRProgress {
    /// Notify that headers have been received
    HeadersReceivedMsg(Option<ResponseHeaderCollection>, Status),
    /// Partial progress (after receiving headers), containing portion of the response
    LoadingMsg(ByteString),
    /// Loading is done
    DoneMsg,
    /// There was an error (Abort or Timeout). For a network or other error, just pass None
    ErroredMsg(Option<Error>),
    /// Timeout was reached
    TimeoutMsg
}

enum SyncOrAsync<'a> {
    Sync(JSRef<'a, XMLHttpRequest>),
    Async(TrustedXHRAddress, ScriptChan)
}


#[jstraceable]
#[must_root]
pub struct XMLHttpRequest {
    eventtarget: XMLHttpRequestEventTarget,
    ready_state: Cell<XMLHttpRequestState>,
    timeout: Cell<u32>,
    with_credentials: Cell<bool>,
    upload: JS<XMLHttpRequestUpload>,
    response_url: DOMString,
    status: Cell<u16>,
    status_text: RefCell<ByteString>,
    response: RefCell<ByteString>,
    response_type: Cell<XMLHttpRequestResponseType>,
    response_xml: MutNullableJS<Document>,
    response_headers: RefCell<ResponseHeaderCollection>,

    // Associated concepts
    request_method: RefCell<Method>,
    request_url: RefCell<Option<Url>>,
    request_headers: RefCell<RequestHeaderCollection>,
    request_body_len: Cell<uint>,
    sync: Cell<bool>,
    upload_complete: Cell<bool>,
    upload_events: Cell<bool>,
    send_flag: Cell<bool>,

    global: GlobalField,
    pinned_count: Cell<uint>,
    timer: RefCell<Timer>,
    fetch_time: Cell<i64>,
    timeout_pinned: Cell<bool>,
    terminate_sender: RefCell<Option<Sender<Error>>>,
}

impl XMLHttpRequest {
    fn new_inherited(global: &GlobalRef) -> XMLHttpRequest {
        XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestTypeId),
            ready_state: Cell::new(Unsent),
            timeout: Cell::new(0u32),
            with_credentials: Cell::new(false),
            upload: JS::from_rooted(XMLHttpRequestUpload::new(global)),
            response_url: "".to_string(),
            status: Cell::new(0),
            status_text: RefCell::new(ByteString::new(vec!())),
            response: RefCell::new(ByteString::new(vec!())),
            response_type: Cell::new(_empty),
            response_xml: Default::default(),
            response_headers: RefCell::new(ResponseHeaderCollection::new()),

            request_method: RefCell::new(Get),
            request_url: RefCell::new(None),
            request_headers: RefCell::new(RequestHeaderCollection::new()),
            request_body_len: Cell::new(0),
            sync: Cell::new(false),
            send_flag: Cell::new(false),

            upload_complete: Cell::new(false),
            upload_events: Cell::new(false),

            global: GlobalField::from_rooted(global),
            pinned_count: Cell::new(0),
            timer: RefCell::new(Timer::new().unwrap()),
            fetch_time: Cell::new(0),
            timeout_pinned: Cell::new(false),
            terminate_sender: RefCell::new(None),
        }
    }
    pub fn new(global: &GlobalRef) -> Temporary<XMLHttpRequest> {
        reflect_dom_object(box XMLHttpRequest::new_inherited(global),
                           global,
                           XMLHttpRequestBinding::Wrap)
    }
    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<XMLHttpRequest>> {
        Ok(XMLHttpRequest::new(global))
    }

    pub fn handle_xhr_progress(addr: TrustedXHRAddress, progress: XHRProgress) {
        unsafe {
            let xhr = JS::from_trusted_xhr_address(addr).root();
            xhr.deref().process_partial_response(progress);
        }
    }

    fn fetch(fetch_type: &SyncOrAsync, resource_task: ResourceTask,
             mut load_data: LoadData, terminate_receiver: Receiver<Error>,
             cors_request: Result<Option<CORSRequest>,()>) -> ErrorResult {
        fn notify_partial_progress(fetch_type: &SyncOrAsync, msg: XHRProgress) {
            match *fetch_type {
                Sync(xhr) => {
                    xhr.process_partial_response(msg);
                },
                Async(addr, ref script_chan) => {
                    let ScriptChan(ref chan) = *script_chan;
                    chan.send(XHRProgressMsg(addr, msg));
                }
            }
        }

        match cors_request {
            Err(_) => return Err(Network), // Happens in case of cross-origin non-http URIs
            Ok(Some(ref req)) => {
                let response = req.http_fetch();
                if response.network_error {
                    return Err(Network)
                } else {
                    load_data.cors = Some(ResourceCORSData {
                        preflight: req.preflight_flag,
                        origin: req.origin.clone()
                    })
                }
            },
            _ => {}
        }

        // Step 10, 13
        let (start_chan, start_port) = channel();
        resource_task.send(Load(load_data, start_chan));
        let response = start_port.recv();
        match terminate_receiver.try_recv() {
            Ok(e) => return Err(e),
            _ => {}
        }
        match cors_request {
            Ok(Some(ref req)) => {
                    match response.metadata.headers {
                        Some(ref h) if allow_cross_origin_request(req, h) => {},
                        _ => return Err(Network)
                    }
            },
            _ => {}
        }
        // XXXManishearth Clear cache entries in case of a network error

        notify_partial_progress(fetch_type, HeadersReceivedMsg(
            response.metadata.headers.clone(), response.metadata.status.clone()));
        let mut buf = vec!();
        loop {
            let progress = response.progress_port.recv();
            match terminate_receiver.try_recv() {
                Ok(e) => return Err(e),
                _ => {}
            }
            match progress {
                Payload(data) => {
                    buf.push_all(data.as_slice());
                    notify_partial_progress(fetch_type, LoadingMsg(ByteString::new(buf.clone())));
                },
                Done(Ok(()))  => {
                    notify_partial_progress(fetch_type, DoneMsg);
                    return Ok(());
                },
                Done(Err(_))  => {
                    notify_partial_progress(fetch_type, ErroredMsg(None));
                    return Err(Network)
                }
            }
        }
    }
}

impl<'a> XMLHttpRequestMethods for JSRef<'a, XMLHttpRequest> {
    fn GetOnreadystatechange(self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("readystatechange")
    }

    fn SetOnreadystatechange(self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("readystatechange", listener)
    }

    fn ReadyState(self) -> u16 {
        self.ready_state.get() as u16
    }

    fn Open(self, method: ByteString, url: DOMString) -> ErrorResult {
        // Clean up from previous requests, if any:
        self.cancel_timeout();
        let uppercase_method = method.as_str().map(|s| {
            let upper = s.to_ascii_upper();
            match upper.as_slice() {
                "DELETE" | "GET" | "HEAD" | "OPTIONS" |
                "POST" | "PUT" | "CONNECT" | "TRACE" |
                "TRACK" => upper,
                _ => s.to_string()
            }
        });
        let maybe_method: Option<Method> = uppercase_method.and_then(|s| {
            // Note: rust-http tests against the uppercase versions
            // Since we want to pass methods not belonging to the short list above
            // without changing capitalization, this will actually sidestep rust-http's type system
            // since methods like "patch" or "PaTcH" will be considered extension methods
            // despite the there being a rust-http method variant for them
            Method::from_str_or_new(s.as_slice())
        });
        // Step 2
        match maybe_method {
            // Step 4
            Some(Connect) | Some(Trace) => Err(Security),
            Some(ExtensionMethod(ref t)) if t.as_slice() == "TRACK" => Err(Security),
            Some(_) if method.is_token() => {

                *self.request_method.borrow_mut() = maybe_method.unwrap();

                // Step 6
                let base = self.global.root().root_ref().get_url();
                let parsed_url = match UrlParser::new().base_url(&base).parse(url.as_slice()) {
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
                // XXXManishearth abort existing requests
                // Step 12
                *self.request_url.borrow_mut() = Some(parsed_url);
                *self.request_headers.borrow_mut() = RequestHeaderCollection::new();
                self.send_flag.set(false);
                *self.status_text.borrow_mut() = ByteString::new(vec!());
                self.status.set(0);

                // Step 13
                if self.ready_state.get() != Opened {
                    self.change_ready_state(Opened);
                }
                Ok(())
            },
            // This includes cases where as_str() returns None, and when is_token() returns false,
            // both of which indicate invalid extension method names
            _ => Err(Syntax), // Step 3
        }
    }
    fn Open_(self, method: ByteString, url: DOMString, async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult {
        self.sync.set(!async);
        self.Open(method, url)
    }
    fn SetRequestHeader(self, name: ByteString, mut value: ByteString) -> ErrorResult {
        if self.ready_state.get() != Opened || self.send_flag.get() {
            return Err(InvalidState); // Step 1, 2
        }
        if !name.is_token() || !value.is_field_value() {
            return Err(Syntax); // Step 3, 4
        }
        let name_str = match name.to_lower().as_str() {
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
                    _ => String::from_str(s)
                }
            },
            None => return Err(Syntax)
        };
        let mut collection = self.request_headers.borrow_mut();


        // Steps 6,7
        let old_header = collection.iter().find(|ref h| -> bool {
            // XXXManishearth following line waiting on the rust upgrade:
            ByteString::new(h.header_name().into_bytes()).eq_ignore_case(&value)
        });
        match old_header {
            Some(h) => {
                unsafe {
                    // By step 4, the value is a subset of valid utf8
                    // So this unsafe block should never fail

                    let mut buf = h.header_value();
                    buf.as_mut_vec().push_all(&[0x2C, 0x20]);
                    buf.as_mut_vec().push_all(value.as_slice());
                    value = ByteString::new(buf.container_into_owned_bytes());

                }
            },
            None => {}
        }

        let mut reader = BufReader::new(value.as_slice());
        let maybe_header: Option<Header> = HeaderEnum::value_from_stream(
                                                            name_str,
                                                            &mut HeaderValueByteIterator::new(&mut reader));
        match maybe_header {
            Some(h) => {
                // Overwrites existing headers, which we want since we have
                // prepended the new header value with the old one already
                collection.insert(h);
                Ok(())
            },
            None => Err(Syntax)
        }
    }
    fn Timeout(self) -> u32 {
        self.timeout.get()
    }
    fn SetTimeout(self, timeout: u32) -> ErrorResult {
        if self.sync.get() {
            // FIXME: Not valid for a worker environment
            Err(InvalidState)
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
    fn WithCredentials(self) -> bool {
        self.with_credentials.get()
    }
    fn SetWithCredentials(self, with_credentials: bool) {
        self.with_credentials.set(with_credentials);
    }
    fn Upload(self) -> Temporary<XMLHttpRequestUpload> {
        Temporary::new(self.upload)
    }
    fn Send(self, data: Option<SendParam>) -> ErrorResult {
        if self.ready_state.get() != Opened || self.send_flag.get() {
            return Err(InvalidState); // Step 1, 2
        }

        let data = match *self.request_method.borrow() {
            Get | Head => None, // Step 3
            _ => data
        };
        let extracted = data.as_ref().map(|d| d.extract());
        self.request_body_len.set(extracted.as_ref().map(|e| e.len()).unwrap_or(0));

        // Step 6
        self.upload_events.set(false);
        // Step 7
        self.upload_complete.set(match extracted {
            None => true,
            Some (ref v) if v.len() == 0 => true,
            _ => false
        });
        let mut addr = None;
        if !self.sync.get() {
            // If one of the event handlers below aborts the fetch,
            // the assertion in release_once() will fail since we haven't pinned it yet.
            // Pin early to avoid dealing with this
            unsafe {
                addr = Some(self.to_trusted());
            }

            // Step 8
            let upload_target = *self.upload.root();
            let event_target: JSRef<EventTarget> = EventTargetCast::from_ref(upload_target);
            if event_target.has_handlers() {
                self.upload_events.set(true);
            }

            // Step 9
            self.send_flag.set(true);
            self.dispatch_response_progress_event("loadstart".to_string());
            if !self.upload_complete.get() {
                self.dispatch_upload_progress_event("loadstart".to_string(), Some(0));
            }
        }

        if self.ready_state.get() == Unsent {
            // The progress events above might have run abort(), in which case we terminate the fetch.
            return Ok(());
        }

        let global = self.global.root();
        let resource_task = global.root_ref().resource_task();
        let mut load_data = LoadData::new(self.request_url.borrow().clone().unwrap());
        load_data.data = extracted;

        // Default headers
        let ref request_headers = self.request_headers;
        if request_headers.borrow().content_type.is_none() {
            let parameters = vec!((String::from_str("charset"), String::from_str("UTF-8")));
            request_headers.borrow_mut().content_type = match data {
                Some(eString(_)) =>
                    Some(MediaType {
                        type_: String::from_str("text"),
                        subtype: String::from_str("plain"),
                        parameters: parameters
                    }),
                Some(eURLSearchParams(_)) =>
                    Some(MediaType {
                        type_: String::from_str("application"),
                        subtype: String::from_str("x-www-form-urlencoded"),
                        parameters: parameters
                    }),
                None => None
            }
        }

        if request_headers.borrow().accept.is_none() {
            request_headers.borrow_mut().accept = Some(String::from_str("*/*"))
        }

        load_data.headers = (*self.request_headers.borrow()).clone();
        load_data.method = (*self.request_method.borrow()).clone();
        let (terminate_sender, terminate_receiver) = channel();
        *self.terminate_sender.borrow_mut() = Some(terminate_sender);

        // CORS stuff
        let referer_url = self.global.root().root_ref().get_url();
        let mode = if self.upload_events.get() {
            ForcedPreflightMode
        } else {
            CORSMode
        };
        let cors_request = CORSRequest::maybe_new(referer_url.clone(), load_data.url.clone(), mode,
                                                  load_data.method.clone(), load_data.headers.clone());
        match cors_request {
            Ok(None) => {
                let mut buf = String::new();
                buf.push_str(referer_url.scheme.as_slice());
                buf.push_str("://".as_slice());
                referer_url.serialize_host().map(|ref h| buf.push_str(h.as_slice()));
                referer_url.port().as_ref().map(|&p| {
                    buf.push_str(":".as_slice());
                    buf.push_str(format!("{:u}", p).as_slice());
                });
                referer_url.serialize_path().map(|ref h| buf.push_str(h.as_slice()));
                self.request_headers.borrow_mut().referer = Some(buf);
            },
            Ok(Some(ref req)) => self.insert_trusted_header("origin".to_string(),
                                                            format!("{}", req.origin)),
            _ => {}
        }

        if self.sync.get() {
            return XMLHttpRequest::fetch(&mut Sync(self), resource_task, load_data,
                                         terminate_receiver, cors_request);
        } else {
            let builder = TaskBuilder::new().named("XHRTask");
            self.fetch_time.set(time::now().to_timespec().sec);
            let script_chan = global.root_ref().script_chan().clone();
            builder.spawn(proc() {
                let _ = XMLHttpRequest::fetch(&mut Async(addr.unwrap(), script_chan),
                                              resource_task, load_data, terminate_receiver, cors_request);
            });
            let timeout = self.timeout.get();
            if timeout > 0 {
                self.set_timeout(timeout);
            }
        }
        Ok(())
    }
    fn Abort(self) {
        self.terminate_sender.borrow().as_ref().map(|s| s.send_opt(Abort));
        match self.ready_state.get() {
            Opened if self.send_flag.get() => self.process_partial_response(ErroredMsg(Some(Abort))),
            HeadersReceived | Loading => self.process_partial_response(ErroredMsg(Some(Abort))),
            _ => {}
        };
        self.ready_state.set(Unsent);
    }
    fn ResponseURL(self) -> DOMString {
        self.response_url.clone()
    }
    fn Status(self) -> u16 {
        self.status.get()
    }
    fn StatusText(self) -> ByteString {
        self.status_text.borrow().clone()
    }
    fn GetResponseHeader(self, name: ByteString) -> Option<ByteString> {
        self.filter_response_headers().iter().find(|h| {
            name.eq_ignore_case(&FromStr::from_str(h.header_name().as_slice()).unwrap())
        }).map(|h| {
            // rust-http doesn't decode properly, we'll convert it back to bytes here
            ByteString::new(h.header_value().as_slice().chars().map(|c| { assert!(c <= '\u00FF'); c as u8 }).collect())
        })
    }
    fn GetAllResponseHeaders(self) -> ByteString {
        let mut writer = MemWriter::new();
        self.filter_response_headers().write_all(&mut writer).ok().expect("Writing response headers failed");
        let mut vec = writer.unwrap();

        // rust-http appends an extra "\r\n" when using write_all
        vec.pop();
        vec.pop();

        ByteString::new(vec)
    }
    fn ResponseType(self) -> XMLHttpRequestResponseType {
        self.response_type.get()
    }
    fn SetResponseType(self, response_type: XMLHttpRequestResponseType) -> ErrorResult {
        match self.global {
            WorkerField(_) if response_type == XMLHttpRequestResponseTypeValues::Document
            => return Ok(()),
            _ => {}
        }
        match self.ready_state.get() {
            Loading | XHRDone => Err(InvalidState),
            _ if self.sync.get() => Err(InvalidAccess),
            _ => {
                self.response_type.set(response_type);
                Ok(())
            }
        }
    }
    fn Response(self, cx: *mut JSContext) -> JSVal {
         match self.response_type.get() {
            _empty | Text => {
                let ready_state = self.ready_state.get();
                if ready_state == XHRDone || ready_state == Loading {
                    self.text_response().to_jsval(cx)
                } else {
                    "".to_string().to_jsval(cx)
                }
            },
            _ if self.ready_state.get() != XHRDone => NullValue(),
            Json => {
                let decoded = UTF_8.decode(self.response.borrow().as_slice(), DecodeReplace).unwrap().to_string();
                let decoded: Vec<u16> = decoded.as_slice().utf16_units().collect();
                let mut vp = UndefinedValue();
                unsafe {
                    if JS_ParseJSON(cx, decoded.as_ptr(), decoded.len() as u32, &mut vp) == 0 {
                        JS_ClearPendingException(cx);
                        return NullValue();
                    }
                }
                vp
            }
            _ => {
                // XXXManishearth handle other response types
                self.response.borrow().to_jsval(cx)
            }
        }
    }
    fn GetResponseText(self) -> Fallible<DOMString> {
        match self.response_type.get() {
            _empty | Text => {
                match self.ready_state.get() {
                    Loading | XHRDone => Ok(self.text_response()),
                    _ => Ok("".to_string())
                }
            },
            _ => Err(InvalidState)
        }
    }
    fn GetResponseXML(self) -> Option<Temporary<Document>> {
        self.response_xml.get()
    }
}

impl Reflectable for XMLHttpRequest {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}

impl XMLHttpRequestDerived for EventTarget {
    fn is_xmlhttprequest(&self) -> bool {
        match self.type_id {
            XMLHttpRequestTargetTypeId(XMLHttpRequestTypeId) => true,
            _ => false
        }
    }
}

pub struct TrustedXHRAddress(pub *const c_void);

impl TrustedXHRAddress {
    pub fn release_once(self) {
        unsafe {
            JS::from_trusted_xhr_address(self).root().release_once();
        }
    }
}


trait PrivateXMLHttpRequestHelpers {
    unsafe fn to_trusted(self) -> TrustedXHRAddress;
    fn release_once(self);
    fn change_ready_state(self, XMLHttpRequestState);
    fn process_partial_response(self, progress: XHRProgress);
    fn insert_trusted_header(self, name: String, value: String);
    fn dispatch_progress_event(self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>);
    fn dispatch_upload_progress_event(self, type_: DOMString, partial_load: Option<u64>);
    fn dispatch_response_progress_event(self, type_: DOMString);
    fn text_response(self) -> DOMString;
    fn set_timeout(self, timeout:u32);
    fn cancel_timeout(self);
    fn filter_response_headers(self) -> ResponseHeaderCollection;
}

impl<'a> PrivateXMLHttpRequestHelpers for JSRef<'a, XMLHttpRequest> {
    // Creates a trusted address to the object, and roots it. Always pair this with a release()
    unsafe fn to_trusted(self) -> TrustedXHRAddress {
        if self.pinned_count.get() == 0 {
            JS_AddObjectRoot(self.global.root().root_ref().get_cx(), self.reflector().rootable());
        }
        let pinned_count = self.pinned_count.get();
        self.pinned_count.set(pinned_count + 1);
        TrustedXHRAddress(self.deref() as *const XMLHttpRequest as *const libc::c_void)
    }

    fn release_once(self) {
        if self.sync.get() {
            // Lets us call this at various termination cases without having to
            // check self.sync every time, since the pinning mechanism only is
            // meaningful during an async fetch
            return;
        }
        assert!(self.pinned_count.get() > 0)
        let pinned_count = self.pinned_count.get();
        self.pinned_count.set(pinned_count - 1);
        if self.pinned_count.get() == 0 {
            unsafe {
                JS_RemoveObjectRoot(self.global.root().root_ref().get_cx(), self.reflector().rootable());
            }
        }
    }

    fn change_ready_state(self, rs: XMLHttpRequestState) {
        assert!(self.ready_state.get() != rs)
        self.ready_state.set(rs);
        let global = self.global.root();
        let event = Event::new(&global.root_ref(),
                               "readystatechange".to_string(),
                               false, true).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        target.dispatch_event_with_target(None, *event).ok();
    }

    fn process_partial_response(self, progress: XHRProgress) {
        match progress {
            HeadersReceivedMsg(headers, status) => {
                // For synchronous requests, this should not fire any events, and just store data
                // XXXManishearth Find a way to track partial progress of the send (onprogresss for XHRUpload)

                // Part of step 13, send() (processing request end of file)
                // Substep 1
                self.upload_complete.set(true);
                // Substeps 2-4
                if !self.sync.get() {
                    self.dispatch_upload_progress_event("progress".to_string(), None);
                    self.dispatch_upload_progress_event("load".to_string(), None);
                    self.dispatch_upload_progress_event("loadend".to_string(), None);
                }
                // Part of step 13, send() (processing response)
                // XXXManishearth handle errors, if any (substep 1)
                // Substep 2
                *self.status_text.borrow_mut() = ByteString::new(status.reason().container_into_owned_bytes());
                self.status.set(status.code());
                match headers {
                    Some(ref h) => {
                        *self.response_headers.borrow_mut() = h.clone();
                    }
                    None => {}
                };
                // Substep 3
                if self.ready_state.get() == Opened && !self.sync.get() {
                    self.change_ready_state(HeadersReceived);
                }
            },
            LoadingMsg(partial_response) => {
                // For synchronous requests, this should not fire any events, and just store data
                // Part of step 13, send() (processing response body)
                // XXXManishearth handle errors, if any (substep 1)

                // Substep 2
                if self.ready_state.get() == HeadersReceived && !self.sync.get() {
                    self.change_ready_state(Loading);
                }
                // Substep 3
                *self.response.borrow_mut() = partial_response;
                // Substep 4
                if !self.sync.get() {
                    self.dispatch_response_progress_event("progress".to_string());
                }
            },
            DoneMsg => {
                // Part of step 13, send() (processing response end of file)
                // XXXManishearth handle errors, if any (substep 1)

                // Substep 3
                if self.ready_state.get() == Loading || self.sync.get() {
                    // Subsubsteps 2-4
                    self.send_flag.set(false);
                    self.change_ready_state(XHRDone);

                    // Subsubsteps 5-7
                    self.dispatch_response_progress_event("progress".to_string());
                    self.dispatch_response_progress_event("load".to_string());
                    self.dispatch_response_progress_event("loadend".to_string());
                }
                self.cancel_timeout();
                self.release_once();
            },
            ErroredMsg(e) => {
                self.send_flag.set(false);
                // XXXManishearth set response to NetworkError
                self.change_ready_state(XHRDone);
                let errormsg = match e {
                    Some(Abort) => "abort",
                    Some(Timeout) => "timeout",
                    None => "error",
                    _ => unreachable!()
                };

                let upload_complete: &Cell<bool> = &self.upload_complete;
                if !upload_complete.get() {
                    upload_complete.set(true);
                    self.dispatch_upload_progress_event("progress".to_string(), None);
                    self.dispatch_upload_progress_event(errormsg.to_string(), None);
                    self.dispatch_upload_progress_event("loadend".to_string(), None);
                }
                self.dispatch_response_progress_event("progress".to_string());
                self.dispatch_response_progress_event(errormsg.to_string());
                self.dispatch_response_progress_event("loadend".to_string());

                self.cancel_timeout();
                self.release_once();
            },
            TimeoutMsg => {
                match self.ready_state.get() {
                    Opened if self.send_flag.get() => self.process_partial_response(ErroredMsg(Some(Timeout))),
                    Loading | HeadersReceived => self.process_partial_response(ErroredMsg(Some(Timeout))),
                    _ => self.release_once()
                };
            }
        }
    }

    fn insert_trusted_header(self, name: String, value: String) {
        // Insert a header without checking spec-compliance
        // Use for hardcoded headers
        let mut collection = self.request_headers.borrow_mut();
        let value_bytes = value.into_bytes();
        let mut reader = BufReader::new(value_bytes.as_slice());
        let maybe_header: Option<Header> = HeaderEnum::value_from_stream(
                                                                String::from_str(name.as_slice()),
                                                                &mut HeaderValueByteIterator::new(&mut reader));
        collection.insert(maybe_header.unwrap());
    }

    fn dispatch_progress_event(self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>) {
        let global = self.global.root();
        let upload_target = *self.upload.root();
        let progressevent = ProgressEvent::new(&global.root_ref(),
                                               type_, false, false,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0)).root();
        let target: JSRef<EventTarget> = if upload {
            EventTargetCast::from_ref(upload_target)
        } else {
            EventTargetCast::from_ref(self)
        };
        let event: JSRef<Event> = EventCast::from_ref(*progressevent);
        target.dispatch_event_with_target(None, event).ok();
    }

    fn dispatch_upload_progress_event(self, type_: DOMString, partial_load: Option<u64>) {
        // If partial_load is None, loading has completed and we can just use the value from the request body

        let total = self.request_body_len.get() as u64;
        self.dispatch_progress_event(true, type_, partial_load.unwrap_or(total), Some(total));
    }

    fn dispatch_response_progress_event(self, type_: DOMString) {
        let len = self.response.borrow().len() as u64;
        let total = self.response_headers.borrow().content_length.map(|x| {x as u64});
        self.dispatch_progress_event(false, type_, len, total);
    }
    fn set_timeout(self, timeout: u32) {
        // Sets up the object to timeout in a given number of milliseconds
        // This will cancel all previous timeouts
        let oneshot = self.timer.borrow_mut()
                          .oneshot(Duration::milliseconds(timeout as i64));
        let addr = unsafe {
            self.to_trusted() // This will increment the pin counter by one
        };
        if self.timeout_pinned.get() {
            // Already pinned due to a timeout, no need to pin it again since the old timeout was cancelled above
            self.release_once();
        }
        self.timeout_pinned.set(true);
        let global = self.global.root();
        let script_chan = global.root_ref().script_chan().clone();
        let terminate_sender = (*self.terminate_sender.borrow()).clone();
        spawn_named("XHR:Timer", proc () {
            match oneshot.recv_opt() {
                Ok(_) => {
                    let ScriptChan(ref chan) = script_chan;
                    terminate_sender.map(|s| s.send_opt(Timeout));
                    chan.send(XHRProgressMsg(addr, TimeoutMsg));
                },
                Err(_) => {
                    // This occurs if xhr.timeout (the sender) goes out of scope (i.e, xhr went out of scope)
                    // or if the oneshot timer was overwritten. The former case should not happen due to pinning.
                    debug!("XHR timeout was overwritten or canceled")
                }
            }
        }
    );
    }
    fn cancel_timeout(self) {
        // Cancels timeouts on the object, if any
        if self.timeout_pinned.get() {
            self.timeout_pinned.set(false);
            self.release_once();
        }
        // oneshot() closes the previous channel, canceling the timeout
        self.timer.borrow_mut().oneshot(Zero::zero());
    }
    fn text_response(self) -> DOMString {
        let mut encoding = UTF_8 as EncodingRef;
        match self.response_headers.borrow().content_type {
            Some(ref x) => {
                for &(ref name, ref value) in x.parameters.iter() {
                    if name.as_slice().eq_ignore_ascii_case("charset") {
                        encoding = encoding_from_whatwg_label(value.as_slice()).unwrap_or(encoding);
                    }
                }
            },
            None => {}
        }
        // According to Simon, decode() should never return an error, so unwrap()ing
        // the result should be fine. XXXManishearth have a closer look at this later
        encoding.decode(self.response.borrow().as_slice(), DecodeReplace).unwrap().to_string()
    }
    fn filter_response_headers(self) -> ResponseHeaderCollection {
        // http://fetch.spec.whatwg.org/#concept-response-header-list
        let mut headers = ResponseHeaderCollection::new();
        for header in self.response_headers.borrow().iter() {
            match header.header_name().as_slice().to_ascii_lower().as_slice() {
                "set-cookie" | "set-cookie2" => {},
                // XXXManishearth additional CORS filtering goes here
                _ => headers.insert(header)
            };
        }
        headers
    }
}

trait Extractable {
    fn extract(&self) -> Vec<u8>;
}
impl Extractable for SendParam {
    fn extract(&self) -> Vec<u8> {
        // http://fetch.spec.whatwg.org/#concept-fetchbodyinit-extract
        let encoding = UTF_8 as EncodingRef;
        match *self {
            eString(ref s) => encoding.encode(s.as_slice(), EncodeReplace).unwrap(),
            eURLSearchParams(ref usp) => usp.root().serialize(None) // Default encoding is UTF8
        }
    }
}
