/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A query condition:
//!
//! https://drafts.csswg.org/mediaqueries-4/#typedef-media-condition
//! https://drafts.csswg.org/css-contain-3/#typedef-container-condition

use super::{QueryFeatureExpression, FeatureType, FeatureFlags};
use crate::parser::ParserContext;
use crate::values::computed;
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
        }
    }
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

    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
        allow_or: AllowOr,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();

        // FIXME(emilio): This can be cleaner with nll.
        let is_negation = match *input.next()? {
            Token::ParenthesisBlock => false,
            Token::Ident(ref ident) if ident.eq_ignore_ascii_case("not") => true,
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
        };

        if is_negation {
            let inner_condition = Self::parse_in_parens(context, input, feature_type)?;
            return Ok(QueryCondition::Not(Box::new(inner_condition)));
        }

        // ParenthesisBlock.
        let first_condition = Self::parse_paren_block(context, input, feature_type)?;
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

    /// Parse a condition in parentheses.
    pub fn parse_in_parens<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_parenthesis_block()?;
        Self::parse_paren_block(context, input, feature_type)
    }

    fn parse_paren_block<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        feature_type: FeatureType,
    ) -> Result<Self, ParseError<'i>> {
        input.parse_nested_block(|input| {
            // Base case.
            if let Ok(inner) = input.try_parse(|i| Self::parse(context, i, feature_type)) {
                return Ok(QueryCondition::InParens(Box::new(inner)));
            }
            let expr = QueryFeatureExpression::parse_in_parenthesis_block(context, input, feature_type)?;
            Ok(QueryCondition::Feature(expr))
        })
    }

    /// Whether this condition matches the device and quirks mode.
    pub fn matches(&self, context: &computed::Context) -> bool {
        match *self {
            QueryCondition::Feature(ref f) => f.matches(context),
            QueryCondition::InParens(ref c) => c.matches(context),
            QueryCondition::Not(ref c) => !c.matches(context),
            QueryCondition::Operation(ref conditions, op) => {
                let mut iter = conditions.iter();
                match op {
                    Operator::And => iter.all(|c| c.matches(context)),
                    Operator::Or => iter.any(|c| c.matches(context)),
                }
            },
        }
    }
}
