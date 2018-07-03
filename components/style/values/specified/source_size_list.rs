/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! https://html.spec.whatwg.org/multipage/#source-size-list

use app_units::Au;
use cssparser::{Delimiter, Parser, Token};
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use media_queries::{Device, MediaCondition};
use parser::{Parse, ParserContext};
use selectors::context::QuirksMode;
use style_traits::ParseError;
use values::computed::{self, ToComputedValue};
use values::specified::{Length, NoCalcLength, ViewportPercentageLength};

/// A value for a `<source-size>`:
///
/// https://html.spec.whatwg.org/multipage/#source-size
#[derive(Debug)]
pub struct SourceSize {
    condition: MediaCondition,
    value: Length,
}

impl Parse for SourceSize {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let condition = MediaCondition::parse(context, input)?;
        let value = Length::parse_non_negative(context, input)?;

        Ok(Self { condition, value })
    }
}

/// A value for a `<source-size-list>`:
///
/// https://html.spec.whatwg.org/multipage/#source-size-list
#[derive(Debug)]
pub struct SourceSizeList {
    source_sizes: Vec<SourceSize>,
    value: Option<Length>,
}

impl SourceSizeList {
    /// Create an empty `SourceSizeList`, which can be used as a fall-back.
    pub fn empty() -> Self {
        Self {
            source_sizes: vec![],
            value: None,
        }
    }

    /// Set content of `value`, which can be used as fall-back during evaluate.
    pub fn set_fallback_value(&mut self, width: Option<Length>) {
        self.value = width;
    }

    /// Evaluate this <source-size-list> to get the final viewport length.
    pub fn evaluate(&self, device: &Device, quirks_mode: QuirksMode) -> Au {
        let matching_source_size = self.source_sizes
            .iter()
            .find(|source_size| source_size.condition.matches(device, quirks_mode));

        computed::Context::for_media_query_evaluation(device, quirks_mode, |context| {
            match matching_source_size {
                Some(source_size) => source_size.value.to_computed_value(context),
                None => match self.value {
                    Some(ref v) => v.to_computed_value(context),
                    None => Length::NoCalc(NoCalcLength::ViewportPercentage(
                        ViewportPercentageLength::Vw(100.),
                    )).to_computed_value(context),
                },
            }
        }).into()
    }
}

enum SourceSizeOrLength {
    SourceSize(SourceSize),
    Length(Length),
}

impl Parse for SourceSizeOrLength {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(size) = input.try(|input| SourceSize::parse(context, input)) {
            return Ok(SourceSizeOrLength::SourceSize(size));
        }

        let length = Length::parse_non_negative(context, input)?;
        Ok(SourceSizeOrLength::Length(length))
    }
}

impl SourceSizeList {
    /// NOTE(emilio): This doesn't match the grammar in the spec, see:
    ///
    /// https://html.spec.whatwg.org/multipage/#parsing-a-sizes-attribute
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Self {
        let mut source_sizes = vec![];

        loop {
            let result = input.parse_until_before(Delimiter::Comma, |input| {
                SourceSizeOrLength::parse(context, input)
            });

            match result {
                Ok(SourceSizeOrLength::Length(value)) => {
                    return Self {
                        source_sizes,
                        value: Some(value),
                    }
                },
                Ok(SourceSizeOrLength::SourceSize(source_size)) => {
                    source_sizes.push(source_size);
                },
                Err(..) => {},
            }

            match input.next() {
                Ok(&Token::Comma) => {},
                Err(..) => break,
                _ => unreachable!(),
            }
        }

        SourceSizeList {
            source_sizes,
            value: None,
        }
    }
}

#[cfg(feature = "gecko")]
unsafe impl HasFFI for SourceSizeList {
    type FFIType = ::gecko_bindings::structs::RawServoSourceSizeList;
}
#[cfg(feature = "gecko")]
unsafe impl HasSimpleFFI for SourceSizeList {}
#[cfg(feature = "gecko")]
unsafe impl HasBoxFFI for SourceSizeList {}
