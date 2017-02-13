/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A grid line type.

use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;
use values::{CSSFloat, HasViewportPercentage};
use values::computed::ComputedValueAsSpecified;
use values::specified::LengthOrPercentage;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-grid/#typedef-grid-row-start-grid-line
#[allow(missing_docs)]
pub struct GridLine {
    pub is_span: bool,
    pub ident: Option<String>,
    pub integer: Option<i32>,
}

impl Default for GridLine {
    fn default() -> Self {
        GridLine {
            is_span: false,
            ident: None,
            integer: None,
        }
    }
}

impl ToCss for GridLine {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if !self.is_span && self.ident.is_none() && self.integer.is_none() {
            return dest.write_str("auto")
        }

        if self.is_span {
            try!(dest.write_str("span"));
        }

        if let Some(i) = self.integer {
            try!(write!(dest, " {}", i));
        }

        if let Some(ref s) = self.ident {
            try!(write!(dest, " {}", s));
        }

        Ok(())
    }
}

impl Parse for GridLine {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut grid_line = Default::default();
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(grid_line)
        }

        for _ in 0..3 {     // Maximum possible entities for <grid-line>
            if input.try(|i| i.expect_ident_matching("span")).is_ok() {
                if grid_line.is_span {
                    return Err(())
                }
                grid_line.is_span = true;
            } else if let Ok(i) = input.try(|i| i.expect_integer()) {
                if i == 0 || grid_line.integer.is_some() {
                    return Err(())
                }
                grid_line.integer = Some(i);
            } else if let Ok(name) = input.try(|i| i.expect_ident()) {
                if grid_line.ident.is_some() {
                    return Err(())
                }
                grid_line.ident = Some(name.into_owned());
            } else {
                break
            }
        }

        if grid_line.is_span {
            if let Some(i) = grid_line.integer {
                if i < 0 {      // disallow negative integers for grid spans
                    return Err(())
                }
            } else {
                grid_line.integer = Some(1);
            }
        }

        Ok(grid_line)
    }
}

impl ComputedValueAsSpecified for GridLine {}
no_viewport_percentage!(GridLine);

define_css_keyword_enum!{ TrackKeyword:
    "auto" => Auto,
    "max-content" => MaxContent,
    "min-content" => MinContent
}

#[allow(missing_docs)]
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-grid/#typedef-track-breadth
pub enum TrackBreadth<L> {
    Breadth(L),
    Flex(CSSFloat),
    Keyword(TrackKeyword),
}

/// Parse a single flexible length.
pub fn parse_flex(input: &mut Parser) -> Result<CSSFloat, ()> {
    match try!(input.next()) {
        Token::Dimension(ref value, ref unit) if unit.to_lowercase() == "fr" && value.value.is_sign_positive()
            => Ok(value.value),
        _ => Err(()),
    }
}

impl Parse for TrackBreadth<LengthOrPercentage> {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(lop) = input.try(LengthOrPercentage::parse_non_negative) {
            Ok(TrackBreadth::Breadth(lop))
        } else {
            if let Ok(f) = input.try(parse_flex) {
                Ok(TrackBreadth::Flex(f))
            } else {
                TrackKeyword::parse(input).map(TrackBreadth::Keyword)
            }
        }
    }
}

impl<L: ToCss> ToCss for TrackBreadth<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TrackBreadth::Breadth(ref lop) => lop.to_css(dest),
            TrackBreadth::Flex(ref value) => write!(dest, "{}fr", value),
            TrackBreadth::Keyword(ref k) => k.to_css(dest),
        }
    }
}

impl HasViewportPercentage for TrackBreadth<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        if let TrackBreadth::Breadth(ref lop) = *self {
            lop.has_viewport_percentage()
        } else {
            false
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-grid/#typedef-track-size
pub enum TrackSize<L> {
    Breadth(TrackBreadth<L>),
    MinMax(TrackBreadth<L>, TrackBreadth<L>),
    FitContent(L),
}

impl<L> Default for TrackSize<L> {
    fn default() -> Self {
        TrackSize::Breadth(TrackBreadth::Keyword(TrackKeyword::Auto))
    }
}

impl Parse for TrackSize<LengthOrPercentage> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(b) = input.try(|i| TrackBreadth::parse(context, i)) {
            Ok(TrackSize::Breadth(b))
        } else {
            if input.try(|i| i.expect_function_matching("minmax")).is_ok() {
                Ok(try!(input.parse_nested_block(|input| {
                    let inflexible_breadth = if let Ok(lop) = input.try(LengthOrPercentage::parse_non_negative) {
                        Ok(TrackBreadth::Breadth(lop))
                    } else {
                        TrackKeyword::parse(input).map(TrackBreadth::Keyword)
                    };

                    try!(input.expect_comma());
                    Ok(TrackSize::MinMax(try!(inflexible_breadth), try!(TrackBreadth::parse(context, input))))
                })))
            } else {
                try!(input.expect_function_matching("fit-content"));
                Ok(try!(LengthOrPercentage::parse(context, input).map(TrackSize::FitContent)))
            }
        }
    }
}

impl<L: ToCss> ToCss for TrackSize<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TrackSize::Breadth(ref b) => b.to_css(dest),
            TrackSize::MinMax(ref infexible, ref flexible) => {
                try!(dest.write_str("minmax("));
                try!(infexible.to_css(dest));
                try!(dest.write_str(","));
                try!(flexible.to_css(dest));
                dest.write_str(")")
            },
            TrackSize::FitContent(ref lop) => {
                try!(dest.write_str("fit-content("));
                try!(lop.to_css(dest));
                dest.write_str(")")
            },
        }
    }
}

impl HasViewportPercentage for TrackSize<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            TrackSize::Breadth(ref b) => b.has_viewport_percentage(),
            TrackSize::MinMax(ref inf_b, ref b) => inf_b.has_viewport_percentage() || b.has_viewport_percentage(),
            TrackSize::FitContent(ref lop) => lop.has_viewport_percentage(),
        }
    }
}
