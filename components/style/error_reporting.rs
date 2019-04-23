/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Types used to report parsing errors.

#![deny(missing_docs)]

use crate::selector_parser::SelectorImpl;
use crate::stylesheets::UrlExtraData;
use cssparser::{BasicParseErrorKind, ParseErrorKind, SourceLocation, Token};
use selectors::SelectorList;
use std::fmt;
use style_traits::ParseError;

/// Errors that can be encountered while parsing CSS.
#[derive(Debug)]
pub enum ContextualParseError<'a> {
    /// A property declaration was not recognized.
    UnsupportedPropertyDeclaration(
        &'a str,
        ParseError<'a>,
        Option<&'a SelectorList<SelectorImpl>>,
    ),
    /// A font face descriptor was not recognized.
    UnsupportedFontFaceDescriptor(&'a str, ParseError<'a>),
    /// A font feature values descriptor was not recognized.
    UnsupportedFontFeatureValuesDescriptor(&'a str, ParseError<'a>),
    /// A keyframe rule was not valid.
    InvalidKeyframeRule(&'a str, ParseError<'a>),
    /// A font feature values rule was not valid.
    InvalidFontFeatureValuesRule(&'a str, ParseError<'a>),
    /// A keyframe property declaration was not recognized.
    UnsupportedKeyframePropertyDeclaration(&'a str, ParseError<'a>),
    /// A rule was invalid for some reason.
    InvalidRule(&'a str, ParseError<'a>),
    /// A rule was not recognized.
    UnsupportedRule(&'a str, ParseError<'a>),
    /// A viewport descriptor declaration was not recognized.
    UnsupportedViewportDescriptorDeclaration(&'a str, ParseError<'a>),
    /// A counter style descriptor declaration was not recognized.
    UnsupportedCounterStyleDescriptorDeclaration(&'a str, ParseError<'a>),
    /// A counter style rule had no symbols.
    InvalidCounterStyleWithoutSymbols(String),
    /// A counter style rule had less than two symbols.
    InvalidCounterStyleNotEnoughSymbols(String),
    /// A counter style rule did not have additive-symbols.
    InvalidCounterStyleWithoutAdditiveSymbols,
    /// A counter style rule had extends with symbols.
    InvalidCounterStyleExtendsWithSymbols,
    /// A counter style rule had extends with additive-symbols.
    InvalidCounterStyleExtendsWithAdditiveSymbols,
    /// A media rule was invalid for some reason.
    InvalidMediaRule(&'a str, ParseError<'a>),
    /// A value was not recognized.
    UnsupportedValue(&'a str, ParseError<'a>),
}

