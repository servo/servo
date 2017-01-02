/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [@supports rules](https://drafts.csswg.org/css-conditional-3/#at-supports)

use cssparser::{parse_important, Parser};
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
    /// `toplevel` is true when parsing a top-level condition in an `@supports` block,
    /// and false when dealing with paren-nested conditions like `not(cond)` or `(cond) and (cond)`.
    /// Declaration and FutureSyntax conditions can't appear in the top level without parens
    pub fn parse(input: &mut Parser, toplevel: bool) -> Result<SupportsCondition, ()> {
        if let Ok(_) = input.try(|i| i.expect_ident_matching("not")) {
            let inner = SupportsCondition::parse_with_parens(input)?;
            return Ok(SupportsCondition::Not(Box::new(inner)));
        }
        if !toplevel {
            if let Ok(decl) = input.try(Declaration::parse) {
                return Ok(SupportsCondition::Declaration(decl))
            }
        }

        if let Ok(first_cond) = input.try(SupportsCondition::parse_with_parens) {
            if input.is_exhausted() {
                // nested parens
                return Ok(SupportsCondition::Parenthesized(Box::new(first_cond)));
            }
            let mut vec = vec![first_cond];
            let ident = input.expect_ident()?;
            let parse_vec = |i| {
                    let mut first = true;
                    while !input.is_exhausted() {
                        if !first {
                            input.expect_ident_matching(i)?;
                        }
                        first = false;
                        vec.push(SupportsCondition::parse_with_parens(input)?);
                    }
                    Ok(vec)
            };
            match_ignore_ascii_case! { ident,
                "and" => Ok(SupportsCondition::And(parse_vec(&ident)?)),
                "or" => Ok(SupportsCondition::Or(parse_vec(&ident)?)),
                _ => Err(())
            }
        } else {
            // parse_with_parens handles general_enclosed
            // the only time that won't happen is at the top level,
            // and general_enclosed isn't valid at the top level anyway
            Err(())
        }
    }

    // https://drafts.csswg.org/css-conditional-3/#supports_condition_in_parens
    fn parse_with_parens(input: &mut Parser) -> Result<SupportsCondition, ()> {
        let pos = input.position();
        if let Ok(_) = input.try(Parser::expect_parenthesis_block) {
            let inner = input.try(|i| {
                i.parse_nested_block(|input| SupportsCondition::parse(input, false))
            });
            // The above will fail in case of a general_enclosed
            // https://drafts.csswg.org/css-conditional-3/#general_enclosed

            // We handle this outside of `SupportsCondition::parse` for two reasons.
            // Firstly, unenclosed function calls are also valid general_enclosed,
            // so `CSS.supports("not foobar(baz)")` is true. Secondly, we need to
            // be able to deal with the case where `SupportsCondition::parse` finishes,
            // but the input is not exhausted (so `parse_nested_block` fails). Such
            // cases are also general_enclosed.

            // general_enclosed is a function or parenthesized block containing
            // arbitrary tokens
            if let Ok(inner) = inner {
                Ok(inner)
            } else {
                input.parse_nested_block(|i| Ok(consume_all(i)))?;
                Ok(SupportsCondition::FutureSyntax(input.slice_from(pos).to_owned()))
            }
        } else if let Ok(_) = input.try(Parser::expect_function) {
                input.parse_nested_block(|i| Ok(consume_all(i)))?;
                Ok(SupportsCondition::FutureSyntax(input.slice_from(pos).to_owned()))
        } else {
            Err(())
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

impl ToCss for SupportsCondition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write {
        match *self {
            SupportsCondition::Not(ref cond) => {
                dest.write_str("not (")?;
                cond.to_css(dest)?;
                dest.write_str(")")
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
                    dest.write_str("(")?;
                    cond.to_css(dest)?;
                    dest.write_str(")")?;
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
                    dest.write_str("(")?;
                    cond.to_css(dest)?;
                    dest.write_str(")")?;
                }
                Ok(())
            }
            SupportsCondition::Declaration(ref decl) => decl.to_css(dest),
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
