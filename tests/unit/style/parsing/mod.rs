/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tests for parsing and serialization of values/properties

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::Origin;

fn parse<T, F: Fn(&ParserContext, &mut Parser) -> Result<T, ()>>(f: F, s: &str) -> Result<T, ()> {
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new(s);
    f(&context, &mut parser)
}

// This is a macro so that the file/line information
// is preserved in the panic
macro_rules! assert_roundtrip_with_context {
    ($fun:expr, $string:expr) => {
        assert_roundtrip_with_context!($fun, $string, $string);
    };
    ($fun:expr,$input:expr, $output:expr) => {
        let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
        let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
        let mut parser = Parser::new($input);
        let parsed = $fun(&context, &mut parser)
                     .expect(&format!("Failed to parse {}", $input));
        let serialized = ToCss::to_css_string(&parsed);
        assert_eq!(serialized, $output);

        let mut parser = Parser::new(&serialized);
        let re_parsed = $fun(&context, &mut parser)
                        .expect(&format!("Failed to parse serialization {}", $input));
        let re_serialized = ToCss::to_css_string(&re_parsed);
        assert_eq!(serialized, re_serialized);
    }
}

macro_rules! parse_longhand {
    ($name:ident, $s:expr) => {{
        let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
        let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
        $name::parse(&context, &mut Parser::new($s)).unwrap()
    }};
}

mod animation;
mod background;
mod basic_shape;
mod border;
mod font;
mod image;
mod inherited_box;
mod inherited_text;
mod mask;
mod position;
mod selectors;
