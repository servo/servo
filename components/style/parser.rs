/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use cssparser::{Parser, SourcePosition};
use log;
use selectors::parser::ParserContext as SelectorParserContext;
use style_traits::ParseErrorReporter;
use stylesheets::Origin;
use url::{Url, UrlParser};

pub struct ParserContext<'a> {
    pub stylesheet_origin: Origin,
    pub base_url: &'a Url,
    pub selector_context: SelectorParserContext,
    pub error_reporter: Box<ParseErrorReporter + Send>
}

impl<'a> ParserContext<'a> {
     pub fn new(stylesheet_origin: Origin, base_url: &'a Url, error_reporter: Box<ParseErrorReporter + Send>)
     -> ParserContext<'a> {
         let mut selector_context = SelectorParserContext::new();
         selector_context.in_user_agent_stylesheet = stylesheet_origin == Origin::UserAgent;
         ParserContext {
             stylesheet_origin: stylesheet_origin,
             base_url: base_url,
             selector_context: selector_context,
             error_reporter: error_reporter,
         }
     }
}


impl<'a> ParserContext<'a> {
    pub fn parse_url(&self, input: &str) -> Url {
        UrlParser::new().base_url(self.base_url).parse(input)
            .unwrap_or_else(|_| Url::parse("about:invalid").unwrap())
    }
}


/// Defaults to a no-op.
/// Set a `RUST_LOG=style::errors` environment variable
/// to log CSS parse errors to stderr.
pub fn log_css_error(input: &mut Parser, position: SourcePosition, message: &str) {
    if log_enabled!(log::LogLevel::Info) {
        let location = input.source_location(position);
        // TODO eventually this will got into a "web console" or something.
        info!("{}:{} {}", location.line, location.column, message)
    }
}
