/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

#![deny(missing_docs)]

use Atom;
pub use cssparser::{RGBA, Token, Parser, serialize_identifier, BasicParseError, CompactCowStr};
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseError;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::hash;
use style_traits::{ToCss, ParseError, StyleParseError};

pub mod animated;
pub mod computed;
pub mod generics;
pub mod specified;

/// A CSS float value.
pub type CSSFloat = f32;

/// A CSS integer value.
pub type CSSInteger = i32;

/// The default font size.
pub const FONT_MEDIUM_PX: i32 = 16;

define_keyword_type!(None_, "none");
define_keyword_type!(Auto, "auto");
define_keyword_type!(Normal, "normal");

/// Convenience void type to disable some properties and values through types.
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
pub enum Impossible {}

impl Parse for Impossible {
    fn parse<'i, 't>(
        _context: &ParserContext,
        _input: &mut Parser<'i, 't>)
    -> Result<Self, ParseError<'i>> {
        Err(StyleParseError::UnspecifiedError.into())
    }
}

/// A struct representing one of two kinds of values.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
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

impl<A: Parse, B: Parse> Parse for Either<A, B> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Either<A, B>, ParseError<'i>> {
        if let Ok(v) = input.try(|i| A::parse(context, i)) {
            Ok(Either::First(v))
        } else {
            B::parse(context, input).map(Either::Second)
        }
    }
}

/// https://drafts.csswg.org/css-values-4/#custom-idents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct CustomIdent(pub Atom);

impl CustomIdent {
    /// Parse an already-tokenizer identifier
    pub fn from_ident<'i>(ident: CompactCowStr<'i>, excluding: &[&str]) -> Result<Self, ParseError<'i>> {
        let valid = match_ignore_ascii_case! { &ident,
            "initial" | "inherit" | "unset" | "default" => false,
            _ => true
        };
        if !valid {
            return Err(SelectorParseError::UnexpectedIdent(ident).into());
        }
        if excluding.iter().any(|s| ident.eq_ignore_ascii_case(s)) {
            Err(StyleParseError::UnspecifiedError.into())
        } else {
            Ok(CustomIdent(Atom::from(Cow::from(ident))))
        }
    }
}

impl ToCss for CustomIdent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_identifier(&self.0.to_string(), dest)
    }
}

/// https://drafts.csswg.org/css-animations/#typedef-keyframes-name
#[derive(Debug, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum KeyframesName {
    /// <custom-ident>
    Ident(CustomIdent),
    /// <string>
    QuotedString(Atom),
}

impl KeyframesName {
    /// https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-name
    pub fn from_ident(value: String) -> Self {
        let custom_ident = CustomIdent::from_ident((&*value).into(), &["none"]).ok();
        match custom_ident {
            Some(ident) => KeyframesName::Ident(ident),
            None => KeyframesName::QuotedString(value.into()),
        }
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

impl PartialEq for KeyframesName {
    fn eq(&self, other: &Self) -> bool {
        self.as_atom() == other.as_atom()
    }
}

impl hash::Hash for KeyframesName {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        self.as_atom().hash(state)
    }
}

impl Parse for KeyframesName {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        match input.next() {
            Ok(Token::Ident(s)) => Ok(KeyframesName::Ident(CustomIdent::from_ident(s, &["none"])?)),
            Ok(Token::QuotedString(s)) => Ok(KeyframesName::QuotedString(Atom::from(Cow::from(s)))),
            Ok(t) => Err(BasicParseError::UnexpectedToken(t).into()),
            Err(e) => Err(e.into()),
        }
    }
}

impl ToCss for KeyframesName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            KeyframesName::Ident(ref ident) => ident.to_css(dest),
            KeyframesName::QuotedString(ref atom) => atom.to_string().to_css(dest),
        }
    }
}

// A type for possible values for min- and max- flavors of width,
// height, block-size, and inline-size.
define_css_keyword_enum!(ExtremumLength:
                         "-moz-max-content" => MaxContent,
                         "-moz-min-content" => MinContent,
                         "-moz-fit-content" => FitContent,
                         "-moz-available" => FillAvailable);
no_viewport_percentage!(ExtremumLength);
