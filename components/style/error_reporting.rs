/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types used to report parsing errors.

#![deny(missing_docs)]

use cssparser::{Parser, SourcePosition, BasicParseError, Token};
use cssparser::ParseError as CssParseError;
use log;
use style_traits::ParseError;
use stylesheets::UrlExtraData;

/// Errors that can be encountered while parsing CSS.
pub enum ContextualParseError<'a> {
    /// A property declaration was not recognized.
    UnsupportedPropertyDeclaration(&'a str, ParseError<'a>),
    /// A font face descriptor was not recognized.
    UnsupportedFontFaceDescriptor(&'a str, ParseError<'a>),
    /// A keyframe rule was not valid.
    InvalidKeyframeRule(&'a str, ParseError<'a>),
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
    InvalidCounterStyleExtendsWithAdditiveSymbols
}

impl<'a> ContextualParseError<'a> {
    /// Turn a parse error into a string representation.
    pub fn to_string(&self) -> String {
        fn token_to_str(t: &Token) -> String {
            match *t {
                Token::Ident(ref i) => format!("identifier {}", i),
                Token::AtKeyword(ref kw) => format!("keyword @{}", kw),
                Token::Hash(ref h) => format!("hash #{}", h),
                Token::IDHash(ref h) => format!("id selector #{}", h),
                Token::QuotedString(ref s) => format!("quoted string \"{}\"", s),
                Token::UnquotedUrl(ref u) => format!("url {}", u),
                Token::Delim(ref d) => format!("delimiter {}", d),
                Token::Number { int_value: Some(i), .. } => format!("number {}", i),
                Token::Number { value, .. } => format!("number {}", value),
                Token::Percentage { int_value: Some(i), .. } => format!("percentage {}", i),
                Token::Percentage { unit_value, .. } => format!("percentage {}", unit_value * 100.),
                Token::Dimension { value, ref unit, .. } => format!("dimension {}{}", value, unit),
                Token::WhiteSpace(_) => format!("whitespace"),
                Token::Comment(_) => format!("comment"),
                Token::Colon => format!("colon (:)"),
                Token::Semicolon => format!("semicolon (;)"),
                Token::Comma => format!("comma (,)"),
                Token::IncludeMatch => format!("include match (~=)"),
                Token::DashMatch => format!("dash match (|=)"),
                Token::PrefixMatch => format!("prefix match (^=)"),
                Token::SuffixMatch => format!("suffix match ($=)"),
                Token::SubstringMatch => format!("substring match (*=)"),
                Token::Column => format!("column (||)"),
                Token::CDO => format!("CDO (<!--)"),
                Token::CDC => format!("CDC (-->)"),
                Token::Function(ref f) => format!("function {}", f),
                Token::ParenthesisBlock => format!("parenthesis ("),
                Token::SquareBracketBlock => format!("square bracket ["),
                Token::CurlyBracketBlock => format!("curly bracket {{"),
                Token::BadUrl(ref _u) => format!("bad url parse error"),
                Token::BadString(ref _s) => format!("bad string parse error"),
                Token::CloseParenthesis => format!("unmatched close parenthesis"),
                Token::CloseSquareBracket => format!("unmatched close square bracket"),
                Token::CloseCurlyBracket => format!("unmatched close curly bracket"),
            }
        }

        fn parse_error_to_str(err: &ParseError) -> String {
            match *err {
                CssParseError::Basic(BasicParseError::UnexpectedToken(ref t)) =>
                    format!("found unexpected {}", token_to_str(t)),
                CssParseError::Basic(BasicParseError::EndOfInput) =>
                    format!("unexpected end of input"),
                CssParseError::Basic(BasicParseError::AtRuleInvalid(ref i)) =>
                    format!("@ rule invalid: {}", i),
                CssParseError::Basic(BasicParseError::AtRuleBodyInvalid) =>
                    format!("@ rule invalid"),
                CssParseError::Basic(BasicParseError::QualifiedRuleInvalid) =>
                    format!("qualified rule invalid"),
                CssParseError::Custom(ref err) =>
                    format!("{:?}", err)
            }
        }

        match *self {
            ContextualParseError::UnsupportedPropertyDeclaration(decl, ref err) =>
                format!("Unsupported property declaration: '{}', {}", decl,
                        parse_error_to_str(err)),
            ContextualParseError::UnsupportedFontFaceDescriptor(decl, ref err) =>
                format!("Unsupported @font-face descriptor declaration: '{}', {}", decl,
                        parse_error_to_str(err)),
            ContextualParseError::InvalidKeyframeRule(rule, ref err) =>
                format!("Invalid keyframe rule: '{}', {}", rule,
                        parse_error_to_str(err)),
            ContextualParseError::UnsupportedKeyframePropertyDeclaration(decl, ref err) =>
                format!("Unsupported keyframe property declaration: '{}', {}", decl,
                        parse_error_to_str(err)),
            ContextualParseError::InvalidRule(rule, ref err) =>
                format!("Invalid rule: '{}', {}", rule, parse_error_to_str(err)),
            ContextualParseError::UnsupportedRule(rule, ref err) =>
                format!("Unsupported rule: '{}', {}", rule, parse_error_to_str(err)),
            ContextualParseError::UnsupportedViewportDescriptorDeclaration(decl, ref err) =>
                format!("Unsupported @viewport descriptor declaration: '{}', {}", decl,
                        parse_error_to_str(err)),
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(decl, ref err) =>
                format!("Unsupported @counter-style descriptor declaration: '{}', {}", decl,
                        parse_error_to_str(err)),
            ContextualParseError::InvalidCounterStyleWithoutSymbols(ref system) =>
                format!("Invalid @counter-style rule: 'system: {}' without 'symbols'", system),
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(ref system) =>
                format!("Invalid @counter-style rule: 'system: {}' less than two 'symbols'", system),
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols =>
                "Invalid @counter-style rule: 'system: additive' without 'additive-symbols'".into(),
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols =>
                "Invalid @counter-style rule: 'system: extends …' with 'symbols'".into(),
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols =>
                "Invalid @counter-style rule: 'system: extends …' with 'additive-symbols'".into(),
        }
    }
}

