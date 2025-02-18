/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Functions for validating and extracting qualified XML names.

use html5ever::{namespace_url, ns, LocalName, Namespace, Prefix};

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::str::DOMString;

/// Check if an element name is valid. See <http://www.w3.org/TR/xml/#NT-Name>
/// for details.
fn is_valid_start(c: char) -> bool {
    matches!(c, ':' |
        'A'..='Z' |
        '_' |
        'a'..='z' |
        '\u{C0}'..='\u{D6}' |
        '\u{D8}'..='\u{F6}' |
        '\u{F8}'..='\u{2FF}' |
        '\u{370}'..='\u{37D}' |
        '\u{37F}'..='\u{1FFF}' |
        '\u{200C}'..='\u{200D}' |
        '\u{2070}'..='\u{218F}' |
        '\u{2C00}'..='\u{2FEF}' |
        '\u{3001}'..='\u{D7FF}' |
        '\u{F900}'..='\u{FDCF}' |
        '\u{FDF0}'..='\u{FFFD}' |
        '\u{10000}'..='\u{EFFFF}')
}

fn is_valid_continuation(c: char) -> bool {
    is_valid_start(c) ||
        matches!(c,
            '-' |
            '.' |
            '0'..='9' |
            '\u{B7}' |
            '\u{300}'..='\u{36F}' |
            '\u{203F}'..='\u{2040}')
}

/// Validate a qualified name. See <https://dom.spec.whatwg.org/#validate> for details.
///
/// On success, this returns a tuple `(prefix, local name)`.
pub(crate) fn validate_and_extract_qualified_name(
    qualified_name: &str,
) -> Fallible<(Option<&str>, &str)> {
    if qualified_name.is_empty() {
        // Qualified names must not be empty
        return Err(Error::InvalidCharacter);
    }
    let mut colon_offset = None;
    let mut at_start_of_name = true;

    for (byte_position, c) in qualified_name.char_indices() {
        if c == ':' {
            if colon_offset.is_some() {
                // Qualified names must not contain more than one colon
                return Err(Error::InvalidCharacter);
            }
            colon_offset = Some(byte_position);
            at_start_of_name = true;
            continue;
        }

        if at_start_of_name {
            if !is_valid_start(c) {
                // Name segments must begin with a valid start character
                return Err(Error::InvalidCharacter);
            }
            at_start_of_name = false;
        } else if !is_valid_continuation(c) {
            // Name segments must consist of valid characters
            return Err(Error::InvalidCharacter);
        }
    }

    let Some(colon_offset) = colon_offset else {
        // Simple case: there is no prefix
        return Ok((None, qualified_name));
    };

    let (prefix, local_name) = qualified_name.split_at(colon_offset);
    let local_name = &local_name[1..]; // Remove the colon

    if prefix.is_empty() || local_name.is_empty() {
        // Neither prefix nor local name can be empty
        return Err(Error::InvalidCharacter);
    }

    Ok((Some(prefix), local_name))
}

/// Validate a namespace and qualified name and extract their parts.
/// See <https://dom.spec.whatwg.org/#validate-and-extract> for details.
pub(crate) fn validate_and_extract(
    namespace: Option<DOMString>,
    qualified_name: &str,
) -> Fallible<(Namespace, Option<Prefix>, LocalName)> {
    // Step 1. If namespace is the empty string, then set it to null.
    let namespace = namespace_from_domstring(namespace);

    // Step 2. Validate qualifiedName.
    // Step 3. Let prefix be null.
    // Step 4. Let localName be qualifiedName.
    // Step 5. If qualifiedName contains a U+003A (:):
    // NOTE: validate_and_extract_qualified_name does all of these things for us, because
    // it's easier to do them together
    let (prefix, local_name) = validate_and_extract_qualified_name(qualified_name)?;
    debug_assert!(!local_name.contains(':'));

    match (namespace, prefix) {
        (ns!(), Some(_)) => {
            // Step 6. If prefix is non-null and namespace is null, then throw a "NamespaceError" DOMException.
            Err(Error::Namespace)
        },
        (ref ns, Some("xml")) if ns != &ns!(xml) => {
            // Step 7. If prefix is "xml" and namespace is not the XML namespace,
            // then throw a "NamespaceError" DOMException.
            Err(Error::Namespace)
        },
        (ref ns, p) if ns != &ns!(xmlns) && (qualified_name == "xmlns" || p == Some("xmlns")) => {
            // Step 8. If either qualifiedName or prefix is "xmlns" and namespace is not the XMLNS namespace,
            // then throw a "NamespaceError" DOMException.
            Err(Error::Namespace)
        },
        (ns!(xmlns), p) if qualified_name != "xmlns" && p != Some("xmlns") => {
            // Step 9. If namespace is the XMLNS namespace and neither qualifiedName nor prefix is "xmlns",
            // then throw a "NamespaceError" DOMException.
            Err(Error::Namespace)
        },
        (ns, p) => {
            // Step 10. Return namespace, prefix, and localName.
            Ok((ns, p.map(Prefix::from), LocalName::from(local_name)))
        },
    }
}

pub(crate) fn matches_name_production(name: &str) -> bool {
    let mut iter = name.chars();

    if iter.next().is_none_or(|c| !is_valid_start(c)) {
        return false;
    }
    iter.all(is_valid_continuation)
}

/// Convert a possibly-null URL to a namespace.
///
/// If the URL is None, returns the empty namespace.
pub(crate) fn namespace_from_domstring(url: Option<DOMString>) -> Namespace {
    match url {
        None => ns!(),
        Some(s) => Namespace::from(s),
    }
}
