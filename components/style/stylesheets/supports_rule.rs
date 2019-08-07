/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [@supports rules](https://drafts.csswg.org/css-conditional-3/#at-supports)

use crate::parser::ParserContext;
use crate::properties::{PropertyDeclaration, PropertyId, SourcePropertyDeclaration};
use crate::selector_parser::{SelectorImpl, SelectorParser};
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::stylesheets::{CssRuleType, CssRules, Namespaces};
use cssparser::parse_important;
use cssparser::{Delimiter, Parser, SourceLocation, Token};
use cssparser::{ParseError as CssParseError, ParserInput};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use selectors::parser::{Selector, SelectorParseErrorKind};
use servo_arc::Arc;
use std::ffi::{CStr, CString};
use std::fmt::{self, Write};
use std::str;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// An [`@supports`][supports] rule.
///
/// [supports]: https://drafts.csswg.org/css-conditional-3/#at-supports
#[derive(Debug, ToShmem)]
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

impl SupportsRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        // Measurement of other fields may be added later.
        self.rules.unconditional_shallow_size_of(ops) +
            self.rules.read_with(guard).size_of(guard, ops)
    }
}

impl ToCssWithGuard for SupportsRule {
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@supports ")?;
        self.condition.to_css(&mut CssWriter::new(dest))?;
        self.rules.read_with(guard).to_css_block(guard, dest)
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
/// <https://drafts.csswg.org/css-conditional-3/#at-supports>
#[derive(Clone, Debug, ToShmem)]
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
    /// A `selector()` function.
    Selector(RawSelector),
    /// `-moz-bool-pref("pref-name")`
    /// Since we need to pass it through FFI to get the pref value,
    /// we store it as CString directly.
    MozBoolPref(CString),
    /// `(any tokens)` or `func(any tokens)`
    FutureSyntax(String),
}

impl SupportsCondition {
    /// Parse a condition
    ///
    /// <https://drafts.csswg.org/css-conditional/#supports_condition>
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("not")).is_ok() {
            let inner = SupportsCondition::parse_in_parens(input)?;
            return Ok(SupportsCondition::Not(Box::new(inner)));
        }

        let in_parens = SupportsCondition::parse_in_parens(input)?;

        let location = input.current_source_location();
        let (keyword, wrapper) = match input.next() {
            // End of input
            Err(..) => return Ok(in_parens),
            Ok(&Token::Ident(ref ident)) => {
                match_ignore_ascii_case! { &ident,
                    "and" => ("and", SupportsCondition::And as fn(_) -> _),
                    "or" => ("or", SupportsCondition::Or as fn(_) -> _),
                    _ => return Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())))
                }
            },
            Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
        };

        let mut conditions = Vec::with_capacity(2);
        conditions.push(in_parens);
        loop {
            conditions.push(SupportsCondition::parse_in_parens(input)?);
            if input
                .try(|input| input.expect_ident_matching(keyword))
                .is_err()
            {
                // Did not find the expected keyword.
                // If we found some other token, it will be rejected by
                // `Parser::parse_entirely` somewhere up the stack.
                return Ok(wrapper(conditions));
            }
        }
    }

    /// Parses a functional supports condition.
    fn parse_functional<'i, 't>(
        function: &str,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        match_ignore_ascii_case! { function,
            // Although this is an internal syntax, it is not necessary
            // to check parsing context as far as we accept any
            // unexpected token as future syntax, and evaluate it to
            // false when not in chrome / ua sheet.
            // See https://drafts.csswg.org/css-conditional-3/#general_enclosed
            "-moz-bool-pref" => {
                let name = {
                    let name = input.expect_string()?;
                    CString::new(name.as_bytes())
                }.map_err(|_| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))?;
                Ok(SupportsCondition::MozBoolPref(name))
            }
            "selector" => {
                let pos = input.position();
                consume_any_value(input)?;
                Ok(SupportsCondition::Selector(RawSelector(
                    input.slice_from(pos).to_owned()
                )))
            }
            _ => {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }
        }
    }

    /// <https://drafts.csswg.org/css-conditional-3/#supports_condition_in_parens>
    fn parse_in_parens<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // Whitespace is normally taken care of in `Parser::next`,
        // but we want to not include it in `pos` for the SupportsCondition::FutureSyntax cases.
        while input.try(Parser::expect_whitespace).is_ok() {}
        let pos = input.position();
        let location = input.current_source_location();
        // FIXME: remove clone() when lifetimes are non-lexical
        match input.next()?.clone() {
            Token::ParenthesisBlock => {
                let nested =
                    input.try(|input| input.parse_nested_block(parse_condition_or_declaration));
                if nested.is_ok() {
                    return nested;
                }
            },
            Token::Function(ident) => {
                let nested = input.try(|input| {
                    input.parse_nested_block(|input| {
                        SupportsCondition::parse_functional(&ident, input)
                    })
                });
                if nested.is_ok() {
                    return nested;
                }
            },
            t => return Err(location.new_unexpected_token_error(t)),
        }
        input.parse_nested_block(consume_any_value)?;
        Ok(SupportsCondition::FutureSyntax(
            input.slice_from(pos).to_owned(),
        ))
    }

    /// Evaluate a supports condition
    pub fn eval(&self, cx: &ParserContext, namespaces: &Namespaces) -> bool {
        match *self {
            SupportsCondition::Not(ref cond) => !cond.eval(cx, namespaces),
            SupportsCondition::Parenthesized(ref cond) => cond.eval(cx, namespaces),
            SupportsCondition::And(ref vec) => vec.iter().all(|c| c.eval(cx, namespaces)),
            SupportsCondition::Or(ref vec) => vec.iter().any(|c| c.eval(cx, namespaces)),
            SupportsCondition::Declaration(ref decl) => decl.eval(cx),
            SupportsCondition::MozBoolPref(ref name) => eval_moz_bool_pref(name, cx),
            SupportsCondition::Selector(ref selector) => selector.eval(cx, namespaces),
            SupportsCondition::FutureSyntax(_) => false,
        }
    }
}

