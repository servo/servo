/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various macro helpers.

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

        impl $crate::parser::Parse for $name {
            #[allow(missing_docs)]
            fn parse(_context: &$crate::parser::ParserContext, input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { &try!(input.expect_ident()),
                    $( $css => Ok($name::$variant), )+
                    _ => Err(())
                }
            }
        }

        impl ::style_traits::values::ToCss for $name {
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

/// A macro for implementing `ComputedValueAsSpecified`, `Parse`
/// and `HasViewportPercentage` traits for the enums defined
/// using `define_css_keyword_enum` macro.
///
/// NOTE: We should either move `Parse` trait to `style_traits`
/// or `define_css_keyword_enum` macro to this crate, but that
/// may involve significant cleanup in both the crates.
macro_rules! add_impls_for_keyword_enum {
    ($name:ident) => {
        impl $crate::parser::Parse for $name {
            #[inline]
            fn parse(_context: &$crate::parser::ParserContext,
                     input: &mut ::cssparser::Parser)
                     -> Result<Self, ()> {
                $name::parse(input)
            }
        }

        impl $crate::values::computed::ComputedValueAsSpecified for $name {}
        no_viewport_percentage!($name);
    };
}

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

        impl $crate::properties::animated_properties::Animatable for $name {
            #[inline]
            fn add_weighted(&self, _other: &Self, _self_progress: f64, _other_progress: f64)
                -> Result<Self, ()> {
                Ok($name)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $css)
            }
        }

        impl $crate::parser::Parse for $name {
            fn parse(_context: &$crate::parser::ParserContext,
                     input: &mut ::cssparser::Parser)
                     -> Result<$name, ()> {
                input.expect_ident_matching($css).map(|_| $name)
            }
        }

        impl $crate::values::computed::ComputedValueAsSpecified for $name {}
        no_viewport_percentage!($name);
    };
}
