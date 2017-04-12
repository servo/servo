/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which CSS code is parsed.

#![deny(missing_docs)]

use cssparser::{Parser, SourcePosition, UnicodeRange};
use error_reporting::ParseErrorReporter;
use style_traits::OneOrMoreCommaSeparated;
use stylesheets::{CssRuleType, Origin, UrlExtraData};

/// The mode to use when parsing lengths.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum LengthParsingMode {
    /// In CSS, lengths must have units, except for zero values, where the unit can be omitted.
    /// https://www.w3.org/TR/css3-values/#lengths
    Default,
    /// In SVG, a coordinate or length value without a unit identifier (e.g., "25") is assumed to be in user units (px).
    /// https://www.w3.org/TR/SVG/coords.html#Units
    SVG,
}

impl LengthParsingMode {
    /// Whether the parsing mode allows unitless lengths for non-zero values to be intpreted as px.
    pub fn allows_unitless_lengths(&self) -> bool {
        *self == LengthParsingMode::SVG
    }
}

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
    /// Line number offsets for inline stylesheets
    pub line_number_offset: u64,
    /// The mode to use when parsing lengths.
    pub length_parsing_mode: LengthParsingMode,
}

impl<'a> ParserContext<'a> {
    /// Create a parser context.
    pub fn new(stylesheet_origin: Origin,
               url_data: &'a UrlExtraData,
               error_reporter: &'a ParseErrorReporter,
               rule_type: Option<CssRuleType>,
               length_parsing_mode: LengthParsingMode)
               -> ParserContext<'a> {
        ParserContext {
            stylesheet_origin: stylesheet_origin,
            url_data: url_data,
            error_reporter: error_reporter,
            rule_type: rule_type,
            line_number_offset: 0u64,
            length_parsing_mode: length_parsing_mode,
        }
    }

    /// Create a parser context for on-the-fly parsing in CSSOM
    pub fn new_for_cssom(url_data: &'a UrlExtraData,
                         error_reporter: &'a ParseErrorReporter,
                         rule_type: Option<CssRuleType>,
                         length_parsing_mode: LengthParsingMode)
                         -> ParserContext<'a> {
        Self::new(Origin::Author, url_data, error_reporter, rule_type, length_parsing_mode)
    }

    /// Create a parser context based on a previous context, but with a modified rule type.
    pub fn new_with_rule_type(context: &'a ParserContext,
                              rule_type: Option<CssRuleType>)
                              -> ParserContext<'a> {
        ParserContext {
            stylesheet_origin: context.stylesheet_origin,
            url_data: context.url_data,
            error_reporter: context.error_reporter,
            rule_type: rule_type,
            line_number_offset: context.line_number_offset,
            length_parsing_mode: context.length_parsing_mode,
        }
    }

    /// Create a parser context for inline CSS which accepts additional line offset argument.
    pub fn new_with_line_number_offset(stylesheet_origin: Origin,
                                       url_data: &'a UrlExtraData,
                                       error_reporter: &'a ParseErrorReporter,
                                       line_number_offset: u64,
                                       length_parsing_mode: LengthParsingMode)
                                       -> ParserContext<'a> {
        ParserContext {
            stylesheet_origin: stylesheet_origin,
            url_data: url_data,
            error_reporter: error_reporter,
            rule_type: None,
            line_number_offset: line_number_offset,
            length_parsing_mode: length_parsing_mode,
        }
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
    let line_number_offset = parsercontext.line_number_offset;
    parsercontext.error_reporter.report_error(input, position,
                                              message, url_data,
                                              line_number_offset);
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