#[cfg(feature = "gecko")]
fn eval_moz_bool_pref(name: &CStr, cx: &ParserContext) -> bool {
    use crate::gecko_bindings::bindings;
    if !cx.in_ua_or_chrome_sheet() {
        return false;
    }
    unsafe { bindings::Gecko_GetBoolPrefValue(name.as_ptr()) }
}

#[cfg(feature = "servo")]
fn eval_moz_bool_pref(_: &CStr, _: &ParserContext) -> bool {
    false
}

/// supports_condition | declaration
/// <https://drafts.csswg.org/css-conditional/#dom-css-supports-conditiontext-conditiontext>
pub fn parse_condition_or_declaration<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<SupportsCondition, ParseError<'i>> {
    if let Ok(condition) = input.try(SupportsCondition::parse) {
        Ok(SupportsCondition::Parenthesized(Box::new(condition)))
    } else {
        Declaration::parse(input).map(SupportsCondition::Declaration)
    }
}

impl ToCss for SupportsCondition {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            SupportsCondition::Not(ref cond) => {
                dest.write_str("not ")?;
                cond.to_css(dest)
            },
            SupportsCondition::Parenthesized(ref cond) => {
                dest.write_str("(")?;
                cond.to_css(dest)?;
                dest.write_str(")")
            },
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
            },
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
            },
            SupportsCondition::Declaration(ref decl) => {
                dest.write_str("(")?;
                decl.to_css(dest)?;
                dest.write_str(")")
            },
            SupportsCondition::Selector(ref selector) => {
                dest.write_str("selector(")?;
                selector.to_css(dest)?;
                dest.write_str(")")
            },
            SupportsCondition::MozBoolPref(ref name) => {
                dest.write_str("-moz-bool-pref(")?;
                let name =
                    str::from_utf8(name.as_bytes()).expect("Should be parsed from valid UTF-8");
                name.to_css(dest)?;
                dest.write_str(")")
            },
            SupportsCondition::FutureSyntax(ref s) => dest.write_str(&s),
        }
    }
}

#[derive(Clone, Debug, ToShmem)]
/// A possibly-invalid CSS selector.
pub struct RawSelector(pub String);

impl ToCss for RawSelector {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str(&self.0)
    }
}

impl RawSelector {
    /// Tries to evaluate a `selector()` function.
    pub fn eval(&self, context: &ParserContext, namespaces: &Namespaces) -> bool {
        #[cfg(feature = "gecko")]
        {
            if !static_prefs::pref!("layout.css.supports-selector.enabled") {
                return false;
            }
        }

        let mut input = ParserInput::new(&self.0);
        let mut input = Parser::new(&mut input);
        input
            .parse_entirely(|input| -> Result<(), CssParseError<()>> {
                let parser = SelectorParser {
                    namespaces,
                    stylesheet_origin: context.stylesheet_origin,
                    url_data: Some(context.url_data),
                };

                #[allow(unused_variables)]
                let selector = Selector::<SelectorImpl>::parse(&parser, input)
                    .map_err(|_| input.new_custom_error(()))?;

                #[cfg(feature = "gecko")]
                {
                    use crate::selector_parser::PseudoElement;
                    use selectors::parser::Component;

                    let has_any_unknown_webkit_pseudo = selector.has_pseudo_element() &&
                        selector.iter_raw_match_order().any(|component| {
                            matches!(
                                *component,
                                Component::PseudoElement(PseudoElement::UnknownWebkit(..))
                            )
                        });
                    if has_any_unknown_webkit_pseudo {
                        return Err(input.new_custom_error(()));
                    }
                }

                Ok(())
            })
            .is_ok()
    }
}

#[derive(Clone, Debug, ToShmem)]
/// A possibly-invalid property declaration
pub struct Declaration(pub String);

impl ToCss for Declaration {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str(&self.0)
    }
}

/// <https://drafts.csswg.org/css-syntax-3/#typedef-any-value>
fn consume_any_value<'i, 't>(input: &mut Parser<'i, 't>) -> Result<(), ParseError<'i>> {
    input.expect_no_error_token().map_err(|err| err.into())
}

impl Declaration {
    /// Parse a declaration
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Declaration, ParseError<'i>> {
        let pos = input.position();
        input.expect_ident()?;
        input.expect_colon()?;
        consume_any_value(input)?;
        Ok(Declaration(input.slice_from(pos).to_owned()))
    }

    /// Determine if a declaration parses
    ///
    /// <https://drafts.csswg.org/css-conditional-3/#support-definition>
    pub fn eval(&self, context: &ParserContext) -> bool {
        debug_assert_eq!(context.rule_type(), CssRuleType::Style);

        let mut input = ParserInput::new(&self.0);
        let mut input = Parser::new(&mut input);
        input
            .parse_entirely(|input| -> Result<(), CssParseError<()>> {
                let prop = input.expect_ident_cloned().unwrap();
                input.expect_colon().unwrap();

                let id =
                    PropertyId::parse(&prop, context).map_err(|_| input.new_custom_error(()))?;

                let mut declarations = SourcePropertyDeclaration::new();
                input.parse_until_before(Delimiter::Bang, |input| {
                    PropertyDeclaration::parse_into(&mut declarations, id, &context, input)
                        .map_err(|_| input.new_custom_error(()))
                })?;
                let _ = input.try(parse_important);
                Ok(())
            })
            .is_ok()
    }
}