impl<'a> fmt::Display for ContextualParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn token_to_str(t: &Token, f: &mut fmt::Formatter) -> fmt::Result {
            match *t {
                Token::Ident(ref i) => write!(f, "identifier {}", i),
                Token::AtKeyword(ref kw) => write!(f, "keyword @{}", kw),
                Token::Hash(ref h) => write!(f, "hash #{}", h),
                Token::IDHash(ref h) => write!(f, "id selector #{}", h),
                Token::QuotedString(ref s) => write!(f, "quoted string \"{}\"", s),
                Token::UnquotedUrl(ref u) => write!(f, "url {}", u),
                Token::Delim(ref d) => write!(f, "delimiter {}", d),
                Token::Number {
                    int_value: Some(i), ..
                } => write!(f, "number {}", i),
                Token::Number { value, .. } => write!(f, "number {}", value),
                Token::Percentage {
                    int_value: Some(i), ..
                } => write!(f, "percentage {}", i),
                Token::Percentage { unit_value, .. } => {
                    write!(f, "percentage {}", unit_value * 100.)
                },
                Token::Dimension {
                    value, ref unit, ..
                } => write!(f, "dimension {}{}", value, unit),
                Token::WhiteSpace(_) => write!(f, "whitespace"),
                Token::Comment(_) => write!(f, "comment"),
                Token::Colon => write!(f, "colon (:)"),
                Token::Semicolon => write!(f, "semicolon (;)"),
                Token::Comma => write!(f, "comma (,)"),
                Token::IncludeMatch => write!(f, "include match (~=)"),
                Token::DashMatch => write!(f, "dash match (|=)"),
                Token::PrefixMatch => write!(f, "prefix match (^=)"),
                Token::SuffixMatch => write!(f, "suffix match ($=)"),
                Token::SubstringMatch => write!(f, "substring match (*=)"),
                Token::CDO => write!(f, "CDO (<!--)"),
                Token::CDC => write!(f, "CDC (-->)"),
                Token::Function(ref name) => write!(f, "function {}", name),
                Token::ParenthesisBlock => write!(f, "parenthesis ("),
                Token::SquareBracketBlock => write!(f, "square bracket ["),
                Token::CurlyBracketBlock => write!(f, "curly bracket {{"),
                Token::BadUrl(ref _u) => write!(f, "bad url parse error"),
                Token::BadString(ref _s) => write!(f, "bad string parse error"),
                Token::CloseParenthesis => write!(f, "unmatched close parenthesis"),
                Token::CloseSquareBracket => write!(f, "unmatched close square bracket"),
                Token::CloseCurlyBracket => write!(f, "unmatched close curly bracket"),
            }
        }

        fn parse_error_to_str(err: &ParseError, f: &mut fmt::Formatter) -> fmt::Result {
            match err.kind {
                ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(ref t)) => {
                    write!(f, "found unexpected ")?;
                    token_to_str(t, f)
                },
                ParseErrorKind::Basic(BasicParseErrorKind::EndOfInput) => {
                    write!(f, "unexpected end of input")
                },
                ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(ref i)) => {
                    write!(f, "@ rule invalid: {}", i)
                },
                ParseErrorKind::Basic(BasicParseErrorKind::AtRuleBodyInvalid) => {
                    write!(f, "@ rule invalid")
                },
                ParseErrorKind::Basic(BasicParseErrorKind::QualifiedRuleInvalid) => {
                    write!(f, "qualified rule invalid")
                },
                ParseErrorKind::Custom(ref err) => write!(f, "{:?}", err),
            }
        }

        match *self {
            ContextualParseError::UnsupportedPropertyDeclaration(decl, ref err, _selectors) => {
                write!(f, "Unsupported property declaration: '{}', ", decl)?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::UnsupportedFontFaceDescriptor(decl, ref err) => {
                write!(
                    f,
                    "Unsupported @font-face descriptor declaration: '{}', ",
                    decl
                )?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::UnsupportedFontFeatureValuesDescriptor(decl, ref err) => {
                write!(
                    f,
                    "Unsupported @font-feature-values descriptor declaration: '{}', ",
                    decl
                )?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::InvalidKeyframeRule(rule, ref err) => {
                write!(f, "Invalid keyframe rule: '{}', ", rule)?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::InvalidFontFeatureValuesRule(rule, ref err) => {
                write!(f, "Invalid font feature value rule: '{}', ", rule)?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::UnsupportedKeyframePropertyDeclaration(decl, ref err) => {
                write!(f, "Unsupported keyframe property declaration: '{}', ", decl)?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::InvalidRule(rule, ref err) => {
                write!(f, "Invalid rule: '{}', ", rule)?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::UnsupportedRule(rule, ref err) => {
                write!(f, "Unsupported rule: '{}', ", rule)?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::UnsupportedViewportDescriptorDeclaration(decl, ref err) => {
                write!(
                    f,
                    "Unsupported @viewport descriptor declaration: '{}', ",
                    decl
                )?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(decl, ref err) => {
                write!(
                    f,
                    "Unsupported @counter-style descriptor declaration: '{}', ",
                    decl
                )?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::InvalidCounterStyleWithoutSymbols(ref system) => write!(
                f,
                "Invalid @counter-style rule: 'system: {}' without 'symbols'",
                system
            ),
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(ref system) => write!(
                f,
                "Invalid @counter-style rule: 'system: {}' less than two 'symbols'",
                system
            ),
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols => write!(
                f,
                "Invalid @counter-style rule: 'system: additive' without 'additive-symbols'"
            ),
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols => write!(
                f,
                "Invalid @counter-style rule: 'system: extends …' with 'symbols'"
            ),
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols => write!(
                f,
                "Invalid @counter-style rule: 'system: extends …' with 'additive-symbols'"
            ),
            ContextualParseError::InvalidMediaRule(media_rule, ref err) => {
                write!(f, "Invalid media rule: {}, ", media_rule)?;
                parse_error_to_str(err, f)
            },
            ContextualParseError::UnsupportedValue(_value, ref err) => parse_error_to_str(err, f),
        }
    }
}

/// A generic trait for an error reporter.
pub trait ParseErrorReporter {
    /// Called when the style engine detects an error.
    ///
    /// Returns the current input being parsed, the source location it was
    /// reported from, and a message.
    fn report_error(
        &self,
        url: &UrlExtraData,
        location: SourceLocation,
        error: ContextualParseError,
    );
}

/// An error reporter that uses [the `log` crate](https://github.com/rust-lang-nursery/log)
/// at `info` level.
///
/// This logging is silent by default, and can be enabled with a `RUST_LOG=style=info`
/// environment variable.
/// (See [`env_logger`](https://rust-lang-nursery.github.io/log/env_logger/).)
#[cfg(feature = "servo")]
pub struct RustLogReporter;

#[cfg(feature = "servo")]
impl ParseErrorReporter for RustLogReporter {
    fn report_error(
        &self,
        url: &UrlExtraData,
        location: SourceLocation,
        error: ContextualParseError,
    ) {
        if log_enabled!(log::Level::Info) {
            info!(
                "Url:\t{}\n{}:{} {}",
                url.as_str(),
                location.line,
                location.column,
                error
            )
        }
    }
}
