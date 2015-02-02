/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use std::collections::HashMap;
use string_cache::Namespace;
use cssparser::{Parser, SourcePosition};
use url::{Url, UrlParser};
use log;

use stylesheets::Origin;


pub struct NamespaceMap {
    pub default: Option<Namespace>,
    pub prefix_map: HashMap<String, Namespace>,
}


pub struct ParserContext<'a> {
    pub stylesheet_origin: Origin,
    pub base_url: &'a Url,
    pub namespaces: NamespaceMap,
}

impl<'a> ParserContext<'a> {
    pub fn new(stylesheet_origin: Origin, base_url: &'a Url) -> ParserContext<'a> {
        ParserContext {
            stylesheet_origin: stylesheet_origin,
            base_url: base_url,
            namespaces: NamespaceMap {
                default: None,
                prefix_map: HashMap::new()
            }
        }
    }
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
