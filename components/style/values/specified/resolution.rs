/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Resolution values:
//!
//! https://drafts.csswg.org/css-values/#resolution

use crate::parser::{Parse, ParserContext};
use crate::values::specified::CalcNode;
use crate::values::CSSFloat;
use cssparser::{Parser, Token};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// A specified resolution.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToShmem)]
pub struct Resolution {
    value: CSSFloat,
    unit: ResolutionUnit,
    was_calc: bool,
}

#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
enum ResolutionUnit {
    /// Dots per inch.
    Dpi,
    /// An alias unit for dots per pixel.
    X,
    /// Dots per pixel.
    Dppx,
    /// Dots per centimeter.
    Dpcm,
}

impl ResolutionUnit {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dpi => "dpi",
            Self::X => "x",
            Self::Dppx => "dppx",
            Self::Dpcm => "dpcm",
        }
    }
}

impl Resolution {
    /// Returns a resolution value from dppx units.
    pub fn from_dppx(value: CSSFloat) -> Self {
        Self {
            value,
            unit: ResolutionUnit::Dppx,
            was_calc: false,
        }
    }

    /// Returns a resolution value from dppx units.
    pub fn from_x(value: CSSFloat) -> Self {
        Self {
            value,
            unit: ResolutionUnit::X,
            was_calc: false,
        }
    }

    /// Returns a resolution value from dppx units.
    pub fn from_dppx_calc(value: CSSFloat) -> Self {
        Self {
            value,
            unit: ResolutionUnit::Dppx,
            was_calc: true,
        }
    }

    /// Convert this resolution value to dppx units.
    pub fn dppx(&self) -> CSSFloat {
        match self.unit {
            ResolutionUnit::X | ResolutionUnit::Dppx => self.value,
            _ => self.dpi() / 96.0,
        }
    }

    /// Convert this resolution value to dpi units.
    pub fn dpi(&self) -> CSSFloat {
        match self.unit {
            ResolutionUnit::Dpi => self.value,
            ResolutionUnit::X | ResolutionUnit::Dppx => self.value * 96.0,
            ResolutionUnit::Dpcm => self.value * 2.54,
        }
    }

    /// Parse a resolution given a value and unit.
    pub fn parse_dimension<'i, 't>(value: CSSFloat, unit: &str) -> Result<Self, ()> {
        let unit = match_ignore_ascii_case! { &unit,
            "dpi" => ResolutionUnit::Dpi,
            "dppx" => ResolutionUnit::Dppx,
            "dpcm" => ResolutionUnit::Dpcm,
            "x" => ResolutionUnit::X,
            _ => return Err(())
        };
        Ok(Self {
            value,
            unit,
            was_calc: false,
        })
    }
}

impl ToCss for Resolution {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        crate::values::serialize_specified_dimension(
            self.value,
            self.unit.as_str(),
            self.was_calc,
            dest,
        )
    }
}

impl Parse for Resolution {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        match *input.next()? {
            Token::Dimension {
                value, ref unit, ..
            } if value >= 0. => Self::parse_dimension(value, unit)
                .map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
            Token::Function(ref name) => {
                let function = CalcNode::math_function(context, name, location)?;
                CalcNode::parse_resolution(context, input, function)
            },
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}
