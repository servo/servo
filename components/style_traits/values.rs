/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper types and traits for the handling of CSS values.

use app_units::Au;
use cssparser::{UnicodeRange, serialize_string};
use std::fmt;

/// Serialises a value according to its CSS representation.
///
/// This trait is implemented for `str` and its friends, serialising the string
/// contents as a CSS quoted string.
///
/// This trait is derivable with `#[derive(ToCss)]`, with the following behaviour:
/// * unit variants get serialised as the `snake-case` representation
///   of their name;
/// * unit variants whose name starts with "Moz" or "Webkit" are prepended
///   with a "-";
/// * variants with fields get serialised as the space-separated serialisations
///   of their fields.
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

impl<'a, T> ToCss for &'a T where T: ToCss + ?Sized {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        (*self).to_css(dest)
    }
}

impl ToCss for str {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_string(self, dest)
    }
}

impl ToCss for String {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_string(self, dest)
    }
}

/// Type used as the associated type in the `OneOrMoreSeparated` trait on a
/// type to indicate that a serialized list of elements of this type is
/// separated by commas.
pub struct CommaSeparator;

/// Type used as the associated type in the `OneOrMoreSeparated` trait on a
/// type to indicate that a serialized list of elements of this type is
/// separated by spaces.
pub struct SpaceSeparator;

/// A trait satisfied by the types corresponding to separators.
pub trait Separator {
    /// The separator string that the satisfying separator type corresponds to.
    fn separator() -> &'static str;
}

impl Separator for CommaSeparator {
    fn separator() -> &'static str {
        ", "
    }
}

impl Separator for SpaceSeparator {
    fn separator() -> &'static str {
        " "
    }
}

/// Trait that indicates that satisfying separator types are comma separators.
/// This seems kind of redundant, but we aren't able to express type equality
/// constraints yet.
/// https://github.com/rust-lang/rust/issues/20041
pub trait IsCommaSeparator {}

impl IsCommaSeparator for CommaSeparator {}

/// Marker trait on T to automatically implement ToCss for Vec<T> when T's are
/// separated by some delimiter `delim`.
pub trait OneOrMoreSeparated {
    /// Associated type indicating which separator is used.
    type S: Separator;
}

impl OneOrMoreSeparated for UnicodeRange {
    type S = CommaSeparator;
}

impl<T> ToCss for Vec<T> where T: ToCss + OneOrMoreSeparated {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.iter();
        iter.next().unwrap().to_css(dest)?;
        for item in iter {
            dest.write_str(<T as OneOrMoreSeparated>::S::separator())?;
            item.to_css(dest)?;
        }
        Ok(())
    }
}

impl<T> ToCss for Box<T> where T: ?Sized + ToCss {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        (**self).to_css(dest)
    }
}

impl ToCss for Au {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}px", self.to_f64_px())
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
impl_to_css_for_predefined_type!(u16);
impl_to_css_for_predefined_type!(u32);
impl_to_css_for_predefined_type!(::cssparser::Token<'a>);
impl_to_css_for_predefined_type!(::cssparser::RGBA);
impl_to_css_for_predefined_type!(::cssparser::Color);
impl_to_css_for_predefined_type!(::cssparser::UnicodeRange);

#[macro_export]
macro_rules! define_css_keyword_enum {
    ($name: ident: values { $( $css: expr => $variant: ident),+, }
                   aliases { $( $alias: expr => $alias_variant: ident ),+, }) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ]
                                                              [ $( $alias => $alias_variant ),+ ]);
    };
    ($name: ident: values { $( $css: expr => $variant: ident),+, }
                   aliases { $( $alias: expr => $alias_variant: ident ),* }) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ]
                                                              [ $( $alias => $alias_variant ),* ]);
    };
    ($name: ident: values { $( $css: expr => $variant: ident),+ }
                   aliases { $( $alias: expr => $alias_variant: ident ),+, }) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ]
                                                              [ $( $alias => $alias_variant ),+ ]);
    };
    ($name: ident: values { $( $css: expr => $variant: ident),+ }
                   aliases { $( $alias: expr => $alias_variant: ident ),* }) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ]
                                                              [ $( $alias => $alias_variant ),* ]);
    };
    ($name: ident: $( $css: expr => $variant: ident ),+,) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ] []);
    };
    ($name: ident: $( $css: expr => $variant: ident ),+) => {
        __define_css_keyword_enum__add_optional_traits!($name [ $( $css => $variant ),+ ] []);
    };
}

