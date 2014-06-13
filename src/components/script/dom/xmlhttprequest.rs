/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseType;
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::XMLHttpRequestResponseTypeValues::{_empty, Json, Text};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast, XMLHttpRequestDerived};
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::{ErrorResult, Fallible, InvalidState, InvalidAccess, Network, Syntax, Security};
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable, OptionalRootedRootable};
use dom::bindings::str::ByteString;
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::Document;
use dom::event::Event;
use dom::eventtarget::{EventTarget, EventTargetHelpers, XMLHttpRequestTargetTypeId};
use dom::progressevent::ProgressEvent;
use dom::window::Window;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequestupload::XMLHttpRequestUpload;

use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecodeReplace, Encoding};

use ResponseHeaderCollection = http::headers::response::HeaderCollection;
use RequestHeaderCollection = http::headers::request::HeaderCollection;
use http::headers::content_type::MediaType;
use http::headers::{HeaderEnum, HeaderValueByteIterator};
use http::headers::request::Header;
use http::method::{Method, Get, Head, Connect, Trace, ExtensionMethod};
use http::status::Status;

use js::jsapi::{JS_AddObjectRoot, JS_ParseJSON, JS_RemoveObjectRoot, JSContext};
use js::jsval::{JSVal, NullValue, UndefinedValue};

use libc;
use libc::c_void;

use net::resource_task::{ResourceTask, Load, LoadData, Payload, Done};
use script_task::{ScriptChan, XHRProgressMsg};
use servo_util::str::DOMString;
use servo_util::url::{parse_url, try_parse_url};

use std::ascii::StrAsciiExt;
use std::cell::Cell;
use std::comm::channel;
use std::io::{BufReader, MemWriter};
use std::from_str::FromStr;
use std::path::BytesContainer;
use std::task::TaskBuilder;

use url::Url;

// As send() start accepting more and more parameter types,
// change this to the appropriate type from UnionTypes, eg
// use SendParam = dom::bindings::codegen::UnionTypes::StringOrFormData;
pub type SendParam = DOMString;

#[deriving(Eq,Encodable)]
pub enum XMLHttpRequestId {
    XMLHttpRequestTypeId,
    XMLHttpRequestUploadTypeId
}

#[deriving(Eq, Encodable)]
enum XMLHttpRequestState {
    Unsent = 0u16,
    Opened = 1u16,
    HeadersReceived = 2u16,
    Loading = 3u16,
    XHRDone = 4u16, // So as not to conflict with the ProgressMsg `Done`
}

pub enum XHRProgress {
    /// Notify that headers have been received
    HeadersReceivedMsg(Option<ResponseHeaderCollection>, Status),
    /// Partial progress (after receiving headers), containing portion of the response
    LoadingMsg(ByteString),
    /// Loading is done
    DoneMsg,
    /// There was an error
    ErroredMsg,
    /// Release the pinned XHR object.
    ReleaseMsg,
}

enum SyncOrAsync<'a, 'b> {
    Sync(&'b mut JSRef<'a, XMLHttpRequest>),
    Async(TrustedXHRAddress, ScriptChan)
}

impl<'a,'b> SyncOrAsync<'a,'b> {
    fn is_async(&self) -> bool {
        match *self {
            Async(_,_) => true,
            _ => false
        }
    }
}
#[deriving(Encodable)]
pub struct XMLHttpRequest {
    eventtarget: XMLHttpRequestEventTarget,
    ready_state: XMLHttpRequestState,
    timeout: u32,
    with_credentials: bool,
    upload: Cell<Option<JS<XMLHttpRequestUpload>>>,
    response_url: DOMString,
    status: u16,
    status_text: ByteString,
    response: ByteString,
    response_type: XMLHttpRequestResponseType,
    response_xml: Cell<Option<JS<Document>>>,
    response_headers: Untraceable<ResponseHeaderCollection>,

