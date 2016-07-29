/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

pub use cssparser::RGBA;

use app_units::Au;
use cssparser::CssStringWriter;
use std::fmt::{self, Write};
use url::Url;

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

        impl ::cssparser::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write {
                match *self {
                    $( $name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

pub mod computed;
pub mod specified;

/// The real ToCss trait can't be implemented for types in crates that don't
/// depend on each other.
pub trait LocalToCss {
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

impl LocalToCss for Au {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}px", self.to_f64_px())
    }
}

impl LocalToCss for Url {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("url(\""));
        try!(write!(CssStringWriter::new(dest), "{}", self));
        try!(dest.write_str("\")"));
        Ok(())
    }
}

pub type CSSFloat = f32;

pub const FONT_MEDIUM_PX: i32 = 16;

pub trait HasViewportPercentage {
    fn has_viewport_percentage(&self) -> bool;
}

pub trait NoViewportPercentage {}

impl<T> HasViewportPercentage for T where T: NoViewportPercentage {
    fn has_viewport_percentage(&self) -> bool {
        false
    }
}

