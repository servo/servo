/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use std::ops::Deref;
use string_cache::{Atom, Namespace};
use util::str::{DOMString, LengthOrPercentageOrAuto, parse_unsigned_integer, parse_legacy_color, parse_length};
use util::str::{split_html_space_chars, str_join};
use values::specified::{Length};

// Duplicated from script::dom::values.
const UNSIGNED_LONG_MAX: u32 = 2147483647;

#[derive(PartialEq, Clone, HeapSizeOf)]
pub enum AttrValue {
    String(DOMString),
    TokenList(DOMString, Vec<Atom>),
    UInt(DOMString, u32),
    Atom(Atom),
    Length(DOMString, Option<Length>),
    Color(DOMString, Option<RGBA>),
    Dimension(DOMString, LengthOrPercentageOrAuto),
}

impl AttrValue {
    pub fn from_serialized_tokenlist(tokens: DOMString) -> AttrValue {
        let atoms =
            split_html_space_chars(&tokens)
            .map(Atom::from)
            .fold(vec![], |mut acc, atom| {
                if !acc.contains(&atom) { acc.push(atom) }
                acc
            });
        AttrValue::TokenList(tokens, atoms)
    }

    pub fn from_atomic_tokens(atoms: Vec<Atom>) -> AttrValue {
        // TODO(ajeffrey): effecient conversion of Vec<Atom> to DOMString
        let tokens = DOMString::from(str_join(&atoms, "\x20"));
        AttrValue::TokenList(tokens, atoms)
    }

    // https://html.spec.whatwg.org/multipage/#reflecting-content-attributes-in-idl-attributes:idl-unsigned-long
    pub fn from_u32(string: DOMString, default: u32) -> AttrValue {
        let result = parse_unsigned_integer(string.chars()).unwrap_or(default);
        let result = if result > UNSIGNED_LONG_MAX {
            default
        } else {
            result
        };
        AttrValue::UInt(string, result)
    }

    // https://html.spec.whatwg.org/multipage/#limited-to-only-non-negative-numbers-greater-than-zero
    pub fn from_limited_u32(string: DOMString, default: u32) -> AttrValue {
        let result = parse_unsigned_integer(string.chars()).unwrap_or(default);
        let result = if result == 0 || result > UNSIGNED_LONG_MAX {
            default
        } else {
            result
        };
        AttrValue::UInt(string, result)
    }

    pub fn from_atomic(string: DOMString) -> AttrValue {
        // FIXME(ajeffrey): convert directly from DOMString to Atom
        let value = Atom::from(&*string);
        AttrValue::Atom(value)
    }

    pub fn from_legacy_color(string: DOMString) -> AttrValue {
        let parsed = parse_legacy_color(&string).ok();
        AttrValue::Color(string, parsed)
    }

    pub fn from_dimension(string: DOMString) -> AttrValue {
        let parsed = parse_length(&string);
        AttrValue::Dimension(string, parsed)
    }

    /// Assumes the `AttrValue` is a `TokenList` and returns its tokens
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `TokenList`
    pub fn as_tokens(&self) -> &[Atom] {
        match *self {
            AttrValue::TokenList(_, ref tokens) => tokens,
            _ => panic!("Tokens not found"),
        }
    }

    /// Assumes the `AttrValue` is an `Atom` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not an `Atom`
    pub fn as_atom(&self) -> &Atom {
        match *self {
            AttrValue::Atom(ref value) => value,
            _ => panic!("Atom not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Color` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Color`
    pub fn as_color(&self) -> Option<&RGBA> {
        match *self {
            AttrValue::Color(_, ref color) => color.as_ref(),
            _ => panic!("Color not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Length` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Length`
    pub fn as_length(&self) -> Option<&Length> {
        match *self {
            AttrValue::Length(_, ref length) => length.as_ref(),
            _ => panic!("Length not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Dimension` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Dimension`
    pub fn as_dimension(&self) -> &LengthOrPercentageOrAuto {
        match *self {
            AttrValue::Dimension(_, ref l) => l,
            _ => panic!("Dimension not found"),
        }
    }

    /// Return the AttrValue as its integer representation, if any.
    /// This corresponds to attribute values returned as `AttrValue::UInt(_)`
    /// by `VirtualMethods::parse_plain_attribute()`.
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `UInt`
    pub fn as_uint(&self) -> u32 {
        if let AttrValue::UInt(_, value) = *self {
            value
        } else {
            panic!("Uint not found");
        }
    }
}

impl Deref for AttrValue {
    type Target = str;

    fn deref(&self) -> &str {
        match *self {
            AttrValue::String(ref value) |
                AttrValue::TokenList(ref value, _) |
                AttrValue::UInt(ref value, _) |
                AttrValue::Length(ref value, _) |
                AttrValue::Color(ref value, _) |
                AttrValue::Dimension(ref value, _) => &value,
            AttrValue::Atom(ref value) => &value,
        }
    }
}

#[derive(Clone, HeapSizeOf, Debug)]
pub struct AttrIdentifier {
    pub local_name: Atom,
    pub name: Atom,
    pub namespace: Namespace,
    pub prefix: Option<Atom>,
}
