/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A query condition:
//!
//! https://drafts.csswg.org/mediaqueries-4/#typedef-media-condition
//! https://drafts.csswg.org/css-contain-3/#typedef-container-condition

use super::{FeatureFlags, FeatureType, QueryFeatureExpression};
use crate::values::computed;
use crate::{error_reporting::ContextualParseError, parser::ParserContext};
use cssparser::{Parser, Token};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// A binary `and` or `or` operator.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToCss, ToShmem)]
#[allow(missing_docs)]
pub enum Operator {
    And,
    Or,
}

/// Whether to allow an `or` condition or not during parsing.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToCss)]
enum AllowOr {
    Yes,
    No,
}

/// https://en.wikipedia.org/wiki/Three-valued_logic#Kleene_and_Priest_logics
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToCss)]
pub enum KleeneValue {
    /// False
    False = 0,
    /// True
    True = 1,
    /// Either true or false, but we’re not sure which yet.
    Unknown,
}

impl From<bool> for KleeneValue {
    fn from(b: bool) -> Self {
        if b {
            Self::True
        } else {
            Self::False
        }
    }
}

impl KleeneValue {
    /// Turns this Kleene value to a bool, taking the unknown value as an
    /// argument.
    pub fn to_bool(self, unknown: bool) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            Self::Unknown => unknown,
        }
    }
}

impl std::ops::Not for KleeneValue {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

// Implements the logical and operation.
impl std::ops::BitAnd for KleeneValue {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        if self == Self::False || other == Self::False {
            return Self::False;
        }
        if self == Self::Unknown || other == Self::Unknown {
            return Self::Unknown;
        }
        Self::True
    }
}

// Implements the logical or operation.
impl std::ops::BitOr for KleeneValue {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        if self == Self::True || other == Self::True {
            return Self::True;
        }
        if self == Self::Unknown || other == Self::Unknown {
            return Self::Unknown;
        }
        Self::False
    }
}

impl std::ops::BitOrAssign for KleeneValue {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl std::ops::BitAndAssign for KleeneValue {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

/// Represents a condition.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub enum QueryCondition {
    /// A simple feature expression, implicitly parenthesized.
    Feature(QueryFeatureExpression),
    /// A negation of a condition.
    Not(Box<QueryCondition>),
    /// A set of joint operations.
    Operation(Box<[QueryCondition]>, Operator),
    /// A condition wrapped in parenthesis.
    InParens(Box<QueryCondition>),
    /// [ <function-token> <any-value>? ) ] | [ ( <any-value>? ) ]
    GeneralEnclosed(String),
}

impl ToCss for QueryCondition {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            // NOTE(emilio): QueryFeatureExpression already includes the
            // parenthesis.
            QueryCondition::Feature(ref f) => f.to_css(dest),
            QueryCondition::Not(ref c) => {
                dest.write_str("not ")?;
                c.to_css(dest)
            },
            QueryCondition::InParens(ref c) => {
                dest.write_char('(')?;
                c.to_css(dest)?;
                dest.write_char(')')
            },
            QueryCondition::Operation(ref list, op) => {
                let mut iter = list.iter();
                iter.next().unwrap().to_css(dest)?;
                for item in iter {
                    dest.write_char(' ')?;
                    op.to_css(dest)?;
                    dest.write_char(' ')?;
                    item.to_css(dest)?;
                }
                Ok(())
            },
            QueryCondition::GeneralEnclosed(ref s) => dest.write_str(&s),
        }
    }
}

/// <https://drafts.csswg.org/css-syntax-3/#typedef-any-value>
fn consume_any_value<'i, 't>(input: &mut Parser<'i, 't>) -> Result<(), ParseError<'i>> {
    input.expect_no_error_token().map_err(Into::into)
}