    // Associated concepts
    request_method: Untraceable<Method>,
    request_url: Untraceable<Url>,
    request_headers: Untraceable<RequestHeaderCollection>,
    request_body: SendParam,
    sync: bool,
    upload_complete: bool,
    upload_events: bool,
    send_flag: bool,

    global: JS<Window>,
    pinned: bool,
}

impl XMLHttpRequest {
    pub fn new_inherited(owner: &JSRef<Window>) -> XMLHttpRequest {
        let xhr = XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestTypeId),
            ready_state: Unsent,
            timeout: 0u32,
            with_credentials: false,
            upload: Cell::new(None),
            response_url: "".to_string(),
            status: 0,
            status_text: ByteString::new(vec!()),
            response: ByteString::new(vec!()),
            response_type: _empty,
            response_xml: Cell::new(None),
            response_headers: Untraceable::new(ResponseHeaderCollection::new()),

            request_method: Untraceable::new(Get),
            request_url: Untraceable::new(parse_url("", None)),
            request_headers: Untraceable::new(RequestHeaderCollection::new()),
            request_body: "".to_string(),
            sync: false,
            send_flag: false,

            upload_complete: false,
            upload_events: false,

            global: owner.unrooted(),
            pinned: false,
        };
        xhr.upload.assign(Some(XMLHttpRequestUpload::new(owner)));
        xhr
    }
    pub fn new(window: &JSRef<Window>) -> Temporary<XMLHttpRequest> {
        reflect_dom_object(box XMLHttpRequest::new_inherited(window),
                           window,
                           XMLHttpRequestBinding::Wrap)
    }
    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<Temporary<XMLHttpRequest>> {
        Ok(XMLHttpRequest::new(owner))
    }

    pub fn handle_xhr_progress(addr: TrustedXHRAddress, progress: XHRProgress) {
        unsafe {
            let mut xhr = JS::from_trusted_xhr_address(addr).root();
            xhr.process_partial_response(progress);
        }
    }

    fn fetch(fetch_type: &mut SyncOrAsync, resource_task: ResourceTask, load_data: LoadData) -> ErrorResult {

        fn notify_partial_progress(fetch_type: &mut SyncOrAsync, msg: XHRProgress) {
            match *fetch_type {
                Sync(ref mut xhr) => {
                    xhr.process_partial_response(msg);
                },
                Async(addr, ref script_chan) => {
                    let ScriptChan(ref chan) = *script_chan;
                    chan.send(XHRProgressMsg(addr, msg));
                }
            }
        }

        // Step 10, 13
        let (start_chan, start_port) = channel();
        resource_task.send(Load(load_data, start_chan));
        let response = start_port.recv();
        notify_partial_progress(fetch_type, HeadersReceivedMsg(
            response.metadata.headers.clone(), response.metadata.status.clone()));
        let mut buf = vec!();
        loop {
            match response.progress_port.recv() {
                Payload(data) => {
                    buf.push_all(data.as_slice());
                    notify_partial_progress(fetch_type, LoadingMsg(ByteString::new(buf.clone())));
                },
                Done(Ok(()))  => {
                    notify_partial_progress(fetch_type, DoneMsg);
                    if fetch_type.is_async() {
                        notify_partial_progress(fetch_type, ReleaseMsg)
                    }
                    return Ok(());
                },
                Done(Err(_))  => {
                    notify_partial_progress(fetch_type, ErroredMsg);
                    if fetch_type.is_async() {
                        notify_partial_progress(fetch_type, ReleaseMsg)
                    }
                    return Err(Network)
                }
            }
        }
    }
}

