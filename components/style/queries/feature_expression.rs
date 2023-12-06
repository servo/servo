/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Parsing for query feature expressions, like `(foo: bar)` or
//! `(width >= 400px)`.

use super::feature::{Evaluator, QueryFeatureDescription};
use super::feature::{FeatureFlags, KeywordDiscriminant};
use crate::parser::{Parse, ParserContext};
use crate::queries::condition::KleeneValue;
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
        let media_features = &crate::gecko::media_features::MEDIA_FEATURES;
        #[cfg(feature = "servo")]
        let media_features = &*crate::servo::media_queries::MEDIA_FEATURES;

        use crate::stylesheets::container_rule::CONTAINER_FEATURES;

        match *self {
            FeatureType::Media => media_features,
            FeatureType::Container => &CONTAINER_FEATURES,
        }
    }

    fn find_feature(&self, name: &Atom) -> Option<(usize, &'static QueryFeatureDescription)> {
        self.features()
            .iter()
            .enumerate()
            .find(|(_, f)| f.name == *name)
    }
}

/// The kind of matching that should be performed on a feature value.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
enum LegacyRange {
    /// At least the specified value.
    Min,
    /// At most the specified value.
    Max,
}

/// The operator that was specified in this feature.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
enum Operator {
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
            Self::Equal => "=",
            Self::LessThan => "<",
            Self::LessThanEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanEqual => ">=",
        })
    }
}

impl Operator {
    fn is_compatible_with(self, right_op: Self) -> bool {
        // Some operators are not compatible with each other in multi-range
        // context.
        match self {
            Self::Equal => false,
            Self::GreaterThan | Self::GreaterThanEqual => {
                matches!(right_op, Self::GreaterThan | Self::GreaterThanEqual)
            },
            Self::LessThan | Self::LessThanEqual => {
                matches!(right_op, Self::LessThan | Self::LessThanEqual)
            },
        }
    }

    fn evaluate(&self, cmp: Ordering) -> bool {
        match *self {
            Self::Equal => cmp == Ordering::Equal,
            Self::GreaterThan => cmp == Ordering::Greater,
            Self::GreaterThanEqual => cmp == Ordering::Equal || cmp == Ordering::Greater,
            Self::LessThan => cmp == Ordering::Less,
            Self::LessThanEqual => cmp == Ordering::Equal || cmp == Ordering::Less,
        }
    }

    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let operator = match *input.next()? {
            Token::Delim('=') => return Ok(Operator::Equal),
            Token::Delim('>') => Operator::GreaterThan,
            Token::Delim('<') => Operator::LessThan,
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
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
            return Ok(operator);
        }

        Ok(match operator {
            Operator::GreaterThan => Operator::GreaterThanEqual,
            Operator::LessThan => Operator::LessThanEqual,
            _ => unreachable!(),
        })
    }
}

#[derive(Clone, Debug, MallocSizeOf, ToShmem, PartialEq)]
enum QueryFeatureExpressionKind {
    /// Just the media feature name.
    Empty,

    /// A single value.
    Single(QueryExpressionValue),

    /// Legacy range syntax (min-*: value) or so.
    LegacyRange(LegacyRange, QueryExpressionValue),

    /// Modern range context syntax:
    /// https://drafts.csswg.org/mediaqueries-5/#mq-range-context
    Range {
        left: Option<(Operator, QueryExpressionValue)>,
        right: Option<(Operator, QueryExpressionValue)>,
    },
}

