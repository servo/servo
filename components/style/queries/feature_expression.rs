/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Parsing for query feature expressions, like `(foo: bar)` or
//! `(width >= 400px)`.

use super::feature::{Evaluator, QueryFeatureDescription};
use super::feature::{KeywordDiscriminant, ParsingRequirements};
use crate::parser::{Parse, ParserContext};
use crate::str::{starts_with_ignore_ascii_case, string_as_ascii_lowercase};
use crate::values::computed::{self, Ratio, ToComputedValue};
use crate::values::specified::{Integer, Length, Number, Resolution};
use crate::values::CSSFloat;
use crate::{Atom, Zero};
use cssparser::{Parser, Token};
use std::cmp::{Ordering, PartialOrd};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// Whether we're parsing a media or container query feature.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub enum FeatureType {
    /// We're parsing a media feature.
    Media,
    /// We're parsing a container feature.
    Container,
}

impl FeatureType {
    fn features(&self) -> &'static [QueryFeatureDescription] {
        #[cfg(feature = "gecko")]
        use crate::gecko::media_features::MEDIA_FEATURES;
        #[cfg(feature = "servo")]
        use crate::servo::media_queries::MEDIA_FEATURES;

        use crate::stylesheets::container_rule::CONTAINER_FEATURES;

        match *self {
            FeatureType::Media => &MEDIA_FEATURES,
            FeatureType::Container => &CONTAINER_FEATURES,
        }
    }

    fn find_feature(&self, name: &Atom) -> Option<(usize, &'static QueryFeatureDescription)> {
        self.features().iter().enumerate().find(|(_, f)| f.name == *name)
    }
}


/// The kind of matching that should be performed on a feature value.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub enum Range {
    /// At least the specified value.
    Min,
    /// At most the specified value.
    Max,
}

/// The operator that was specified in this feature.
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
/// Ranged features are not allowed with operations (that'd make no sense).
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
    fn evaluate<T>(range_or_op: Option<Self>, query_value: Option<T>, value: T) -> bool
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
    fn evaluate_with_query_value<T>(range_or_op: Option<Self>, query_value: T, value: T) -> bool
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

/// A feature expression contains a reference to the feature, the value the
/// query contained, and the range to evaluate.
#[derive(Clone, Debug, MallocSizeOf, ToShmem, PartialEq)]
pub struct QueryFeatureExpression {
    feature_type: FeatureType,
    feature_index: usize,
    value: Option<QueryExpressionValue>,
    range_or_operator: Option<RangeOrOperator>,
}

impl ToCss for QueryFeatureExpression {
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
    let operator = match first_delim {
        '=' => return Ok(Some(Operator::Equal)),
        '>' => Operator::GreaterThan,
        '<' => Operator::LessThan,
        _ => return Err(()),
    };

    // https://drafts.csswg.org/mediaqueries-4/#mq-syntax:
    //
    //     No whitespace is allowed between the “<” or “>”
    //     <delim-token>s and the following “=” <delim-token>, if it’s
    //     present.
    //
    // TODO(emilio): Maybe we should ignore comments as well?
    // https://github.com/w3c/csswg-drafts/issues/6248
    let parsed_equal = input
        .try_parse(|i| {
            let t = i.next_including_whitespace().map_err(|_| ())?;
            if !matches!(t, Token::Delim('=')) {
                return Err(());
            }
            Ok(())
        })
        .is_ok();

    if !parsed_equal {
        return Ok(Some(operator));
    }

    Ok(Some(match operator {
        Operator::GreaterThan => Operator::GreaterThanEqual,
        Operator::LessThan => Operator::LessThanEqual,
        _ => unreachable!(),
    }))
}

#[allow(unused_variables)]
fn disabled_by_pref(feature: &Atom, context: &ParserContext) -> bool {
    #[cfg(feature = "gecko")]
    {
        if *feature == atom!("forced-colors") {
            // forced-colors is always enabled in the ua and chrome. On
            // the web it is hidden behind a preference, which is defaulted
            // to 'true' as of bug 1659511.
            return !context.in_ua_or_chrome_sheet() &&
                !static_prefs::pref!("layout.css.forced-colors.enabled");
        }
        // prefers-contrast is always enabled in the ua and chrome. On
        // the web it is hidden behind a preference.
        if *feature == atom!("prefers-contrast") {
            return !context.in_ua_or_chrome_sheet() &&
                !static_prefs::pref!("layout.css.prefers-contrast.enabled");
        }
    }
    false
}

