/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The context within which CSS code is parsed.

use crate::context::QuirksMode;
use crate::error_reporting::{ContextualParseError, ParseErrorReporter};
use crate::stylesheets::{CssRuleType, Namespaces, Origin, UrlExtraData};
use crate::use_counters::UseCounters;
use cssparser::{Parser, SourceLocation, UnicodeRange};
use style_traits::{OneOrMoreSeparated, ParseError, ParsingMode, Separator};

/// Asserts that all ParsingMode flags have a matching ParsingMode value in gecko.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_parsing_mode_match() {
    use crate::gecko_bindings::structs;

    macro_rules! check_parsing_modes {
        ( $( $a:ident => $b:path ),*, ) => {
            if cfg!(debug_assertions) {
                let mut modes = ParsingMode::all();
                $(
                    assert_eq!(structs::$a as usize, $b.bits() as usize, stringify!($b));
                    modes.remove($b);
                )*
                assert_eq!(modes, ParsingMode::empty(), "all ParsingMode bits should have an assertion");
            }
        }
    }

    check_parsing_modes! {
        ParsingMode_Default => ParsingMode::DEFAULT,
        ParsingMode_AllowUnitlessLength => ParsingMode::ALLOW_UNITLESS_LENGTH,
        ParsingMode_AllowAllNumericValues => ParsingMode::ALLOW_ALL_NUMERIC_VALUES,
    }
}

/// The data that the parser needs from outside in order to parse a stylesheet.
pub struct ParserContext<'a> {
    /// The `Origin` of the stylesheet, whether it's a user, author or
    /// user-agent stylesheet.
    pub stylesheet_origin: Origin,
    /// The extra data we need for resolving url values.
    pub url_data: &'a UrlExtraData,
    /// The current rule type, if any.
    pub rule_type: Option<CssRuleType>,
    /// The mode to use when parsing.
    pub parsing_mode: ParsingMode,
    /// The quirks mode of this stylesheet.
    pub quirks_mode: QuirksMode,
    /// The active error reporter, or none if error reporting is disabled.
    error_reporter: Option<&'a ParseErrorReporter>,
    /// The currently active namespaces.
    pub namespaces: Option<&'a Namespaces>,
    /// The use counters we want to record while parsing style rules, if any.
    pub use_counters: Option<&'a UseCounters>,
}

impl<'a> ParserContext<'a> {
    /// Create a parser context.
    #[inline]
    pub fn new(
        stylesheet_origin: Origin,
        url_data: &'a UrlExtraData,
        rule_type: Option<CssRuleType>,
        parsing_mode: ParsingMode,
        quirks_mode: QuirksMode,
        error_reporter: Option<&'a ParseErrorReporter>,
        use_counters: Option<&'a UseCounters>,
    ) -> Self {
        Self {
            stylesheet_origin,
            url_data,
            rule_type,
            parsing_mode,
            quirks_mode,
            error_reporter,
            namespaces: None,
            use_counters,
        }
    }

    /// Create a parser context for on-the-fly parsing in CSSOM
    #[inline]
    pub fn new_for_cssom(
        url_data: &'a UrlExtraData,
        rule_type: Option<CssRuleType>,
        parsing_mode: ParsingMode,
        quirks_mode: QuirksMode,
        error_reporter: Option<&'a ParseErrorReporter>,
        use_counters: Option<&'a UseCounters>,
    ) -> Self {
        Self::new(
            Origin::Author,
            url_data,
            rule_type,
            parsing_mode,
            quirks_mode,
            error_reporter,
            use_counters,
        )
    }

    /// Create a parser context based on a previous context, but with a modified
    /// rule type.
    #[inline]
    pub fn new_with_rule_type(
        context: &'a ParserContext,
        rule_type: CssRuleType,
        namespaces: &'a Namespaces,
    ) -> ParserContext<'a> {
        Self {
            stylesheet_origin: context.stylesheet_origin,
            url_data: context.url_data,
            rule_type: Some(rule_type),
            parsing_mode: context.parsing_mode,
            quirks_mode: context.quirks_mode,
            namespaces: Some(namespaces),
            error_reporter: context.error_reporter,
            use_counters: context.use_counters,
        }
    }

    /// Whether we're in a @page rule.
    #[inline]
    pub fn in_page_rule(&self) -> bool {
        self.rule_type
            .map_or(false, |rule_type| rule_type == CssRuleType::Page)
    }

    /// Get the rule type, which assumes that one is available.
    pub fn rule_type(&self) -> CssRuleType {
        self.rule_type
            .expect("Rule type expected, but none was found.")
    }

    /// Returns whether CSS error reporting is enabled.
    #[inline]
    pub fn error_reporting_enabled(&self) -> bool {
        self.error_reporter.is_some()
    }

    /// Record a CSS parse error with this context’s error reporting.
    pub fn log_css_error(&self, location: SourceLocation, error: ContextualParseError) {
        let error_reporter = match self.error_reporter {
            Some(r) => r,
            None => return,
        };

        error_reporter.report_error(self.url_data, location, error)
    }

    /// Whether we're in a user-agent stylesheet.
    #[inline]
    pub fn in_ua_sheet(&self) -> bool {
        self.stylesheet_origin == Origin::UserAgent
    }

    /// Returns whether chrome-only rules should be parsed.
    #[inline]
    pub fn chrome_rules_enabled(&self) -> bool {
        self.url_data.is_chrome() || self.stylesheet_origin == Origin::User
    }

    /// Whether we're in a user-agent stylesheet or chrome rules are enabled.
    #[inline]
    pub fn in_ua_or_chrome_sheet(&self) -> bool {
        self.in_ua_sheet() || self.chrome_rules_enabled()
    }
}

/// A trait to abstract parsing of a specified value given a `ParserContext` and
/// CSS input.
///
/// This can be derived on keywords with `#[derive(Parse)]`.
///
/// The derive code understands the following attributes on each of the variants:
///
///  * `#[parse(aliases = "foo,bar")]` can be used to alias a value with another
///    at parse-time.
///
///  * `#[parse(condition = "function")]` can be used to make the parsing of the
///    value conditional on `function`, which needs to fulfill
///    `fn(&ParserContext) -> bool`.
pub trait Parse: Sized {
    /// Parse a value of this type.
    ///
    /// Returns an error on failure.
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>>;
}

impl<T> Parse for Vec<T>
where
    T: Parse + OneOrMoreSeparated,
    <T as OneOrMoreSeparated>::S: Separator,
{
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        <T as OneOrMoreSeparated>::S::parse(input, |i| T::parse(context, i))
    }
}

impl Parse for UnicodeRange {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        UnicodeRange::parse(input).map_err(|e| e.into())
    }
}