impl QueryFeatureExpressionKind {
    /// Evaluate a given range given an optional query value and a value from
    /// the browser.
    fn evaluate<T>(
        &self,
        context_value: T,
        mut compute: impl FnMut(&QueryExpressionValue) -> T,
    ) -> bool
    where
        T: PartialOrd + Zero,
    {
        match *self {
            Self::Empty => return !context_value.is_zero(),
            Self::Single(ref value) => {
                let value = compute(value);
                let cmp = match context_value.partial_cmp(&value) {
                    Some(c) => c,
                    None => return false,
                };
                cmp == Ordering::Equal
            },
            Self::LegacyRange(ref range, ref value) => {
                let value = compute(value);
                let cmp = match context_value.partial_cmp(&value) {
                    Some(c) => c,
                    None => return false,
                };
                cmp == Ordering::Equal ||
                    match range {
                        LegacyRange::Min => cmp == Ordering::Greater,
                        LegacyRange::Max => cmp == Ordering::Less,
                    }
            },
            Self::Range {
                ref left,
                ref right,
            } => {
                debug_assert!(left.is_some() || right.is_some());
                if let Some((ref op, ref value)) = left {
                    let value = compute(value);
                    let cmp = match value.partial_cmp(&context_value) {
                        Some(c) => c,
                        None => return false,
                    };
                    if !op.evaluate(cmp) {
                        return false;
                    }
                }
                if let Some((ref op, ref value)) = right {
                    let value = compute(value);
                    let cmp = match context_value.partial_cmp(&value) {
                        Some(c) => c,
                        None => return false,
                    };
                    if !op.evaluate(cmp) {
                        return false;
                    }
                }
                true
            },
        }
    }

    /// Non-ranged features only need to compare to one value at most.
    fn non_ranged_value(&self) -> Option<&QueryExpressionValue> {
        match *self {
            Self::Empty => None,
            Self::Single(ref v) => Some(v),
            Self::LegacyRange(..) | Self::Range { .. } => {
                debug_assert!(false, "Unexpected ranged value in non-ranged feature!");
                None
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
    kind: QueryFeatureExpressionKind,
}

impl ToCss for QueryFeatureExpression {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_char('(')?;

        match self.kind {
            QueryFeatureExpressionKind::Empty => self.write_name(dest)?,
            QueryFeatureExpressionKind::Single(ref v) |
            QueryFeatureExpressionKind::LegacyRange(_, ref v) => {
                self.write_name(dest)?;
                dest.write_str(": ")?;
                v.to_css(dest, self)?;
            },
            QueryFeatureExpressionKind::Range {
                ref left,
                ref right,
            } => {
                if let Some((ref op, ref val)) = left {
                    val.to_css(dest, self)?;
                    dest.write_char(' ')?;
                    op.to_css(dest)?;
                    dest.write_char(' ')?;
                }
                self.write_name(dest)?;
                if let Some((ref op, ref val)) = right {
                    dest.write_char(' ')?;
                    op.to_css(dest)?;
                    dest.write_char(' ')?;
                    val.to_css(dest, self)?;
                }
            },
        }
        dest.write_char(')')
    }
}

fn consume_operation_or_colon<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Option<Operator>, ParseError<'i>> {
    if input.try_parse(|input| input.expect_colon()).is_ok() {
        return Ok(None);
    }
    Operator::parse(input).map(|op| Some(op))
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

        // prefers-reduced-transparency is always enabled in the ua and chrome. On
        // the web it is hidden behind a preference (see Bug 1822176).
        if *feature == atom!("prefers-reduced-transparency") {
            return !context.in_ua_or_chrome_sheet() &&
                !static_prefs::pref!("layout.css.prefers-reduced-transparency.enabled");
        }

        // inverted-colors is always enabled in the ua and chrome. On
        // the web it is hidden behind a preferenc.
        if *feature == atom!("inverted-colors") {
            return !context.in_ua_or_chrome_sheet() &&
                !static_prefs::pref!("layout.css.inverted-colors.enabled");
        }
    }
    false
}

impl QueryFeatureExpression {
    fn new(
        feature_type: FeatureType,
        feature_index: usize,
        kind: QueryFeatureExpressionKind,
    ) -> Self {
        debug_assert!(feature_index < feature_type.features().len());
        Self {
            feature_type,
            feature_index,
            kind,
        }
    }

    fn write_name<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        let feature = self.feature();
        if feature.flags.contains(FeatureFlags::WEBKIT_PREFIX) {
            dest.write_str("-webkit-")?;
        }

        if let QueryFeatureExpressionKind::LegacyRange(range, _) = self.kind {
            match range {
                LegacyRange::Min => dest.write_str("min-")?,
                LegacyRange::Max => dest.write_str("max-")?,
            }
        }

