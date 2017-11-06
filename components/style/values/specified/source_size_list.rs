/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/#source-size-list

use app_units::Au;
use cssparser::Parser;
use media_queries::{Device, Expression as MediaExpression};
use parser::{Parse, ParserContext};
use selectors::context::QuirksMode;
use style_traits::ParseError;
use values::computed::{self, ToComputedValue};
use values::specified::Length;

/// A value for a `<source-size>`:
///
/// https://html.spec.whatwg.org/multipage/#source-size
pub struct SourceSize {
    condition: MediaExpression,
    value: Length,
}

impl Parse for SourceSize {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let condition = MediaExpression::parse(context, input)?;
        let value = Length::parse_non_negative(context, input)?;

        Ok(Self { condition, value })
    }
}

/// A value for a `<source-size-list>`:
///
/// https://html.spec.whatwg.org/multipage/#source-size-list
pub struct SourceSizeList {
    source_sizes: Vec<SourceSize>,
    value: Length,
}

impl SourceSizeList {
    /// Evaluate this <source-size-list> to get the final viewport length.
    pub fn evaluate(&self, device: &Device, quirks_mode: QuirksMode) -> Au {
        let matching_source_size = self.source_sizes.iter().find(|source_size| {
            source_size.condition.matches(device, quirks_mode)
        });

        computed::Context::for_media_query_evaluation(device, quirks_mode, |context| {
            match matching_source_size {
                Some(source_size) => {
                    source_size.value.to_computed_value(context)
                }
                None => {
                    self.value.to_computed_value(context)
                }
            }
        }).into()
    }
}

impl Parse for SourceSizeList {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let source_sizes = input.try(|input| {
            input.parse_comma_separated(|input| {
                SourceSize::parse(context, input)
            })
        }).unwrap_or(vec![]);

        let value = Length::parse_non_negative(context, input)?;

        Ok(Self { source_sizes, value })
    }
}
