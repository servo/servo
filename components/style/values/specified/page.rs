/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified @page at-rule properties

use crate::parser::{Parse, ParserContext};
use crate::values::generics;
use crate::values::generics::size::Size2D;
use crate::values::specified::length::NonNegativeLength;
use cssparser::Parser;
use style_traits::ParseError;

pub use generics::page::Orientation;
pub use generics::page::PaperSize;
/// Specified value of the @page size descriptor
pub type PageSize = generics::page::PageSize<Size2D<NonNegativeLength>>;

impl Parse for PageSize {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Try to parse as <page-size> [ <orientation> ]
        if let Ok(paper_size) = input.try_parse(PaperSize::parse) {
            if let Ok(orientation) = input.try_parse(Orientation::parse) {
                return Ok(PageSize::PaperSizeAndOrientation(paper_size, orientation));
            }
            return Ok(PageSize::PaperSize(paper_size));
        }
        // Try to parse as <orientation> [ <page-size> ]
        if let Ok(orientation) = input.try_parse(Orientation::parse) {
            if let Ok(paper_size) = input.try_parse(PaperSize::parse) {
                return Ok(PageSize::PaperSizeAndOrientation(paper_size, orientation));
            }
            return Ok(PageSize::Orientation(orientation));
        }
        // Try to parse dimensions
        if let Ok(size) =
            input.try_parse(|i| Size2D::parse_with(context, i, NonNegativeLength::parse))
        {
            return Ok(PageSize::Size(size));
        }
        // auto value
        input.expect_ident_matching("auto")?;
        Ok(PageSize::Auto)
    }
}
