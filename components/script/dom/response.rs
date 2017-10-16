/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use body::{BodyOperations, BodyType, consume_body, consume_body_with_promise};
use core::cell::Cell;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::HeadersBinding::{HeadersInit, HeadersMethods};
use dom::bindings::codegen::Bindings::ResponseBinding;
use dom::bindings::codegen::Bindings::ResponseBinding::{ResponseMethods, ResponseType as DOMResponseType};
use dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::{ByteString, USVString};
use dom::globalscope::GlobalScope;
use dom::headers::{Headers, Guard};
use dom::headers::{is_vchar, is_obs_text};
use dom::promise::Promise;
use dom::xmlhttprequest::Extractable;
use dom_struct::dom_struct;
use hyper::header::Headers as HyperHeaders;
use hyper::status::StatusCode;
use hyper_serde::Serde;
use net_traits::response::{ResponseBody as NetTraitsResponseBody};
use servo_url::ServoUrl;
use std::cell::Ref;
use std::mem;
use std::rc::Rc;
use std::str::FromStr;
use url::Position;

#[dom_struct]
pub struct Response {
    reflector_: Reflector,
    headers_reflector: MutNullableDom<Headers>,
    mime_type: DomRefCell<Vec<u8>>,
    body_used: Cell<bool>,
    /// `None` can be considered a StatusCode of `0`.
    #[ignore_heap_size_of = "Defined in hyper"]
    status: DomRefCell<Option<StatusCode>>,
    raw_status: DomRefCell<Option<(u16, Vec<u8>)>>,
    response_type: DomRefCell<DOMResponseType>,
    url: DomRefCell<Option<ServoUrl>>,
    url_list: DomRefCell<Vec<ServoUrl>>,
    // For now use the existing NetTraitsResponseBody enum
    body: DomRefCell<NetTraitsResponseBody>,
    #[ignore_heap_size_of = "Rc"]
    body_promise: DomRefCell<Option<(Rc<Promise>, BodyType)>>,
}

