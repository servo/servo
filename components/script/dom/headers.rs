/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::str::{self, FromStr};

use data_url::mime::Mime as DataUrlMime;
use dom_struct::dom_struct;
use http::header::{HeaderMap as HyperHeaders, HeaderName, HeaderValue};
use js::rust::HandleObject;
use net_traits::fetch::headers::get_value_from_header_list;
use net_traits::request::is_cors_safelisted_request_header;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HeadersBinding::{HeadersInit, HeadersMethods};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{is_token, ByteString};
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct Headers {
    reflector_: Reflector,
    guard: Cell<Guard>,
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[no_trace]
    header_list: DomRefCell<HyperHeaders>,
}

// https://fetch.spec.whatwg.org/#concept-headers-guard
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum Guard {
    Immutable,
    Request,
    RequestNoCors,
    Response,
    None,
}

impl Headers {
    pub fn new_inherited() -> Headers {
        Headers {
            reflector_: Reflector::new(),
            guard: Cell::new(Guard::None),
            header_list: DomRefCell::new(HyperHeaders::new()),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<Headers> {
        Self::new_with_proto(global, None)
    }

    fn new_with_proto(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<Headers> {
        reflect_dom_object_with_proto(Box::new(Headers::new_inherited()), global, proto)
    }

    // https://fetch.spec.whatwg.org/#dom-headers
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        init: Option<HeadersInit>,
    ) -> Fallible<DomRoot<Headers>> {
        let dom_headers_new = Headers::new_with_proto(global, proto);
        dom_headers_new.fill(init)?;
        Ok(dom_headers_new)
    }
}

impl HeadersMethods for Headers {
    // https://fetch.spec.whatwg.org/#concept-headers-append
    fn Append(&self, name: ByteString, value: ByteString) -> ErrorResult {
        // Step 1
        let value = normalize_value(value);

        // Step 2
        // https://fetch.spec.whatwg.org/#headers-validate
        let (mut valid_name, valid_value) = validate_name_and_value(name, value)?;
        valid_name = valid_name.to_lowercase();

        if self.guard.get() == Guard::Immutable {
            return Err(Error::Type("Guard is immutable".to_string()));
        }
        if self.guard.get() == Guard::Request && is_forbidden_header_name(&valid_name) {
            return Ok(());
        }
        if self.guard.get() == Guard::Response && is_forbidden_response_header(&valid_name) {
            return Ok(());
        }

        // Step 3
        if self.guard.get() == Guard::RequestNoCors {
            let tmp_value = if let Some(mut value) =
                get_value_from_header_list(&valid_name, &self.header_list.borrow())
            {
                value.extend(b", ");
                value.extend(valid_value.clone());
                value
            } else {
                valid_value.clone()
            };

            if !is_cors_safelisted_request_header(&valid_name, &tmp_value) {
                return Ok(());
            }
        }

        // Step 4
        match HeaderValue::from_bytes(&valid_value) {
            Ok(value) => {
                self.header_list
                    .borrow_mut()
                    .append(HeaderName::from_str(&valid_name).unwrap(), value);
            },
            Err(_) => {
                // can't add the header, but we don't need to panic the browser over it
                warn!(
                    "Servo thinks \"{:?}\" is a valid HTTP header value but HeaderValue doesn't.",
                    valid_value
                );
            },
        };

        // Step 5
        if self.guard.get() == Guard::RequestNoCors {
            self.remove_privileged_no_cors_request_headers();
        }

        Ok(())
    }

    // https://fetch.spec.whatwg.org/#dom-headers-delete
    fn Delete(&self, name: ByteString) -> ErrorResult {
        // Step 1
        let valid_name = validate_name(name)?;
        // Step 2
        if self.guard.get() == Guard::Immutable {
            return Err(Error::Type("Guard is immutable".to_string()));
        }
        // Step 3
        if self.guard.get() == Guard::Request && is_forbidden_header_name(&valid_name) {
            return Ok(());
        }
        // Step 4
        if self.guard.get() == Guard::RequestNoCors &&
            !is_cors_safelisted_request_header(&valid_name, &b"invalid".to_vec())
        {
            return Ok(());
        }
        // Step 5
        if self.guard.get() == Guard::Response && is_forbidden_response_header(&valid_name) {
            return Ok(());
        }
        // Step 6
        self.header_list.borrow_mut().remove(&valid_name);
        Ok(())
    }

    // https://fetch.spec.whatwg.org/#dom-headers-get
    fn Get(&self, name: ByteString) -> Fallible<Option<ByteString>> {
        // Step 1
        let valid_name = validate_name(name)?;
        Ok(
            get_value_from_header_list(&valid_name, &self.header_list.borrow())
                .map(ByteString::new),
        )
    }

    // https://fetch.spec.whatwg.org/#dom-headers-getsetcookie
    fn GetSetCookie(&self) -> Vec<ByteString> {
        self.header_list
            .borrow()
            .get_all("set-cookie")
            .iter()
            .map(|v| ByteString::new(v.as_bytes().to_vec()))
            .collect()
    }

    // https://fetch.spec.whatwg.org/#dom-headers-has
    fn Has(&self, name: ByteString) -> Fallible<bool> {
        // Step 1
        let valid_name = validate_name(name)?;
        // Step 2
        Ok(self.header_list.borrow_mut().get(&valid_name).is_some())
    }

    // https://fetch.spec.whatwg.org/#dom-headers-set
    fn Set(&self, name: ByteString, value: ByteString) -> Fallible<()> {
        // Step 1
        let value = normalize_value(value);
        // Step 2
        let (mut valid_name, valid_value) = validate_name_and_value(name, value)?;
        valid_name = valid_name.to_lowercase();
        // Step 3
        if self.guard.get() == Guard::Immutable {
            return Err(Error::Type("Guard is immutable".to_string()));
        }
        // Step 4
        if self.guard.get() == Guard::Request && is_forbidden_header_name(&valid_name) {
            return Ok(());
        }
        // Step 5
        if self.guard.get() == Guard::RequestNoCors &&
            !is_cors_safelisted_request_header(&valid_name, &valid_value)
        {
            return Ok(());
        }
        // Step 6
        if self.guard.get() == Guard::Response && is_forbidden_response_header(&valid_name) {
            return Ok(());
        }
        // Step 7
        // https://fetch.spec.whatwg.org/#concept-header-list-set
        self.header_list.borrow_mut().insert(
            HeaderName::from_str(&valid_name).unwrap(),
            HeaderValue::from_bytes(&valid_value).unwrap(),
        );
        Ok(())
    }
}

impl Headers {
    pub fn copy_from_headers(&self, headers: DomRoot<Headers>) -> ErrorResult {
        for (name, value) in headers.header_list.borrow().iter() {
            self.Append(
                ByteString::new(Vec::from(name.as_str())),
                ByteString::new(Vec::from(value.as_bytes())),
            )?;
        }
        Ok(())
    }

    // https://fetch.spec.whatwg.org/#concept-headers-fill
    pub fn fill(&self, filler: Option<HeadersInit>) -> ErrorResult {
        match filler {
            Some(HeadersInit::ByteStringSequenceSequence(v)) => {
                for mut seq in v {
                    if seq.len() == 2 {
                        let val = seq.pop().unwrap();
                        let name = seq.pop().unwrap();
                        self.Append(name, val)?;
                    } else {
                        return Err(Error::Type(
                            format!("Each header object must be a sequence of length 2 - found one with length {}",
                                    seq.len())));
                    }
                }
                Ok(())
            },
            Some(HeadersInit::ByteStringByteStringRecord(m)) => {
                for (key, value) in m.iter() {
                    self.Append(key.clone(), value.clone())?;
                }
                Ok(())
            },
            None => Ok(()),
        }
    }

    pub fn for_request(global: &GlobalScope) -> DomRoot<Headers> {
        let headers_for_request = Headers::new(global);
        headers_for_request.guard.set(Guard::Request);
        headers_for_request
    }

    pub fn for_response(global: &GlobalScope) -> DomRoot<Headers> {
        let headers_for_response = Headers::new(global);
        headers_for_response.guard.set(Guard::Response);
        headers_for_response
    }

    pub fn set_guard(&self, new_guard: Guard) {
        self.guard.set(new_guard)
    }

    pub fn get_guard(&self) -> Guard {
        self.guard.get()
    }

    pub fn set_headers(&self, hyper_headers: HyperHeaders) {
        *self.header_list.borrow_mut() = hyper_headers;
    }

    pub fn get_headers_list(&self) -> HyperHeaders {
        self.header_list.borrow_mut().clone()
    }

    // https://fetch.spec.whatwg.org/#concept-header-extract-mime-type
    pub fn extract_mime_type(&self) -> Vec<u8> {
        extract_mime_type(&self.header_list.borrow()).unwrap_or_default()
    }

    // https://fetch.spec.whatwg.org/#concept-header-list-sort-and-combine
    pub fn sort_and_combine(&self) -> Vec<(String, Vec<u8>)> {
        let borrowed_header_list = self.header_list.borrow();
        let mut header_vec = vec![];

        for name in borrowed_header_list.keys() {
            let name = name.as_str();
            if name == "set-cookie" {
                for value in borrowed_header_list.get_all(name).iter() {
                    header_vec.push((name.to_owned(), value.as_bytes().to_vec()));
                }
            } else if let Some(value) = get_value_from_header_list(name, &borrowed_header_list) {
                header_vec.push((name.to_owned(), value));
            }
        }

        header_vec.sort_by(|a, b| a.0.cmp(&b.0));
        header_vec
    }

    // https://fetch.spec.whatwg.org/#ref-for-privileged-no-cors-request-header-name
    pub fn remove_privileged_no_cors_request_headers(&self) {
        // https://fetch.spec.whatwg.org/#privileged-no-cors-request-header-name
        self.header_list.borrow_mut().remove("range");
    }
}

impl Iterable for Headers {
    type Key = ByteString;
    type Value = ByteString;

    fn get_iterable_length(&self) -> u32 {
        let sorted_header_vec = self.sort_and_combine();
        sorted_header_vec.len() as u32
    }

    fn get_value_at_index(&self, n: u32) -> ByteString {
        let sorted_header_vec = self.sort_and_combine();
        let value = sorted_header_vec[n as usize].1.clone();
        ByteString::new(value)
    }

    fn get_key_at_index(&self, n: u32) -> ByteString {
        let sorted_header_vec = self.sort_and_combine();
        let key = sorted_header_vec[n as usize].0.clone();
        ByteString::new(key.into_bytes().to_vec())
    }
}

// https://fetch.spec.whatwg.org/#forbidden-response-header-name
fn is_forbidden_response_header(name: &str) -> bool {
    matches!(name, "set-cookie" | "set-cookie2")
}

// https://fetch.spec.whatwg.org/#forbidden-header-name
pub fn is_forbidden_header_name(name: &str) -> bool {
    let disallowed_headers = [
        "accept-charset",
        "accept-encoding",
        "access-control-request-headers",
        "access-control-request-method",
        "connection",
        "content-length",
        "cookie",
        "cookie2",
        "date",
        "dnt",
        "expect",
        "host",
        "keep-alive",
        "origin",
        "referer",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
        "via",
    ];

    let disallowed_header_prefixes = ["sec-", "proxy-"];

    disallowed_headers.iter().any(|header| *header == name) ||
        disallowed_header_prefixes
            .iter()
            .any(|prefix| name.starts_with(prefix))
}

// There is some unresolved confusion over the definition of a name and a value.
//
// As of December 2019, WHATWG has no formal grammar production for value;
// https://fetch.spec.whatg.org/#concept-header-value just says not to have
// newlines, nulls, or leading/trailing whitespace. It even allows
// octets that aren't a valid UTF-8 encoding, and WPT tests reflect this.
// The HeaderValue class does not fully reflect this, so headers
// containing bytes with values 1..31 or 127 can't be created, failing
// WPT tests but probably not affecting anything important on the real Internet.
fn validate_name_and_value(name: ByteString, value: ByteString) -> Fallible<(String, Vec<u8>)> {
    let valid_name = validate_name(name)?;
    if !is_legal_header_value(&value) {
        return Err(Error::Type("Header value is not valid".to_string()));
    }
    Ok((valid_name, value.into()))
}

fn validate_name(name: ByteString) -> Fallible<String> {
    if !is_field_name(&name) {
        return Err(Error::Type("Name is not valid".to_string()));
    }
    match String::from_utf8(name.into()) {
        Ok(ns) => Ok(ns),
        _ => Err(Error::Type("Non-UTF8 header name found".to_string())),
    }
}

// Removes trailing and leading HTTP whitespace bytes.
// https://fetch.spec.whatwg.org/#concept-header-value-normalize
pub fn normalize_value(value: ByteString) -> ByteString {
    match (
        index_of_first_non_whitespace(&value),
        index_of_last_non_whitespace(&value),
    ) {
        (Some(begin), Some(end)) => ByteString::new(value[begin..end + 1].to_owned()),
        _ => ByteString::new(vec![]),
    }
}

fn is_http_whitespace(byte: u8) -> bool {
    byte == b'\t' || byte == b'\n' || byte == b'\r' || byte == b' '
}

fn index_of_first_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate() {
        if !is_http_whitespace(byte) {
            return Some(index);
        }
    }
    None
}

fn index_of_last_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate().rev() {
        if !is_http_whitespace(byte) {
            return Some(index);
        }
    }
    None
}

