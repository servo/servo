/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::str::{self, FromStr};

use dom_struct::dom_struct;
use http::header::{HeaderMap as HyperHeaders, HeaderName, HeaderValue};
use js::rust::HandleObject;
use net_traits::fetch::headers::{
    extract_mime_type, get_decode_and_split_header_value, get_value_from_header_list,
    is_forbidden_method,
};
use net_traits::request::is_cors_safelisted_request_header;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HeadersBinding::{HeadersInit, HeadersMethods};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{ByteString, is_token};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Headers {
    reflector_: Reflector,
    guard: Cell<Guard>,
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[no_trace]
    header_list: DomRefCell<HyperHeaders>,
}

// https://fetch.spec.whatwg.org/#concept-headers-guard
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum Guard {
    Immutable,
    Request,
    RequestNoCors,
    Response,
    None,
}

impl Headers {
    pub(crate) fn new_inherited() -> Headers {
        Headers {
            reflector_: Reflector::new(),
            guard: Cell::new(Guard::None),
            header_list: DomRefCell::new(HyperHeaders::new()),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Headers> {
        Self::new_with_proto(global, None, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Headers> {
        reflect_dom_object_with_proto(Box::new(Headers::new_inherited()), global, proto, can_gc)
    }
}

impl HeadersMethods<crate::DomTypeHolder> for Headers {
    // https://fetch.spec.whatwg.org/#dom-headers
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        init: Option<HeadersInit>,
    ) -> Fallible<DomRoot<Headers>> {
        let dom_headers_new = Headers::new_with_proto(global, proto, can_gc);
        dom_headers_new.fill(init)?;
        Ok(dom_headers_new)
    }

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
        if self.guard.get() == Guard::Request &&
            is_forbidden_request_header(&valid_name, &valid_value)
        {
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
        let (mut valid_name, valid_value) = validate_name_and_value(name, ByteString::new(vec![]))?;

        valid_name = valid_name.to_lowercase();

        // Step 2
        if self.guard.get() == Guard::Immutable {
            return Err(Error::Type("Guard is immutable".to_string()));
        }
        // Step 3
        if self.guard.get() == Guard::Request &&
            is_forbidden_request_header(&valid_name, &valid_value)
        {
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
        if self.guard.get() == Guard::Request &&
            is_forbidden_request_header(&valid_name, &valid_value)
        {
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
        match HeaderValue::from_bytes(&valid_value) {
            Ok(value) => {
                self.header_list
                    .borrow_mut()
                    .insert(HeaderName::from_str(&valid_name).unwrap(), value);
            },
            Err(_) => {
                // can't add the header, but we don't need to panic the browser over it
                warn!(
                    "Servo thinks \"{:?}\" is a valid HTTP header value but HeaderValue doesn't.",
                    valid_value
                );
            },
        };
        Ok(())
    }
}

impl Headers {
    pub(crate) fn copy_from_headers(&self, headers: DomRoot<Headers>) -> ErrorResult {
        for (name, value) in headers.header_list.borrow().iter() {
            self.Append(
                ByteString::new(Vec::from(name.as_str())),
                ByteString::new(Vec::from(value.as_bytes())),
            )?;
        }
        Ok(())
    }

    // https://fetch.spec.whatwg.org/#concept-headers-fill
    pub(crate) fn fill(&self, filler: Option<HeadersInit>) -> ErrorResult {
        match filler {
            Some(HeadersInit::ByteStringSequenceSequence(v)) => {
                for mut seq in v {
                    if seq.len() == 2 {
                        let val = seq.pop().unwrap();
                        let name = seq.pop().unwrap();
                        self.Append(name, val)?;
                    } else {
                        return Err(Error::Type(format!(
                            "Each header object must be a sequence of length 2 - found one with length {}",
                            seq.len()
                        )));
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

    pub(crate) fn for_request(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Headers> {
        let headers_for_request = Headers::new(global, can_gc);
        headers_for_request.guard.set(Guard::Request);
        headers_for_request
    }

    pub(crate) fn for_response(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Headers> {
        let headers_for_response = Headers::new(global, can_gc);
        headers_for_response.guard.set(Guard::Response);
        headers_for_response
    }

    pub(crate) fn set_guard(&self, new_guard: Guard) {
        self.guard.set(new_guard)
    }

    pub(crate) fn get_guard(&self) -> Guard {
        self.guard.get()
    }

    pub(crate) fn set_headers(&self, hyper_headers: HyperHeaders) {
        *self.header_list.borrow_mut() = hyper_headers;
    }

    pub(crate) fn get_headers_list(&self) -> HyperHeaders {
        self.header_list.borrow_mut().clone()
    }

    // https://fetch.spec.whatwg.org/#concept-header-extract-mime-type
    pub(crate) fn extract_mime_type(&self) -> Vec<u8> {
        extract_mime_type(&self.header_list.borrow()).unwrap_or_default()
    }

    // https://fetch.spec.whatwg.org/#concept-header-list-sort-and-combine
    pub(crate) fn sort_and_combine(&self) -> Vec<(String, Vec<u8>)> {
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
    pub(crate) fn remove_privileged_no_cors_request_headers(&self) {
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

/// This function will internally convert `name` to lowercase for matching, so explicitly converting
/// before calling is not necessary
///
/// <https://fetch.spec.whatwg.org/#forbidden-request-header>
pub(crate) fn is_forbidden_request_header(name: &str, value: &[u8]) -> bool {
    let forbidden_header_names = [
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
        // This list is defined in the fetch spec, however the draft spec for private-network-access
        // proposes this additional forbidden name, which is currently included in WPT tests. See:
        // https://wicg.github.io/private-network-access/#forbidden-header-names
        "access-control-request-private-network",
    ];

    // Step 1: If name is a byte-case-insensitive match for one of (forbidden_header_names), return
    // true
    let lowercase_name = name.to_lowercase();

    if forbidden_header_names
        .iter()
        .any(|header| *header == lowercase_name.as_str())
    {
        return true;
    }

    let forbidden_header_prefixes = ["sec-", "proxy-"];

    // Step 2: If name when byte-lowercased starts with `proxy-` or `sec-`, then return true.
    if forbidden_header_prefixes
        .iter()
        .any(|prefix| lowercase_name.starts_with(prefix))
    {
        return true;
    }

    let potentially_forbidden_header_names = [
        "x-http-method",
        "x-http-method-override",
        "x-method-override",
    ];

    // Step 3: If name is a byte-case-insensitive match for one of (potentially_forbidden_header_names)
    if potentially_forbidden_header_names
        .iter()
        .any(|header| *header == lowercase_name)
    {
        // Step 3.1: Let parsedValues be the result of getting, decoding, and splitting value.
        let parsed_values = get_decode_and_split_header_value(value.to_vec());

        // Step 3.2: For each method of parsedValues: if the isomorphic encoding of method is a
        // forbidden method, then return true.
        return parsed_values
            .iter()
            .any(|s| is_forbidden_method(s.as_bytes()));
    }

    // Step 4: Return false.
    false
}

// https://fetch.spec.whatwg.org/#forbidden-response-header-name
fn is_forbidden_response_header(name: &str) -> bool {
    matches!(name, "set-cookie" | "set-cookie2")
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
pub(crate) fn is_vchar(x: u8) -> bool {
    matches!(x, 0x21..=0x7E)
}

// http://tools.ietf.org/html/rfc7230#section-3.2.6
pub(crate) fn is_obs_text(x: u8) -> bool {
    matches!(x, 0x80..=0xFF)
}
