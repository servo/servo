/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;
use std::rc::Rc;
use std::str::FromStr;

use dom_struct::dom_struct;
use http::header::HeaderMap as HyperHeaders;
use http::StatusCode;
use hyper_serde::Serde;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use servo_url::ServoUrl;
use url::Position;

use crate::body::{consume_body, BodyMixin, BodyType, Extractable, ExtractedBody};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HeadersBinding::HeadersMethods;
use crate::dom::bindings::codegen::Bindings::ResponseBinding;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::{
    ResponseMethods, ResponseType as DOMResponseType,
};
use crate::dom::bindings::codegen::Bindings::XMLHttpRequestBinding::BodyInit;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{ByteString, USVString};
use crate::dom::globalscope::GlobalScope;
use crate::dom::headers::{is_obs_text, is_vchar, Guard, Headers};
use crate::dom::promise::Promise;
use crate::dom::readablestream::{ExternalUnderlyingSource, ReadableStream};
use crate::script_runtime::{JSContext as SafeJSContext, StreamConsumer};

#[dom_struct]
pub struct Response {
    reflector_: Reflector,
    headers_reflector: MutNullableDom<Headers>,
    /// `None` can be considered a StatusCode of `0`.
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[no_trace]
    status: DomRefCell<Option<StatusCode>>,
    raw_status: DomRefCell<Option<(u16, Vec<u8>)>>,
    response_type: DomRefCell<DOMResponseType>,
    #[no_trace]
    url: DomRefCell<Option<ServoUrl>>,
    #[no_trace]
    url_list: DomRefCell<Vec<ServoUrl>>,
    /// The stream of <https://fetch.spec.whatwg.org/#body>.
    body_stream: MutNullableDom<ReadableStream>,
    #[ignore_malloc_size_of = "StreamConsumer"]
    stream_consumer: DomRefCell<Option<StreamConsumer>>,
    redirected: DomRefCell<bool>,
}