        // NB: CssStringWriter not needed, feature names are under control.
        write!(dest, "{}", feature.name)?;

        Ok(())
    }

    fn feature(&self) -> &'static QueryFeatureDescription {
        &self.feature_type.features()[self.feature_index]
    }

    /// Returns the feature flags for our feature.
    pub fn feature_flags(&self) -> FeatureFlags {
        self.feature().flags
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
        input.parse_nested_block(|input| {
            Self::parse_in_parenthesis_block(context, input, feature_type)
        })
    }

    fn parse_feature_name<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<(usize, Option<LegacyRange>), ParseError<'i>> {
        let mut flags = FeatureFlags::empty();
        let location = input.current_source_location();
        let ident = input.expect_ident()?;

        if context.in_ua_or_chrome_sheet() {
            flags.insert(FeatureFlags::CHROME_AND_UA_ONLY);
        }

        let mut feature_name = &**ident;
        if starts_with_ignore_ascii_case(feature_name, "-webkit-") {
            feature_name = &feature_name[8..];
            flags.insert(FeatureFlags::WEBKIT_PREFIX);
        }

        let range = if starts_with_ignore_ascii_case(feature_name, "min-") {
            feature_name = &feature_name[4..];
            Some(LegacyRange::Min)
        } else if starts_with_ignore_ascii_case(feature_name, "max-") {
            feature_name = &feature_name[4..];
            Some(LegacyRange::Max)
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
            !flags.contains(feature.flags.parsing_requirements()) ||
            (range.is_some() && !feature.allows_ranges())
        {
            return Err(location.new_custom_error(
                StyleParseErrorKind::MediaQueryExpectedFeatureName(ident.clone()),
            ));
        }

        Ok((feature_index, range))
    }

    /// Parses the following range syntax:
    ///
    ///   (feature-value <operator> feature-name)
    ///   (feature-value <operator> feature-name <operator> feature-value)
    fn parse_multi_range_syntax<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        let start = input.state();

        // To parse the values, we first need to find the feature name. We rely
        // on feature values for ranged features not being able to be top-level
        // <ident>s, which holds.
        let feature_index = loop {
            // NOTE: parse_feature_name advances the input.
            if let Ok((index, range)) = Self::parse_feature_name(context, input, feature_type) {
                if range.is_some() {
                    // Ranged names are not allowed here.
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                break index;
            }
            if input.is_exhausted() {
                return Err(start
                    .source_location()
                    .new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
        };

        input.reset(&start);

        let feature = &feature_type.features()[feature_index];
        let left_val = QueryExpressionValue::parse(feature, context, input)?;
        let left_op = Operator::parse(input)?;

        {
            let (parsed_index, _) = Self::parse_feature_name(context, input, feature_type)?;
            debug_assert_eq!(
                parsed_index, feature_index,
                "How did we find a different feature?"
            );
        }

        let right_op = input.try_parse(Operator::parse).ok();
        let right = match right_op {
            Some(op) => {
                if !left_op.is_compatible_with(op) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                Some((op, QueryExpressionValue::parse(feature, context, input)?))
            },
            None => None,
        };
        Ok(Self::new(
            feature_type,
            feature_index,
            QueryFeatureExpressionKind::Range {
                left: Some((left_op, left_val)),
                right,
            },
        ))
    }

    /// Parse a feature expression where we've already consumed the parenthesis.
    pub fn parse_in_parenthesis_block<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        let (feature_index, range) =
            match input.try_parse(|input| Self::parse_feature_name(context, input, feature_type)) {
                Ok(v) => v,
                Err(e) => {
                    if let Ok(expr) = Self::parse_multi_range_syntax(context, input, feature_type) {
                        return Ok(expr);
                    }
                    return Err(e);
                },
            };
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

                return Ok(Self::new(
                    feature_type,
                    feature_index,
                    QueryFeatureExpressionKind::Empty,
                ));
            },
            Ok(operator) => operator,
        };

        let feature = &feature_type.features()[feature_index];

        let value = QueryExpressionValue::parse(feature, context, input).map_err(|err| {
            err.location
                .new_custom_error(StyleParseErrorKind::MediaQueryExpectedFeatureValue)
        })?;

        let kind = match range {
            Some(range) => {
                if operator.is_some() {
                    return Err(
                        input.new_custom_error(StyleParseErrorKind::MediaQueryUnexpectedOperator)
                    );
                }
                QueryFeatureExpressionKind::LegacyRange(range, value)
            },
            None => match operator {
                Some(operator) => {
                    if !feature.allows_ranges() {
                        return Err(input
                            .new_custom_error(StyleParseErrorKind::MediaQueryUnexpectedOperator));
                    }
                    QueryFeatureExpressionKind::Range {
                        left: None,
                        right: Some((operator, value)),
                    }
                },
                None => QueryFeatureExpressionKind::Single(value),
            },
        };

        Ok(Self::new(feature_type, feature_index, kind))
    }

    /// Returns whether this query evaluates to true for the given device.
    pub fn matches(&self, context: &computed::Context) -> KleeneValue {
        macro_rules! expect {
            ($variant:ident, $v:expr) => {
                match *$v {
                    QueryExpressionValue::$variant(ref v) => v,
                    _ => unreachable!("Unexpected QueryExpressionValue"),
                }
            };
        }

        KleeneValue::from(match self.feature().evaluator {
            Evaluator::Length(eval) => {
                let v = eval(context);
                self.kind
                    .evaluate(v, |v| expect!(Length, v).to_computed_value(context))
            },
            Evaluator::OptionalLength(eval) => {
                let v = match eval(context) {
                    Some(v) => v,
                    None => return KleeneValue::Unknown,
                };
                self.kind
                    .evaluate(v, |v| expect!(Length, v).to_computed_value(context))
            },
            Evaluator::Integer(eval) => {
                let v = eval(context);
                self.kind.evaluate(v, |v| *expect!(Integer, v))
            },
            Evaluator::Float(eval) => {
                let v = eval(context);
                self.kind.evaluate(v, |v| *expect!(Float, v))
            },
            Evaluator::NumberRatio(eval) => {
                let ratio = eval(context);
                // A ratio of 0/0 behaves as the ratio 1/0, so we need to call used_value()
                // to convert it if necessary.
                // FIXME: we may need to update here once
                // https://github.com/w3c/csswg-drafts/issues/4954 got resolved.
                self.kind
                    .evaluate(ratio, |v| expect!(NumberRatio, v).used_value())
            },
            Evaluator::OptionalNumberRatio(eval) => {
                let ratio = match eval(context) {
                    Some(v) => v,
                    None => return KleeneValue::Unknown,
                };
                // See above for subtleties here.
                self.kind
                    .evaluate(ratio, |v| expect!(NumberRatio, v).used_value())
            },
            Evaluator::Resolution(eval) => {
                let v = eval(context).dppx();
                self.kind.evaluate(v, |v| {
                    expect!(Resolution, v).to_computed_value(context).dppx()
                })
            },
            Evaluator::Enumerated { evaluator, .. } => {
                let computed = self
                    .kind
                    .non_ranged_value()
                    .map(|v| *expect!(Enumerated, v));
                return evaluator(context, computed);
            },
            Evaluator::BoolInteger(eval) => {
                let computed = self
                    .kind
                    .non_ranged_value()
                    .map(|v| *expect!(BoolInteger, v));
                let boolean = eval(context);
                computed.map_or(boolean, |v| v == boolean)
            },
        })
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
    /// An integer.
    Integer(i32),
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
            Evaluator::OptionalLength(..) | Evaluator::Length(..) => {
                let length = Length::parse(context, input)?;
                QueryExpressionValue::Length(length)
            },
            Evaluator::Integer(..) => {
                let integer = Integer::parse(context, input)?;
                QueryExpressionValue::Integer(integer.value())
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
            Evaluator::OptionalNumberRatio(..) | Evaluator::NumberRatio(..) => {
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