// http://tools.ietf.org/html/rfc7230#section-3.2
fn is_field_name(name: &ByteString) -> bool {
    is_token(name)
}

// https://fetch.spec.whatg.org/#concept-header-value
fn is_legal_header_value(value: &ByteString) -> bool {
    let value_len = value.len();
    if value_len == 0 {
        return true;
    }
    match value[0] {
        b' ' | b'\t' => return false,
        _ => {},
    };
    match value[value_len - 1] {
        b' ' | b'\t' => return false,
        _ => {},
    };
    for &ch in &value[..] {
        match ch {
            b'\0' | b'\n' | b'\r' => return false,
            _ => {},
        }
    }
    true
    // If accepting non-UTF8 header values causes breakage,
    // removing the above "true" and uncommenting the below code
    // would ameliorate it while still accepting most reasonable headers:
    //match str::from_utf8(value) {
    //    Ok(_) => true,
    //    Err(_) => {
    //        warn!(
    //            "Rejecting spec-legal but non-UTF8 header value: {:?}",
    //            value
    //        );
    //        false
    //    },
    // }
}

// https://tools.ietf.org/html/rfc5234#appendix-B.1
pub fn is_vchar(x: u8) -> bool {
    matches!(x, 0x21..=0x7E)
}

// http://tools.ietf.org/html/rfc7230#section-3.2.6
pub fn is_obs_text(x: u8) -> bool {
    matches!(x, 0x80..=0xFF)
}

