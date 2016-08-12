/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HeadersBinding;
use dom::bindings::codegen::Bindings::HeadersBinding::HeadersMethods;
use dom::bindings::codegen::UnionTypes::HeadersOrByteStringSequenceSequence;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, is_token};
use hyper::header::Headers as HyperHeaders;
use std::cell::Cell;
use std::result::Result;

#[dom_struct]
pub struct Headers {
    reflector_: Reflector,
    guard: Cell<Guard>,
    #[ignore_heap_size_of = "Defined in hyper"]
    header_list: DOMRefCell<HyperHeaders>
}

// https://fetch.spec.whatwg.org/#concept-headers-guard
#[derive(Copy, Clone, JSTraceable, HeapSizeOf, PartialEq)]
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
            header_list: DOMRefCell::new(HyperHeaders::new()),
        }
    }

    pub fn new(global: GlobalRef) -> Root<Headers> {
        reflect_dom_object(box Headers::new_inherited(), global, HeadersBinding::Wrap)
    }

    // https://fetch.spec.whatwg.org/#dom-headers
    pub fn Constructor(global: GlobalRef, init: Option<HeadersBinding::HeadersInit>)
                       -> Fallible<Root<Headers>> {
        let dom_headers_new = Headers::new(global);
        try!(dom_headers_new.fill(init));
        Ok(dom_headers_new)
    }
}

impl HeadersMethods for Headers {
    // https://fetch.spec.whatwg.org/#concept-headers-append
    fn Append(&self, name: ByteString, value: ByteString) -> ErrorResult {
        // Step 1
        let value = normalize_value(value);
        // Step 2
        let (mut valid_name, valid_value) = try!(validate_name_and_value(name, value));
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
        if self.guard.get() == Guard::RequestNoCors && !is_cors_safelisted_request_header(&valid_name) {
            return Ok(());
        }
        // Step 6
        if self.guard.get() == Guard::Response && is_forbidden_response_header(&valid_name) {
            return Ok(());
        }
        // Step 7
        let mut combined_value: Vec<u8> = vec![];
        if let Some(v) = self.header_list.borrow().get_raw(&valid_name) {
            combined_value = v[0].clone();
            combined_value.push(b","[0]);
        }
        combined_value.extend(valid_value.iter().cloned());
        self.header_list.borrow_mut().set_raw(valid_name, vec![combined_value]);
        Ok(())
    }

    // https://fetch.spec.whatwg.org/#dom-headers-delete
    fn Delete(&self, name: ByteString) -> ErrorResult {
        // Step 1
        let valid_name = try!(validate_name(name));
        // Step 2
        if self.guard.get() == Guard::Immutable {
            return Err(Error::Type("Guard is immutable".to_string()));
        }
        // Step 3
        if self.guard.get() == Guard::Request && is_forbidden_header_name(&valid_name) {
            return Ok(());
        }
        // Step 4
        if self.guard.get() == Guard::RequestNoCors && !is_cors_safelisted_request_header(&valid_name) {
            return Ok(());
        }
        // Step 5
        if self.guard.get() == Guard::Response && is_forbidden_response_header(&valid_name) {
            return Ok(());
        }
        // Step 6
        self.header_list.borrow_mut().remove_raw(&valid_name);
        Ok(())
    }

    // https://fetch.spec.whatwg.org/#dom-headers-get
    fn Get(&self, name: ByteString) -> Fallible<Option<ByteString>> {
        // Step 1
        let valid_name = &try!(validate_name(name));
        Ok(self.header_list.borrow().get_raw(&valid_name).map(|v| {
            ByteString::new(v[0].clone())
        }))
    }

    // https://fetch.spec.whatwg.org/#dom-headers-has
    fn Has(&self, name: ByteString) -> Fallible<bool> {
        // Step 1
        let valid_name = try!(validate_name(name));
        // Step 2
        Ok(self.header_list.borrow_mut().get_raw(&valid_name).is_some())
    }

