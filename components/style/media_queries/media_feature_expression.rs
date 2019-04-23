/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Parsing for media feature expressions, like `(foo: bar)` or
//! `(width >= 400px)`.

use super::media_feature::{Evaluator, MediaFeatureDescription};
use super::media_feature::{KeywordDiscriminant, ParsingRequirements};
use super::Device;
use crate::context::QuirksMode;
#[cfg(feature = "gecko")]
use crate::gecko::media_features::MEDIA_FEATURES;
#[cfg(feature = "gecko")]
use crate::gecko_bindings::structs;
use crate::parser::{Parse, ParserContext};
#[cfg(feature = "servo")]
use crate::servo::media_queries::MEDIA_FEATURES;
use crate::str::{starts_with_ignore_ascii_case, string_as_ascii_lowercase};
use crate::values::computed::{self, ToComputedValue};
use crate::values::specified::{Integer, Length, Number, Resolution};
use crate::values::{serialize_atom_identifier, CSSFloat};
use crate::{Atom, Zero};
use cssparser::{Parser, Token};
use std::cmp::{Ordering, PartialOrd};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// An aspect ratio, with a numerator and denominator.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub struct AspectRatio(pub u32, pub u32);

impl ToCss for AspectRatio {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.0.to_css(dest)?;
        dest.write_char('/')?;
        self.1.to_css(dest)
    }
}

impl PartialOrd for AspectRatio {
    fn partial_cmp(&self, other: &AspectRatio) -> Option<Ordering> {
        u64::partial_cmp(
            &(self.0 as u64 * other.1 as u64),
            &(self.1 as u64 * other.0 as u64),
        )
    }
}

/// The kind of matching that should be performed on a media feature value.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub enum Range {
    /// At least the specified value.
    Min,
    /// At most the specified value.
    Max,
}

/// The operator that was specified in this media feature.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub enum Operator {
    /// =
    Equal,
    /// >
    GreaterThan,
    /// >=
    GreaterThanEqual,
    /// <
    LessThan,
    /// <=
    LessThanEqual,
}

impl ToCss for Operator {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(match *self {
            Operator::Equal => "=",
            Operator::LessThan => "<",
            Operator::LessThanEqual => "<=",
            Operator::GreaterThan => ">",
            Operator::GreaterThanEqual => ">=",
        })
    }
}

/// Either a `Range` or an `Operator`.
///
/// Ranged media features are not allowed with operations (that'd make no
/// sense).
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub enum RangeOrOperator {
    /// A `Range`.
    Range(Range),
    /// An `Operator`.
    Operator(Operator),
}

impl RangeOrOperator {
    /// Evaluate a given range given an optional query value and a value from
    /// the browser.
    pub fn evaluate<T>(range_or_op: Option<Self>, query_value: Option<T>, value: T) -> bool
    where
        T: PartialOrd + Zero,
    {
        match query_value {
            Some(v) => Self::evaluate_with_query_value(range_or_op, v, value),
            None => !value.is_zero(),
        }
    }

    /// Evaluate a given range given a non-optional query value and a value from
    /// the browser.
    pub fn evaluate_with_query_value<T>(range_or_op: Option<Self>, query_value: T, value: T) -> bool
    where
        T: PartialOrd,
    {
        let cmp = match value.partial_cmp(&query_value) {
            Some(c) => c,
            None => return false,
        };

        let range_or_op = match range_or_op {
            Some(r) => r,
            None => return cmp == Ordering::Equal,
        };

        match range_or_op {
            RangeOrOperator::Range(range) => {
                cmp == Ordering::Equal ||
                    match range {
                        Range::Min => cmp == Ordering::Greater,
                        Range::Max => cmp == Ordering::Less,
                    }
            },
            RangeOrOperator::Operator(op) => match op {
                Operator::Equal => cmp == Ordering::Equal,
                Operator::GreaterThan => cmp == Ordering::Greater,
                Operator::GreaterThanEqual => cmp == Ordering::Equal || cmp == Ordering::Greater,
                Operator::LessThan => cmp == Ordering::Less,
                Operator::LessThanEqual => cmp == Ordering::Equal || cmp == Ordering::Less,
            },
        }
    }
}

