/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which CSS code is parsed.

use cssparser::{Parser, SourcePosition};
use error_reporting::ParseErrorReporter;
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::refptr::{GeckoArcPrincipal, GeckoArcURI};
use selector_impl::TheSelectorImpl;
use selectors::parser::ParserContext as SelectorParserContext;
use stylesheets::Origin;
use url::Url;

#[cfg(not(feature = "gecko"))]
pub struct ParserContextExtraData;

#[cfg(feature = "gecko")]
pub struct ParserContextExtraData {
    pub base: Option<GeckoArcURI>,
    pub referrer: Option<GeckoArcURI>,
    pub principal: Option<GeckoArcPrincipal>,
}

impl ParserContextExtraData {
    #[cfg(not(feature = "gecko"))]
    pub fn default() -> ParserContextExtraData {
        ParserContextExtraData
    }

    #[cfg(feature = "gecko")]
    pub fn default() -> ParserContextExtraData {
        ParserContextExtraData { base: None, referrer: None, principal: None }
    }
}

pub struct ParserContext<'a> {
    pub stylesheet_origin: Origin,
    pub base_url: &'a Url,
    pub selector_context: SelectorParserContext<TheSelectorImpl>,
    pub error_reporter: Box<ParseErrorReporter + Send>,
    pub extra_data: ParserContextExtraData,
}

impl<'a> ParserContext<'a> {
    pub fn new_with_extra_data(stylesheet_origin: Origin, base_url: &'a Url,
                               error_reporter: Box<ParseErrorReporter + Send>,
                               extra_data: ParserContextExtraData)
                               -> ParserContext<'a> {
        let mut selector_context = SelectorParserContext::new();
        selector_context.in_user_agent_stylesheet = stylesheet_origin == Origin::UserAgent;
        ParserContext {
            stylesheet_origin: stylesheet_origin,
            base_url: base_url,
            selector_context: selector_context,
            error_reporter: error_reporter,
            extra_data: extra_data,
        }
    }

    pub fn new(stylesheet_origin: Origin, base_url: &'a Url, error_reporter: Box<ParseErrorReporter + Send>)
               -> ParserContext<'a> {
        let extra_data = ParserContextExtraData::default();
        ParserContext::new_with_extra_data(stylesheet_origin, base_url, error_reporter, extra_data)
    }
}

/// Defaults to a no-op.
/// Set a `RUST_LOG=style::errors` environment variable
/// to log CSS parse errors to stderr.
pub fn log_css_error(input: &mut Parser, position: SourcePosition, message: &str, parsercontext: &ParserContext) {
    parsercontext.error_reporter.report_error(input, position, message);
}

// XXXManishearth Replace all specified value parse impls with impls of this
// trait. This will make it easy to write more generic values in the future.
// There may need to be two traits -- one for parsing with context, and one
// for parsing without
pub trait Parse {
    fn parse(input: &mut Parser) -> Result<Self, ()> where Self: Sized;
}
