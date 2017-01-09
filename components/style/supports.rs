/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [@supports rules](https://drafts.csswg.org/css-conditional-3/#at-supports)

use cssparser::{parse_important, Parser, Token};
use parser::ParserContext;
use properties::{PropertyDeclaration, PropertyId};
use std::fmt;
use style_traits::ToCss;

#[derive(Debug)]
/// An @supports condition
///
/// https://drafts.csswg.org/css-conditional-3/#at-supports
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
    pub fn parse(input: &mut Parser) -> Result<SupportsCondition, ()> {
        if let Ok(_) = input.try(|i| i.expect_ident_matching("not")) {
            let inner = SupportsCondition::parse_in_parens(input)?;
            return Ok(SupportsCondition::Not(Box::new(inner)));
        }

        let in_parens = SupportsCondition::parse_in_parens(input)?;

        let (keyword, wrapper) = match input.next() {
            Err(()) => {
                // End of input
                return Ok(in_parens)
            }
            Ok(Token::Ident(ident)) => {
                match_ignore_ascii_case! { ident,
                    "and" => ("and", SupportsCondition::And as fn(_) -> _),
                    "or" => ("or", SupportsCondition::Or as fn(_) -> _),
                    _ => return Err(())
                }
            }
            _ => return Err(())
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
    fn parse_in_parens(input: &mut Parser) -> Result<SupportsCondition, ()> {
        // Whitespace is normally taken care of in `Parser::next`,
        // but we want to not include it in `pos` for the SupportsCondition::FutureSyntax cases.
        while input.try(Parser::expect_whitespace).is_ok() {}
        let pos = input.position();
        match input.next()? {
            Token::ParenthesisBlock => {
                input.parse_nested_block(|input| {
                    // `input.try()` not needed here since the alternative uses `consume_all()`.
                    parse_condition_or_declaration(input).or_else(|()| {
                        consume_all(input);
                        Ok(SupportsCondition::FutureSyntax(input.slice_from(pos).to_owned()))
                    })
                })
            }
            Token::Function(_) => {
                input.parse_nested_block(|i| Ok(consume_all(i))).unwrap();
                Ok(SupportsCondition::FutureSyntax(input.slice_from(pos).to_owned()))
            }
            _ => Err(())
        }
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
pub fn parse_condition_or_declaration(input: &mut Parser) -> Result<SupportsCondition, ()> {
    if let Ok(condition) = input.try(SupportsCondition::parse) {
        Ok(SupportsCondition::Parenthesized(Box::new(condition)))
    } else {
        Declaration::parse(input).map(SupportsCondition::Declaration)
    }
}

impl ToCss for SupportsCondition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write {
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

#[derive(Debug)]
/// A possibly-invalid property declaration
pub struct Declaration {
    /// The property name
    pub prop: String,
    /// The property value
    pub val: String,
}

impl ToCss for Declaration {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write {
        dest.write_str(&self.prop)?;
        dest.write_str(":")?;
        // no space, the `val` already contains any possible spaces
        dest.write_str(&self.val)
    }
}

/// Slurps up input till exhausted, return string from source position
fn parse_anything(input: &mut Parser) -> String {
    let pos = input.position();
    consume_all(input);
    input.slice_from(pos).to_owned()
}

/// consume input till done
fn consume_all(input: &mut Parser) {
    while let Ok(_) = input.next() {}
}

impl Declaration {
    /// Parse a declaration
    pub fn parse(input: &mut Parser) -> Result<Declaration, ()> {
        let prop = input.expect_ident()?.into_owned();
        input.expect_colon()?;
        let val = parse_anything(input);
        Ok(Declaration { prop: prop, val: val })
    }

    /// Determine if a declaration parses
    ///
    /// https://drafts.csswg.org/css-conditional-3/#support-definition
    pub fn eval(&self, cx: &ParserContext) -> bool {
        use properties::PropertyDeclarationParseResult::*;
        let id = if let Ok(id) = PropertyId::parse((&*self.prop).into()) {
            id
        } else {
            return false
        };
        let mut input = Parser::new(&self.val);
        let mut list = Vec::new();
        let res = PropertyDeclaration::parse(id, cx, &mut input,
                                             &mut list, /* in_keyframe */ false);
        let _ = input.try(parse_important);
        if !input.is_exhausted() {
            return false;
        }
        match res {
            UnknownProperty => false,
            ExperimentalProperty => false, // only happens for experimental props
                                           // that haven't been enabled
            InvalidValue => false,
            AnimationPropertyInKeyframeBlock => unreachable!(),
            ValidOrIgnoredDeclaration => true,
        }
    }
}
