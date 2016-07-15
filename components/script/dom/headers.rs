use dom::bindings::str::{ByteString, is_token};
use std::result::Result;
use dom::bindings::error::Error;
use hyper;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::reflector::Reflector;

#[dom_struct]
pub struct Headers {
    reflector_: Reflector,
    guard: Guard,
    #[ignore_heap_size_of = "Defined in hyper"]
    header_list: DOMRefCell<hyper::header::Headers>
}

#[derive(PartialEq)]
enum Group {
    CorsSafelistedRequestHeader,
    CorsNonwildcardRequestHeader,
    CorsSafelistedResponseHeader,
    Forbidden,
    ForbiddenResponseHeader,
}

#[derive(JSTraceable, HeapSizeOf)]
enum Guard {
    Immutable,
    Request,
    RequestNoCors,
    Response,
    None,
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            reflector_: Reflector::new(),
            guard: Guard::None,
            header_list: DOMRefCell::new(hyper::header::Headers::new()),
        }
    }

    // https://fetch.spec.whatwg.org/#concept-headers-append
    pub fn Append(&self, name: ByteString, value: ByteString) -> Result<(), Error> {
        // 1) Normalize value
        let value = normalize_value(value);

        // 2) If name is not a name or value is not a value, throw a TypeError.
        try!(validate_name_and_value(&name, &value));

        // 3-6) Check if name and value are legal according to guard.
        // 3) If guard is "immutable", throw a TypeError.
        // 4) Otherwise, if guard is "request" and name is a forbidden header name, return.
        // 5) Otherwise, if guard is "request-no-cors"
        //    and name/value is not a CORS-safelisted request-header, return.
        // 6) Otherwise, if guard is "response" and
        //    name is a forbidden response-header name, return.

        let name_string: String;
        match String::from_utf8(name.into()) {
            Ok(ns) => name_string = ns,
            Err(_) => return Err(Error::Type(String::from("Non-UTF8 header name found"))),
        }
        let group = try!(find_group(&name_string));

        match check_guard_against_header_group(&self.guard, group) {
            Ok(true) => {
                // 7) Append name/value to header list.
                self.header_list.borrow_mut().set_raw(name_string, vec![value.into()]);
                Ok(())
            }
            Ok(false) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

// TODO
// "Content-Type" once parsed, the value should be
// `application/x-www-form-urlencoded`, `multipart/form-data`, or
// `text/plain`
// "DPR", "Downlink", "Save-Data", "Viewport-Width", "Width": once
// parsed, the value should not be failure
fn is_cors_safelisted_request(name: &str) -> bool {
    match name {
        "accept" |
        "accept-language" |
        "content-language" => true,
        _ => false,
    }
}

fn is_cors_non_wildcard_request(name: &str) -> bool {
    match name {
        "authorization" => true,
        _ => false,
    }
}

// TODO
// given a CORS-exposed header-name list list, is a header name that is
// one of the following:
// Cache-Control, Content-Language,
// Content-Type, Expires, Last-Modified, Pragma
// Any value in list that is not a forbidden response-header name.
fn is_cors_safelisted_response(name: &str) -> bool {
    match name {
        "cache-control" |
        "content-language" |
        "content-type" |
        "expires" |
        "last-modified" |
        "pragma" => true,
        _ => false,
    }
}

fn is_forbidden_response(name: &str) -> bool {
    match name {
        "set-cookie" |
        "set-cookie2"  => true,
        _ => false,
    }
}

fn is_forbidden(name: &str) -> bool {
    if name.starts_with("proxy-") ||
        name.starts_with("sec-") {
            true
        } else {
            match name {
                "accept-charset" |
                "accept-encoding" |
                "access-control-request-headers" |
                "access-control-request-method" |
                "connection" |
                "content-length" |
                "cookie" |
                "cookie2" |
                "date" |
                "dnt" |
                "expect" |
                "host" |
                "keep-alive" |
                "origin" |
                "referer" |
                "te" |
                "trailer" |
                "transfer-encoding" |
                "upgrade" |
                "via" => true,
                _ => false,
            }
        }
}

fn find_group(name: &str) -> Result<Group, Error> {
    if is_cors_safelisted_request(&name) {
        Ok(Group::CorsSafelistedRequestHeader)
    } else if is_cors_non_wildcard_request(&name) {
        Ok(Group::CorsNonwildcardRequestHeader)
    } else if is_cors_safelisted_response(&name) {
        Ok(Group::CorsSafelistedResponseHeader)
    } else if is_forbidden_response(&name) {
        Ok(Group::ForbiddenResponseHeader)
    } else if is_forbidden(&name) {
        Ok(Group::Forbidden)
    } else {
        Err(Error::Type(String::from("Name does not have a group")))
    }
}

fn validate_name_and_value(name: &ByteString, value: &ByteString) -> Result<(), Error> {
    if is_field_name(name) && is_field_content(value) {
        Ok(())
    } else {
        Err(Error::Type(String::from("Name and/or Value are not valid")))
    }
}

// Checks the guard and the name/value's group to see if the combination is legal
fn check_guard_against_header_group(guard: &Guard, group: Group) -> Result<bool, Error> {
    match (guard, group) {
        (&Guard::Immutable, _) =>
            Err(Error::Type(String::from("Guard is immutable"))),
        (&Guard::Request, Group::Forbidden) => Ok(false),
        (&Guard::RequestNoCors, ref group)
            if group != &Group::CorsSafelistedRequestHeader => Ok(false),
        (&Guard::Response, Group::ForbiddenResponseHeader) => Ok(false),
        _ => Ok(true),
    }
}

/// Removes trailing and leading HTTP whitespace bytes.
pub fn normalize_value(value: ByteString) -> ByteString {
    let opt_first_index = index_of_first_non_whitespace(&value);
    match opt_first_index {
        None => ByteString::new(vec![]),
        Some(0) => {
            let mut value: Vec<u8> = value.into();
            loop {
                match value.last().map(|ref_byte| *ref_byte) {
                    None => panic!("Should have found non-whitespace character first."),
                    Some(byte) if is_HTTP_whitespace(byte) => value.pop(),
                    Some(_) => return ByteString::new(value),
                };
            }
        }
        Some(first_index) => {
            let opt_last_index = index_of_last_non_whitespace(&value);
            match opt_last_index {
                None => panic!("Should have found non-whitespace character first."),
                Some(last_index) => {
                    let capacity = last_index - first_index + 1;
                    let mut normalized_value = Vec::with_capacity(capacity);
                    for byte in &value[first_index..last_index + 1] {
                        normalized_value.push(*byte);
                    }
                    ByteString::new(normalized_value)
                }
            }
        }
    }
}

fn is_HTTP_whitespace(byte: u8) -> bool {
    return byte == 0x09 ||   // horizontal tab
        byte == 0x0A ||      // new line
        byte == 0x0D ||      // return character
        byte == 0x20;        // space
}

fn index_of_first_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate() {
        if is_HTTP_whitespace(byte) {
            continue;
        }
        return Some(index)
    }
    None
}

fn index_of_last_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate().rev() {
        if is_HTTP_whitespace(byte) {
            continue;
        }
        return Some(index)
    }
    None
}

