/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::XMLHttpRequestBinding;
use dom::bindings::str::ByteString;
use self::XMLHttpRequestBinding::XMLHttpRequestResponseType;
use self::XMLHttpRequestBinding::XMLHttpRequestResponseTypeValues::{_empty, Text};
use dom::bindings::codegen::InheritTypes::{EventTargetCast, XMLHttpRequestDerived};
use dom::bindings::error::{ErrorResult, InvalidState, Network, Syntax, Security};
use dom::document::Document;
use dom::event::{Event, EventMethods};
use dom::eventtarget::{EventTarget, EventTargetHelpers, XMLHttpRequestTargetTypeId};
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable};
use dom::bindings::trace::Untraceable;
use js::jsapi::{JS_AddObjectRoot, JS_RemoveObjectRoot, JSContext};
use js::jsval::{JSVal, NullValue};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use dom::xmlhttprequesteventtarget::XMLHttpRequestEventTarget;
use dom::xmlhttprequestupload::XMLHttpRequestUpload;
use net::resource_task::{ResourceTask, Load, Payload, Done};
use script_task::{ScriptChan, XHRProgressMsg};
use servo_util::str::DOMString;
use servo_util::url::{parse_url, try_parse_url};
use url::Url;

use libc;
use libc::c_void;

use std::comm::channel;
use std::io::MemWriter;

use std::task;

use ResponseHeaderCollection = http::headers::response::HeaderCollection;
use RequestHeaderCollection = http::headers::request::HeaderCollection;

