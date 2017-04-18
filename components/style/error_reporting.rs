/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types used to report parsing errors.

#![deny(missing_docs)]

use cssparser::{Parser, SourcePosition};
use log;
use stylesheets::UrlExtraData;

/// Errors that can be encountered while parsing CSS.
pub enum ParseError<'a> {
    /// A property declaration was not recognized.
    UnsupportedPropertyDeclaration(&'a str),
    /// A font face descriptor was not recognized.
    UnsupportedFontFaceDescriptor(&'a str),
    /// A keyframe rule was not valid.
    InvalidKeyframeRule(&'a str),
    /// A keyframe property declaration was not recognized.
    UnsupportedKeyframePropertyDeclaration(&'a str),
    /// A rule was invalid for some reason.
    InvalidRule(&'a str),
    /// A rule was not recognized.
    UnsupportedRule(&'a str),
    /// A viewport descriptor declaration was not recognized.
    UnsupportedViewportDescriptorDeclaration(&'a str),
    /// A counter style descriptor declaration was not recognized.
    UnsupportedCounterStyleDescriptorDeclaration(&'a str),
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

impl<'a> ParseError<'a> {
    /// Turn a parse error into a string representation.
    pub fn to_string(&self) -> String {
        match *self {
            ParseError::UnsupportedPropertyDeclaration(decl) =>
                format!("Unsupported property declaration: '{}'", decl),
            ParseError::UnsupportedFontFaceDescriptor(decl) =>
                format!("Unsupported @font-face descriptor declaration: '{}'", decl),
            ParseError::InvalidKeyframeRule(rule) =>
                format!("Invalid keyframe rule: '{}'", rule),
            ParseError::UnsupportedKeyframePropertyDeclaration(decl) =>
                format!("Unsupported keyframe property declaration: '{}'", decl),
            ParseError::InvalidRule(rule) =>
                format!("Invalid rule: '{}'", rule),
            ParseError::UnsupportedRule(rule) =>
                format!("Unsupported rule: '{}'", rule),
            ParseError::UnsupportedViewportDescriptorDeclaration(decl) =>
                format!("Unsupported @viewport descriptor declaration: '{}'", decl),
            ParseError::UnsupportedCounterStyleDescriptorDeclaration(decl) =>
                format!("Unsupported @counter-style descriptor declaration: '{}'", decl),
            ParseError::InvalidCounterStyleWithoutSymbols(ref system) =>
                format!("Invalid @counter-style rule: 'system: {}' without 'symbols'", system),
            ParseError::InvalidCounterStyleNotEnoughSymbols(ref system) =>
                format!("Invalid @counter-style rule: 'system: {}' less than two 'symbols'", system),
            ParseError::InvalidCounterStyleWithoutAdditiveSymbols =>
                "Invalid @counter-style rule: 'system: additive' without 'additive-symbols'".into(),
            ParseError::InvalidCounterStyleExtendsWithSymbols =>
                "Invalid @counter-style rule: 'system: extends …' with 'symbols'".into(),
            ParseError::InvalidCounterStyleExtendsWithAdditiveSymbols =>
                "Invalid @counter-style rule: 'system: extends …' with 'additive-symbols'".into(),
        }
    }
}

/// A generic trait for an error reporter.
pub trait ParseErrorReporter : Sync + Send {
    /// Called when the style engine detects an error.
    ///
    /// Returns the current input being parsed, the source position it was
    /// reported from, and a message.
    fn report_error<'a>(&self,
                        input: &mut Parser,
                        position: SourcePosition,
                        error: ParseError<'a>,
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
                        error: ParseError<'a>,
                        url: &UrlExtraData,
                        line_number_offset: u64) {
        if log_enabled!(log::LogLevel::Info) {
            let location = input.source_location(position);
            let line_offset = location.line + line_number_offset as usize;
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
            _: ParseError<'a>,
            _: &UrlExtraData,
            _: u64) {
        // do nothing
    }
}

/// Create an instance of the default error reporter for Servo.
#[cfg(feature = "servo")]
pub fn create_error_reporter() -> RustLogReporter {
    RustLogReporter
}

/// Create an instance of the default error reporter for Stylo.
#[cfg(feature = "gecko")]
pub fn create_error_reporter() -> RustLogReporter {
    RustLogReporter
}
