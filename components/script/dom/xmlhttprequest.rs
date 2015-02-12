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
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::{InvalidState, InvalidAccess};
use dom::bindings::error::Error::{Network, Syntax, Security, Abort, Timeout};
use dom::bindings::global::{GlobalField, GlobalRef, GlobalRoot};
use dom::bindings::js::{MutNullableJS, JS, JSRef, Temporary, OptionalRootedRootable};
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
use script_task::{ScriptChan, ScriptMsg, Runnable};

use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecoderTrap, Encoding, EncodingRef, EncoderTrap};

use hyper::header::Headers;
use hyper::header::{Accept, ContentLength, ContentType, QualityItem};
use hyper::http::RawStatus;
use hyper::mime::{self, Mime};
use hyper::method::Method;

use js::jsapi::{JS_ParseJSON, JSContext};
use js::jsapi::JS_ClearPendingException;
use js::jsval::{JSVal, NullValue, UndefinedValue};

use net::resource_task::{ResourceTask, ResourceCORSData, LoadData, LoadResponse};
use net::resource_task::ControlMsg::Load;
use net::resource_task::ProgressMsg::{Payload, Done};
use cors::{allow_cross_origin_request, CORSRequest, RequestMode};
use util::str::DOMString;
use util::task::spawn_named;

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::default::Default;
use std::old_io::Timer;
use std::str::FromStr;
use std::time::duration::Duration;
use time;
use url::{Url, UrlParser};

use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams;
use dom::bindings::codegen::UnionTypes::StringOrURLSearchParams::{eString, eURLSearchParams};
pub type SendParam = StringOrURLSearchParams;

#[derive(PartialEq, Copy)]
#[jstraceable]
enum XMLHttpRequestState {
    Unsent = 0,
    Opened = 1,
    HeadersReceived = 2,
    Loading = 3,
    XHRDone = 4, // So as not to conflict with the ProgressMsg `Done`
}

struct XHRProgressHandler {
    addr: TrustedXHRAddress,
    progress: XHRProgress,
}

impl XHRProgressHandler {
    fn new(addr: TrustedXHRAddress, progress: XHRProgress) -> XHRProgressHandler {
        XHRProgressHandler { addr: addr, progress: progress }
    }
}

impl Runnable for XHRProgressHandler {
    fn handler(self: Box<XHRProgressHandler>) {
        let this = *self;
        XMLHttpRequest::handle_progress(this.addr, this.progress);
    }
}

#[derive(PartialEq, Clone, Copy)]
#[jstraceable]
pub struct GenerationId(uint);

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

enum SyncOrAsync<'a> {
    Sync(JSRef<'a, XMLHttpRequest>),
    Async(TrustedXHRAddress, Box<ScriptChan+Send>)
}

enum TerminateReason {
    AbortedOrReopened,
    TimedOut,
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
    response_xml: MutNullableJS<Document>,
    response_headers: DOMRefCell<Headers>,

    // Associated concepts
    request_method: DOMRefCell<Method>,
    request_url: DOMRefCell<Option<Url>>,
    request_headers: DOMRefCell<Headers>,
    request_body_len: Cell<uint>,
    sync: Cell<bool>,
    upload_complete: Cell<bool>,
    upload_events: Cell<bool>,
    send_flag: Cell<bool>,

    global: GlobalField,
    timer: DOMRefCell<Timer>,
    fetch_time: Cell<i64>,
    terminate_sender: DOMRefCell<Option<Sender<TerminateReason>>>,
    generation_id: Cell<GenerationId>,
}

impl XMLHttpRequest {
    fn new_inherited(global: GlobalRef) -> XMLHttpRequest {
        XMLHttpRequest {
            eventtarget: XMLHttpRequestEventTarget::new_inherited(XMLHttpRequestEventTargetTypeId::XMLHttpRequest),
            ready_state: Cell::new(XMLHttpRequestState::Unsent),
            timeout: Cell::new(0u32),
            with_credentials: Cell::new(false),
            upload: JS::from_rooted(XMLHttpRequestUpload::new(global)),
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
            timer: DOMRefCell::new(Timer::new().unwrap()),
            fetch_time: Cell::new(0),
            terminate_sender: DOMRefCell::new(None),
            generation_id: Cell::new(GenerationId(0))
        }
    }
    pub fn new(global: GlobalRef) -> Temporary<XMLHttpRequest> {
        reflect_dom_object(box XMLHttpRequest::new_inherited(global),
                           global,
                           XMLHttpRequestBinding::Wrap)
    }
    pub fn Constructor(global: GlobalRef) -> Fallible<Temporary<XMLHttpRequest>> {
        Ok(XMLHttpRequest::new(global))
    }

