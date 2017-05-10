/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

#![deny(missing_docs)]

use Atom;
pub use cssparser::{RGBA, Token, Parser, serialize_identifier, serialize_string};
use parser::{Parse, ParserContext};
use properties::animated_properties::Animatable;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::hash;
use style_traits::ToCss;

macro_rules! define_numbered_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+,) => {
        define_numbered_css_keyword_enum!($name: $( $css => $variant = $value ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+) => {
        #[allow(non_camel_case_types, missing_docs)]
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum $name {
            $( $variant = $value ),+
        }

        impl Parse for $name {
            #[allow(missing_docs)]
            fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { &try!(input.expect_ident()),
                    $( $css => Ok($name::$variant), )+
                    _ => Err(())
                }
            }
        }

        impl ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                where W: ::std::fmt::Write,
            {
                match *self {
                    $( $name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

/// A macro used to implement HasViewportPercentage trait
/// for a given type that may never contain viewport units.
macro_rules! no_viewport_percentage {
    ($name: ident) => {
        impl HasViewportPercentage for $name {
            #[inline]
            fn has_viewport_percentage(&self) -> bool {
                false
            }
        }
    };
}

/// A macro for implementing `ComputedValueAsSpecified`, `Parse`
/// and `HasViewportPercentage` traits for the enums defined
/// using `define_css_keyword_enum` macro.
///
/// NOTE: We should either move `Parse` trait to `style_traits`
/// or `define_css_keyword_enum` macro to this crate, but that
/// may involve significant cleanup in both the crates.
macro_rules! add_impls_for_keyword_enum {
    ($name:ident) => {
        impl Parse for $name {
            #[inline]
            fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<Self, ()> {
                $name::parse(input)
            }
        }

        impl ComputedValueAsSpecified for $name {}
        no_viewport_percentage!($name);
    };
}

pub mod computed;
pub mod generics;
pub mod specified;

/// A CSS float value.
pub type CSSFloat = f32;

/// A CSS integer value.
pub type CSSInteger = i32;

/// The default font size.
pub const FONT_MEDIUM_PX: i32 = 16;

/// A trait used to query whether this value has viewport units.
pub trait HasViewportPercentage {
    /// Returns true if this value has viewport units.
    fn has_viewport_percentage(&self) -> bool;
}

impl<T: HasViewportPercentage> HasViewportPercentage for Box<T> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        (**self).has_viewport_percentage()
    }
}

use self::computed::ComputedValueAsSpecified;

macro_rules! define_keyword_type {
    ($name: ident, $css: expr) => {
        #[derive(Clone, PartialEq, Copy)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[allow(missing_docs)]
        pub struct $name;

        impl ::style_traits::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result where W: ::std::fmt::Write {
                write!(dest, $css)
            }
        }

        impl Animatable for $name {
            #[inline]
            fn interpolate(&self, _other: &Self, _progress: f64) -> Result<Self, ()> {
                Ok($name)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $css)
            }
        }

        impl Parse for $name {
            fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                input.expect_ident_matching($css).map(|_| $name)
            }
        }

        impl ComputedValueAsSpecified for $name {}
        no_viewport_percentage!($name);
    };
}

define_keyword_type!(None_, "none");
define_keyword_type!(Auto, "auto");
define_keyword_type!(Normal, "normal");

#[derive(Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A struct representing one of two kinds of values.
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

impl<A: ToCss, B: ToCss> ToCss for Either<A, B> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Either::First(ref v) => v.to_css(dest),
            Either::Second(ref v) => v.to_css(dest),
        }
    }
}

impl<A: HasViewportPercentage, B: HasViewportPercentage> HasViewportPercentage for Either<A, B> {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            Either::First(ref v) => v.has_viewport_percentage(),
            Either::Second(ref v) => v.has_viewport_percentage(),
        }
    }
}

impl<A: Parse, B: Parse> Parse for Either<A, B> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Either<A, B>, ()> {
        if let Ok(v) = input.try(|i| A::parse(context, i)) {
            Ok(Either::First(v))
        } else {
            B::parse(context, input).map(Either::Second)
        }
    }
}

use self::computed::{Context, ToComputedValue};

impl<A: ToComputedValue, B: ToComputedValue> ToComputedValue for Either<A, B> {
    type ComputedValue = Either<A::ComputedValue, B::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            Either::First(ref a) => Either::First(a.to_computed_value(context)),
            Either::Second(ref a) => Either::Second(a.to_computed_value(context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            Either::First(ref a) => Either::First(ToComputedValue::from_computed_value(a)),
            Either::Second(ref a) => Either::Second(ToComputedValue::from_computed_value(a)),
        }
    }
}

/// https://drafts.csswg.org/css-values-4/#custom-idents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct CustomIdent(pub Atom);

impl CustomIdent {
    /// Parse an already-tokenizer identifier
    pub fn from_ident(ident: Cow<str>, excluding: &[&str]) -> Result<Self, ()> {
        match_ignore_ascii_case! { &ident,
            "initial" | "inherit" | "unset" | "default" => return Err(()),
            _ => {}
        };
        if excluding.iter().any(|s| ident.eq_ignore_ascii_case(s)) {
            Err(())
        } else {
            Ok(CustomIdent(ident.into()))
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
        match CustomIdent::from_ident((&*value).into(), &["none"]) {
            Ok(ident) => KeyframesName::Ident(ident),
            Err(()) => KeyframesName::QuotedString(value.into()),
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
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.next() {
            Ok(Token::Ident(s)) => Ok(KeyframesName::Ident(CustomIdent::from_ident(s, &["none"])?)),
            Ok(Token::QuotedString(s)) => Ok(KeyframesName::QuotedString(s.into())),
            _ => Err(())
        }
    }
}

impl ToCss for KeyframesName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            KeyframesName::Ident(ref ident) => ident.to_css(dest),
            KeyframesName::QuotedString(ref atom) => serialize_string(&atom.to_string(), dest),
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
