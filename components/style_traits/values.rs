/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::CssStringWriter;
use std::fmt::{self, Write};
use url::Url;

/// The real ToCss trait can't be implemented for types in crates that don't
/// depend on each other.
pub trait ToCss {
    /// Serialize `self` in CSS syntax, writing to `dest`.
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write;

    /// Serialize `self` in CSS syntax and return a string.
    ///
    /// (This is a convenience wrapper for `to_css` and probably should not be overridden.)
    #[inline]
    fn to_css_string(&self) -> String {
        let mut s = String::new();
        self.to_css(&mut s).unwrap();
        s
    }
}

impl ToCss for Au {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}px", self.to_f64_px())
    }
}

impl ToCss for Url {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("url(\""));
        try!(write!(CssStringWriter::new(dest), "{}", self));
        try!(dest.write_str("\")"));
        Ok(())
    }
}

macro_rules! impl_to_css_for_predefined_type {
    ($name: ty) => {
        impl<'a> ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                ::cssparser::ToCss::to_css(self, dest)
            }
        }
    };
}

impl_to_css_for_predefined_type!(f32);
impl_to_css_for_predefined_type!(i32);
impl_to_css_for_predefined_type!(u32);
impl_to_css_for_predefined_type!(::cssparser::Token<'a>);
impl_to_css_for_predefined_type!(::cssparser::RGBA);
impl_to_css_for_predefined_type!(::cssparser::Color);

#[macro_export]
macro_rules! define_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident ),+,) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ]);
    };
    ($name: ident: $( $css: expr => $variant: ident ),+) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ]);
    };
}

#[cfg(feature = "servo")]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_optional_traits {
    ($name: ident [ $( $css: expr => $variant: ident ),+ ]) => {
        __define_css_keyword_enum__actual! {
            $name [ Deserialize, Serialize, HeapSizeOf ] [ $( $css => $variant ),+ ]
        }
    };
}

#[cfg(not(feature = "servo"))]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_optional_traits {
    ($name: ident [ $( $css: expr => $variant: ident ),+ ]) => {
        __define_css_keyword_enum__actual! {
            $name [] [ $( $css => $variant ),+ ]
        }
    };
}

#[macro_export]
macro_rules! __define_css_keyword_enum__actual {
    ($name: ident [ $( $derived_trait: ident),* ] [ $( $css: expr => $variant: ident ),+ ]) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug $(, $derived_trait )* )]
        pub enum $name {
            $( $variant ),+
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

pub mod specified {
    use app_units::Au;

    #[repr(u8)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum AllowedNumericType {
        All,
        NonNegative
    }

    impl AllowedNumericType {
        #[inline]
        pub fn is_ok(&self, value: f32) -> bool {
            match *self {
                AllowedNumericType::All => true,
                AllowedNumericType::NonNegative => value >= 0.,
            }
        }

        #[inline]
        pub fn clamp(&self, val: Au) -> Au {
            use std::cmp;
            match *self {
                AllowedNumericType::All => val,
                AllowedNumericType::NonNegative => cmp::max(Au(0), val),
            }
        }
    }
}
