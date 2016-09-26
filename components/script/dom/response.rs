/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::cell::Cell;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HeadersBinding::HeadersMethods;
use dom::bindings::codegen::Bindings::ResponseBinding;
use dom::bindings::codegen::Bindings::ResponseBinding::{ResponseMethods, ResponseType as DOMResponseType};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, USVString};
use dom::headers::{Headers, Guard};
use dom::headers::{is_vchar, is_obs_text};
use hyper::status::StatusCode;
use net_traits::response::{ResponseBody as NetTraitsResponseBody};
use std::str::FromStr;
use url::Position;
use url::Url;

#[dom_struct]
pub struct Response {
    reflector_: Reflector,
    headers_reflector: MutNullableHeap<JS<Headers>>,
    mime_type: DOMRefCell<Vec<u8>>,
    body_used: Cell<bool>,
    /// `None` can be considered a StatusCode of `0`.
    #[ignore_heap_size_of = "Defined in hyper"]
    status: DOMRefCell<Option<StatusCode>>,
    raw_status: DOMRefCell<Option<(u16, Vec<u8>)>>,
    response_type: DOMRefCell<DOMResponseType>,
    url: DOMRefCell<Option<Url>>,
    url_list: DOMRefCell<Vec<Url>>,
    // For now use the existing NetTraitsResponseBody enum, until body
    // is implemented.
    body: DOMRefCell<NetTraitsResponseBody>,
}

impl Response {
    pub fn new_inherited() -> Response {
        Response {
            reflector_: Reflector::new(),
            headers_reflector: Default::default(),
            mime_type: DOMRefCell::new("".to_string().into_bytes()),
            body_used: Cell::new(false),
            status: DOMRefCell::new(Some(StatusCode::Ok)),
            raw_status: DOMRefCell::new(Some((200, b"OK".to_vec()))),
            response_type: DOMRefCell::new(DOMResponseType::Default),
            url: DOMRefCell::new(None),
            url_list: DOMRefCell::new(vec![]),
            body: DOMRefCell::new(NetTraitsResponseBody::Empty),
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response
    pub fn new(global: GlobalRef) -> Root<Response> {
        reflect_dom_object(box Response::new_inherited(), global, ResponseBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef, _body: Option<USVString>, init: &ResponseBinding::ResponseInit)
                       -> Fallible<Root<Response>> {
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
            // TODO: Figure out how/if we should make r's response's
            // header list and r's Headers object the same thing. For
            // now just working with r's Headers object. Also, the
            // header list should already be empty so this step may be
            // unnecessary.
            r.Headers().empty_header_list();

            // Step 6.2
            try!(r.Headers().fill(Some(headers_member.clone())));
        }

        // Step 7
        if let Some(_) = _body {
            // Step 7.1
            if is_null_body_status(init.status) {
                return Err(Error::Type(
                    "Body is non-null but init's status member is a null body status".to_string()));
            };

            // Step 7.2
            let content_type: Option<ByteString> = None;

            // Step 7.3
            // TODO: Extract body and implement step 7.3.

            // Step 7.4
            if let Some(content_type_contents) = content_type {
                if !r.Headers().Has(ByteString::new(b"Content-Type".to_vec())).unwrap() {
                    try!(r.Headers().Append(ByteString::new(b"Content-Type".to_vec()), content_type_contents));
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
    pub fn Error(global: GlobalRef) -> Root<Response> {
        let r = Response::new(global);
        *r.response_type.borrow_mut() = DOMResponseType::Error;
        r.Headers().set_guard(Guard::Immutable);
        *r.raw_status.borrow_mut() = Some((0, b"".to_vec()));
        r
    }

    // https://fetch.spec.whatwg.org/#dom-response-redirect
    pub fn Redirect(global: GlobalRef, url: USVString, status: u16) -> Fallible<Root<Response>> {
        // Step 1
        // TODO: `entry settings object` is not implemented in Servo yet.
        let base_url = global.get_url();
        let parsed_url = base_url.join(&url.0);

        // Step 2
        let url = match parsed_url {
            Ok(url) => url,
            Err(_) => return Err(Error::Type("Url could not be parsed".to_string())),
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
        try!(r.Headers().Set(ByteString::new(b"Location".to_vec()), url_bytestring));

        // Step 4 continued
        // Headers Guard is set to Immutable here to prevent error in Step 6
        r.Headers().set_guard(Guard::Immutable);

        // Step 7
        Ok(r)
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
    fn Headers(&self) -> Root<Headers> {
        self.headers_reflector.or_init(|| Headers::for_response(self.global().r()))
    }

    // https://fetch.spec.whatwg.org/#dom-response-clone
    fn Clone(&self) -> Fallible<Root<Response>> {
        // Step 1
        // TODO: This step relies on body and stream, which are still unimplemented.

        // Step 2
        let new_response = Response::new(self.global().r());
        new_response.Headers().set_guard(self.Headers().get_guard());

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
}

fn serialize_without_fragment(url: &Url) -> &str {
    &url[..Position::AfterQuery]
}