fn is_field_name(name: &ByteString) -> bool {
    // http://tools.ietf.org/html/rfc7230#section-3.2
    is_token(&*name)
}

fn is_field_content(value: &ByteString) -> bool {
    // http://tools.ietf.org/html/rfc2616#section-2.2
    // http://www.rfc-editor.org/errata_search.php?rfc=7230
    // Errata ID: 4189
    // field-content = field-vchar [ 1*( SP / HTAB / field-vchar )
    //                               field-vchar ]
    if value.len() == 0 {
        return false;
    }
    if !is_field_vchar(value[0]) {
        return false;
    }

    for &ch in &value[1..value.len() - 1] {
        if !is_field_vchar(ch) || !is_space(ch) || !is_htab(ch) {
            return false;
        }
    }

    if !is_field_vchar(value[value.len() - 1]) {
        return false;
    }

    return true;
}

fn is_space(x: u8) -> bool {
    return x == 0x20;
}

fn is_htab(x: u8) -> bool {
    return x == 0x09;
}

fn is_field_vchar(x: u8) -> bool {
    is_vchar(x) || is_obs_text(x)
}

fn is_vchar(x: u8) -> bool {
    // http://tools.ietf.org/html/rfc2616#section-2.2
    // field-vchar = VCHAR / obs-text
    match x {
        0...31 | 127 => false, // CTLs
        x if x > 127 => false, // non-CHARs
        _ => true,
    }
}

fn is_obs_text(x: u8) -> bool {
    // http://tools.ietf.org/html/rfc7230#section-3.2.6
    match x {
        0x80...0xFF => true,
        _ => false,
    }
}