#[cfg(feature = "servo")]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_optional_traits {
    ($name: ident [ $( $css: expr => $variant: ident ),+ ]
                  [ $( $alias: expr => $alias_variant: ident),* ]) => {
        __define_css_keyword_enum__actual! {
            $name [ Deserialize, Serialize, HeapSizeOf ]
                  [ $( $css => $variant ),+ ]
                  [ $( $alias => $alias_variant ),* ]
        }
    };
}

#[cfg(not(feature = "servo"))]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_optional_traits {
    ($name: ident [ $( $css: expr => $variant: ident ),+ ] [ $( $alias: expr => $alias_variant: ident),* ]) => {
        __define_css_keyword_enum__actual! {
            $name [] [ $( $css => $variant ),+ ] [ $( $alias => $alias_variant ),* ]
        }
    };
}

#[macro_export]
macro_rules! __define_css_keyword_enum__actual {
    ($name: ident [ $( $derived_trait: ident),* ]
                  [ $( $css: expr => $variant: ident ),+ ]
                  [ $( $alias: expr => $alias_variant: ident ),* ]) => {
        #[allow(non_camel_case_types, missing_docs)]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq $(, $derived_trait )* )]
        pub enum $name {
            $( $variant ),+
        }

        impl $name {
            /// Parse this property from a CSS input stream.
            pub fn parse<'i, 't>(input: &mut ::cssparser::Parser<'i, 't>)
                                 -> Result<$name, $crate::ParseError<'i>> {
                let ident = input.expect_ident()?;
                Self::from_ident(&ident)
                    .map_err(|()| ::cssparser::ParseError::Basic(
                        ::cssparser::BasicParseError::UnexpectedToken(
                            ::cssparser::Token::Ident(ident))))
            }

            /// Parse this property from an already-tokenized identifier.
            pub fn from_ident(ident: &str) -> Result<$name, ()> {
                match_ignore_ascii_case! { ident,
                                           $( $css => Ok($name::$variant), )+
                                           $( $alias => Ok($name::$alias_variant), )*
                                           _ => Err(())
                }
            }
        }

        impl ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                where W: ::std::fmt::Write
            {
                match *self {
                    $( $name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

/// Helper types for the handling of specified values.
pub mod specified {
    use ParsingMode;
    use app_units::Au;
    use std::cmp;

    /// Whether to allow negative lengths or not.
    #[repr(u8)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum AllowedLengthType {
        /// Allow all kind of lengths.
        All,
        /// Allow only non-negative lengths.
        NonNegative
    }

    impl Default for AllowedLengthType {
        #[inline]
        fn default() -> Self {
            AllowedLengthType::All
        }
    }

    impl AllowedLengthType {
        /// Whether value is valid for this allowed length type.
        #[inline]
        pub fn is_ok(&self, parsing_mode: ParsingMode, value: f32) -> bool {
            if parsing_mode.allows_all_numeric_values() {
                return true;
            }
            match *self {
                AllowedLengthType::All => true,
                AllowedLengthType::NonNegative => value >= 0.,
            }
        }

        /// Clamp the value following the rules of this numeric type.
        #[inline]
        pub fn clamp(&self, val: Au) -> Au {
            match *self {
                AllowedLengthType::All => val,
                AllowedLengthType::NonNegative => cmp::max(Au(0), val),
            }
        }
    }

    /// Whether to allow negative lengths or not.
    #[repr(u8)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
    pub enum AllowedNumericType {
        /// Allow all kind of numeric values.
        All,
        /// Allow only non-negative numeric values.
        NonNegative,
        /// Allow only numeric values greater or equal to 1.0.
        AtLeastOne,
    }

    impl AllowedNumericType {
        /// Whether the value fits the rules of this numeric type.
        #[inline]
        pub fn is_ok(&self, parsing_mode: ParsingMode, val: f32) -> bool {
            if parsing_mode.allows_all_numeric_values() {
                return true;
            }
            match *self {
                AllowedNumericType::All => true,
                AllowedNumericType::NonNegative => val >= 0.0,
                AllowedNumericType::AtLeastOne => val >= 1.0,
            }
        }

        /// Clamp the value following the rules of this numeric type.
        #[inline]
        pub fn clamp(&self, val: f32) -> f32 {
            match *self {
                AllowedNumericType::NonNegative if val < 0. => 0.,
                AllowedNumericType::AtLeastOne if val < 1. => 1.,
                _ => val,
            }
        }
    }
}


/// Wrap CSS types for serialization with `write!` or `format!` macros.
/// Used by ToCss of SpecifiedOperation.
pub struct Css<T>(pub T);

impl<T: ToCss> fmt::Display for Css<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.to_css(f)
    }
}
