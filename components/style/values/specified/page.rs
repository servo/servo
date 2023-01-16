/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified @page at-rule properties and named-page style properties

use crate::parser::{Parse, ParserContext};
use crate::values::generics::size::Size2D;
use crate::values::specified::length::NonNegativeLength;
use crate::values::{generics, CustomIdent};
use cssparser::Parser;
use style_traits::ParseError;

pub use generics::page::PageOrientation;
pub use generics::page::PageSizeOrientation;
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
            let orientation = input
                .try_parse(PageSizeOrientation::parse)
                .unwrap_or(PageSizeOrientation::Portrait);
            return Ok(PageSize::PaperSize(paper_size, orientation));
        }
        // Try to parse as <orientation> [ <page-size> ]
        if let Ok(orientation) = input.try_parse(PageSizeOrientation::parse) {
            if let Ok(paper_size) = input.try_parse(PaperSize::parse) {
                return Ok(PageSize::PaperSize(paper_size, orientation));
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

/// Page name value.
///
/// https://drafts.csswg.org/css-page-3/#using-named-pages
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum PageName {
    /// `auto` value.
    Auto,
    /// Page name value
    PageName(CustomIdent),
}

impl Parse for PageName {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        Ok(match_ignore_ascii_case! { ident,
            "auto" => PageName::auto(),
            _ => PageName::PageName(CustomIdent::from_ident(location, ident, &[])?),
        })
    }
}

impl PageName {
    /// `auto` value.
    #[inline]
    pub fn auto() -> Self {
        PageName::Auto
    }

    /// Whether this is the `auto` value.
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(*self, PageName::Auto)
    }
}