    // https://fetch.spec.whatwg.org/#dom-headers-set
    fn Set(&self, name: ByteString, value: ByteString) -> Fallible<()> {
        // Step 1
        let value = normalize_value(value);
        // Step 2
        let (mut valid_name, valid_value) = try!(validate_name_and_value(name, value));
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
        if self.guard.get() == Guard::RequestNoCors && !is_cors_safelisted_request_header(&valid_name) {
            return Ok(());
        }
        // Step 6
        if self.guard.get() == Guard::Response && is_forbidden_response_header(&valid_name) {
            return Ok(());
        }
        // Step 7
        // https://fetch.spec.whatwg.org/#concept-header-list-set
        self.header_list.borrow_mut().set_raw(valid_name, vec![valid_value]);
        Ok(())
    }
}

impl Headers {
    // https://fetch.spec.whatwg.org/#concept-headers-fill
    pub fn fill(&self, filler: Option<HeadersBinding::HeadersInit>) -> ErrorResult {
        match filler {
            // Step 1
            Some(HeadersOrByteStringSequenceSequence::Headers(h)) => {
                for header in h.header_list.borrow().iter() {
                    try!(self.Append(
                        ByteString::new(Vec::from(header.name())),
                        ByteString::new(Vec::from(header.value_string().into_bytes()))
                    ));
                }
                Ok(())
            },
            // Step 2
            Some(HeadersOrByteStringSequenceSequence::ByteStringSequenceSequence(v)) => {
                for mut seq in v {
                    if seq.len() == 2 {
                        let val = seq.pop().unwrap();
                        let name = seq.pop().unwrap();
                        try!(self.Append(name, val));
                    } else {
                        return Err(Error::Type(
                            format!("Each header object must be a sequence of length 2 - found one with length {}",
                                    seq.len())));
                    }
                }
                Ok(())
            },
            // Step 3 TODO constructor for when init is an open-ended dictionary
            None => Ok(()),
        }
    }

    pub fn for_request(global: GlobalRef) -> Root<Headers> {
        let headers_for_request = Headers::new(global);
        headers_for_request.guard.set(Guard::Request);
        headers_for_request
    }

     pub fn set_guard(&self, new_guard: Guard) {
        self.guard.set(new_guard)
    }

    pub fn get_guard(&self) -> Guard {
        self.guard.get()
    }

    pub fn empty_header_list(&self) {
        *self.header_list.borrow_mut() = HyperHeaders::new();
    }

    // https://fetch.spec.whatwg.org/#concept-header-extract-mime-type
    pub fn extract_mime_type(&self) -> Vec<u8> {
        self.header_list.borrow().get_raw("content-type").map_or(vec![], |v| v[0].clone())
    }
}

// TODO
// "Content-Type" once parsed, the value should be
// `application/x-www-form-urlencoded`, `multipart/form-data`,
// or `text/plain`.
// "DPR", "Downlink", "Save-Data", "Viewport-Width", "Width":
// once parsed, the value should not be failure.
// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
fn is_cors_safelisted_request_header(name: &str) -> bool {
    match name {
        "accept" |
        "accept-language" |
        "content-language" => true,
        _ => false,
    }
}

// https://fetch.spec.whatwg.org/#forbidden-response-header-name
fn is_forbidden_response_header(name: &str) -> bool {
    match name {
        "set-cookie" |
        "set-cookie2"  => true,
        _ => false,
    }
}

// https://fetch.spec.whatwg.org/#forbidden-header-name
pub fn is_forbidden_header_name(name: &str) -> bool {
    let disallowed_headers =
        ["accept-charset", "accept-encoding",
         "access-control-request-headers",
         "access-control-request-method",
         "connection", "content-length",
         "cookie", "cookie2", "date", "dnt",
         "expect", "host", "keep-alive", "origin",
         "referer", "te", "trailer", "transfer-encoding",
         "upgrade", "via"];

    let disallowed_header_prefixes = ["sec-", "proxy-"];

    disallowed_headers.iter().any(|header| *header == name) ||
        disallowed_header_prefixes.iter().any(|prefix| name.starts_with(prefix))
}