pub trait XMLHttpRequestMethods<'a> {
    fn GetOnreadystatechange(&self) -> Option<EventHandlerNonNull>;
    fn SetOnreadystatechange(&self, listener: Option<EventHandlerNonNull>);
    fn ReadyState(&self) -> u16;
    fn Open(&mut self, _method: ByteString, _url: DOMString) -> ErrorResult;
    fn Open_(&mut self, _method: ByteString, _url: DOMString, _async: bool,
             _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult;
    fn SetRequestHeader(&mut self, name: ByteString, mut value: ByteString) -> ErrorResult;
    fn Timeout(&self) -> u32;
    fn SetTimeout(&mut self, timeout: u32);
    fn WithCredentials(&self) -> bool;
    fn SetWithCredentials(&mut self, with_credentials: bool);
    fn Upload(&self) -> Temporary<XMLHttpRequestUpload>;
    fn Send(&mut self, _data: Option<SendParam>) -> ErrorResult;
    fn Abort(&self);
    fn ResponseURL(&self) -> DOMString;
    fn Status(&self) -> u16;
    fn StatusText(&self) -> ByteString;
    fn GetResponseHeader(&self, name: ByteString) -> Option<ByteString>;
    fn GetAllResponseHeaders(&self) -> ByteString;
    fn OverrideMimeType(&self, _mime: DOMString);
    fn ResponseType(&self) -> XMLHttpRequestResponseType;
    fn SetResponseType(&mut self, response_type: XMLHttpRequestResponseType) -> ErrorResult;
    fn Response(&self, _cx: *mut JSContext) -> JSVal;
    fn GetResponseText(&self) -> Fallible<DOMString>;
    fn GetResponseXML(&self) -> Option<Temporary<Document>>;
}