impl Response {
    pub fn new_inherited() -> Response {
        Response {
            reflector_: Reflector::new(),
            headers_reflector: Default::default(),
            mime_type: DomRefCell::new("".to_string().into_bytes()),
            body_used: Cell::new(false),
            status: DomRefCell::new(Some(StatusCode::Ok)),
            raw_status: DomRefCell::new(Some((200, b"OK".to_vec()))),
            response_type: DomRefCell::new(DOMResponseType::Default),
            url: DomRefCell::new(None),
            url_list: DomRefCell::new(vec![]),
            body: DomRefCell::new(NetTraitsResponseBody::Empty),
            body_promise: DomRefCell::new(None),
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response
    pub fn new(global: &GlobalScope) -> DomRoot<Response> {
        reflect_dom_object(Box::new(Response::new_inherited()), global, ResponseBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope, body: Option<BodyInit>, init: &ResponseBinding::ResponseInit)
                       -> Fallible<DomRoot<Response>> {
        // Step 1
        if init.status < 200 || init.status > 599 {
            return Err(Error::Range(
                format!("init's status member should be in the range 200 to 599, inclusive, but is {}"
                        , init.status)));
        }

        // Step 2
        if !is_valid_status_text(&init.statusText) {
            return Err(Error::Type("init's statusText member does not match the reason-phrase token production"
                                   .to_string()));
        }

        // Step 3
        let r = Response::new(global);

        // Step 4
        *r.status.borrow_mut() = Some(StatusCode::from_u16(init.status));

        // Step 5
        *r.raw_status.borrow_mut() = Some((init.status, init.statusText.clone().into()));

        // Step 6
        if let Some(ref headers_member) = init.headers {
            // Step 6.1
            r.Headers().empty_header_list();

            // Step 6.2
            r.Headers().fill(Some(headers_member.clone()))?;
        }

        // Step 7
        if let Some(ref body) = body {
            // Step 7.1
            if is_null_body_status(init.status) {
                return Err(Error::Type(
                    "Body is non-null but init's status member is a null body status".to_string()));
            };

            // Step 7.3
            let (extracted_body, content_type) = body.extract();
            *r.body.borrow_mut() = NetTraitsResponseBody::Done(extracted_body);

            // Step 7.4
            if let Some(content_type_contents) = content_type {
                if !r.Headers().Has(ByteString::new(b"Content-Type".to_vec())).unwrap() {
                    r.Headers().Append(ByteString::new(b"Content-Type".to_vec()),
                                            ByteString::new(content_type_contents.as_bytes().to_vec()))?;
                }
            };
        }

        // Step 8
        *r.mime_type.borrow_mut() = r.Headers().extract_mime_type();

        // Step 9
        // TODO: `entry settings object` is not implemented in Servo yet.

        // Step 10
        // TODO: Write this step once Promises are merged in

        // Step 11
        Ok(r)
    }

    // https://fetch.spec.whatwg.org/#dom-response-error
    pub fn Error(global: &GlobalScope) -> DomRoot<Response> {
        let r = Response::new(global);
        *r.response_type.borrow_mut() = DOMResponseType::Error;
        r.Headers().set_guard(Guard::Immutable);
        *r.raw_status.borrow_mut() = Some((0, b"".to_vec()));
        r
    }

    // https://fetch.spec.whatwg.org/#dom-response-redirect
    pub fn Redirect(global: &GlobalScope, url: USVString, status: u16) -> Fallible<DomRoot<Response>> {
        // Step 1
        let base_url = global.api_base_url();
        let parsed_url = base_url.join(&url.0);

        // Step 2
        let url = match parsed_url {
            Ok(url) => url,
            Err(_) => return Err(Error::Type("ServoUrl could not be parsed".to_string())),
        };

        // Step 3
        if !is_redirect_status(status) {
            return Err(Error::Range("status is not a redirect status".to_string()));
        }

        // Step 4
        // see Step 4 continued
        let r = Response::new(global);

        // Step 5
        *r.status.borrow_mut() = Some(StatusCode::from_u16(status));
        *r.raw_status.borrow_mut() = Some((status, b"".to_vec()));

        // Step 6
        let url_bytestring = ByteString::from_str(url.as_str()).unwrap_or(ByteString::new(b"".to_vec()));
        r.Headers().Set(ByteString::new(b"Location".to_vec()), url_bytestring)?;

        // Step 4 continued
        // Headers Guard is set to Immutable here to prevent error in Step 6
        r.Headers().set_guard(Guard::Immutable);

        // Step 7
        Ok(r)
    }

    // https://fetch.spec.whatwg.org/#concept-body-locked
    fn locked(&self) -> bool {
        // TODO: ReadableStream is unimplemented. Just return false
        // for now.
        false
    }
}

impl BodyOperations for Response {
    fn get_body_used(&self) -> bool {
        self.BodyUsed()
    }

    fn set_body_promise(&self, p: &Rc<Promise>, body_type: BodyType) {
        assert!(self.body_promise.borrow().is_none());
        self.body_used.set(true);
        *self.body_promise.borrow_mut() = Some((p.clone(), body_type));
    }

    fn is_locked(&self) -> bool {
        self.locked()
    }

    fn take_body(&self) -> Option<Vec<u8>> {
        let body = mem::replace(&mut *self.body.borrow_mut(), NetTraitsResponseBody::Empty);
        match body {
            NetTraitsResponseBody::Done(bytes) => {
                Some(bytes)
            },
            body => {
                mem::replace(&mut *self.body.borrow_mut(), body);
                None
            },
        }
    }

    fn get_mime_type(&self) -> Ref<Vec<u8>> {
        self.mime_type.borrow()
    }
}

// https://fetch.spec.whatwg.org/#redirect-status
fn is_redirect_status(status: u16) -> bool {
    status == 301 || status == 302 || status == 303 || status == 307 || status == 308
}

// https://tools.ietf.org/html/rfc7230#section-3.1.2
fn is_valid_status_text(status_text: &ByteString) -> bool {
    // reason-phrase  = *( HTAB / SP / VCHAR / obs-text )
    for byte in status_text.iter() {
        if !(*byte == b'\t' || *byte == b' ' || is_vchar(*byte) || is_obs_text(*byte)) {
            return false;
        }
    }
    true
}

// https://fetch.spec.whatwg.org/#null-body-status
fn is_null_body_status(status: u16) -> bool {
    status == 101 || status == 204 || status == 205 || status == 304
}

impl ResponseMethods for Response {
    // https://fetch.spec.whatwg.org/#dom-response-type
    fn Type(&self) -> DOMResponseType {
        *self.response_type.borrow()//into()
    }

    // https://fetch.spec.whatwg.org/#dom-response-url
    fn Url(&self) -> USVString {
        USVString(String::from((*self.url.borrow()).as_ref().map(|u| serialize_without_fragment(u)).unwrap_or("")))
    }

    // https://fetch.spec.whatwg.org/#dom-response-redirected
    fn Redirected(&self) -> bool {
        let url_list_len = self.url_list.borrow().len();
        url_list_len > 1
    }

    // https://fetch.spec.whatwg.org/#dom-response-status
    fn Status(&self) -> u16 {
        match *self.raw_status.borrow() {
            Some((s, _)) => s,
            None => 0,
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response-ok
    fn Ok(&self) -> bool {
        match *self.status.borrow() {
            Some(s) => {
                let status_num = s.to_u16();
                return status_num >= 200 && status_num <= 299;
            }
            None => false,
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response-statustext
    fn StatusText(&self) -> ByteString {
        match *self.raw_status.borrow() {
            Some((_, ref st)) => ByteString::new(st.clone()),
            None => ByteString::new(b"OK".to_vec()),
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response-headers
    fn Headers(&self) -> DomRoot<Headers> {
        self.headers_reflector.or_init(|| Headers::for_response(&self.global()))
    }

    // https://fetch.spec.whatwg.org/#dom-response-clone
    fn Clone(&self) -> Fallible<DomRoot<Response>> {
        // Step 1
        if self.is_locked() || self.body_used.get() {
            return Err(Error::Type("cannot clone a disturbed response".to_string()));
        }

        // Step 2
        let new_response = Response::new(&self.global());
        new_response.Headers().set_guard(self.Headers().get_guard());
        new_response.Headers().fill(Some(HeadersInit::Headers(self.Headers())))?;

        // https://fetch.spec.whatwg.org/#concept-response-clone
        // Instead of storing a net_traits::Response internally, we
        // only store the relevant fields, and only clone them here
        *new_response.response_type.borrow_mut() = self.response_type.borrow().clone();
        *new_response.status.borrow_mut() = self.status.borrow().clone();
        *new_response.raw_status.borrow_mut() = self.raw_status.borrow().clone();
        *new_response.url.borrow_mut() = self.url.borrow().clone();
        *new_response.url_list.borrow_mut() = self.url_list.borrow().clone();

        if *self.body.borrow() != NetTraitsResponseBody::Empty {
            *new_response.body.borrow_mut() = self.body.borrow().clone();
        }

        // Step 3
        // TODO: This step relies on promises, which are still unimplemented.

        // Step 4
        Ok(new_response)
    }

    // https://fetch.spec.whatwg.org/#dom-body-bodyused
    fn BodyUsed(&self) -> bool {
        self.body_used.get()
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-text
    fn Text(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Text)
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-blob
    fn Blob(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Blob)
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-formdata
    fn FormData(&self) -> Rc<Promise> {
        consume_body(self, BodyType::FormData)
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-json
    fn Json(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Json)
    }
}

fn serialize_without_fragment(url: &ServoUrl) -> &str {
    &url[..Position::AfterQuery]
}

impl Response {
    pub fn set_type(&self, new_response_type: DOMResponseType) {
        *self.response_type.borrow_mut() = new_response_type;
    }

    pub fn set_headers(&self, option_hyper_headers: Option<Serde<HyperHeaders>>) {
        self.Headers().set_headers(match option_hyper_headers {
            Some(hyper_headers) => hyper_headers.into_inner(),
            None => HyperHeaders::new(),
        });
    }

    pub fn set_raw_status(&self, status: Option<(u16, Vec<u8>)>) {
        *self.raw_status.borrow_mut() = status;
    }

    pub fn set_final_url(&self, final_url: ServoUrl) {
        *self.url.borrow_mut() = Some(final_url);
    }

    #[allow(unrooted_must_root)]
    pub fn finish(&self, body: Vec<u8>) {
        *self.body.borrow_mut() = NetTraitsResponseBody::Done(body);
        if let Some((p, body_type)) = self.body_promise.borrow_mut().take() {
            consume_body_with_promise(self, body_type, &p);
        }
    }
}
