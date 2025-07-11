/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Functions for validating names as defined in the DOM Standard: <https://dom.spec.whatwg.org/#namespaces>

use html5ever::{LocalName, Namespace, Prefix, ns};
use script_bindings::error::{Error, Fallible};
use script_bindings::str::DOMString;

/// <https://infra.spec.whatwg.org/#xml-namespace>
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

/// <https://infra.spec.whatwg.org/#xmlns-namespace>
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

/// <https://dom.spec.whatwg.org/#valid-namespace-prefix>
fn is_valid_namespace_prefix(p: &str) -> bool {
    // A string is a valid namespace prefix if its length
    // is at least 1 and it does not contain ASCII whitespace,
    // U+0000 NULL, U+002F (/), or U+003E (>).

    if p.is_empty() {
        return false;
    }

    !p.chars()
        .any(|c| c.is_ascii_whitespace() || matches!(c, '\u{0000}' | '\u{002F}' | '\u{003E}'))
}

/// <https://dom.spec.whatwg.org/#valid-attribute-local-name>
pub(crate) fn is_valid_attribute_local_name(name: &str) -> bool {
    // A string is a valid attribute local name if its length
    // is at least 1 and it does not contain ASCII whitespace,
    // U+0000 NULL, U+002F (/), U+003D (=), or U+003E (>).

    if name.is_empty() {
        return false;
    }

    !name.chars().any(|c| {
        c.is_ascii_whitespace() || matches!(c, '\u{0000}' | '\u{002F}' | '\u{003D}' | '\u{003E}')
    })
}

/// <https://dom.spec.whatwg.org/#valid-element-local-name>
pub(crate) fn is_valid_element_local_name(name: &str) -> bool {
    // Step 1. If name’s length is 0, then return false.
    if name.is_empty() {
        return false;
    }

    let mut iter = name.chars();

    // SAFETY: we have already checked that the &str is not empty
    let c0 = iter.next().unwrap();

    // Step 2. If name’s 0th code point is an ASCII alpha, then:
    if c0.is_ascii_alphabetic() {
        for c in iter {
            // Step 2.1 If name contains ASCII whitespace,
            // U+0000 NULL, U+002F (/), or U+003E (>), then return false.
            if c.is_ascii_whitespace() || matches!(c, '\u{0000}' | '\u{002F}' | '\u{003E}') {
                return false;
            }
        }
        true
    }
    // Step 3. If name’s 0th code point is not U+003A (:), U+005F (_),
    // or in the range U+0080 to U+10FFFF, inclusive, then return false.
    else if matches!(c0, '\u{003A}' | '\u{005F}' | '\u{0080}'..='\u{10FFF}') {
        for c in iter {
            // Step 4. If name’s subsequent code points,
            // if any, are not ASCII alphas, ASCII digits,
            // U+002D (-), U+002E (.), U+003A (:), U+005F (_),
            // or in the range U+0080 to U+10FFFF, inclusive,
            // then return false.
            if !c.is_ascii_alphanumeric() &&
                !matches!(
                    c,
                    '\u{002D}' | '\u{002E}' | '\u{003A}' | '\u{005F}' | '\u{0080}'..='\u{10FFF}'
                )
            {
                return false;
            }
        }
        true
    } else {
        false
    }
}