impl QueryFeatureExpression {
    fn new(
        feature_type: FeatureType,
        feature_index: usize,
        value: Option<QueryExpressionValue>,
        range_or_operator: Option<RangeOrOperator>,
    ) -> Self {
        debug_assert!(feature_index < feature_type.features().len());
        Self {
            feature_type,
            feature_index,
            value,
            range_or_operator,
        }
    }

    fn feature(&self) -> &'static QueryFeatureDescription {
        &self.feature_type.features()[self.feature_index]
    }

    /// Parse a feature expression of the form:
    ///
    /// ```
    /// (media-feature: media-value)
    /// ```
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_parenthesis_block()?;
        input.parse_nested_block(|input| Self::parse_in_parenthesis_block(context, input, feature_type))
    }

    /// Parse a feature expression where we've already consumed the parenthesis.
    pub fn parse_in_parenthesis_block<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        let mut requirements = ParsingRequirements::empty();
        let location = input.current_source_location();
        let ident = input.expect_ident()?;

        if context.in_ua_or_chrome_sheet() {
            requirements.insert(ParsingRequirements::CHROME_AND_UA_ONLY);
        }

        let mut feature_name = &**ident;

        if starts_with_ignore_ascii_case(feature_name, "-webkit-") {
            feature_name = &feature_name[8..];
            requirements.insert(ParsingRequirements::WEBKIT_PREFIX);
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

        let (feature_index, feature) = match feature_type.find_feature(&atom) {
            Some((i, f)) => (i, f),
            None => {
                return Err(location.new_custom_error(
                    StyleParseErrorKind::MediaQueryExpectedFeatureName(ident.clone()),
                ))
            },
        };

        if disabled_by_pref(&feature.name, context) ||
            !requirements.contains(feature.requirements) ||
            (range.is_some() && !feature.allows_ranges())
        {
            return Err(location.new_custom_error(
                StyleParseErrorKind::MediaQueryExpectedFeatureName(ident.clone()),
            ));
        }

        let operator = input.try_parse(consume_operation_or_colon);
        let operator = match operator {
            Err(..) => {
                // If there's no colon, this is a query of the form
                // '(<feature>)', that is, there's no value specified.
                //
                // Gecko doesn't allow ranged expressions without a
                // value, so just reject them here too.
                if range.is_some() {
                    return Err(
                        input.new_custom_error(StyleParseErrorKind::RangedExpressionWithNoValue)
                    );
                }

                return Ok(Self::new(feature_type, feature_index, None, None));
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

        let value = QueryExpressionValue::parse(feature, context, input).map_err(|err| {
            err.location
                .new_custom_error(StyleParseErrorKind::MediaQueryExpectedFeatureValue)
        })?;

        Ok(Self::new(feature_type, feature_index, Some(value), range_or_operator))
    }

    /// Returns whether this query evaluates to true for the given device.
    pub fn matches(&self, context: &computed::Context) -> bool {
        let value = self.value.as_ref();

        macro_rules! expect {
            ($variant:ident) => {
                value.map(|value| match *value {
                    QueryExpressionValue::$variant(ref v) => v,
                    _ => unreachable!("Unexpected QueryExpressionValue"),
                })
            };
        }

        match self.feature().evaluator {
            Evaluator::Length(eval) => {
                let computed = expect!(Length).map(|specified| {
                    specified.to_computed_value(context)
                });
                let length = eval(context);
                RangeOrOperator::evaluate(self.range_or_operator, computed, length)
            },
            Evaluator::Integer(eval) => {
                let computed = expect!(Integer).cloned();
                let integer = eval(context);
                RangeOrOperator::evaluate(self.range_or_operator, computed, integer)
            },
            Evaluator::Float(eval) => {
                let computed = expect!(Float).cloned();
                let float = eval(context);
                RangeOrOperator::evaluate(self.range_or_operator, computed, float)
            }
            Evaluator::NumberRatio(eval) => {
                // A ratio of 0/0 behaves as the ratio 1/0, so we need to call used_value()
                // to convert it if necessary.
                // FIXME: we may need to update here once
                // https://github.com/w3c/csswg-drafts/issues/4954 got resolved.
                let computed = match expect!(NumberRatio).cloned() {
                    Some(ratio) => ratio.used_value(),
                    None => return true,
                };
                let ratio = eval(context);
                RangeOrOperator::evaluate_with_query_value(self.range_or_operator, computed, ratio)
            },
            Evaluator::Resolution(eval) => {
                let computed = expect!(Resolution).map(|specified| {
                    specified.to_computed_value(context).dppx()
                });
                let resolution = eval(context).dppx();
                RangeOrOperator::evaluate(self.range_or_operator, computed, resolution)
            },
            Evaluator::Enumerated { evaluator, .. } => {
                debug_assert!(self.range_or_operator.is_none(), "Ranges with keywords?");
                evaluator(context, expect!(Enumerated).cloned())
            },
            Evaluator::BoolInteger(eval) => {
                debug_assert!(self.range_or_operator.is_none(), "Ranges with bools?");
                let computed = expect!(BoolInteger).cloned();
                let boolean = eval(context);
                computed.map_or(boolean, |v| v == boolean)
            },
        }
    }
}

/// A value found or expected in a expression.
///
/// FIXME(emilio): How should calc() serialize in the Number / Integer /
/// BoolInteger / NumberRatio case, as computed or as specified value?
///
/// If the first, this would need to store the relevant values.
///
/// See: https://github.com/w3c/csswg-drafts/issues/1968
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub enum QueryExpressionValue {
    /// A length.
    Length(Length),
    /// A (non-negative) integer.
    Integer(u32),
    /// A floating point value.
    Float(CSSFloat),
    /// A boolean value, specified as an integer (i.e., either 0 or 1).
    BoolInteger(bool),
    /// A single non-negative number or two non-negative numbers separated by '/',
    /// with optional whitespace on either side of the '/'.
    NumberRatio(Ratio),
    /// A resolution.
    Resolution(Resolution),
    /// An enumerated value, defined by the variant keyword table in the
    /// feature's `mData` member.
    Enumerated(KeywordDiscriminant),
}

impl QueryExpressionValue {
    fn to_css<W>(&self, dest: &mut CssWriter<W>, for_expr: &QueryFeatureExpression) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            QueryExpressionValue::Length(ref l) => l.to_css(dest),
            QueryExpressionValue::Integer(v) => v.to_css(dest),
            QueryExpressionValue::Float(v) => v.to_css(dest),
            QueryExpressionValue::BoolInteger(v) => dest.write_str(if v { "1" } else { "0" }),
            QueryExpressionValue::NumberRatio(ratio) => ratio.to_css(dest),
            QueryExpressionValue::Resolution(ref r) => r.to_css(dest),
            QueryExpressionValue::Enumerated(value) => match for_expr.feature().evaluator {
                Evaluator::Enumerated { serializer, .. } => dest.write_str(&*serializer(value)),
                _ => unreachable!(),
            },
        }
    }

    fn parse<'i, 't>(
        for_feature: &QueryFeatureDescription,
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<QueryExpressionValue, ParseError<'i>> {
        Ok(match for_feature.evaluator {
            Evaluator::Length(..) => {
                let length = Length::parse_non_negative(context, input)?;
                QueryExpressionValue::Length(length)
            },
            Evaluator::Integer(..) => {
                let integer = Integer::parse_non_negative(context, input)?;
                QueryExpressionValue::Integer(integer.value() as u32)
            },
            Evaluator::BoolInteger(..) => {
                let integer = Integer::parse_non_negative(context, input)?;
                let value = integer.value();
                if value > 1 {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                QueryExpressionValue::BoolInteger(value == 1)
            },
            Evaluator::Float(..) => {
                let number = Number::parse(context, input)?;
                QueryExpressionValue::Float(number.get())
            },
            Evaluator::NumberRatio(..) => {
                use crate::values::specified::Ratio as SpecifiedRatio;
                let ratio = SpecifiedRatio::parse(context, input)?;
                QueryExpressionValue::NumberRatio(Ratio::new(ratio.0.get(), ratio.1.get()))
            },
            Evaluator::Resolution(..) => {
                QueryExpressionValue::Resolution(Resolution::parse(context, input)?)
            },
            Evaluator::Enumerated { parser, .. } => {
                QueryExpressionValue::Enumerated(parser(context, input)?)
            },
        })
    }
}