/// A feature expression contains a reference to the media feature, the value
/// the media query contained, and the range to evaluate.
#[derive(Clone, Debug, MallocSizeOf, ToShmem)]
pub struct MediaFeatureExpression {
    feature_index: usize,
    value: Option<MediaExpressionValue>,
    range_or_operator: Option<RangeOrOperator>,
}

impl PartialEq for MediaFeatureExpression {
    fn eq(&self, other: &Self) -> bool {
        self.feature_index == other.feature_index &&
            self.value == other.value &&
            self.range_or_operator == other.range_or_operator
    }
}

impl ToCss for MediaFeatureExpression {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str("(")?;

        let feature = self.feature();

        if feature
            .requirements
            .contains(ParsingRequirements::WEBKIT_PREFIX)
        {
            dest.write_str("-webkit-")?;
        }

        if let Some(RangeOrOperator::Range(range)) = self.range_or_operator {
            match range {
                Range::Min => dest.write_str("min-")?,
                Range::Max => dest.write_str("max-")?,
            }
        }

        // NB: CssStringWriter not needed, feature names are under control.
        write!(dest, "{}", feature.name)?;

        if let Some(RangeOrOperator::Operator(op)) = self.range_or_operator {
            dest.write_char(' ')?;
            op.to_css(dest)?;
            dest.write_char(' ')?;
        } else if self.value.is_some() {
            dest.write_str(": ")?;
        }

        if let Some(ref val) = self.value {
            val.to_css(dest, self)?;
        }

        dest.write_str(")")
    }
}

/// Consumes an operation or a colon, or returns an error.
fn consume_operation_or_colon(input: &mut Parser) -> Result<Option<Operator>, ()> {
    let first_delim = {
        let next_token = match input.next() {
            Ok(t) => t,
            Err(..) => return Err(()),
        };

        match *next_token {
            Token::Colon => return Ok(None),
            Token::Delim(oper) => oper,
            _ => return Err(()),
        }
    };
    Ok(Some(match first_delim {
        '=' => Operator::Equal,
        '>' => {
            if input.try(|i| i.expect_delim('=')).is_ok() {
                Operator::GreaterThanEqual
            } else {
                Operator::GreaterThan
            }
        },
        '<' => {
            if input.try(|i| i.expect_delim('=')).is_ok() {
                Operator::LessThanEqual
            } else {
                Operator::LessThan
            }
        },
        _ => return Err(()),
    }))
}

impl MediaFeatureExpression {
    fn new(
        feature_index: usize,
        value: Option<MediaExpressionValue>,
        range_or_operator: Option<RangeOrOperator>,
    ) -> Self {
        debug_assert!(feature_index < MEDIA_FEATURES.len());
        Self {
            feature_index,
            value,
            range_or_operator,
        }
    }

