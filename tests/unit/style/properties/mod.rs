/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::context::QuirksMode;
use style::parser::{LengthParsingMode, ParserContext};
use style::stylesheets::{CssRuleType, Origin};

fn parse<T, F: Fn(&ParserContext, &mut Parser) -> Result<T, ()>>(f: F, s: &str) -> Result<T, ()> {
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style),
                                     LengthParsingMode::Default,
                                     QuirksMode::NoQuirks);
    let mut parser = Parser::new(s);
    f(&context, &mut parser)
}

macro_rules! assert_roundtrip_with_context {
    ($fun:expr, $string:expr) => {
        assert_roundtrip_with_context!($fun, $string, $string);
    };
    ($fun:expr, $input:expr, $output:expr) => {{
        let serialized = parse(|context, i| {
            let parsed = $fun(context, i)
                         .expect(&format!("Failed to parse {}", $input));
            let serialized = ToCss::to_css_string(&parsed);
            assert_eq!(serialized, $output);
            Ok(serialized)
        }, $input).unwrap();

        parse(|context, i| {
            let re_parsed = $fun(context, i)
                            .expect(&format!("Failed to parse serialization {}", $input));
            let re_serialized = ToCss::to_css_string(&re_parsed);
            assert_eq!(serialized, re_serialized);
            Ok(())
        }, &serialized).unwrap()
    }}
}

mod background;
mod scaffolding;
mod serialization;
mod viewport;
