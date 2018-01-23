/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper types and traits for the handling of CSS values.

use app_units::Au;
use cssparser::{ParseError, Parser, Token, UnicodeRange, serialize_string};
use cssparser::ToCss as CssparserToCss;
use servo_arc::Arc;
use std::fmt::{self, Write};

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
/// * if `#[css(comma)]` is found on a variant, its fields are separated by
///   commas, otherwise, by spaces;
/// * if `#[css(function)]` is found on a variant, the variant name gets
///   serialised like unit variants and its fields are surrounded by parentheses;
/// * if `#[css(iterable)]` is found on a function variant, that variant needs
///   to have a single member, and that member needs to be iterable. The
///   iterable will be serialized as the arguments for the function.
/// * if `#[css(dimension)]` is found on a variant, that variant needs
///   to have a single member. The variant would be serialized as a CSS
///   dimension token, like: <member><identifier>.
/// * finally, one can put `#[css(derive_debug)]` on the whole type, to
///   implement `Debug` by a single call to `ToCss::to_css`.
pub trait ToCss {
    /// Serialize `self` in CSS syntax, writing to `dest`.
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write;

    /// Serialize `self` in CSS syntax and return a string.
    ///
    /// (This is a convenience wrapper for `to_css` and probably should not be overridden.)
    #[inline]
    fn to_css_string(&self) -> String {
        let mut s = String::new();
        self.to_css(&mut CssWriter::new(&mut s)).unwrap();
        s
    }
}

impl<'a, T> ToCss for &'a T where T: ToCss + ?Sized {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
        (*self).to_css(dest)
    }
}

impl ToCss for str {
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
        serialize_string(self, dest)
    }
}

impl ToCss for String {
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
        serialize_string(self, dest)
    }
}

impl<T> ToCss for Option<T>
where
    T: ToCss,
{
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
        self.as_ref().map_or(Ok(()), |value| value.to_css(dest))
    }
}

/// A writer tailored for serialising CSS.
///
/// Coupled with SequenceWriter, this allows callers to transparently handle
/// things like comma-separated values etc.
pub struct CssWriter<'w, W: 'w> {
    inner: &'w mut W,
    prefix: Option<&'static str>,
}

impl<'w, W> CssWriter<'w, W>
where
    W: Write,
{
    /// Creates a new `CssWriter`.
    #[inline]
    pub fn new(inner: &'w mut W) -> Self {
        Self { inner, prefix: Some("") }
    }
}

impl<'w, W> Write for CssWriter<'w, W>
where
    W: Write,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.is_empty() {
            return Ok(());
        }
        if let Some(prefix) = self.prefix.take() {
            // We are going to write things, but first we need to write
            // the prefix that was set by `SequenceWriter::item`.
            if !prefix.is_empty() {
                self.inner.write_str(prefix)?;
            }
        }
        self.inner.write_str(s)
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        if let Some(prefix) = self.prefix.take() {
            // See comment in `write_str`.
            if !prefix.is_empty() {
                self.inner.write_str(prefix)?;
            }
        }
        self.inner.write_char(c)
    }
}

#[macro_export]
macro_rules! serialize_function {
    ($dest: expr, $name: ident($( $arg: expr, )+)) => {
        serialize_function!($dest, $name($($arg),+))
    };
    ($dest: expr, $name: ident($first_arg: expr $( , $arg: expr )*)) => {
        {
            $dest.write_str(concat!(stringify!($name), "("))?;
            $first_arg.to_css($dest)?;
            $(
                $dest.write_str(", ")?;
                $arg.to_css($dest)?;
            )*
            $dest.write_char(')')
        }
    }
}

/// Convenience wrapper to serialise CSS values separated by a given string.
pub struct SequenceWriter<'a, 'b: 'a, W: 'b> {
    inner: &'a mut CssWriter<'b, W>,
    separator: &'static str,
}