// There is some unresolved confusion over the definition of a name and a value.
// The fetch spec [1] defines a name as "a case-insensitive byte
// sequence that matches the field-name token production. The token
// productions are viewable in [2]." A field-name is defined as a
// token, which is defined in [3].
// ISSUE 1:
// It defines a value as "a byte sequence that matches the field-content token production."
// To note, there is a difference between field-content and
// field-value (which is made up of field-content and obs-fold). The
// current definition does not allow for obs-fold (which are white
// space and newlines) in values. So perhaps a value should be defined
// as "a byte sequence that matches the field-value token production."
// However, this would then allow values made up entirely of white space and newlines.
// RELATED ISSUE 2:
// According to a previously filed Errata ID: 4189 in [4], "the
// specified field-value rule does not allow single field-vchar
// surrounded by whitespace anywhere". They provided a fix for the
// field-content production, but ISSUE 1 has still not been resolved.
// The production definitions likely need to be re-written.
// [1] https://fetch.spec.whatwg.org/#concept-header-value
// [2] https://tools.ietf.org/html/rfc7230#section-3.2
// [3] https://tools.ietf.org/html/rfc7230#section-3.2.6
// [4] https://www.rfc-editor.org/errata_search.php?rfc=7230
fn validate_name_and_value(name: ByteString, value: ByteString)
                           -> Result<(String, Vec<u8>), Error> {
    let valid_name = try!(validate_name(name));
    if !is_field_content(&value) {
        return Err(Error::Type("Value is not valid".to_string()));
    }
    Ok((valid_name, value.into()))
}

fn validate_name(name: ByteString) -> Result<String, Error> {
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
    match (index_of_first_non_whitespace(&value), index_of_last_non_whitespace(&value)) {
        (Some(begin), Some(end)) => ByteString::new(value[begin..end + 1].to_owned()),
        _ => ByteString::new(vec![]),
    }
}

fn is_HTTP_whitespace(byte: u8) -> bool {
    byte == b'\t' || byte == b'\n' || byte == b'\r' || byte == b' '
}

fn index_of_first_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate() {
        if !is_HTTP_whitespace(byte) {
            return Some(index);
        }
    }
    None
}

fn index_of_last_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate().rev() {
        if !is_HTTP_whitespace(byte) {
            return Some(index);
        }
    }
    None
}

// http://tools.ietf.org/html/rfc7230#section-3.2
fn is_field_name(name: &ByteString) -> bool {
    is_token(&*name)
}

// https://tools.ietf.org/html/rfc7230#section-3.2
// http://www.rfc-editor.org/errata_search.php?rfc=7230
// Errata ID: 4189
// field-content = field-vchar [ 1*( SP / HTAB / field-vchar )
//                               field-vchar ]
fn is_field_content(value: &ByteString) -> bool {
    let value_len = value.len();

    if value_len == 0 {
        return false;
    }
    if !is_field_vchar(value[0]) {
        return false;
    }

    if value_len > 2 {
        for &ch in &value[1..value_len - 1] {
            if !is_field_vchar(ch) && !is_space(ch) && !is_htab(ch) {
                return false;
            }
        }
    }

    if !is_field_vchar(value[value_len - 1]) {
        return false;
    }

    return true;
}

fn is_space(x: u8) -> bool {
    x == b' '
}

fn is_htab(x: u8) -> bool {
    x == b'\t'
}

// https://tools.ietf.org/html/rfc7230#section-3.2
fn is_field_vchar(x: u8) -> bool {
    is_vchar(x) || is_obs_text(x)
}

// https://tools.ietf.org/html/rfc5234#appendix-B.1
fn is_vchar(x: u8) -> bool {
    match x {
        0x21...0x7E => true,
        _ => false,
    }
}

// http://tools.ietf.org/html/rfc7230#section-3.2.6
fn is_obs_text(x: u8) -> bool {
    match x {
        0x80...0xFF => true,
        _ => false,
    }
}