#[allow(non_snake_case)]
impl Response {
    pub fn new_inherited(global: &GlobalScope) -> Response {
        let stream = ReadableStream::new_with_external_underlying_source(
            global,
            ExternalUnderlyingSource::FetchResponse,
        );
        Response {
            reflector_: Reflector::new(),
            headers_reflector: Default::default(),
            status: DomRefCell::new(Some(StatusCode::OK)),
            raw_status: DomRefCell::new(Some((200, b"".to_vec()))),
            response_type: DomRefCell::new(DOMResponseType::Default),
            url: DomRefCell::new(None),
            url_list: DomRefCell::new(vec![]),
            body_stream: MutNullableDom::new(Some(&*stream)),
            stream_consumer: DomRefCell::new(None),
            redirected: DomRefCell::new(false),
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response
    pub fn new(global: &GlobalScope) -> DomRoot<Response> {
        Self::new_with_proto(global, None)
    }

    fn new_with_proto(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<Response> {
        reflect_dom_object_with_proto(Box::new(Response::new_inherited(global)), global, proto)
    }

    // https://fetch.spec.whatwg.org/#initialize-a-response
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        body: Option<BodyInit>,
        init: &ResponseBinding::ResponseInit,
    ) -> Fallible<DomRoot<Response>> {
        // Step 1
        if init.status < 200 || init.status > 599 {
            return Err(Error::Range(format!(
                "init's status member should be in the range 200 to 599, inclusive, but is {}",
                init.status
            )));
        }

        // Step 2
        if !is_valid_status_text(&init.statusText) {
            return Err(Error::Type(
                "init's statusText member does not match the reason-phrase token production"
                    .to_string(),
            ));
        }

        let r = Response::new_with_proto(global, proto);

        // Step 3
        *r.status.borrow_mut() = Some(StatusCode::from_u16(init.status).unwrap());

        // Step 4
        *r.raw_status.borrow_mut() = Some((init.status, init.statusText.clone().into()));

        // Step 5
        if let Some(ref headers_member) = init.headers {
            r.Headers().fill(Some(headers_member.clone()))?;
        }

        // Step 6
        if let Some(ref body) = body {
            // Step 6.1
            if is_null_body_status(init.status) {
                return Err(Error::Type(
                    "Body is non-null but init's status member is a null body status".to_string(),
                ));
            };

            // Step 6.2
            let ExtractedBody {
                stream,
                total_bytes: _,
                content_type,
                source: _,
            } = body.extract(global)?;

            r.body_stream.set(Some(&*stream));

            // Step 6.3
            if let Some(content_type_contents) = content_type {
                if !r
                    .Headers()
                    .Has(ByteString::new(b"Content-Type".to_vec()))
                    .unwrap()
                {
                    r.Headers().Append(
                        ByteString::new(b"Content-Type".to_vec()),
                        ByteString::new(content_type_contents.as_bytes().to_vec()),
                    )?;
                }
            };
        } else {
            // Reset FetchResponse to an in-memory stream with empty byte sequence here for
            // no-init-body case
            let stream = ReadableStream::new_from_bytes(global, Vec::with_capacity(0));
            r.body_stream.set(Some(&*stream));
        }

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
    pub fn Redirect(
        global: &GlobalScope,
        url: USVString,
        status: u16,
    ) -> Fallible<DomRoot<Response>> {
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
        *r.status.borrow_mut() = Some(StatusCode::from_u16(status).unwrap());
        *r.raw_status.borrow_mut() = Some((status, b"".to_vec()));

        // Step 6
        let url_bytestring =
            ByteString::from_str(url.as_str()).unwrap_or(ByteString::new(b"".to_vec()));
        r.Headers()
            .Set(ByteString::new(b"Location".to_vec()), url_bytestring)?;

        // Step 4 continued
        // Headers Guard is set to Immutable here to prevent error in Step 6
        r.Headers().set_guard(Guard::Immutable);

        // Step 7
        Ok(r)
    }

    pub fn error_stream(&self, error: Error) {
        if let Some(body) = self.body_stream.get() {
            body.error_native(error);
        }
    }
}

impl BodyMixin for Response {
    fn is_disturbed(&self) -> bool {
        self.body_stream
            .get()
            .map_or(false, |stream| stream.is_disturbed())
    }

    fn is_locked(&self) -> bool {
        self.body_stream
            .get()
            .map_or(false, |stream| stream.is_locked())
    }

    fn body(&self) -> Option<DomRoot<ReadableStream>> {
        self.body_stream.get()
    }

    fn get_mime_type(&self) -> Vec<u8> {
        let headers = self.Headers();
        headers.extract_mime_type()
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
        *self.response_type.borrow() //into()
    }

    // https://fetch.spec.whatwg.org/#dom-response-url
    fn Url(&self) -> USVString {
        USVString(String::from(
            (*self.url.borrow())
                .as_ref()
                .map(serialize_without_fragment)
                .unwrap_or(""),
        ))
    }

    // https://fetch.spec.whatwg.org/#dom-response-redirected
    fn Redirected(&self) -> bool {
        return *self.redirected.borrow();
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
                let status_num = s.as_u16();
                (200..=299).contains(&status_num)
            },
            None => false,
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response-statustext
    fn StatusText(&self) -> ByteString {
        match *self.raw_status.borrow() {
            Some((_, ref st)) => ByteString::new(st.clone()),
            None => ByteString::new(b"".to_vec()),
        }
    }

    // https://fetch.spec.whatwg.org/#dom-response-headers
    fn Headers(&self) -> DomRoot<Headers> {
        self.headers_reflector
            .or_init(|| Headers::for_response(&self.global()))
    }

    // https://fetch.spec.whatwg.org/#dom-response-clone
    fn Clone(&self) -> Fallible<DomRoot<Response>> {
        // Step 1
        if self.is_locked() || self.is_disturbed() {
            return Err(Error::Type("cannot clone a disturbed response".to_string()));
        }

        // Step 2
        let new_response = Response::new(&self.global());
        new_response.Headers().copy_from_headers(self.Headers())?;
        new_response.Headers().set_guard(self.Headers().get_guard());

        // https://fetch.spec.whatwg.org/#concept-response-clone
        // Instead of storing a net_traits::Response internally, we
        // only store the relevant fields, and only clone them here
        *new_response.response_type.borrow_mut() = *self.response_type.borrow();
        *new_response.status.borrow_mut() = *self.status.borrow();
        new_response
            .raw_status
            .borrow_mut()
            .clone_from(&self.raw_status.borrow());
        new_response.url.borrow_mut().clone_from(&self.url.borrow());
        new_response
            .url_list
            .borrow_mut()
            .clone_from(&self.url_list.borrow());

        if let Some(stream) = self.body_stream.get().clone() {
            new_response.body_stream.set(Some(&*stream));
        }

        // Step 3
        // TODO: This step relies on promises, which are still unimplemented.

        // Step 4
        Ok(new_response)
    }

    // https://fetch.spec.whatwg.org/#dom-body-bodyused
    fn BodyUsed(&self) -> bool {
        self.is_disturbed()
    }

    /// <https://fetch.spec.whatwg.org/#dom-body-body>
    fn GetBody(&self, _cx: SafeJSContext) -> Option<NonNull<JSObject>> {
        self.body().map(|stream| stream.get_js_stream())
    }

    // https://fetch.spec.whatwg.org/#dom-body-text
    fn Text(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Text)
    }

    // https://fetch.spec.whatwg.org/#dom-body-blob
    fn Blob(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Blob)
    }

    // https://fetch.spec.whatwg.org/#dom-body-formdata
    fn FormData(&self) -> Rc<Promise> {
        consume_body(self, BodyType::FormData)
    }

    // https://fetch.spec.whatwg.org/#dom-body-json
    fn Json(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Json)
    }

