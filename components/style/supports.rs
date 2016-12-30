/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [@supports rules](https://drafts.csswg.org/css-conditional-3/#at-supports)

use cssparser::Parser;
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
                return Ok(first_cond)
            }
            let mut vec = vec![first_cond];
            if let Ok(ident) = input.try(|i| i.expect_ident()) {
                match &*ident {
                    "and" | "or" => {
                        let mut first = true;
                        while !input.is_exhausted() {
                            if !first {
                                input.expect_ident_matching(&ident)?;
                            }
                            first = false;
                            vec.push(SupportsCondition::parse_with_parens(input)?);
                        }
                        if ident == "and" {
                            Ok(SupportsCondition::And(vec))
                        } else {
                            Ok(SupportsCondition::Or(vec))
                        }
                    }
                    _ => Err(())
                }
            } else {
                Err(())
            }
        } else {
            // https://drafts.csswg.org/css-conditional-3/#general_enclosed
            if toplevel {
                return Err(())
            }
            let pos = input.position();
            if let Ok(_) = input.try(Parser::expect_function) {
                input.parse_nested_block(|i| Ok(consume_all(i)))?;
                Ok(SupportsCondition::FutureSyntax(input.slice_from(pos).to_owned()))
            } else if let Ok(_) = input.try(Parser::expect_parenthesis_block) {
                input.parse_nested_block(|i| Ok(consume_all(i)))?;
                Ok(SupportsCondition::FutureSyntax(input.slice_from(pos).to_owned()))
            } else {
                Err(())
            }
        }
    }

    fn parse_with_parens(input: &mut Parser) -> Result<SupportsCondition, ()> {
        input.expect_parenthesis_block()?;
        input.parse_nested_block(|input| SupportsCondition::parse(input, false))
    }

    /// Evaluate a supports condition
    pub fn eval(&self, cx: &ParserContext) -> bool {
        match *self {
            SupportsCondition::Not(ref cond) => !cond.eval(cx),
            SupportsCondition::And(ref vec) => vec.iter().all(|c| c.eval(cx)),
            SupportsCondition::Or(ref vec) => !vec.iter().all(|c| !c.eval(cx)),
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
            SupportsCondition::And(ref vec) => {
                let mut first = true;
                for cond in vec {
                    if !first {
                        dest.write_str(" and ")?;
                    }
                    first = true;
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
                        dest.write_str(" and ")?;
                    }
                    first = true;
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
    prop: String,
    val: String,
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
        match res {
            UnknownProperty => false,
            ExperimentalProperty => false, // only happens for experimental props
                                           // that haven't been enabled
            InvalidValue => false,
            AnimationPropertyInKeyframeBlock => false, // should never happen
            ValidOrIgnoredDeclaration => true,
        }
    }
}