/// A generic trait for an error reporter.
pub trait ParseErrorReporter {
    /// Called when the style engine detects an error.
    ///
    /// Returns the current input being parsed, the source position it was
    /// reported from, and a message.
    fn report_error<'a>(&self,
                        input: &mut Parser,
                        position: SourcePosition,
                        error: ContextualParseError<'a>,
                        url: &UrlExtraData,
                        line_number_offset: u64);
}

/// An error reporter that uses [the `log` crate](https://github.com/rust-lang-nursery/log)
/// at `info` level.
///
/// This logging is silent by default, and can be enabled with a `RUST_LOG=style=info`
/// environment variable.
/// (See [`env_logger`](https://rust-lang-nursery.github.io/log/env_logger/).)
pub struct RustLogReporter;

impl ParseErrorReporter for RustLogReporter {
    fn report_error<'a>(&self,
                        input: &mut Parser,
                        position: SourcePosition,
                        error: ContextualParseError<'a>,
                        url: &UrlExtraData,
                        line_number_offset: u64) {
        if log_enabled!(log::LogLevel::Info) {
            let location = input.source_location(position);
            let line_offset = location.line + line_number_offset as u32;
            info!("Url:\t{}\n{}:{} {}", url.as_str(), line_offset, location.column, error.to_string())
        }
    }
}

/// Error reporter which silently forgets errors
pub struct NullReporter;

impl ParseErrorReporter for NullReporter {
    fn report_error<'a>(&self,
            _: &mut Parser,
            _: SourcePosition,
            _: ContextualParseError<'a>,
            _: &UrlExtraData,
            _: u64) {
        // do nothing
    }
}