    pub fn handle_progress(addr: TrustedXHRAddress, progress: XHRProgress) {
        let xhr = addr.to_temporary().root();
        xhr.r().process_partial_response(progress);
    }

    #[allow(unsafe_blocks)]
    fn fetch(fetch_type: &SyncOrAsync, resource_task: ResourceTask,
             mut load_data: LoadData, terminate_receiver: Receiver<TerminateReason>,
             cors_request: Result<Option<CORSRequest>,()>, gen_id: GenerationId,
             start_port: Receiver<LoadResponse>) -> ErrorResult {

        fn notify_partial_progress(fetch_type: &SyncOrAsync, msg: XHRProgress) {
            match *fetch_type {
                SyncOrAsync::Sync(xhr) => {
                    xhr.process_partial_response(msg);
                },
                SyncOrAsync::Async(ref addr, ref script_chan) => {
                    script_chan.send(ScriptMsg::RunnableMsg(box XHRProgressHandler::new(addr.clone(), msg)));
                }
            }
        }

        macro_rules! notify_error_and_return(
            ($err:expr) => ({
                notify_partial_progress(fetch_type, XHRProgress::Errored(gen_id, $err));
                return Err($err)
            });
        );

        macro_rules! terminate(
            ($reason:expr) => (
                match $reason {
                    TerminateReason::AbortedOrReopened => {
                        return Err(Abort)
                    }
                    TerminateReason::TimedOut => {
                        notify_error_and_return!(Timeout);
                    }
                }
            );
        );


        match cors_request {
            Err(_) => {
                // Happens in case of cross-origin non-http URIs
                notify_error_and_return!(Network);
            }

            Ok(Some(ref req)) => {
                let (chan, cors_port) = channel();
                let req2 = req.clone();
                // TODO: this exists only to make preflight check non-blocking
                // perhaps should be handled by the resource_loader?
                spawn_named("XHR:Cors".to_owned(), move || {
                    let response = req2.http_fetch();
                    chan.send(response).unwrap();
                });

                select! (
                    response = cors_port.recv() => {
                        let response = response.unwrap();
                        if response.network_error {
                            notify_error_and_return!(Network);
                        } else {
                            load_data.cors = Some(ResourceCORSData {
                                preflight: req.preflight_flag,
                                origin: req.origin.clone()
                            });
                        }
                    },
                    reason = terminate_receiver.recv() => terminate!(reason.unwrap())
                );
            }
            _ => {}
        }

        // Step 10, 13
        resource_task.send(Load(load_data)).unwrap();


        let progress_port;
        select! (
            response = start_port.recv() => {
                let response = response.unwrap();
                match cors_request {
                    Ok(Some(ref req)) => {
                        match response.metadata.headers {
                            Some(ref h) if allow_cross_origin_request(req, h) => {},
                            _ => notify_error_and_return!(Network)
                        }
                    },

                    _ => {}
                };
                // XXXManishearth Clear cache entries in case of a network error
                notify_partial_progress(fetch_type, XHRProgress::HeadersReceived(gen_id,
                    response.metadata.headers.clone(), response.metadata.status.clone()));

                progress_port = response.progress_port;
            },
            reason = terminate_receiver.recv() => terminate!(reason.unwrap())
        );

        let mut buf = vec!();
        loop {
            // Under most circumstances, progress_port will contain lots of Payload
            // events. Since select! does not have any fairness or priority, it
            // might always remove the progress_port event, even when there is
            // a terminate event waiting in the terminate_receiver. If this happens,
            // a timeout or abort will take too long to be processed. To avoid this,
            // in each iteration, we check for a terminate event before we block.
            match terminate_receiver.try_recv() {
                Ok(reason) => terminate!(reason),
                Err(_) => ()
            };

            select! (
                progress = progress_port.recv() => match progress.unwrap() {
                    Payload(data) => {
                        buf.push_all(data.as_slice());
                        notify_partial_progress(fetch_type,
                                                XHRProgress::Loading(gen_id, ByteString::new(buf.clone())));
                    },
                    Done(Ok(()))  => {
                        notify_partial_progress(fetch_type, XHRProgress::Done(gen_id));
                        return Ok(());
                    },
                    Done(Err(_))  => {
                        notify_error_and_return!(Network);
                    }
                },
                reason = terminate_receiver.recv() => terminate!(reason.unwrap())
            );
        }
    }
}