    fn feature(&self) -> &'static MediaFeatureDescription {
        &MEDIA_FEATURES[self.feature_index]
    }

    /// Parse a media expression of the form:
    ///
    /// ```
    /// (media-feature: media-value)
    /// ```
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_parenthesis_block()?;
        input.parse_nested_block(|input| Self::parse_in_parenthesis_block(context, input))
    }

    /// Parse a media feature expression where we've already consumed the
    /// parenthesis.
    pub fn parse_in_parenthesis_block<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // FIXME: remove extra indented block when lifetimes are non-lexical
        let feature_index;
        let feature;
        let range;
        {
            let location = input.current_source_location();
            let ident = input.expect_ident()?;

            let mut requirements = ParsingRequirements::empty();

            if context.in_ua_or_chrome_sheet() {
                requirements.insert(ParsingRequirements::CHROME_AND_UA_ONLY);
            }

            let result = {
                let mut feature_name = &**ident;

                #[cfg(feature = "gecko")]
                {
                    if unsafe { structs::StaticPrefs_sVarCache_layout_css_prefixes_webkit } &&
                        starts_with_ignore_ascii_case(feature_name, "-webkit-")
                    {
                        feature_name = &feature_name[8..];
                        requirements.insert(ParsingRequirements::WEBKIT_PREFIX);
                        if unsafe {
                            structs::StaticPrefs_sVarCache_layout_css_prefixes_device_pixel_ratio_webkit
                        } {
                            requirements.insert(
                                ParsingRequirements::WEBKIT_DEVICE_PIXEL_RATIO_PREF_ENABLED,
                            );
                        }
                    }
                }

                let range = if starts_with_ignore_ascii_case(feature_name, "min-") {
                    feature_name = &feature_name[4..];
                    Some(Range::Min)
                } else if starts_with_ignore_ascii_case(feature_name, "max-") {
                    feature_name = &feature_name[4..];
                    Some(Range::Max)
                } else {
                    None
                };

                let atom = Atom::from(string_as_ascii_lowercase(feature_name));
                match MEDIA_FEATURES
                    .iter()
                    .enumerate()
                    .find(|(_, f)| f.name == atom)
                {
                    Some((i, f)) => Ok((i, f, range)),
                    None => Err(()),
                }
            };

            match result {
                Ok((i, f, r)) => {
                    feature_index = i;
                    feature = f;
                    range = r;
                },
                Err(()) => {
                    return Err(location.new_custom_error(
                        StyleParseErrorKind::MediaQueryExpectedFeatureName(ident.clone()),
                    ));
                },
            }

            if !(feature.requirements & !requirements).is_empty() {
                return Err(location.new_custom_error(
                    StyleParseErrorKind::MediaQueryExpectedFeatureName(ident.clone()),
                ));
            }

            if range.is_some() && !feature.allows_ranges() {
                return Err(location.new_custom_error(
                    StyleParseErrorKind::MediaQueryExpectedFeatureName(ident.clone()),
                ));
            }
        }

        let operator = input.try(consume_operation_or_colon);
        let operator = match operator {
            Err(..) => {
                // If there's no colon, this is a media query of the
                // form '(<feature>)', that is, there's no value
                // specified.
                //
                // Gecko doesn't allow ranged expressions without a
                // value, so just reject them here too.
                if range.is_some() {
                    return Err(
                        input.new_custom_error(StyleParseErrorKind::RangedExpressionWithNoValue)
                    );
                }

                return Ok(Self::new(feature_index, None, None));
            },
            Ok(operator) => operator,
        };

        let range_or_operator = match range {
            Some(range) => {
                if operator.is_some() {
                    return Err(
                        input.new_custom_error(StyleParseErrorKind::MediaQueryUnexpectedOperator)
                    );
                }
                Some(RangeOrOperator::Range(range))
            },
            None => match operator {
                Some(operator) => {
                    if !feature.allows_ranges() {
                        return Err(input
                            .new_custom_error(StyleParseErrorKind::MediaQueryUnexpectedOperator));
                    }
                    Some(RangeOrOperator::Operator(operator))
                },
                None => None,
            },
        };

        let value = MediaExpressionValue::parse(feature, context, input).map_err(|err| {
            err.location
                .new_custom_error(StyleParseErrorKind::MediaQueryExpectedFeatureValue)
        })?;

        Ok(Self::new(feature_index, Some(value), range_or_operator))
    }

    /// Returns whether this media query evaluates to true for the given device.
    pub fn matches(&self, device: &Device, quirks_mode: QuirksMode) -> bool {
        let value = self.value.as_ref();

        macro_rules! expect {
            ($variant:ident) => {
                value.map(|value| match *value {
                    MediaExpressionValue::$variant(ref v) => v,
                    _ => unreachable!("Unexpected MediaExpressionValue"),
                })
            };
        }

        match self.feature().evaluator {
            Evaluator::Length(eval) => {
                let computed = expect!(Length).map(|specified| {
                    computed::Context::for_media_query_evaluation(device, quirks_mode, |context| {
                        specified.to_computed_value(context)
                    })
                });
                eval(device, computed, self.range_or_operator)
            },
            Evaluator::Integer(eval) => {
                eval(device, expect!(Integer).cloned(), self.range_or_operator)
            },
            Evaluator::Float(eval) => eval(device, expect!(Float).cloned(), self.range_or_operator),
            Evaluator::IntRatio(eval) => {
                eval(device, expect!(IntRatio).cloned(), self.range_or_operator)
            },
            Evaluator::Resolution(eval) => {
                let computed = expect!(Resolution).map(|specified| {
                    computed::Context::for_media_query_evaluation(device, quirks_mode, |context| {
                        specified.to_computed_value(context)
                    })
                });
                eval(device, computed, self.range_or_operator)
            },
            Evaluator::Enumerated { evaluator, .. } => {
                evaluator(device, expect!(Enumerated).cloned(), self.range_or_operator)
            },
            Evaluator::Ident(eval) => eval(device, expect!(Ident).cloned(), self.range_or_operator),
            Evaluator::BoolInteger(eval) => eval(
                device,
                expect!(BoolInteger).cloned(),
                self.range_or_operator,
            ),
        }
    }
}

