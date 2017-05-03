/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Functions for validating and extracting qualified XML names.

use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::str::DOMString;
use html5ever::{Prefix, LocalName, Namespace};

/// Validate a qualified name. See https://dom.spec.whatwg.org/#validate for details.
pub fn validate_qualified_name(qualified_name: &str) -> ErrorResult {
    match xml_name_type(qualified_name) {
        XMLName::InvalidXMLName => {
            // Step 1.
            Err(Error::InvalidCharacter)
        },
        XMLName::Name => {
            // Step 2.
            Err(Error::Namespace)
        },
        XMLName::QName => Ok(()),
    }
}

/// Validate a namespace and qualified name and extract their parts.
/// See https://dom.spec.whatwg.org/#validate-and-extract for details.
pub fn validate_and_extract(namespace: Option<DOMString>,
                            qualified_name: &str)
                            -> Fallible<(Namespace, Option<Prefix>, LocalName)> {
    // Step 1.
    let namespace = namespace_from_domstring(namespace);

    // Step 2.
    try!(validate_qualified_name(qualified_name));

    let colon = ':';

    // Step 5.
    let mut parts = qualified_name.splitn(2, colon);

    let (maybe_prefix, local_name) = {
        let maybe_prefix = parts.next();
        let maybe_local_name = parts.next();

        debug_assert!(parts.next().is_none());

        if let Some(local_name) = maybe_local_name {
            debug_assert!(!maybe_prefix.unwrap().is_empty());

            (maybe_prefix, local_name)
        } else {
            (None, maybe_prefix.unwrap())
        }
    };

    debug_assert!(!local_name.contains(colon));

    match (namespace, maybe_prefix) {
        (ns!(), Some(_)) => {
            // Step 6.
            Err(Error::Namespace)
        },
        (ref ns, Some("xml")) if ns != &ns!(xml) => {
            // Step 7.
            Err(Error::Namespace)
        },
        (ref ns, p) if ns != &ns!(xmlns) && (qualified_name == "xmlns" || p == Some("xmlns")) => {
            // Step 8.
            Err(Error::Namespace)
        },
        (ns!(xmlns), p) if qualified_name != "xmlns" && p != Some("xmlns") => {
            // Step 9.
            Err(Error::Namespace)
        },
        (ns, p) => {
            // Step 10.
            Ok((ns, p.map(Prefix::from), LocalName::from(local_name)))
        }
    }
}

/// Results of `xml_name_type`.
#[derive(PartialEq)]
#[allow(missing_docs)]
pub enum XMLName {
    QName,
    Name,
    InvalidXMLName,
}

/// Check if an element name is valid. See http://www.w3.org/TR/xml/#NT-Name
/// for details.
pub fn xml_name_type(name: &str) -> XMLName {
    fn is_valid_start(c: char) -> bool {
        match c {
            ':' |
            'A'...'Z' |
            '_' |
            'a'...'z' |
            '\u{C0}'...'\u{D6}' |
            '\u{D8}'...'\u{F6}' |
            '\u{F8}'...'\u{2FF}' |
            '\u{370}'...'\u{37D}' |
            '\u{37F}'...'\u{1FFF}' |
            '\u{200C}'...'\u{200D}' |
            '\u{2070}'...'\u{218F}' |
            '\u{2C00}'...'\u{2FEF}' |
            '\u{3001}'...'\u{D7FF}' |
            '\u{F900}'...'\u{FDCF}' |
            '\u{FDF0}'...'\u{FFFD}' |
            '\u{10000}'...'\u{EFFFF}' => true,
            _ => false,
        }
    }

    fn is_valid_continuation(c: char) -> bool {
        is_valid_start(c) ||
        match c {
            '-' |
            '.' |
            '0'...'9' |
            '\u{B7}' |
            '\u{300}'...'\u{36F}' |
            '\u{203F}'...'\u{2040}' => true,
            _ => false,
        }
    }

    let mut iter = name.chars();
    let mut non_qname_colons = false;
    let mut seen_colon = false;
    let mut last = match iter.next() {
        None => return XMLName::InvalidXMLName,
        Some(c) => {
            if !is_valid_start(c) {
                return XMLName::InvalidXMLName;
            }
            if c == ':' {
                non_qname_colons = true;
            }
            c
        }
    };

    for c in iter {
        if !is_valid_continuation(c) {
            return XMLName::InvalidXMLName;
        }
        if c == ':' {
            if seen_colon {
                non_qname_colons = true;
            } else {
                seen_colon = true;
            }
        }
        last = c
    }

    if last == ':' {
        non_qname_colons = true
    }

    if non_qname_colons {
        XMLName::Name
    } else {
        XMLName::QName
    }
}

/// Convert a possibly-null URL to a namespace.
///
/// If the URL is None, returns the empty namespace.
pub fn namespace_from_domstring(url: Option<DOMString>) -> Namespace {
    match url {
        None => ns!(),
        Some(s) => Namespace::from(s),
    }
}
