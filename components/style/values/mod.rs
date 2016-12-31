/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

#![deny(missing_docs)]

pub use cssparser::{RGBA, Parser};
use parser::{Parse, ParserContext};
use std::fmt::{self, Debug};
use style_traits::ToCss;

macro_rules! define_numbered_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+,) => {
        define_numbered_css_keyword_enum!($name: $( $css => $variant = $value ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+) => {
        #[allow(non_camel_case_types, missing_docs)]
        #[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Copy, RustcEncodable, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum $name {
            $( $variant = $value ),+
        }

        impl $name {
            #[allow(missing_docs)]
            pub fn parse(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { try!(input.expect_ident()),
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

pub mod computed;
pub mod specified;

/// A CSS float value.
pub type CSSFloat = f32;

/// The default font size.
pub const FONT_MEDIUM_PX: i32 = 16;

/// A trait used to query whether this value has viewport units.
pub trait HasViewportPercentage {
    /// Returns true if this value has viewport units.
    fn has_viewport_percentage(&self) -> bool;
}

/// A trait used as a marker to represent that a given type may never contain
/// viewport units.
pub trait NoViewportPercentage {}

impl<T> HasViewportPercentage for T
    where T: NoViewportPercentage,
{
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        false
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
        impl NoViewportPercentage for $name {}
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