impl<'a> XMLHttpRequestMethods for JSRef<'a, XMLHttpRequest> {
    event_handler!(readystatechange, GetOnreadystatechange, SetOnreadystatechange);

    fn ReadyState(self) -> u16 {
        self.ready_state.get() as u16
    }

    fn Open(self, method: ByteString, url: DOMString) -> ErrorResult {
        //FIXME(seanmonstar): use a Trie instead?
        let maybe_method = method.as_str().and_then(|s| {
            // Note: hyper tests against the uppercase versions
            // Since we want to pass methods not belonging to the short list above
            // without changing capitalization, this will actually sidestep rust-http's type system
            // since methods like "patch" or "PaTcH" will be considered extension methods
            // despite the there being a rust-http method variant for them
            let upper = s.to_ascii_uppercase();
            match upper.as_slice() {
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
            Some(Method::Extension(ref t)) if t.as_slice() == "TRACK" => Err(Security),
            Some(_) if method.is_token() => {

                *self.request_method.borrow_mut() = maybe_method.unwrap();

                // Step 6
                let base = self.global.root().r().get_url();
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
    fn Open_(self, method: ByteString, url: DOMString, async: bool,
                 _username: Option<DOMString>, _password: Option<DOMString>) -> ErrorResult {
        self.sync.set(!async);
        self.Open(method, url)
    }
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
                buf.push_all(value.as_slice());
                debug!("SetRequestHeader: new value = {:?}", buf);
                value = ByteString::new(buf);
            },
            None => {}
        }

        headers.set_raw(name_str.to_owned(), vec![value.as_slice().to_vec()]);
        Ok(())
    }
    fn Timeout(self) -> u32 {
        self.timeout.get()
    }
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
    fn WithCredentials(self) -> bool {
        self.with_credentials.get()
    }
    // Spec for SetWithCredentials: https://xhr.spec.whatwg.org/#dom-xmlhttprequest-withcredentials
    fn SetWithCredentials(self, with_credentials: bool) -> ErrorResult {
        match self.ready_state.get() {
            XMLHttpRequestState::HeadersReceived |
            XMLHttpRequestState::Loading |
            XMLHttpRequestState::XHRDone => Err(InvalidState),
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
    fn Upload(self) -> Temporary<XMLHttpRequestUpload> {
        Temporary::new(self.upload)
    }
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
            Some (ref v) if v.len() == 0 => true,
            _ => false
        });

        if !self.sync.get() {
            // Step 8
            let upload_target = self.upload.root();
            let event_target: JSRef<EventTarget> = EventTargetCast::from_ref(upload_target.r());
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
        let resource_task = global.r().resource_task();
        let (start_chan, start_port) = channel();
        let mut load_data = LoadData::new(self.request_url.borrow().clone().unwrap(), start_chan);
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
            load_data.preserved_headers.set(Accept(vec![QualityItem::new(mime, 1.0)]));
        }

        load_data.method = (*self.request_method.borrow()).clone();
        let (terminate_sender, terminate_receiver) = channel();
        *self.terminate_sender.borrow_mut() = Some(terminate_sender);

        // CORS stuff
        let referer_url = self.global.root().r().get_url();
        let mode = if self.upload_events.get() {
            RequestMode::ForcedPreflight
        } else {
            RequestMode::CORS
        };
        let mut combined_headers = load_data.headers.clone();
        combined_headers.extend(load_data.preserved_headers.iter());
        let cors_request = CORSRequest::maybe_new(referer_url.clone(), load_data.url.clone(), mode,
                                                  load_data.method.clone(), combined_headers);
        match cors_request {
            Ok(None) => {
                let mut buf = String::new();
                buf.push_str(referer_url.scheme.as_slice());
                buf.push_str("://".as_slice());
                referer_url.serialize_host().map(|ref h| buf.push_str(h.as_slice()));
                referer_url.port().as_ref().map(|&p| {
                    buf.push_str(":".as_slice());
                    buf.push_str(format!("{}", p).as_slice());
                });
                referer_url.serialize_path().map(|ref h| buf.push_str(h.as_slice()));
                self.request_headers.borrow_mut().set_raw("Referer".to_owned(), vec![buf.into_bytes()]);
            },
            Ok(Some(ref req)) => self.insert_trusted_header("origin".to_owned(),
                                                            format!("{}", req.origin)),
            _ => {}
        }

        debug!("request_headers = {:?}", *self.request_headers.borrow());

        let gen_id = self.generation_id.get();
        if self.sync.get() {
            return XMLHttpRequest::fetch(&mut SyncOrAsync::Sync(self), resource_task, load_data,
                                         terminate_receiver, cors_request, gen_id, start_port);
        } else {
            self.fetch_time.set(time::now().to_timespec().sec);
            let script_chan = global.r().script_chan();
            // Pin the object before launching the fetch task. This is to ensure that
            // the object will stay alive as long as there are (possibly cancelled)
            // inflight events queued up in the script task's port.
            let addr = Trusted::new(self.global.root().r().get_cx(), self,
                                    script_chan.clone());
            spawn_named("XHRTask".to_owned(), move || {
                let _ = XMLHttpRequest::fetch(&mut SyncOrAsync::Async(addr, script_chan),
                                              resource_task,
                                              load_data,
                                              terminate_receiver,
                                              cors_request,
                                              gen_id,
                                              start_port);
            });
            let timeout = self.timeout.get();
            if timeout > 0 {
                self.set_timeout(timeout);
            }
        }
        Ok(())
    }
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
            name.eq_ignore_case(&FromStr::from_str(h.name()).unwrap())
        }).map(|h| {
            ByteString::new(h.value_string().into_bytes())
        })
    }
    fn GetAllResponseHeaders(self) -> ByteString {
        ByteString::new(self.filter_response_headers().to_string().into_bytes())
    }
    fn ResponseType(self) -> XMLHttpRequestResponseType {
        self.response_type.get()
    }
    fn SetResponseType(self, response_type: XMLHttpRequestResponseType) -> ErrorResult {
        match self.global.root() {
            GlobalRoot::Worker(_) if response_type == XMLHttpRequestResponseType::Document
            => return Ok(()),
            _ => {}
        }
        match self.ready_state.get() {
            XMLHttpRequestState::Loading | XMLHttpRequestState::XHRDone => Err(InvalidState),
            _ if self.sync.get() => Err(InvalidAccess),
            _ => {
                self.response_type.set(response_type);
                Ok(())
            }
        }
    }
    #[allow(unsafe_blocks)]
    fn Response(self, cx: *mut JSContext) -> JSVal {
         match self.response_type.get() {
            _empty | Text => {
                let ready_state = self.ready_state.get();
                if ready_state == XMLHttpRequestState::XHRDone || ready_state == XMLHttpRequestState::Loading {
                    self.text_response().to_jsval(cx)
                } else {
                    "".to_jsval(cx)
                }
            },
            _ if self.ready_state.get() != XMLHttpRequestState::XHRDone => NullValue(),
            Json => {
                let decoded = UTF_8.decode(self.response.borrow().as_slice(), DecoderTrap::Replace).unwrap().to_owned();
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
                    XMLHttpRequestState::Loading | XMLHttpRequestState::XHRDone => Ok(self.text_response()),
                    _ => Ok("".to_owned())
                }
            },
            _ => Err(InvalidState)
        }
    }
    fn GetResponseXML(self) -> Option<Temporary<Document>> {
        self.response_xml.get()
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
    fn process_partial_response(self, progress: XHRProgress);
    fn terminate_ongoing_fetch(self);
    fn insert_trusted_header(self, name: String, value: String);
    fn dispatch_progress_event(self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>);
    fn dispatch_upload_progress_event(self, type_: DOMString, partial_load: Option<u64>);
    fn dispatch_response_progress_event(self, type_: DOMString);
    fn text_response(self) -> DOMString;
    fn set_timeout(self, timeout:u32);
    fn cancel_timeout(self);
    fn filter_response_headers(self) -> Headers;
}

