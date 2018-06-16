/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A media query condition:
//!
//! https://drafts.csswg.org/mediaqueries-4/#typedef-media-condition

use cssparser::{Parser, Token};
use parser::ParserContext;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

use super::MediaFeatureExpression;


/// A binary `and` or `or` operator.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, Parse, ToCss)]
#[allow(missing_docs)]
pub enum Operator {
    And,
    Or,
}

/// Represents a media condition.
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
            if let Ok(expr) = input.try(|i| MediaFeatureExpression::parse_in_parenthesis_block(context, i)) {
                return Ok(MediaCondition::Feature(expr));
            }

            let inner = Self::parse(context, input)?;
            Ok(MediaCondition::InParens(Box::new(inner)))
        })
    }
}
