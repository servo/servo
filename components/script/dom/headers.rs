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
use net_traits::trim_http_whitespace;

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

/// <https://fetch.spec.whatwg.org/#concept-headers-guard>
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
    /// <https://fetch.spec.whatwg.org/#dom-headers>
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

    /// <https://fetch.spec.whatwg.org/#concept-headers-append>
    fn Append(&self, name: ByteString, value: ByteString) -> ErrorResult {
        // 1. Normalize value.
        let value = trim_http_whitespace(&value);

        // 2. If validating (name, value) for headers returns false, then return.
        let Some((mut valid_name, valid_value)) =
            self.validate_name_and_value(name, ByteString::new(value.into()))?
        else {
            return Ok(());
        };

        valid_name = valid_name.to_lowercase();

        // 3. If headers’s guard is "request-no-cors":
        if self.guard.get() == Guard::RequestNoCors {
            // 3.1. Let temporaryValue be the result of getting name from headers’s header list.
            let tmp_value = if let Some(mut value) =
                get_value_from_header_list(&valid_name, &self.header_list.borrow())
            {
                // 3.3. Otherwise, set temporaryValue to temporaryValue, followed by 0x2C 0x20, followed by value.
                value.extend(b", ");
                value.extend(valid_value.to_vec());
                value
            } else {
                // 3.2. If temporaryValue is null, then set temporaryValue to value.
                valid_value.to_vec()
            };
            // 3.4. If (name, temporaryValue) is not a no-CORS-safelisted request-header, then return.
            if !is_cors_safelisted_request_header(&valid_name, &tmp_value) {
                return Ok(());
            }
        }

        // 4. Append (name, value) to headers’s header list.
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

        // 5. If headers’s guard is "request-no-cors", then remove privileged no-CORS request-headers from headers.
        if self.guard.get() == Guard::RequestNoCors {
            self.remove_privileged_no_cors_request_headers();
        }

        Ok(())
    }

    /// <https://fetch.spec.whatwg.org/#dom-headers-delete>
    fn Delete(&self, name: ByteString) -> ErrorResult {
        // Step 1 If validating (name, ``) for this returns false, then return.
        let name_and_value = self.validate_name_and_value(name, ByteString::new(vec![]))?;
        let Some((mut valid_name, _valid_value)) = name_and_value else {
            return Ok(());
        };

        valid_name = valid_name.to_lowercase();

        // Step 2 If this’s guard is "request-no-cors", name is not a no-CORS-safelisted request-header name,
        // and name is not a privileged no-CORS request-header name, then return.
        if self.guard.get() == Guard::RequestNoCors &&
            !is_cors_safelisted_request_header(&valid_name, &b"invalid".to_vec())
        {
            return Ok(());
        }

        // 3. If this’s header list does not contain name, then return.
        // 4. Delete name from this’s header list.
        self.header_list.borrow_mut().remove(valid_name);

        // 5. If this’s guard is "request-no-cors", then remove privileged no-CORS request-headers from this.
        if self.guard.get() == Guard::RequestNoCors {
            self.remove_privileged_no_cors_request_headers();
        }

        Ok(())
    }

    /// <https://fetch.spec.whatwg.org/#dom-headers-get>
    fn Get(&self, name: ByteString) -> Fallible<Option<ByteString>> {
        // 1. If name is not a header name, then throw a TypeError.
        let valid_name = validate_name(name)?;

        // 2. Return the result of getting name from this’s header list.
        Ok(
            get_value_from_header_list(&valid_name, &self.header_list.borrow())
                .map(ByteString::new),
        )
    }

    /// <https://fetch.spec.whatwg.org/#dom-headers-getsetcookie>
    fn GetSetCookie(&self) -> Vec<ByteString> {
        // 1. If this’s header list does not contain `Set-Cookie`, then return « ».
        // 2. Return the values of all headers in this’s header list whose name is a
        // byte-case-insensitive match for `Set-Cookie`, in order.
        self.header_list
            .borrow()
            .get_all("set-cookie")
            .iter()
            .map(|v| ByteString::new(v.as_bytes().to_vec()))
            .collect()
    }

    /// <https://fetch.spec.whatwg.org/#dom-headers-has>
    fn Has(&self, name: ByteString) -> Fallible<bool> {
        // 1. If name is not a header name, then throw a TypeError.
        let valid_name = validate_name(name)?;
        // 2. Return true if this’s header list contains name; otherwise false.
        Ok(self.header_list.borrow_mut().get(&valid_name).is_some())
    }

    /// <https://fetch.spec.whatwg.org/#dom-headers-set>
    fn Set(&self, name: ByteString, value: ByteString) -> Fallible<()> {
        // 1. Normalize value
        let value = trim_http_whitespace(&value);

        // 2. If validating (name, value) for this returns false, then return.
        let Some((mut valid_name, valid_value)) =
            self.validate_name_and_value(name, ByteString::new(value.into()))?
        else {
            return Ok(());
        };
        valid_name = valid_name.to_lowercase();

        // 3. If this’s guard is "request-no-cors" and (name, value) is not a
        // no-CORS-safelisted request-header, then return.
        if self.guard.get() == Guard::RequestNoCors &&
            !is_cors_safelisted_request_header(&valid_name, &valid_value.to_vec())
        {
            return Ok(());
        }

        // 4. Set (name, value) in this’s header list.
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

        // 5. If this’s guard is "request-no-cors", then remove privileged no-CORS request-headers from this.
        if self.guard.get() == Guard::RequestNoCors {
            self.remove_privileged_no_cors_request_headers();
        }

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

    /// <https://fetch.spec.whatwg.org/#concept-headers-fill>
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

    /// <https://fetch.spec.whatwg.org/#concept-header-extract-mime-type>
    pub(crate) fn extract_mime_type(&self) -> Vec<u8> {
        extract_mime_type(&self.header_list.borrow()).unwrap_or_default()
    }

    /// <https://fetch.spec.whatwg.org/#concept-header-list-sort-and-combine>
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

    /// <https://fetch.spec.whatwg.org/#ref-for-privileged-no-cors-request-header-name>
    pub(crate) fn remove_privileged_no_cors_request_headers(&self) {
        // <https://fetch.spec.whatwg.org/#privileged-no-cors-request-header-name>
        self.header_list.borrow_mut().remove("range");
    }

    /// <https://fetch.spec.whatwg.org/#headers-validate>
    pub(crate) fn validate_name_and_value(
        &self,
        name: ByteString,
        value: ByteString,
    ) -> Fallible<Option<(String, ByteString)>> {
        // 1. If name is not a header name or value is not a header value, then throw a TypeError.
        let valid_name = validate_name(name)?;
        if !is_legal_header_value(&value) {
            return Err(Error::Type("Header value is not valid".to_string()));
        }
        // 2. If headers’s guard is "immutable", then throw a TypeError.
        if self.guard.get() == Guard::Immutable {
            return Err(Error::Type("Guard is immutable".to_string()));
        }
        // 3. If headers’s guard is "request" and (name, value) is a forbidden request-header, then return false.
        if self.guard.get() == Guard::Request && is_forbidden_request_header(&valid_name, &value) {
            return Ok(None);
        }
        // 4. If headers’s guard is "response" and name is a forbidden response-header name, then return false.
        if self.guard.get() == Guard::Response && is_forbidden_response_header(&valid_name) {
            return Ok(None);
        }

        Ok(Some((valid_name, value)))
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
        "set-cookie",
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

/// <https://fetch.spec.whatwg.org/#forbidden-response-header-name>
fn is_forbidden_response_header(name: &str) -> bool {
    // A forbidden response-header name is a header name that is a byte-case-insensitive match for one of
    let name = name.to_ascii_lowercase();
    matches!(name.as_str(), "set-cookie" | "set-cookie2")
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

/// <http://tools.ietf.org/html/rfc7230#section-3.2>
fn is_field_name(name: &ByteString) -> bool {
    is_token(name)
}

// As of December 2019, WHATWG has no formal grammar production for value;
// https://fetch.spec.whatg.org/#concept-header-value just says not to have
// newlines, nulls, or leading/trailing whitespace. It even allows
// octets that aren't a valid UTF-8 encoding, and WPT tests reflect this.
// The HeaderValue class does not fully reflect this, so headers
// containing bytes with values 1..31 or 127 can't be created, failing
// WPT tests but probably not affecting anything important on the real Internet.
/// <https://fetch.spec.whatg.org/#concept-header-value>
fn is_legal_header_value(value: &[u8]) -> bool {
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
    for &ch in value {
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

/// <https://tools.ietf.org/html/rfc5234#appendix-B.1>
pub(crate) fn is_vchar(x: u8) -> bool {
    matches!(x, 0x21..=0x7E)
}

/// <http://tools.ietf.org/html/rfc7230#section-3.2.6>
pub(crate) fn is_obs_text(x: u8) -> bool {
    matches!(x, 0x80..=0xFF)
}
