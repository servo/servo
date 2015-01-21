/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use cssparser::{Parser, SourcePosition};
use url::{Url, UrlParser};
use log;

use stylesheets::Origin;
use namespaces::NamespaceMap;


pub struct ParserContext<'a> {
    pub stylesheet_origin: Origin,
    pub base_url: &'a Url,
    pub namespaces: NamespaceMap,
}


impl<'a> ParserContext<'a> {
    pub fn in_user_agent_stylesheet(&self) -> bool {
        self.stylesheet_origin == Origin::UserAgent
    }

    pub fn parse_url(&self, input: &str) -> Url {
        UrlParser::new().base_url(self.base_url).parse(input)
            .unwrap_or_else(|_| Url::parse("about:invalid").unwrap())
    }
}


/// Defaults to a no-op.
/// Set a `RUST_LOG=style::errors` environment variable
/// to log CSS parse errors to stderr.
pub fn log_css_error(input: &mut Parser, position: SourcePosition, message: &str) {
    if log_enabled!(log::INFO) {
        let location = input.source_location(position);
        // TODO eventually this will got into a "web console" or something.
        info!("{}:{} {}", location.line, location.column, message)
    }
}
