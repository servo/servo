/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [@supports rules](https://drafts.csswg.org/css-conditional-3/#at-supports)

use cssparser::{BasicParseError, ParseError as CssParseError, ParserInput};
use cssparser::{Delimiter, parse_important, Parser, SourceLocation, Token};
use parser::ParserContext;
use properties::{PropertyId, PropertyDeclaration, SourcePropertyDeclaration};
use selectors::parser::SelectorParseError;
use servo_arc::Arc;
use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::{ToCss, ParseError, StyleParseError};
use stylesheets::{CssRuleType, CssRules};

/// An [`@supports`][supports] rule.
///
/// [supports]: https://drafts.csswg.org/css-conditional-3/#at-supports
#[derive(Debug)]
pub struct SupportsRule {
    /// The parsed condition
    pub condition: SupportsCondition,
    /// Child rules
    pub rules: Arc<Locked<CssRules>>,
    /// The result of evaluating the condition
    pub enabled: bool,
    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for SupportsRule {
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        dest.write_str("@supports ")?;
        self.condition.to_css(dest)?;
        dest.write_str(" {")?;
        for rule in self.rules.read_with(guard).0.iter() {
            dest.write_str(" ")?;
            rule.to_css(guard, dest)?;
        }
        dest.write_str(" }")
    }
}

impl DeepCloneWithLock for SupportsRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        let rules = self.rules.read_with(guard);
        SupportsRule {
            condition: self.condition.clone(),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock, guard, params))),
            enabled: self.enabled,
            source_location: self.source_location.clone(),
        }
    }
}

/// An @supports condition
///
/// https://drafts.csswg.org/css-conditional-3/#at-supports
#[derive(Clone, Debug)]
pub enum SupportsCondition {
    /// `not (condition)`
    Not(Box<SupportsCondition>),
    /// `(condition)`
    Parenthesized(Box<SupportsCondition>),
    /// `(condition) and (condition) and (condition) ..`
    And(Vec<SupportsCondition>),
    /// `(condition) or (condition) or (condition) ..`
    Or(Vec<SupportsCondition>),
    /// `property-ident: value` (value can be any tokens)
    Declaration(Declaration),
    /// `(any tokens)` or `func(any tokens)`
    FutureSyntax(String),
}

impl SupportsCondition {
    /// Parse a condition
    ///
    /// https://drafts.csswg.org/css-conditional/#supports_condition
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<SupportsCondition, ParseError<'i>> {
        if let Ok(_) = input.try(|i| i.expect_ident_matching("not")) {
            let inner = SupportsCondition::parse_in_parens(input)?;
            return Ok(SupportsCondition::Not(Box::new(inner)));
        }

        let in_parens = SupportsCondition::parse_in_parens(input)?;

        let (keyword, wrapper) = match input.next() {
            Err(_) => {
                // End of input
                return Ok(in_parens)
            }
            Ok(Token::Ident(ident)) => {
                match_ignore_ascii_case! { &ident,
                    "and" => ("and", SupportsCondition::And as fn(_) -> _),
                    "or" => ("or", SupportsCondition::Or as fn(_) -> _),
                    _ => return Err(SelectorParseError::UnexpectedIdent(ident.clone()).into())
                }
            }
            Ok(t) => return Err(CssParseError::Basic(BasicParseError::UnexpectedToken(t)))
        };

        let mut conditions = Vec::with_capacity(2);
        conditions.push(in_parens);
        loop {
            conditions.push(SupportsCondition::parse_in_parens(input)?);
            if input.try(|input| input.expect_ident_matching(keyword)).is_err() {
                // Did not find the expected keyword.
                // If we found some other token,
                // it will be rejected by `Parser::parse_entirely` somewhere up the stack.
                return Ok(wrapper(conditions))
            }
        }
    }

    /// https://drafts.csswg.org/css-conditional-3/#supports_condition_in_parens
    fn parse_in_parens<'i, 't>(input: &mut Parser<'i, 't>) -> Result<SupportsCondition, ParseError<'i>> {
        // Whitespace is normally taken care of in `Parser::next`,
        // but we want to not include it in `pos` for the SupportsCondition::FutureSyntax cases.
        while input.try(Parser::expect_whitespace).is_ok() {}
        let pos = input.position();
        match input.next()? {
            Token::ParenthesisBlock => {
                let nested = input.try(|input| {
                    input.parse_nested_block(|i| parse_condition_or_declaration(i))
                });
                if nested.is_ok() {
                    return nested;
                }
            }
            Token::Function(_) => {}
            t => return Err(CssParseError::Basic(BasicParseError::UnexpectedToken(t))),
        }
        input.parse_nested_block(|i| consume_any_value(i))?;
        Ok(SupportsCondition::FutureSyntax(input.slice_from(pos).to_owned()))
    }

    /// Evaluate a supports condition
    pub fn eval(&self, cx: &ParserContext) -> bool {
        match *self {
            SupportsCondition::Not(ref cond) => !cond.eval(cx),
            SupportsCondition::Parenthesized(ref cond) => cond.eval(cx),
            SupportsCondition::And(ref vec) => vec.iter().all(|c| c.eval(cx)),
            SupportsCondition::Or(ref vec) => vec.iter().any(|c| c.eval(cx)),
            SupportsCondition::Declaration(ref decl) => decl.eval(cx),
            SupportsCondition::FutureSyntax(_) => false
        }
    }
}