    // https://fetch.spec.whatwg.org/#dom-body-arraybuffer
    fn ArrayBuffer(&self) -> Rc<Promise> {
        consume_body(self, BodyType::ArrayBuffer)
    }
}

fn serialize_without_fragment(url: &ServoUrl) -> &str {
    &url[..Position::AfterQuery]
}

impl Response {
    pub fn set_type(&self, new_response_type: DOMResponseType) {
        *self.response_type.borrow_mut() = new_response_type;
        self.set_response_members_by_type(new_response_type);
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

    pub fn set_redirected(&self, is_redirected: bool) {
        *self.redirected.borrow_mut() = is_redirected;
    }

    fn set_response_members_by_type(&self, response_type: DOMResponseType) {
        match response_type {
            DOMResponseType::Error => {
                *self.status.borrow_mut() = None;
                self.set_raw_status(None);
                self.set_headers(None);
            },
            DOMResponseType::Opaque => {
                *self.url_list.borrow_mut() = vec![];
                *self.status.borrow_mut() = None;
                self.set_raw_status(None);
                self.set_headers(None);
                self.body_stream.set(None);
            },
            DOMResponseType::Opaqueredirect => {
                *self.status.borrow_mut() = None;
                self.set_raw_status(None);
                self.set_headers(None);
                self.body_stream.set(None);
            },
            DOMResponseType::Default => {},
            DOMResponseType::Basic => {},
            DOMResponseType::Cors => {},
        }
    }

    pub fn set_stream_consumer(&self, sc: Option<StreamConsumer>) {
        *self.stream_consumer.borrow_mut() = sc;
    }

    pub fn stream_chunk(&self, chunk: Vec<u8>) {
        // Note, are these two actually mutually exclusive?
        if let Some(stream_consumer) = self.stream_consumer.borrow_mut().as_ref() {
            stream_consumer.consume_chunk(chunk.as_slice());
        } else if let Some(body) = self.body_stream.get() {
            body.enqueue_native(chunk);
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn finish(&self) {
        if let Some(body) = self.body_stream.get() {
            body.close_native();
        }
        if let Some(stream_consumer) = self.stream_consumer.borrow_mut().take() {
            stream_consumer.stream_end();
        }
    }
}