impl<'a> XMLHttpRequestMethods<'a> for JSRef<'a, XMLHttpRequest> {
    fn GetOnreadystatechange(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.get_event_handler_common("readystatechange")
    }

    fn SetOnreadystatechange(&self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.set_event_handler_common("readystatechange", listener)
    }

    fn ReadyState(&self) -> u16 {
        self.ready_state as u16
    }

    fn Open(&mut self, method: ByteString, url: DOMString) -> ErrorResult {
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
        let base: Option<Url> = Some(self.global.root().get_url());
        match maybe_method {
            // Step 4
            Some(Connect) | Some(Trace) => Err(Security),
            Some(ExtensionMethod(ref t)) if t.as_slice() == "TRACK" => Err(Security),
            Some(_) if method.is_token() => {

                *self.request_method = maybe_method.unwrap();

                // Step 6
                let parsed_url = match try_parse_url(url.as_slice(), base) {
                    Ok(parsed) => parsed,
                    Err(_) => return Err(Syntax) // Step 7
                };
                // XXXManishearth Do some handling of username/passwords
                if self.sync {
                    // FIXME: This should only happen if the global environment is a document environment
                    if self.timeout != 0 || self.with_credentials || self.response_type != _empty {
                        return Err(InvalidAccess)
                    }
                }
                // XXXManishearth abort existing requests
                // Step 12
                *self.request_url = parsed_url;
                *self.request_headers = RequestHeaderCollection::new();
                self.send_flag = false;
                self.status_text = ByteString::new(vec!());
                self.status = 0;

                // Step 13
                if self.ready_state != Opened {
                    self.change_ready_state(Opened);
                }
                Ok(())
            },
            // This includes cases where as_str() returns None, and when is_token() returns false,
            // both of which indicate invalid extension method names
            _ => Err(Syntax), // Step 3
        }
    }
    fn Open_(&mut self, method: ByteString, url: DOMString, async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult {
        self.sync = !async;
        self.Open(method, url)
    }
    fn SetRequestHeader(&mut self, name: ByteString, mut value: ByteString) -> ErrorResult {
        if self.ready_state != Opened || self.send_flag {
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
        let collection = self.request_headers.deref_mut();


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
                    buf.push_bytes(&[0x2C, 0x20]);
                    buf.push_bytes(value.as_slice());
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
    fn Timeout(&self) -> u32 {
        self.timeout
    }
    fn SetTimeout(&mut self, timeout: u32) {
        self.timeout = timeout
    }
    fn WithCredentials(&self) -> bool {
        self.with_credentials
    }
    fn SetWithCredentials(&mut self, with_credentials: bool) {
        self.with_credentials = with_credentials
    }
    fn Upload(&self) -> Temporary<XMLHttpRequestUpload> {
        Temporary::new(self.upload.get().get_ref().clone())
    }
    fn Send(&mut self, data: Option<DOMString>) -> ErrorResult {
        if self.ready_state != Opened || self.send_flag {
            return Err(InvalidState); // Step 1, 2
        }

        let data = match *self.request_method {
            Get | Head => None, // Step 3
            _ => data
        };

        // Step 6
        self.upload_events = false;
        // Step 7
        self.upload_complete = match data {
            None => true,
            Some (ref s) if s.len() == 0 => true,
            _ => false
        };
        if !self.sync {
            // Step 8
            let upload_target = &*self.upload.get().root().unwrap();
            let event_target: &JSRef<EventTarget> = EventTargetCast::from_ref(upload_target);
            if event_target.has_handlers() {
                self.upload_events = true;
            }

            // Step 9
            self.send_flag = true;
            self.dispatch_response_progress_event("loadstart".to_string());
            if !self.upload_complete {
                self.dispatch_upload_progress_event("loadstart".to_string(), Some(0));
            }
        }

        let mut global = self.global.root();
        let resource_task = global.page().resource_task.deref().clone();
        let mut load_data = LoadData::new((*self.request_url).clone());
        load_data.data = data;

        // Default headers

        if self.request_headers.content_type.is_none() {
            self.request_headers.content_type = Some(MediaType {
                type_: String::from_str("text"),
                subtype: String::from_str("plain"),
                parameters: vec!((String::from_str("charset"), String::from_str("UTF-8")))
            });
        }

        if self.request_headers.accept.is_none() {
            self.request_headers.accept = Some(String::from_str("*/*"))
        }

        // XXXManishearth this is to be replaced with Origin for CORS (with no path)
        let referer_url = self.global.root().get_url();
        let mut buf = String::new();
        buf.push_str(referer_url.scheme.as_slice());
        buf.push_str("://".as_slice());
        buf.push_str(referer_url.host.as_slice());
        referer_url.port.as_ref().map(|p| {
            buf.push_str(":".as_slice());
            buf.push_str(p.as_slice());
        });
        buf.push_str(referer_url.path.as_slice());
        self.request_headers.referer = Some(buf);

        load_data.headers = (*self.request_headers).clone();
        load_data.method = (*self.request_method).clone();
        if self.sync {
            return XMLHttpRequest::fetch(&mut Sync(self), resource_task, load_data);
        } else {
            let builder = TaskBuilder::new().named("XHRTask");
            unsafe {
                let addr = self.to_trusted();
                let script_chan = global.script_chan.clone();
                builder.spawn(proc() {
                    let _ = XMLHttpRequest::fetch(&mut Async(addr, script_chan), resource_task, load_data);
                })
            }
        }
        Ok(())
    }
    fn Abort(&self) {

    }
    fn ResponseURL(&self) -> DOMString {
        self.response_url.clone()
    }
    fn Status(&self) -> u16 {
        self.status
    }
    fn StatusText(&self) -> ByteString {
        self.status_text.clone()
    }
    fn GetResponseHeader(&self, name: ByteString) -> Option<ByteString> {
        self.response_headers.deref().iter().find(|h| {
            name.eq_ignore_case(&FromStr::from_str(h.header_name().as_slice()).unwrap())
        }).map(|h| {
            FromStr::from_str(h.header_value().as_slice()).unwrap()
        })
    }
    fn GetAllResponseHeaders(&self) -> ByteString {
        let mut writer = MemWriter::new();
        self.response_headers.deref().write_all(&mut writer).ok().expect("Writing response headers failed");
        ByteString::new(writer.unwrap())
    }
    fn OverrideMimeType(&self, _mime: DOMString) {

    }
    fn ResponseType(&self) -> XMLHttpRequestResponseType {
        self.response_type
    }
    fn SetResponseType(&mut self, response_type: XMLHttpRequestResponseType) -> ErrorResult {
        if self.sync {
            // FIXME: When Workers are implemented, there should be
            // an additional check that this is a document environment
            return Err(InvalidState);
        }
        match self.ready_state {
            Loading | XHRDone => Err(InvalidState),
            _ => {
                self.response_type = response_type;
                Ok(())
            }
        }
    }
    fn Response(&self, cx: *mut JSContext) -> JSVal {
         match self.response_type {
            _empty | Text => {
                if self.ready_state == XHRDone || self.ready_state == Loading {
                    self.text_response().to_jsval(cx)
                } else {
                    "".to_string().to_jsval(cx)
                }
            },
            _ if self.ready_state != XHRDone => NullValue(),
            Json => {
                let decoded = UTF_8.decode(self.response.as_slice(), DecodeReplace).unwrap().to_string().to_utf16();
                let mut vp = UndefinedValue();
                unsafe {
                    JS_ParseJSON(cx, decoded.as_ptr(), decoded.len() as u32, &mut vp);
                }
                vp
            }
            _ => {
                // XXXManishearth handle other response types
                self.response.to_jsval(cx)
            }
        }
    }
    fn GetResponseText(&self) -> Fallible<DOMString> {
        match self.response_type {
            _empty | Text => {
                match self.ready_state {
                    Loading | XHRDone => Ok(self.text_response()),
                    _ => Ok("".to_string())
                }
            },
            _ => Err(InvalidState)
        }
    }
    fn GetResponseXML(&self) -> Option<Temporary<Document>> {
        self.response_xml.get().map(|response| Temporary::new(response))
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

pub struct TrustedXHRAddress(pub *c_void);

impl TrustedXHRAddress {
    pub fn release(self) {
        unsafe {
            JS::from_trusted_xhr_address(self).root().release();
        }
    }
}


trait PrivateXMLHttpRequestHelpers {
    unsafe fn to_trusted(&mut self) -> TrustedXHRAddress;
    fn release(&mut self);
    fn change_ready_state(&mut self, XMLHttpRequestState);
    fn process_partial_response(&mut self, progress: XHRProgress);
    fn insert_trusted_header(&mut self, name: String, value: String);
    fn dispatch_progress_event(&self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>);
    fn dispatch_upload_progress_event(&self, type_: DOMString, partial_load: Option<u64>);
    fn dispatch_response_progress_event(&self, type_: DOMString);
    fn text_response(&self) -> DOMString;
}

impl<'a> PrivateXMLHttpRequestHelpers for JSRef<'a, XMLHttpRequest> {
    // Creates a trusted address to the object, and roots it. Always pair this with a release()
    unsafe fn to_trusted(&mut self) -> TrustedXHRAddress {
        assert!(self.pinned == false);
        self.pinned = true;
        JS_AddObjectRoot(self.global.root().get_cx(), self.reflector().rootable());
        TrustedXHRAddress(self.deref() as *XMLHttpRequest as *libc::c_void)
    }

    fn release(&mut self) {
        assert!(self.pinned);
        unsafe {
            JS_RemoveObjectRoot(self.global.root().get_cx(), self.reflector().rootable());
        }
        self.pinned = false;
    }

    fn change_ready_state(&mut self, rs: XMLHttpRequestState) {
        assert!(self.ready_state != rs)
        self.ready_state = rs;
        let win = &*self.global.root();
        let event =
            Event::new(win, "readystatechange".to_string(), false, true).root();
        let target: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        target.dispatch_event_with_target(None, &*event).ok();
    }

    fn process_partial_response(&mut self, progress: XHRProgress) {
        match progress {
            HeadersReceivedMsg(headers, status) => {
                // XXXManishearth Find a way to track partial progress of the send (onprogresss for XHRUpload)

                // Part of step 13, send() (processing request end of file)
                // Substep 1
                self.upload_complete = true;
                // Substeps 2-4
                self.dispatch_upload_progress_event("progress".to_string(), None);
                self.dispatch_upload_progress_event("load".to_string(), None);
                self.dispatch_upload_progress_event("loadend".to_string(), None);

                // Part of step 13, send() (processing response)
                // XXXManishearth handle errors, if any (substep 1)
                // Substep 2
                self.status_text = ByteString::new(status.reason().container_into_owned_bytes());
                self.status = status.code();
                match headers {
                    Some(ref h) => *self.response_headers = h.clone(),
                    None => {}
                };
                // Substep 3
                if self.ready_state == Opened {
                    self.change_ready_state(HeadersReceived);
                }
            },
            LoadingMsg(partial_response) => {
                // Part of step 13, send() (processing response body)
                // XXXManishearth handle errors, if any (substep 1)

                // Substep 2
                if self.ready_state == HeadersReceived {
                    self.change_ready_state(Loading);
                }
                // Substep 3
                self.response = partial_response;
                // Substep 4
                self.dispatch_response_progress_event("progress".to_string());
            },
            DoneMsg => {
                // Part of step 13, send() (processing response end of file)
                // XXXManishearth handle errors, if any (substep 1)

                // Substep 3
                if self.ready_state == Loading {
                    // Subsubsteps 2-4
                    self.send_flag = false;
                    self.change_ready_state(XHRDone);

                    // Subsubsteps 5-7
                    self.dispatch_response_progress_event("progress".to_string());
                    self.dispatch_response_progress_event("load".to_string());
                    self.dispatch_response_progress_event("loadend".to_string());
                }
            },
            ErroredMsg => {
                self.send_flag = false;
                // XXXManishearth set response to NetworkError
                // XXXManishearth also handle terminated requests (timeout/abort/fatal)
                self.change_ready_state(XHRDone);
                if !self.sync {
                    if !self.upload_complete {
                        self.upload_complete = true;
                        self.dispatch_upload_progress_event("progress".to_string(), None);
                        self.dispatch_upload_progress_event("load".to_string(), None);
                        self.dispatch_upload_progress_event("loadend".to_string(), None);
                    }
                    self.dispatch_response_progress_event("progress".to_string());
                    self.dispatch_response_progress_event("load".to_string());
                    self.dispatch_response_progress_event("loadend".to_string());
                }

            },
            ReleaseMsg => {
                self.release();
            }
        }
    }

    fn insert_trusted_header(&mut self, name: String, value: String) {
        // Insert a header without checking spec-compliance
        // Use for hardcoded headers
        let collection = self.request_headers.deref_mut();
        let value_bytes = value.into_bytes();
        let mut reader = BufReader::new(value_bytes.as_slice());
        let maybe_header: Option<Header> = HeaderEnum::value_from_stream(
                                                                String::from_str(name.as_slice()),
                                                                &mut HeaderValueByteIterator::new(&mut reader));
        collection.insert(maybe_header.unwrap());
    }

    fn dispatch_progress_event(&self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>) {
        let win = &*self.global.root();
        let upload_target = &*self.upload.get().root().unwrap();
        let progressevent = ProgressEvent::new(win, type_, false, false,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0)).root();
        let target: &JSRef<EventTarget> = if upload {
            EventTargetCast::from_ref(upload_target)
        } else {
            EventTargetCast::from_ref(self)
        };
        let event: &JSRef<Event> = EventCast::from_ref(&*progressevent);
        target.dispatch_event_with_target(None, event).ok();
    }

    fn dispatch_upload_progress_event(&self, type_: DOMString, partial_load: Option<u64>) {
        // If partial_load is None, loading has completed and we can just use the value from the request body

        let total = self.request_body.len() as u64;
        self.dispatch_progress_event(true, type_, partial_load.unwrap_or(total), Some(total));
    }

    fn dispatch_response_progress_event(&self, type_: DOMString) {
        let len = self.response.len() as u64;
        let total = self.response_headers.deref().content_length.map(|x| {x as u64});
        self.dispatch_progress_event(false, type_, len, total);
    }

    fn text_response(&self) -> DOMString {
        let mut encoding = UTF_8 as &Encoding:Send;
        match self.response_headers.content_type {
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
        encoding.decode(self.response.as_slice(), DecodeReplace).unwrap().to_string()
    }
}
