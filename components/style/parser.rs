/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which CSS code is parsed.

use cssparser::{Parser, SourcePosition};
use error_reporting::ParseErrorReporter;
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::refptr::{GeckoArcPrincipal, GeckoArcURI};
use servo_url::ServoUrl;
use stylesheets::{MemoryHoleReporter, Origin};

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
    pub base_url: &'a ServoUrl,
    pub error_reporter: Box<ParseErrorReporter + Send>,
    pub extra_data: ParserContextExtraData,
}

impl<'a> ParserContext<'a> {
    pub fn new_with_extra_data(stylesheet_origin: Origin, base_url: &'a ServoUrl,
                               error_reporter: Box<ParseErrorReporter + Send>,
                               extra_data: ParserContextExtraData)
                               -> ParserContext<'a> {
        ParserContext {
            stylesheet_origin: stylesheet_origin,
            base_url: base_url,
            error_reporter: error_reporter,
            extra_data: extra_data,
        }
    }

    pub fn new(stylesheet_origin: Origin, base_url: &'a ServoUrl, error_reporter: Box<ParseErrorReporter + Send>)
               -> ParserContext<'a> {
        let extra_data = ParserContextExtraData::default();
        ParserContext::new_with_extra_data(stylesheet_origin, base_url, error_reporter, extra_data)
    }

    pub fn new_for_cssom(base_url: &'a ServoUrl) -> ParserContext<'a> {
        Self::new(Origin::User, base_url, Box::new(MemoryHoleReporter))
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
pub trait Parse {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> where Self: Sized;
}
