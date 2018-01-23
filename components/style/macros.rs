/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various macro helpers.

macro_rules! exclusive_value {
    (($value:ident, $set:expr) => $ident:path) => {
        if $value.intersects($set) {
            return Err(())
        } else {
            $ident
        }
    }
}

#[cfg(feature = "gecko")]
macro_rules! impl_gecko_keyword_conversions {
    ($name: ident, $utype: ty) => {
        impl From<$utype> for $name {
            fn from(bits: $utype) -> $name {
                $name::from_gecko_keyword(bits)
            }
        }

        impl From<$name> for $utype {
            fn from(v: $name) -> $utype {
                v.to_gecko_keyword()
            }
        }
    };
}

macro_rules! trivial_to_computed_value {
    ($name:ty) => {
        impl $crate::values::computed::ToComputedValue for $name {
            type ComputedValue = $name;

            fn to_computed_value(&self, _: &$crate::values::computed::Context) -> Self {
                self.clone()
            }

            fn from_computed_value(other: &Self) -> Self {
                other.clone()
            }
        }
    }
}

/// A macro to parse an identifier, or return an `UnexpectedIndent` error
/// otherwise.
///
/// FIXME(emilio): The fact that `UnexpectedIdent` is a `SelectorParseError`
/// doesn't make a lot of sense to me.
macro_rules! try_match_ident_ignore_ascii_case {
    ($input:expr, $( $match_body:tt )*) => {
        let location = $input.current_source_location();
        let ident = $input.expect_ident_cloned()?;
        (match_ignore_ascii_case! { &ident,
            $( $match_body )*
            _ => Err(()),
        })
        .map_err(|()| {
            location.new_custom_error(
                ::selectors::parser::SelectorParseErrorKind::UnexpectedIdent(ident.clone())
            )
        })
    }
}

macro_rules! define_numbered_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+,) => {
        define_numbered_css_keyword_enum!($name: $( $css => $variant = $value ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+) => {
        #[allow(non_camel_case_types, missing_docs)]
        #[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
        pub enum $name {
            $( $variant = $value ),+
        }

        impl $crate::parser::Parse for $name {
            fn parse<'i, 't>(
                _context: &$crate::parser::ParserContext,
                input: &mut ::cssparser::Parser<'i, 't>,
            ) -> Result<$name, ::style_traits::ParseError<'i>> {
                try_match_ident_ignore_ascii_case! { input,
                    $( $css => Ok($name::$variant), )+
                }
            }
        }

        impl ::style_traits::ToCss for $name {
            fn to_css<W>(
                &self,
                dest: &mut ::style_traits::CssWriter<W>,
            ) -> ::std::fmt::Result
            where
                W: ::std::fmt::Write,
            {
                match *self {
                    $( $name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

/// A macro for implementing `ToComputedValue`, and `Parse` traits for
/// the enums defined using `define_css_keyword_enum` macro.
///
/// NOTE: We should either move `Parse` trait to `style_traits`
/// or `define_css_keyword_enum` macro to this crate, but that
/// may involve significant cleanup in both the crates.
macro_rules! add_impls_for_keyword_enum {
    ($name:ident) => {
        impl $crate::parser::Parse for $name {
            #[inline]
            fn parse<'i, 't>(
                _context: &$crate::parser::ParserContext,
                input: &mut ::cssparser::Parser<'i, 't>,
            ) -> Result<Self, ::style_traits::ParseError<'i>> {
                $name::parse(input)
            }
        }

        trivial_to_computed_value!($name);
    };
}

macro_rules! define_keyword_type {
    ($name: ident, $css: expr) => {
        #[allow(missing_docs)]
        #[derive(Animate, Clone, ComputeSquaredDistance, Copy, MallocSizeOf, PartialEq)]
        #[derive(ToAnimatedZero, ToComputedValue, ToCss)]
        pub struct $name;

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($css)
            }
        }

        impl $crate::parser::Parse for $name {
            fn parse<'i, 't>(
                _context: &$crate::parser::ParserContext,
                input: &mut ::cssparser::Parser<'i, 't>
            ) -> Result<$name, ::style_traits::ParseError<'i>> {
                input.expect_ident_matching($css).map(|_| $name).map_err(|e| e.into())
            }
        }

        impl $crate::values::animated::AnimatedValueAsComputed for $name {}
    };
}

#[cfg(feature = "gecko")]
macro_rules! impl_bitflags_conversions {
    ($name: ident) => {
        impl From<u8> for $name {
            fn from(bits: u8) -> $name {
                $name::from_bits(bits).expect("bits contain valid flag")
            }
        }

        impl From<$name> for u8 {
            fn from(v: $name) -> u8 {
                v.bits()
            }
        }
    };
}