impl<'a> PrivateXMLHttpRequestHelpers for JSRef<'a, XMLHttpRequest> {
    fn change_ready_state(self, rs: XMLHttpRequestState) {
        assert!(self.ready_state.get() != rs);
        self.ready_state.set(rs);
        let global = self.global.root();
        let event = Event::new(global.r(),
                               "readystatechange".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::Cancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        event.r().fire(target);
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

                // Part of step 11, send() (processing response end of file)
                // XXXManishearth handle errors, if any (substep 2)

                // Subsubsteps 5-7
                self.send_flag.set(false);
                self.change_ready_state(XMLHttpRequestState::XHRDone);
                return_if_fetch_was_terminated!();
                // Subsubsteps 10-12
                self.dispatch_response_progress_event("progress".to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event("load".to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_response_progress_event("loadend".to_owned());
            },
            XHRProgress::Errored(_, e) => {
                self.send_flag.set(false);
                // XXXManishearth set response to NetworkError
                self.change_ready_state(XMLHttpRequestState::XHRDone);
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
        self.terminate_sender.borrow().as_ref().map(|s| s.send(TerminateReason::AbortedOrReopened));
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
                                               type_, false, false,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0)).root();
        let target: JSRef<EventTarget> = if upload {
            EventTargetCast::from_ref(upload_target.r())
        } else {
            EventTargetCast::from_ref(self)
        };
        let event: JSRef<Event> = EventCast::from_ref(progressevent.r());
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
    fn set_timeout(self, timeout: u32) {
        // Sets up the object to timeout in a given number of milliseconds
        // This will cancel all previous timeouts
        let oneshot = self.timer.borrow_mut()
                          .oneshot(Duration::milliseconds(timeout as i64));
        let terminate_sender = (*self.terminate_sender.borrow()).clone();
        spawn_named("XHR:Timer".to_owned(), move || {
            match oneshot.recv() {
                Ok(_) => {
                    terminate_sender.map(|s| s.send(TerminateReason::TimedOut));
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
        // oneshot() closes the previous channel, canceling the timeout
        self.timer.borrow_mut().oneshot(Duration::zero());
    }

    fn text_response(self) -> DOMString {
        let mut encoding = UTF_8 as EncodingRef;
        match self.response_headers.borrow().get() {
            Some(&ContentType(mime::Mime(_, _, ref params))) => {
                for &(ref name, ref value) in params.iter() {
                    if name == &mime::Attr::Charset {
                        encoding = encoding_from_whatwg_label(value.to_string().as_slice()).unwrap_or(encoding);
                    }
                }
            },
            None => {}
        }

        // According to Simon, decode() should never return an error, so unwrap()ing
        // the result should be fine. XXXManishearth have a closer look at this later
        encoding.decode(self.response.borrow().as_slice(), DecoderTrap::Replace).unwrap().to_owned()
    }
    fn filter_response_headers(self) -> Headers {
        // http://fetch.spec.whatwg.org/#concept-response-header-list
        use std::fmt;
        use hyper::header::{Header, HeaderFormat};
        use hyper::header::SetCookie;

        // a dummy header so we can use headers.remove::<SetCookie2>()
        #[derive(Clone)]
        struct SetCookie2;
        impl Header for SetCookie2 {
            fn header_name() -> &'static str {
                "set-cookie2"
            }

            fn parse_header(_: &[Vec<u8>]) -> Option<SetCookie2> {
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
}

trait Extractable {
    fn extract(&self) -> Vec<u8>;
}
impl Extractable for SendParam {
    fn extract(&self) -> Vec<u8> {
        // http://fetch.spec.whatwg.org/#concept-fetchbodyinit-extract
        let encoding = UTF_8 as EncodingRef;
        match *self {
            eString(ref s) => encoding.encode(s.as_slice(), EncoderTrap::Replace).unwrap(),
            eURLSearchParams(ref usp) => usp.root().r().serialize(None) // Default encoding is UTF8
        }
    }
}
