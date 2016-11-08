/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

pub use cssparser::RGBA;

use std::fmt::{self, Debug};
use style_traits::{FromCss, ToCss};

macro_rules! define_numbered_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+,) => {
        define_numbered_css_keyword_enum!($name: $( $css => $variant = $value ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Copy, RustcEncodable, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum $name {
            $( $variant = $value ),+
        }

        impl $name {
            pub fn parse(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    $( $css => Ok($name::$variant), )+
                    _ => Err(())
                }
            }
        }

        impl ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write {
                match *self {
                    $( $name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

macro_rules! impl_specified_for_computed {
    ($specified: ty, $computed: ty) => {
        impl ToComputedValue for $specified {
            type ComputedValue = $computed;

            fn to_computed_value(&self, context: &Context) -> $computed {
                match *self {
                    Either::First(a) => Either::First(a.to_computed_value(context)),
                    Either::Second(a) => Either::Second(a.to_computed_value(context)),
                }
            }

            #[inline]
            fn from_computed_value(computed: &$computed) -> Self {
                match *computed {
                    Either::First(a) => Either::First(ToComputedValue::from_computed_value(&a)),
                    Either::Second(a) => Either::Second(ToComputedValue::from_computed_value(&a)),
                }
            }
        }
    };
}

pub trait HasViewportPercentage {
    fn has_viewport_percentage(&self) -> bool;
}

pub trait NoViewportPercentage {}

impl<T> HasViewportPercentage for T where T: NoViewportPercentage {
    fn has_viewport_percentage(&self) -> bool {
        false
    }
}

pub trait CssType: Debug + FromCss + ToCss {}

impl<T> CssType for T where T: Debug + FromCss + ToCss {}

use self::computed::ComputedValueAsSpecified;

macro_rules! define_keyword_type {
    ($name: ident, $css: expr) => {
        #[derive(Clone, PartialEq, Copy)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct $name;

        impl ::style_traits::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result where W: ::std::fmt::Write {
                write!(dest, $css)
            }
        }

        impl ::style_traits::FromCss for $name {
            fn from_css(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                input.expect_ident_matching($css).map(|_| $name)
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, $css)
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
pub enum Either<A: CssType, B: CssType> {
    First(A),
    Second(B),
}

impl<A: CssType, B: CssType> Debug for Either<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Either::First(ref v) => write!(f, "{:?}", v),
            Either::Second(ref v) => write!(f, "{:?}", v),
        }
    }
}

impl<A: CssType, B: CssType> ToCss for Either<A, B> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Either::First(ref v) => v.to_css(dest),
            Either::Second(ref v) => v.to_css(dest),
        }
    }
}

impl<A: CssType, B: CssType> Either<A, B> {
    pub fn parse(input: &mut ::cssparser::Parser) -> Result<Either<A, B>, ()> {
        if let Ok(v) = input.try(|i| A::from_css(i)) {
            Ok(Either::First(v))
        } else if let Ok(v) = B::from_css(input) {
            Ok(Either::Second(v))
        } else {
            Err(())
        }
    }
}

macro_rules! impl_either_type_getter {
    ($name: ident, $ty: ty, $variant: path) => {
        impl<T: CssType> Either<$ty, T> {
            pub fn $name(&self) -> Option<&$ty> {
                match *self {
                    $variant(ref v) => Some(v),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! impl_either_type_viewport_percent {
    ($name: ty, $variant: path) => {
        impl HasViewportPercentage for $name {
            fn has_viewport_percentage(&self) -> bool {
                match *self {
                    $variant(ref length) => length.has_viewport_percentage(),
                    _ => false,
                }
            }
        }
    };
}


pub mod computed;
pub mod specified;

pub type CSSFloat = f32;

pub const FONT_MEDIUM_PX: i32 = 16;