/// <https://dom.spec.whatwg.org/#valid-doctype-name>
pub(crate) fn is_valid_doctype_name(name: &str) -> bool {
    // A string is a valid doctype name if it does not contain
    // ASCII whitespace, U+0000 NULL, or U+003E (>).
    !name
        .chars()
        .any(|c| c.is_ascii_whitespace() || matches!(c, '\u{0000}' | '\u{003E}'))
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

/// Context for [`validate_and_extract`] a namespace and qualified name
///
/// <https://dom.spec.whatwg.org/#validate-and-extract>
#[derive(Clone, Copy, Debug)]
pub(crate) enum Context {
    Attribute,
    Element,
}

/// <https://dom.spec.whatwg.org/#validate-and-extract>
pub(crate) fn validate_and_extract(
    namespace: Option<DOMString>,
    qualified_name: &str,
    context: Context,
) -> Fallible<(Namespace, Option<Prefix>, LocalName)> {
    // Step 1. If namespace is the empty string, then set it to null.
    let namespace = namespace_from_domstring(namespace);

    // Step 2. Let prefix be null.
    let mut prefix = None;
    // Step 3. Let localName be qualifiedName.
    let mut local_name = qualified_name;
    // Step 4. If qualifiedName contains a U+003A (:):
    if let Some(idx) = qualified_name.find(':') {
        //     Step 4.1. Let splitResult be the result of running
        //          strictly split given qualifiedName and U+003A (:).
        let p = &qualified_name[..idx];

        // Step 5. If prefix is not a valid namespace prefix,
        // then throw an "InvalidCharacterError" DOMException.
        if !is_valid_namespace_prefix(p) {
            debug!("Not a valid namespace prefix");
            return Err(Error::InvalidCharacter);
        }

        //     Step 4.2. Set prefix to splitResult[0].
        prefix = Some(p);

        //     Step 4.3. Set localName to splitResult[1].
        let remaining = &qualified_name[(idx + 1).min(qualified_name.len())..];
        match remaining.find(':') {
            Some(end) => local_name = &remaining[..end],
            None => local_name = remaining,
        };
    }

    if let Some(p) = prefix {
        // Step 5. If prefix is not a valid namespace prefix,
        // then throw an "InvalidCharacterError" DOMException.
        if !is_valid_namespace_prefix(p) {
            debug!("Not a valid namespace prefix");
            return Err(Error::InvalidCharacter);
        }
    }

    match context {
        // Step 6. If context is "attribute" and localName
        //      is not a valid attribute local name, then
        //      throw an "InvalidCharacterError" DOMException.
        Context::Attribute => {
            if !is_valid_attribute_local_name(local_name) {
                debug!("Not a valid attribute name");
                return Err(Error::InvalidCharacter);
            }
        },
        // Step 7. If context is "element" and localName
        //      is not a valid element local name, then
        //      throw an "InvalidCharacterError" DOMException.
        Context::Element => {
            if !is_valid_element_local_name(local_name) {
                debug!("Not a valid element name");
                return Err(Error::InvalidCharacter);
            }
        },
    }

    match prefix {
        // Step 8. If prefix is non-null and namespace is null,
        //      then throw a "NamespaceError" DOMException.
        Some(_) if namespace.is_empty() => Err(Error::Namespace),
        // Step 9. If prefix is "xml" and namespace is not the XML namespace,
        //      then throw a "NamespaceError" DOMException.
        Some("xml") if *namespace != *XML_NAMESPACE => Err(Error::Namespace),
        // Step 10. If either qualifiedName or prefix is "xmlns" and namespace
        //      is not the XMLNS namespace, then throw a "NamespaceError" DOMException.
        p if (qualified_name == "xmlns" || p == Some("xmlns")) &&
            *namespace != *XMLNS_NAMESPACE =>
        {
            Err(Error::Namespace)
        },
        Some(_) if qualified_name == "xmlns" && *namespace != *XMLNS_NAMESPACE => {
            Err(Error::Namespace)
        },
        // Step 11. If namespace is the XMLNS namespace and neither qualifiedName
        //      nor prefix is "xmlns", then throw a "NamespaceError" DOMException.
        p if *namespace == *XMLNS_NAMESPACE &&
            (qualified_name != "xmlns" && p != Some("xmlns")) =>
        {
            Err(Error::Namespace)
        },
        // Step 12. Return (namespace, prefix, localName).
        _ => Ok((
            namespace,
            prefix.map(Prefix::from),
            LocalName::from(local_name),
        )),
    }
}