impl QueryCondition {
    /// Parse a single condition.
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, feature_type, AllowOr::Yes)
    }

    fn visit<F>(&self, visitor: &mut F)
    where
        F: FnMut(&Self),
    {
        visitor(self);
        match *self {
            Self::Feature(..) => {},
            Self::GeneralEnclosed(..) => {},
            Self::Not(ref cond) => cond.visit(visitor),
            Self::Operation(ref conds, _op) => {
                for cond in conds.iter() {
                    cond.visit(visitor);
                }
            },
            Self::InParens(ref cond) => cond.visit(visitor),
        }
    }

    /// Returns the union of all flags in the expression. This is useful for
    /// container queries.
    pub fn cumulative_flags(&self) -> FeatureFlags {
        let mut result = FeatureFlags::empty();
        self.visit(&mut |condition| {
            if let Self::Feature(ref f) = condition {
                result.insert(f.feature_flags())
            }
        });
        result
    }

    /// Parse a single condition, disallowing `or` expressions.
    ///
    /// To be used from the legacy query syntax.
    pub fn parse_disallow_or<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, feature_type, AllowOr::No)
    }

    /// https://drafts.csswg.org/mediaqueries-5/#typedef-media-condition or
    /// https://drafts.csswg.org/mediaqueries-5/#typedef-media-condition-without-or
    /// (depending on `allow_or`).
    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
        allow_or: AllowOr,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        if input.try_parse(|i| i.expect_ident_matching("not")).is_ok() {
            let inner_condition = Self::parse_in_parens(context, input, feature_type)?;
            return Ok(QueryCondition::Not(Box::new(inner_condition)));
        }

        let first_condition = Self::parse_in_parens(context, input, feature_type)?;
        let operator = match input.try_parse(Operator::parse) {
            Ok(op) => op,
            Err(..) => return Ok(first_condition),
        };

        if allow_or == AllowOr::No && operator == Operator::Or {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        let mut conditions = vec![];
        conditions.push(first_condition);
        conditions.push(Self::parse_in_parens(context, input, feature_type)?);

        let delim = match operator {
            Operator::And => "and",
            Operator::Or => "or",
        };

        loop {
            if input.try_parse(|i| i.expect_ident_matching(delim)).is_err() {
                return Ok(QueryCondition::Operation(
                    conditions.into_boxed_slice(),
                    operator,
                ));
            }

            conditions.push(Self::parse_in_parens(context, input, feature_type)?);
        }
    }

    fn parse_in_parenthesis_block<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        // Base case. Make sure to preserve this error as it's more generally
        // relevant.
        let feature_error = match input.try_parse(|input| {
            QueryFeatureExpression::parse_in_parenthesis_block(context, input, feature_type)
        }) {
            Ok(expr) => return Ok(Self::Feature(expr)),
            Err(e) => e,
        };
        if let Ok(inner) = Self::parse(context, input, feature_type) {
            return Ok(Self::InParens(Box::new(inner)));
        }
        Err(feature_error)
    }

    /// Parse a condition in parentheses, or `<general-enclosed>`.
    ///
    /// https://drafts.csswg.org/mediaqueries/#typedef-media-in-parens
    pub fn parse_in_parens<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        input.skip_whitespace();
        let start = input.position();
        let start_location = input.current_source_location();
        match *input.next()? {
            Token::ParenthesisBlock => {
                let nested = input.try_parse(|input| {
                    input.parse_nested_block(|input| {
                        Self::parse_in_parenthesis_block(context, input, feature_type)
                    })
                });
                match nested {
                    Ok(nested) => return Ok(nested),
                    Err(e) => {
                        // We're about to swallow the error in a `<general-enclosed>`
                        // condition, so report it while we can.
                        let loc = e.location;
                        let error =
                            ContextualParseError::InvalidMediaRule(input.slice_from(start), e);
                        context.log_css_error(loc, error);
                    },
                }
            },
            Token::Function(..) => {
                // TODO: handle `style()` queries, etc.
            },
            ref t => return Err(start_location.new_unexpected_token_error(t.clone())),
        }
        input.parse_nested_block(consume_any_value)?;
        Ok(Self::GeneralEnclosed(input.slice_from(start).to_owned()))
    }

    /// Whether this condition matches the device and quirks mode.
    /// https://drafts.csswg.org/mediaqueries/#evaluating
    /// https://drafts.csswg.org/mediaqueries/#typedef-general-enclosed
    /// Kleene 3-valued logic is adopted here due to the introduction of
    /// <general-enclosed>.
    pub fn matches(&self, context: &computed::Context) -> KleeneValue {
        match *self {
            QueryCondition::Feature(ref f) => f.matches(context),
            QueryCondition::GeneralEnclosed(_) => KleeneValue::Unknown,
            QueryCondition::InParens(ref c) => c.matches(context),
            QueryCondition::Not(ref c) => !c.matches(context),
            QueryCondition::Operation(ref conditions, op) => {
                debug_assert!(!conditions.is_empty(), "We never create an empty op");
                match op {
                    Operator::And => {
                        let mut result = KleeneValue::True;
                        for c in conditions.iter() {
                            result &= c.matches(context);
                            if result == KleeneValue::False {
                                break;
                            }
                        }
                        result
                    },
                    Operator::Or => {
                        let mut result = KleeneValue::False;
                        for c in conditions.iter() {
                            result |= c.matches(context);
                            if result == KleeneValue::True {
                                break;
                            }
                        }
                        result
                    },
                }
            },
        }
    }
}
