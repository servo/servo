/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A media query condition:
//!
//! https://drafts.csswg.org/mediaqueries-4/#typedef-media-condition

use context::QuirksMode;
use cssparser::{Parser, Token};
use parser::ParserContext;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use super::{Device, MediaFeatureExpression};


/// A binary `and` or `or` operator.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToCss)]
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

/// Represents a media condition.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum MediaCondition {
    /// A simple media feature expression, implicitly parenthesized.
    Feature(MediaFeatureExpression),
    /// A negation of a condition.
    Not(Box<MediaCondition>),
    /// A set of joint operations.
    Operation(Box<[MediaCondition]>, Operator),
    /// A condition wrapped in parenthesis.
    InParens(Box<MediaCondition>),
}

impl ToCss for MediaCondition {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            // NOTE(emilio): MediaFeatureExpression already includes the
            // parenthesis.
            MediaCondition::Feature(ref f) => f.to_css(dest),
            MediaCondition::Not(ref c) => {
                dest.write_str("not ")?;
                c.to_css(dest)
            }
            MediaCondition::InParens(ref c) => {
                dest.write_char('(')?;
                c.to_css(dest)?;
                dest.write_char(')')
            }
            MediaCondition::Operation(ref list, op) => {
                let mut iter = list.iter();
                iter.next().unwrap().to_css(dest)?;
                for item in iter {
                    dest.write_char(' ')?;
                    op.to_css(dest)?;
                    dest.write_char(' ')?;
                    item.to_css(dest)?;
                }
                Ok(())
            }
        }
    }
}

impl MediaCondition {
    /// Parse a single media condition.
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowOr::Yes)
    }

    /// Parse a single media condition, disallowing `or` expressions.
    ///
    /// To be used from the legacy media query syntax.
    pub fn parse_disallow_or<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowOr::No)
    }

    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_or: AllowOr,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();

        // FIXME(emilio): This can be cleaner with nll.
        let is_negation = match *input.next()? {
            Token::ParenthesisBlock => false,
            Token::Ident(ref ident) if ident.eq_ignore_ascii_case("not") => true,
            ref t => {
                return Err(location.new_unexpected_token_error(t.clone()))
            }
        };

        if is_negation {
            let inner_condition = Self::parse_in_parens(context, input)?;
            return Ok(MediaCondition::Not(Box::new(inner_condition)))
        }

        // ParenthesisBlock.
        let first_condition = Self::parse_paren_block(context, input)?;
        let operator = match input.try(Operator::parse) {
            Ok(op) => op,
            Err(..) => return Ok(first_condition),
        };

        if allow_or == AllowOr::No && operator == Operator::Or {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        let mut conditions = vec![];
        conditions.push(first_condition);
        conditions.push(Self::parse_in_parens(context, input)?);

        let delim = match operator {
            Operator::And => "and",
            Operator::Or => "or",
        };

        loop {
            if input.try(|i| i.expect_ident_matching(delim)).is_err() {
                return Ok(MediaCondition::Operation(
                    conditions.into_boxed_slice(),
                    operator,
                ));
            }

            conditions.push(Self::parse_in_parens(context, input)?);
        }
    }

    /// Parse a media condition in parentheses.
    pub fn parse_in_parens<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_parenthesis_block()?;
        Self::parse_paren_block(context, input)
    }

    fn parse_paren_block<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.parse_nested_block(|input| {
            // Base case.
            if let Ok(inner) = input.try(|i| Self::parse(context, i)) {
                return Ok(MediaCondition::InParens(Box::new(inner)))
            }
            let expr = MediaFeatureExpression::parse_in_parenthesis_block(context, input)?;
            Ok(MediaCondition::Feature(expr))
        })
    }

    /// Whether this condition matches the device and quirks mode.
    pub fn matches(&self, device: &Device, quirks_mode: QuirksMode) -> bool {
        match *self {
            MediaCondition::Feature(ref f) => f.matches(device, quirks_mode),
            MediaCondition::InParens(ref c) => c.matches(device, quirks_mode),
            MediaCondition::Not(ref c) => !c.matches(device, quirks_mode),
            MediaCondition::Operation(ref conditions, op) => {
                let mut iter = conditions.iter();
                match op {
                    Operator::And => {
                        iter.all(|c| c.matches(device, quirks_mode))
                    }
                    Operator::Or => {
                        iter.any(|c| c.matches(device, quirks_mode))
                    }
                }
            }
        }
    }
}
