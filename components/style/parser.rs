/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which CSS code is parsed.

#![deny(missing_docs)]

use cssparser::{Parser, SourcePosition, UnicodeRange};
use error_reporting::ParseErrorReporter;
use style_traits::OneOrMoreCommaSeparated;
use stylesheets::{CssRuleType, Origin, UrlExtraData};

/// The data that the parser needs from outside in order to parse a stylesheet.
pub struct ParserContext<'a> {
    /// The `Origin` of the stylesheet, whether it's a user, author or
    /// user-agent stylesheet.
    pub stylesheet_origin: Origin,
    /// The extra data we need for resolving url values.
    pub url_data: &'a UrlExtraData,
    /// An error reporter to report syntax errors.
    pub error_reporter: &'a ParseErrorReporter,
    /// The current rule type, if any.
    pub rule_type: Option<CssRuleType>,
}

impl<'a> ParserContext<'a> {
    /// Create a parser context.
    pub fn new(stylesheet_origin: Origin,
               url_data: &'a UrlExtraData,
               error_reporter: &'a ParseErrorReporter,
               rule_type: Option<CssRuleType>)
               -> ParserContext<'a> {
        ParserContext {
            stylesheet_origin: stylesheet_origin,
            url_data: url_data,
            error_reporter: error_reporter,
            rule_type: rule_type,
        }
    }

    /// Create a parser context for on-the-fly parsing in CSSOM
    pub fn new_for_cssom(url_data: &'a UrlExtraData,
                         error_reporter: &'a ParseErrorReporter,
                         rule_type: Option<CssRuleType>)
                         -> ParserContext<'a> {
        Self::new(Origin::Author, url_data, error_reporter, rule_type)
    }

    /// Create a parser context based on a previous context, but with a modified rule type.
    pub fn new_with_rule_type(context: &'a ParserContext,
                              rule_type: Option<CssRuleType>)
                              -> ParserContext<'a> {
        Self::new(context.stylesheet_origin,
                  context.url_data,
                  context.error_reporter,
                  rule_type)
    }

    /// Get the rule type, which assumes that one is available.
    pub fn rule_type(&self) -> CssRuleType {
        self.rule_type.expect("Rule type expected, but none was found.")
    }
}

/// Defaults to a no-op.
/// Set a `RUST_LOG=style::errors` environment variable
/// to log CSS parse errors to stderr.
pub fn log_css_error(input: &mut Parser,
                     position: SourcePosition,
                     message: &str,
                     parsercontext: &ParserContext) {
    let url_data = parsercontext.url_data;
    parsercontext.error_reporter.report_error(input, position, message, url_data);
}

// XXXManishearth Replace all specified value parse impls with impls of this
// trait. This will make it easy to write more generic values in the future.
/// A trait to abstract parsing of a specified value given a `ParserContext` and
/// CSS input.
pub trait Parse : Sized {
    /// Parse a value of this type.
    ///
    /// Returns an error on failure.
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()>;
}

impl<T> Parse for Vec<T> where T: Parse + OneOrMoreCommaSeparated {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.parse_comma_separated(|input| T::parse(context, input))
    }
}

/// Parse a non-empty space-separated or comma-separated list of objects parsed by parse_one
pub fn parse_space_or_comma_separated<F, T>(input: &mut Parser, mut parse_one: F)
        -> Result<Vec<T>, ()>
        where F: FnMut(&mut Parser) -> Result<T, ()> {
    let first = parse_one(input)?;
    let mut vec = vec![first];
    loop {
        let _ = input.try(|i| i.expect_comma());
        if let Ok(val) = input.try(|i| parse_one(i)) {
            vec.push(val)
        } else {
            break
        }
    }
    Ok(vec)
}
impl Parse for UnicodeRange {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        UnicodeRange::parse(input)
    }
}
