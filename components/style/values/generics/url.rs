/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for url properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::ParseError;

/// An image url or none, used for example in list-style-image
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedValue, ToAnimatedZero, ToComputedValue,
         ToCss)]
pub enum UrlOrNone<Url> {
    /// `none`
    None,
    /// `A URL`
    Url(Url),
}

impl<Url> UrlOrNone<Url> {
    /// Initial "none" value for properties such as `list-style-image`
    pub fn none() -> Self {
        UrlOrNone::None
    }
}

impl<Url> Parse for UrlOrNone<Url>
where
    Url: Parse,
{
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<UrlOrNone<Url>, ParseError<'i>> {
        if let Ok(url) = input.try(|input| Url::parse(context, input)) {
            return Ok(UrlOrNone::Url(url));
        }
        input.expect_ident_matching("none")?;
        Ok(UrlOrNone::None)
    }
}