// https://fetch.spec.whatwg.org/#concept-header-extract-mime-type
// This function uses data_url::Mime to parse the MIME Type because
// mime::Mime does not provide a parser following the Fetch spec
// see https://github.com/hyperium/mime/issues/106
pub fn extract_mime_type(headers: &HyperHeaders) -> Option<Vec<u8>> {
    let mut charset: Option<String> = None;
    let mut essence: String = "".to_string();
    let mut mime_type: Option<DataUrlMime> = None;

    // Step 4
    let headers_values = headers.get_all(http::header::CONTENT_TYPE).iter();

    // Step 5
    if headers_values.size_hint() == (0, Some(0)) {
        return None;
    }

    // Step 6
    for header_value in headers_values {
        // Step 6.1
        match DataUrlMime::from_str(header_value.to_str().unwrap_or("")) {
            // Step 6.2
            Err(_) => continue,
            Ok(temp_mime) => {
                let temp_essence = format!("{}/{}", temp_mime.type_, temp_mime.subtype);

                // Step 6.2
                if temp_essence == "*/*" {
                    continue;
                }

                let temp_charset = &temp_mime.get_parameter("charset");

                // Step 6.3
                mime_type = Some(DataUrlMime {
                    type_: temp_mime.type_.to_string(),
                    subtype: temp_mime.subtype.to_string(),
                    parameters: temp_mime.parameters.clone(),
                });

                // Step 6.4
                if temp_essence != essence {
                    charset = temp_charset.map(|c| c.to_string());
                    temp_essence.clone_into(&mut essence);
                } else {
                    // Step 6.5
                    if temp_charset.is_none() && charset.is_some() {
                        let DataUrlMime {
                            type_: t,
                            subtype: st,
                            parameters: p,
                        } = mime_type.unwrap();
                        let mut params = p;
                        params.push(("charset".to_string(), charset.clone().unwrap()));
                        mime_type = Some(DataUrlMime {
                            type_: t.to_string(),
                            subtype: st.to_string(),
                            parameters: params,
                        })
                    }
                }
            },
        }
    }

    // Step 7, 8
    mime_type.map(|m| format!("{}", m).into_bytes())
}