/// A value found or expected in a media expression.
///
/// FIXME(emilio): How should calc() serialize in the Number / Integer /
/// BoolInteger / IntRatio case, as computed or as specified value?
///
/// If the first, this would need to store the relevant values.
///
/// See: https://github.com/w3c/csswg-drafts/issues/1968
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub enum MediaExpressionValue {
    /// A length.
    Length(Length),
    /// A (non-negative) integer.
    Integer(u32),
    /// A floating point value.
    Float(CSSFloat),
    /// A boolean value, specified as an integer (i.e., either 0 or 1).
    BoolInteger(bool),
    /// Two integers separated by '/', with optional whitespace on either side
    /// of the '/'.
    IntRatio(AspectRatio),
    /// A resolution.
    Resolution(Resolution),
    /// An enumerated value, defined by the variant keyword table in the
    /// feature's `mData` member.
    Enumerated(KeywordDiscriminant),
    /// An identifier.
    Ident(Atom),
}

impl MediaExpressionValue {
    fn to_css<W>(&self, dest: &mut CssWriter<W>, for_expr: &MediaFeatureExpression) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            MediaExpressionValue::Length(ref l) => l.to_css(dest),
            MediaExpressionValue::Integer(v) => v.to_css(dest),
            MediaExpressionValue::Float(v) => v.to_css(dest),
            MediaExpressionValue::BoolInteger(v) => dest.write_str(if v { "1" } else { "0" }),
            MediaExpressionValue::IntRatio(ratio) => ratio.to_css(dest),
            MediaExpressionValue::Resolution(ref r) => r.to_css(dest),
            MediaExpressionValue::Ident(ref ident) => serialize_atom_identifier(ident, dest),
            MediaExpressionValue::Enumerated(value) => match for_expr.feature().evaluator {
                Evaluator::Enumerated { serializer, .. } => dest.write_str(&*serializer(value)),
                _ => unreachable!(),
            },
        }
    }

    fn parse<'i, 't>(
        for_feature: &MediaFeatureDescription,
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MediaExpressionValue, ParseError<'i>> {
        Ok(match for_feature.evaluator {
            Evaluator::Length(..) => {
                let length = Length::parse_non_negative(context, input)?;
                MediaExpressionValue::Length(length)
            },
            Evaluator::Integer(..) => {
                let integer = Integer::parse_non_negative(context, input)?;
                MediaExpressionValue::Integer(integer.value() as u32)
            },
            Evaluator::BoolInteger(..) => {
                let integer = Integer::parse_non_negative(context, input)?;
                let value = integer.value();
                if value > 1 {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                MediaExpressionValue::BoolInteger(value == 1)
            },
            Evaluator::Float(..) => {
                let number = Number::parse(context, input)?;
                MediaExpressionValue::Float(number.get())
            },
            Evaluator::IntRatio(..) => {
                let a = Integer::parse_positive(context, input)?;
                input.expect_delim('/')?;
                let b = Integer::parse_positive(context, input)?;
                MediaExpressionValue::IntRatio(AspectRatio(a.value() as u32, b.value() as u32))
            },
            Evaluator::Resolution(..) => {
                MediaExpressionValue::Resolution(Resolution::parse(context, input)?)
            },
            Evaluator::Enumerated { parser, .. } => {
                MediaExpressionValue::Enumerated(parser(context, input)?)
            },
            Evaluator::Ident(..) => {
                MediaExpressionValue::Ident(Atom::from(input.expect_ident()?.as_ref()))
            },
        })
    }
}
