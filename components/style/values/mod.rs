/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

#![deny(missing_docs)]

use crate::parser::{Parse, ParserContext};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::Atom;
pub use cssparser::{serialize_identifier, serialize_name, CowRcStr, Parser};
pub use cssparser::{SourceLocation, Token, RGBA};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Debug, Write};
use std::hash;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use to_shmem::impl_trivial_to_shmem;

#[cfg(feature = "gecko")]
pub use crate::gecko::url::CssUrl;
#[cfg(feature = "servo")]
pub use crate::servo::url::CssUrl;

pub mod animated;
pub mod computed;
pub mod distance;
pub mod generics;
pub mod resolved;
pub mod specified;

/// A CSS float value.
pub type CSSFloat = f32;

/// A CSS integer value.
pub type CSSInteger = i32;

define_keyword_type!(None_, "none");
define_keyword_type!(Auto, "auto");

/// Serialize an identifier which is represented as an atom.
#[cfg(feature = "gecko")]
pub fn serialize_atom_identifier<W>(ident: &Atom, dest: &mut W) -> fmt::Result
where
    W: Write,
{
    ident.with_str(|s| serialize_identifier(s, dest))
}

/// Serialize an identifier which is represented as an atom.
#[cfg(feature = "servo")]
pub fn serialize_atom_identifier<Static, W>(
    ident: &::string_cache::Atom<Static>,
    dest: &mut W,
) -> fmt::Result
where
    Static: ::string_cache::StaticAtomSet,
    W: Write,
{
    serialize_identifier(&ident, dest)
}

/// Serialize a name which is represented as an Atom.
#[cfg(feature = "gecko")]
pub fn serialize_atom_name<W>(ident: &Atom, dest: &mut W) -> fmt::Result
where
    W: Write,
{
    ident.with_str(|s| serialize_name(s, dest))
}

/// Serialize a name which is represented as an Atom.
#[cfg(feature = "servo")]
pub fn serialize_atom_name<Static, W>(
    ident: &::string_cache::Atom<Static>,
    dest: &mut W,
) -> fmt::Result
where
    Static: ::string_cache::StaticAtomSet,
    W: Write,
{
    serialize_name(&ident, dest)
}

/// Serialize a normalized value into percentage.
pub fn serialize_percentage<W>(value: CSSFloat, dest: &mut CssWriter<W>) -> fmt::Result
where
    W: Write,
{
    (value * 100.).to_css(dest)?;
    dest.write_str("%")
}

/// Convenience void type to disable some properties and values through types.
#[cfg_attr(feature = "servo", derive(Deserialize, MallocSizeOf, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
)]
pub enum Impossible {}

// FIXME(nox): This should be derived but the derive code cannot cope
// with uninhabited enums.
impl ComputeSquaredDistance for Impossible {
    #[inline]
    fn compute_squared_distance(&self, _other: &Self) -> Result<SquaredDistance, ()> {
        match *self {}
    }
}

impl_trivial_to_shmem!(Impossible);

impl Parse for Impossible {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// A struct representing one of two kinds of values.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum Either<A, B> {
    /// The first value.
    First(A),
    /// The second kind of value.
    Second(B),
}

impl<A: Debug, B: Debug> Debug for Either<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Either::First(ref v) => v.fmt(f),
            Either::Second(ref v) => v.fmt(f),
        }
    }
}

/// <https://drafts.csswg.org/css-values-4/#custom-idents>
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct CustomIdent(pub Atom);

impl CustomIdent {
    /// Parse an already-tokenizer identifier
    pub fn from_ident<'i>(
        location: SourceLocation,
        ident: &CowRcStr<'i>,
        excluding: &[&str],
    ) -> Result<Self, ParseError<'i>> {
        let valid = match_ignore_ascii_case! { ident,
            "initial" | "inherit" | "unset" | "default" | "revert" => false,
            _ => true
        };
        if !valid {
            return Err(
                location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone()))
            );
        }
        if excluding.iter().any(|s| ident.eq_ignore_ascii_case(s)) {
            Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        } else {
            Ok(CustomIdent(Atom::from(ident.as_ref())))
        }
    }
}

impl ToCss for CustomIdent {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        serialize_atom_identifier(&self.0, dest)
    }
}

/// <https://drafts.csswg.org/css-animations/#typedef-keyframes-name>
#[derive(
    Clone, Debug, MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem,
)]
pub enum KeyframesName {
    /// <custom-ident>
    Ident(CustomIdent),
    /// <string>
    QuotedString(Atom),
}

impl KeyframesName {
    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-name>
    pub fn from_ident(value: &str) -> Self {
        let location = SourceLocation { line: 0, column: 0 };
        let custom_ident = CustomIdent::from_ident(location, &value.into(), &["none"]).ok();
        match custom_ident {
            Some(ident) => KeyframesName::Ident(ident),
            None => KeyframesName::QuotedString(value.into()),
        }
    }

    /// Create a new KeyframesName from Atom.
    #[cfg(feature = "gecko")]
    pub fn from_atom(atom: Atom) -> Self {
        debug_assert_ne!(atom, atom!(""));

        // FIXME: We might want to preserve <string>, but currently Gecko
        // stores both of <custom-ident> and <string> into nsAtom, so
        // we can't tell it.
        KeyframesName::Ident(CustomIdent(atom))
    }

    /// The name as an Atom
    pub fn as_atom(&self) -> &Atom {
        match *self {
            KeyframesName::Ident(ref ident) => &ident.0,
            KeyframesName::QuotedString(ref atom) => atom,
        }
    }
}

impl Eq for KeyframesName {}

/// A trait that returns whether a given type is the `auto` value or not. So far
/// only needed for background-size serialization, which special-cases `auto`.
pub trait IsAuto {
    /// Returns whether the value is the `auto` value.
    fn is_auto(&self) -> bool;
}

impl PartialEq for KeyframesName {
    fn eq(&self, other: &Self) -> bool {
        self.as_atom() == other.as_atom()
    }
}

impl hash::Hash for KeyframesName {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_atom().hash(state)
    }
}

impl Parse for KeyframesName {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        match *input.next()? {
            Token::Ident(ref s) => Ok(KeyframesName::Ident(CustomIdent::from_ident(
                location,
                s,
                &["none"],
            )?)),
            Token::QuotedString(ref s) => Ok(KeyframesName::QuotedString(Atom::from(s.as_ref()))),
            ref t => Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}

impl ToCss for KeyframesName {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            KeyframesName::Ident(ref ident) => ident.to_css(dest),
            KeyframesName::QuotedString(ref atom) => atom.to_string().to_css(dest),
        }
    }
}
