/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various macro helpers.

/// A macro to parse an identifier, or return an `UnexpectedIndent` error
/// otherwise.
///
/// FIXME(emilio): The fact that `UnexpectedIdent` is a `SelectorParseError`
/// doesn't make a lot of sense to me.
macro_rules! try_match_ident_ignore_ascii_case {
    ($ident:expr, $( $match_body:tt )*) => {
        let __ident = $ident;
        (match_ignore_ascii_case! { &*__ident,
            $( $match_body )*
            _ => Err(()),
        })
        .map_err(|()| {
            ::selectors::parser::SelectorParseError::UnexpectedIdent(__ident).into()
        })
    }
}

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
            fn parse<'i, 't>(_context: &$crate::parser::ParserContext,
                             input: &mut ::cssparser::Parser<'i, 't>)
                             -> Result<$name, ::style_traits::ParseError<'i>> {
                try_match_ident_ignore_ascii_case! { input.expect_ident()?,
                    $( $css => Ok($name::$variant), )+
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
            fn parse<'i, 't>(_context: &$crate::parser::ParserContext,
                             input: &mut ::cssparser::Parser<'i, 't>)
                             -> Result<Self, ::style_traits::ParseError<'i>> {
                $name::parse(input)
            }
        }

        impl $crate::values::computed::ComputedValueAsSpecified for $name {}
        no_viewport_percentage!($name);
    };
}

macro_rules! define_keyword_type {
    ($name: ident, $css: expr) => {
        #[allow(missing_docs)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[derive(Clone, Copy, PartialEq, ToCss)]
        pub struct $name;

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
            fn parse<'i, 't>(_context: &$crate::parser::ParserContext,
                             input: &mut ::cssparser::Parser<'i, 't>)
                             -> Result<$name, ::style_traits::ParseError<'i>> {
                input.expect_ident_matching($css).map(|_| $name).map_err(|e| e.into())
            }
        }

        impl $crate::values::computed::ComputedValueAsSpecified for $name {}
        impl $crate::values::animated::AnimatedValueAsComputed for $name {}
        no_viewport_percentage!($name);

        impl $crate::values::animated::ToAnimatedZero for $name {
            #[inline]
            fn to_animated_zero(&self) -> Result<Self, ()> { Ok($name) }
        }
    };
}