impl<'a, 'b, W> SequenceWriter<'a, 'b, W>
where
    W: Write + 'b,
{
    /// Create a new sequence writer.
    #[inline]
    pub fn new(inner: &'a mut CssWriter<'b, W>, separator: &'static str) -> Self {
        if inner.prefix.is_none() {
            // See comment in `item`.
            inner.prefix = Some("");
        }
        Self { inner, separator }
    }

    /// Serialises a CSS value, writing any separator as necessary.
    ///
    /// The separator is never written before any `item` produces any output,
    /// and is written in subsequent calls only if the `item` produces some
    /// output on its own again. This lets us handle `Option<T>` fields by
    /// just not printing anything on `None`.
    #[inline]
    pub fn item<T>(&mut self, item: &T) -> fmt::Result
    where
        T: ToCss,
    {
        let old_prefix = self.inner.prefix;
        if old_prefix.is_none() {
            // If there is no prefix in the inner writer, a previous
            // call to this method produced output, which means we need
            // to write the separator next time we produce output again.
            self.inner.prefix = Some(self.separator);
        }
        item.to_css(&mut self.inner)?;
        match (old_prefix, self.inner.prefix) {
            (_, None) => {
                // This call produced output and cleaned up after itself.
            }
            (None, Some(p)) => {
                // Some previous call to `item` produced output,
                // but this one did not, prefix should be the same as
                // the one we set.
                debug_assert_eq!(self.separator, p);
                // We clean up here even though it's not necessary just
                // to be able to do all these assertion checks.
                self.inner.prefix = None;
            }
            (Some(old), Some(new)) => {
                // No previous call to `item` produced output, and this one
                // either.
                debug_assert_eq!(old, new);
            }
        }
        Ok(())
    }
}

/// Type used as the associated type in the `OneOrMoreSeparated` trait on a
/// type to indicate that a serialized list of elements of this type is
/// separated by commas.
pub struct Comma;

/// Type used as the associated type in the `OneOrMoreSeparated` trait on a
/// type to indicate that a serialized list of elements of this type is
/// separated by spaces.
pub struct Space;

/// Type used as the associated type in the `OneOrMoreSeparated` trait on a
/// type to indicate that a serialized list of elements of this type is
/// separated by commas, but spaces without commas are also allowed when
/// parsing.
pub struct CommaWithSpace;

/// A trait satisfied by the types corresponding to separators.
pub trait Separator {
    /// The separator string that the satisfying separator type corresponds to.
    fn separator() -> &'static str;

    /// Parses a sequence of values separated by this separator.
    ///
    /// The given closure is called repeatedly for each item in the sequence.
    ///
    /// Successful results are accumulated in a vector.
    ///
    /// This method returns `Err(_)` the first time a closure does or if
    /// the separators aren't correct.
    fn parse<'i, 't, F, T, E>(
        parser: &mut Parser<'i, 't>,
        parse_one: F,
    ) -> Result<Vec<T>, ParseError<'i, E>>
    where
        F: for<'tt> FnMut(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>;
}

impl Separator for Comma {
    fn separator() -> &'static str {
        ", "
    }

    fn parse<'i, 't, F, T, E>(
        input: &mut Parser<'i, 't>,
        parse_one: F,
    ) -> Result<Vec<T>, ParseError<'i, E>>
    where
        F: for<'tt> FnMut(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>
    {
        input.parse_comma_separated(parse_one)
    }
}

impl Separator for Space {
    fn separator() -> &'static str {
        " "
    }

    fn parse<'i, 't, F, T, E>(
        input: &mut Parser<'i, 't>,
        mut parse_one: F,
    ) -> Result<Vec<T>, ParseError<'i, E>>
    where
        F: for<'tt> FnMut(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>
    {
        input.skip_whitespace();  // Unnecessary for correctness, but may help try() rewind less.
        let mut results = vec![parse_one(input)?];
        loop {
            input.skip_whitespace();  // Unnecessary for correctness, but may help try() rewind less.
            if let Ok(item) = input.try(&mut parse_one) {
                results.push(item);
            } else {
                return Ok(results)
            }
        }
    }
}