/// supports_condition | declaration
/// https://drafts.csswg.org/css-conditional/#dom-css-supports-conditiontext-conditiontext
pub fn parse_condition_or_declaration<'i, 't>(input: &mut Parser<'i, 't>)
                                              -> Result<SupportsCondition, ParseError<'i>> {
    if let Ok(condition) = input.try(SupportsCondition::parse) {
        Ok(SupportsCondition::Parenthesized(Box::new(condition)))
    } else {
        Declaration::parse(input).map(SupportsCondition::Declaration)
    }
}

impl ToCss for SupportsCondition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            SupportsCondition::Not(ref cond) => {
                dest.write_str("not ")?;
                cond.to_css(dest)
            }
            SupportsCondition::Parenthesized(ref cond) => {
                dest.write_str("(")?;
                cond.to_css(dest)?;
                dest.write_str(")")
            }
            SupportsCondition::And(ref vec) => {
                let mut first = true;
                for cond in vec {
                    if !first {
                        dest.write_str(" and ")?;
                    }
                    first = false;
                    cond.to_css(dest)?;
                }
                Ok(())
            }
            SupportsCondition::Or(ref vec) => {
                let mut first = true;
                for cond in vec {
                    if !first {
                        dest.write_str(" or ")?;
                    }
                    first = false;
                    cond.to_css(dest)?;
                }
                Ok(())
            }
            SupportsCondition::Declaration(ref decl) => {
                dest.write_str("(")?;
                decl.to_css(dest)?;
                dest.write_str(")")
            }
            SupportsCondition::FutureSyntax(ref s) => dest.write_str(&s),
        }
    }
}

#[derive(Clone, Debug)]
/// A possibly-invalid property declaration
pub struct Declaration {
    /// The property name
    pub prop: String,
    /// The property value
    pub val: String,
}

impl ToCss for Declaration {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        dest.write_str(&self.prop)?;
        dest.write_str(":")?;
        // no space, the `val` already contains any possible spaces
        dest.write_str(&self.val)
    }
}

/// https://drafts.csswg.org/css-syntax-3/#typedef-any-value
fn consume_any_value<'i, 't>(input: &mut Parser<'i, 't>) -> Result<(), ParseError<'i>> {
    input.expect_no_error_token().map_err(|err| err.into())
}

impl Declaration {
    /// Parse a declaration
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Declaration, ParseError<'i>> {
        let prop = input.expect_ident()?.into_owned();
        input.expect_colon()?;
        let pos = input.position();
        consume_any_value(input)?;
        Ok(Declaration { prop: prop, val: input.slice_from(pos).to_owned() })
    }

    /// Determine if a declaration parses
    ///
    /// https://drafts.csswg.org/css-conditional-3/#support-definition
    pub fn eval(&self, cx: &ParserContext) -> bool {
        let id = if let Ok(id) = PropertyId::parse((&*self.prop).into()) {
            id
        } else {
            return false
        };
        let mut input = ParserInput::new(&self.val);
        let mut input = Parser::new(&mut input);
        let context = ParserContext::new_with_rule_type(cx, Some(CssRuleType::Style));
        let mut declarations = SourcePropertyDeclaration::new();
        let res = input.parse_until_before(Delimiter::Bang, |input| {
            PropertyDeclaration::parse_into(&mut declarations, id, &context, input)
                .map_err(|e| StyleParseError::PropertyDeclaration(e).into())
        });
        let _ = input.try(parse_important);
        res.is_ok() && input.is_exhausted()
    }
}