// As send() start accepting more and more parameter types,
// change this to the appropriate type from UnionTypes, eg
// use SendParam = dom::bindings::codegen::UnionTypes::StringOrFormData;
type SendParam = DOMString;

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
    HeadersReceivedMsg(Option<ResponseHeaderCollection>),
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
            Sync(_) => true,
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
    upload: Option<JS<XMLHttpRequestUpload>>,
    response_url: DOMString,
    status: u16,
    status_text: ByteString,
    response: ByteString,
    response_type: XMLHttpRequestResponseType,
    response_text: DOMString,
    response_xml: Option<JS<Document>>,
    response_headers: Untraceable<ResponseHeaderCollection>,

    // Associated concepts
    request_method: ByteString,
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
        let mut xhr = XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestTypeId),
            ready_state: Unsent,
            timeout: 0u32,
            with_credentials: false,
            upload: None,
            response_url: "".to_owned(),
            status: 0,
            status_text: ByteString::new(vec!()),
            response: ByteString::new(vec!()),
            response_type: _empty,
            response_text: "".to_owned(),
            response_xml: None,
            response_headers: Untraceable::new(ResponseHeaderCollection::new()),

            request_method: ByteString::new(vec!()),
            request_url: Untraceable::new(parse_url("", None)),
            request_headers: Untraceable::new(RequestHeaderCollection::new()),
            request_body: "".to_owned(),
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
        reflect_dom_object(~XMLHttpRequest::new_inherited(window),
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

    fn fetch(fetch_type: &mut SyncOrAsync, resource_task: ResourceTask, url: Url) -> ErrorResult {

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
        resource_task.send(Load(url, start_chan));
        let response = start_port.recv();
        notify_partial_progress(fetch_type, HeadersReceivedMsg(response.metadata.headers.clone()));
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
    fn ReadyState(&self) -> u16;
    fn Open(&mut self, _method: ByteString, _url: DOMString) -> ErrorResult;
    fn Open_(&mut self, _method: ByteString, _url: DOMString, _async: bool,
             _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult;
    fn SetRequestHeader(&self, _name: ByteString, _value: ByteString);
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
    fn GetResponseHeader(&self, _name: ByteString) -> Option<ByteString>;
    fn GetAllResponseHeaders(&self) -> ByteString;
    fn OverrideMimeType(&self, _mime: DOMString);
    fn ResponseType(&self) -> XMLHttpRequestResponseType;
    fn SetResponseType(&mut self, response_type: XMLHttpRequestResponseType);
    fn Response(&self, _cx: *JSContext) -> JSVal;
    fn ResponseText(&self) -> DOMString;
    fn GetResponseXML(&self) -> Option<Temporary<Document>>;
}

impl<'a> XMLHttpRequestMethods<'a> for JSRef<'a, XMLHttpRequest> {
    fn ReadyState(&self) -> u16 {
        self.ready_state as u16
    }
    fn Open(&mut self, method: ByteString, url: DOMString) -> ErrorResult {
        self.request_method = method;

        // Step 2
        let base: Option<Url> = Some(self.global.root().get_url());
        match self.request_method.to_lower().as_str() {
            Some("get") => {

                // Step 5
                self.request_method = self.request_method.to_lower();

                // Step 6
                let parsed_url = match try_parse_url(url, base) {
                    Ok(parsed) => parsed,
                    Err(_) => return Err(Syntax) // Step 7
                };
                // XXXManishearth Do some handling of username/passwords, and abort existing requests

                // Step 12
                self.request_url = Untraceable::new(parsed_url);
                self.request_headers = Untraceable::new(RequestHeaderCollection::new());
                self.send_flag = false;
                // XXXManishearth Set response to a NetworkError

                // Step 13
                self.change_ready_state(Opened);
                //XXXManishearth fire a progressevent
                Ok(())
            },
            // XXXManishearth Handle other standard methods
            Some("connect") | Some("trace") | Some("track") => {
                Err(Security) // Step 4
            },
            None => Err(Syntax),
            _ => {
                if self.request_method.is_token() {
                    Ok(()) // XXXManishearth handle extension methods
                } else {
                    Err(Syntax) // Step 3
                }
            }
        }
    }
    fn Open_(&mut self, method: ByteString, url: DOMString, async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult {
        self.sync = !async;
        self.Open(method, url)
    }
    fn SetRequestHeader(&self, _name: ByteString, _value: ByteString) {

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
        Temporary::new(self.upload.get_ref().clone())
    }
    fn Send(&mut self, data: Option<DOMString>) -> ErrorResult {
        // XXXManishearth handle POSTdata, and headers
        if self.ready_state != Opened || self.send_flag {
            return Err(InvalidState); // Step 1, 2
        }

        let data = match self.request_method.to_lower().as_str() {
            Some("get") | Some("head") => None, // Step 3
            _ => data
        };

        // Step 6
        self.upload_complete = false;
        self.upload_events = false;
        // XXXManishearth handle upload events

        // Step 9
        self.send_flag = true;
        let mut global = self.global.root();
        let resource_task = global.page().resource_task.deref().clone();
        let url = self.request_url.clone();
        if self.sync {
            return XMLHttpRequest::fetch(&mut Sync(self), resource_task, url);
        } else {
            let mut builder = task::task().named("XHRTask");
            unsafe {
                let addr = self.to_trusted();
                let script_chan = global.script_chan.clone();
                builder.spawn(proc() {
                    XMLHttpRequest::fetch(&mut Async(addr, script_chan), resource_task, url);
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
        None
    }
    fn GetAllResponseHeaders(&self) -> ByteString {
        let mut writer = MemWriter::new();
        self.response_headers.deref().write_all(&mut writer);
        ByteString::new(writer.unwrap())
    }
    fn OverrideMimeType(&self, _mime: DOMString) {

    }
    fn ResponseType(&self) -> XMLHttpRequestResponseType {
        self.response_type
    }
    fn SetResponseType(&mut self, response_type: XMLHttpRequestResponseType) {
        self.response_type = response_type
    }
    fn Response(&self, cx: *JSContext) -> JSVal {
         match self.response_type {
            _empty | Text => {
                if self.ready_state == XHRDone || self.ready_state == Loading {
                    self.response.to_jsval(cx)
                } else {
                    "".to_owned().to_jsval(cx)
                }
            },
            _ => {
                if self.ready_state == XHRDone {
                    // XXXManishearth we may not be able to store
                    // other response types as DOMStrings
                    self.response.to_jsval(cx)
                } else {
                    NullValue()
                }
            }
        }
    }
    fn ResponseText(&self) -> DOMString {
        self.response_text.clone()
    }
    fn GetResponseXML(&self) -> Option<Temporary<Document>> {
        self.response_xml.clone().map(|response| Temporary::new(response))
    }
}

impl Reflectable for XMLHttpRequest {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.eventtarget.mut_reflector()
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
        self.ready_state = rs;
        let win = &*self.global.root();
        let mut event = Event::new(win).root();
        event.InitEvent("readystatechange".to_owned(), false, true);
        let target: &JSRef<EventTarget> = EventTargetCast::from_ref(self);
        target.dispatch_event_with_target(None, &mut *event).ok();
    }

    fn process_partial_response(&mut self, progress: XHRProgress) {
        match progress {
            HeadersReceivedMsg(headers) => {
                match headers {
                    Some(ref h) => *self.response_headers = h.clone(),
                    None => {}
                };
                self.change_ready_state(HeadersReceived);
            },
            LoadingMsg(partial_response) => {
                self.response = partial_response;
                if self.ready_state == HeadersReceived {
                    self.change_ready_state(Loading);
                }
            },
            DoneMsg => {
                self.send_flag = false;
                self.change_ready_state(XHRDone);
            },
            ErroredMsg => {
                 self.send_flag = false;
                // XXXManishearth set response to NetworkError
                if !self.upload_complete {
                    self.upload_complete = true;
                    // XXXManishearth handle upload progress
                }
                // XXXManishearth fire some progress events
                self.change_ready_state(XHRDone);
            },
            ReleaseMsg => {
                self.release();
            }
        }
    }
}