impl Separator for CommaWithSpace {
    fn separator() -> &'static str {
        ", "
    }

    fn parse<'i, 't, F, T, E>(
        input: &mut Parser<'i, 't>,
        mut parse_one: F,
    ) -> Result<Vec<T>, ParseError<'i, E>>
    where
        F: for<'tt> FnMut(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>
    {
        input.skip_whitespace();  // Unnecessary for correctness, but may help try() rewind less.
        let mut results = vec![parse_one(input)?];
        loop {
            input.skip_whitespace();  // Unnecessary for correctness, but may help try() rewind less.
            let comma_location = input.current_source_location();
            let comma = input.try(|i| i.expect_comma()).is_ok();
            input.skip_whitespace();  // Unnecessary for correctness, but may help try() rewind less.
            if let Ok(item) = input.try(&mut parse_one) {
                results.push(item);
            } else if comma {
                return Err(comma_location.new_unexpected_token_error(Token::Comma));
            } else {
                break;
            }
        }
        Ok(results)
    }
}

/// Marker trait on T to automatically implement ToCss for Vec<T> when T's are
/// separated by some delimiter `delim`.
pub trait OneOrMoreSeparated {
    /// Associated type indicating which separator is used.
    type S: Separator;
}

impl OneOrMoreSeparated for UnicodeRange {
    type S = Comma;
}

impl<T> ToCss for Vec<T> where T: ToCss + OneOrMoreSeparated {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
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
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
        where W: Write,
    {
        (**self).to_css(dest)
    }
}

impl<T> ToCss for Arc<T> where T: ?Sized + ToCss {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
        where W: Write,
    {
        (**self).to_css(dest)
    }
}

impl ToCss for Au {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
        self.to_f64_px().to_css(dest)?;
        dest.write_str("px")
    }
}

macro_rules! impl_to_css_for_predefined_type {
    ($name: ty) => {
        impl<'a> ToCss for $name {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: Write {
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
            $name [ Deserialize, Serialize, MallocSizeOf ]
                  [ $( $css => $variant ),+ ]
                  [ $( $alias => $alias_variant ),* ]
        }
    };
}

#[cfg(not(feature = "servo"))]
#[macro_export]
macro_rules! __define_css_keyword_enum__add_optional_traits {
    ($name: ident [ $( $css: expr => $variant: ident ),+ ]
                  [ $( $alias: expr => $alias_variant: ident),* ]) => {
        __define_css_keyword_enum__actual! {
            $name [ MallocSizeOf ]
                  [ $( $css => $variant ),+ ]
                  [ $( $alias => $alias_variant ),* ]
        }
    };
}

#[macro_export]
macro_rules! __define_css_keyword_enum__actual {
    ($name: ident [ $( $derived_trait: ident),* ]
                  [ $( $css: expr => $variant: ident ),+ ]
                  [ $( $alias: expr => $alias_variant: ident ),* ]) => {
        #[allow(non_camel_case_types, missing_docs)]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq$(, $derived_trait )* )]
        pub enum $name {
            $( $variant ),+
        }

        impl $name {
            /// Parse this property from a CSS input stream.
            pub fn parse<'i, 't>(input: &mut ::cssparser::Parser<'i, 't>)
                                 -> Result<$name, $crate::ParseError<'i>> {
                use cssparser::Token;
                let location = input.current_source_location();
                match *input.next()? {
                    Token::Ident(ref ident) => {
                        Self::from_ident(ident).map_err(|()| {
                            location.new_unexpected_token_error(Token::Ident(ident.clone()))
                        })
                    }
                    ref token => Err(location.new_unexpected_token_error(token.clone()))
                }
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

        impl $crate::ToCss for $name {
            fn to_css<W>(&self, dest: &mut $crate::CssWriter<W>) -> ::std::fmt::Result
                where W: ::std::fmt::Write
            {
                match *self {
                    $( $name::$variant => ::std::fmt::Write::write_str(dest, $css) ),+
                }
            }
        }
    }
}

/// Helper types for the handling of specified values.
pub mod specified {
    use ParsingMode;

    /// Whether to allow negative lengths or not.
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, PartialOrd)]
    pub enum AllowedNumericType {
        /// Allow all kind of numeric values.
        All,
        /// Allow only non-negative numeric values.
        NonNegative,
        /// Allow only numeric values greater or equal to 1.0.
        AtLeastOne,
    }

    impl Default for AllowedNumericType {
        #[inline]
        fn default() -> Self {
            AllowedNumericType::All
        }
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
