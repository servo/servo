/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tests for parsing and serialization of values/properties

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::{LengthParsingMode, ParserContext};
use style::stylesheets::{CssRuleType, Origin};

fn parse<T, F: Fn(&ParserContext, &mut Parser) -> Result<T, ()>>(f: F, s: &str) -> Result<T, ()> {
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style),
                                     LengthParsingMode::Default);
    let mut parser = Parser::new(s);
    f(&context, &mut parser)
}

// This is a macro so that the file/line information
// is preserved in the panic
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

macro_rules! assert_roundtrip {
    ($fun:expr, $string:expr) => {
        assert_roundtrip!($fun, $string, $string);
    };
    ($fun:expr, $input:expr, $output:expr) => {
        let mut parser = Parser::new($input);
        let parsed = $fun(&mut parser)
                     .expect(&format!("Failed to parse {}", $input));
        let serialized = ToCss::to_css_string(&parsed);
        assert_eq!(serialized, $output);

        let mut parser = Parser::new(&serialized);
        let re_parsed = $fun(&mut parser)
                        .expect(&format!("Failed to parse serialization {}", $input));
        let re_serialized = ToCss::to_css_string(&re_parsed);
        assert_eq!(serialized, re_serialized)
    }
}

macro_rules! assert_parser_exhausted {
    ($fun:expr, $string:expr, $should_exhausted:expr) => {{
        parse(|context, input| {
            let parsed = $fun(context, input);
            assert_eq!(parsed.is_ok(), true);
            assert_eq!(input.is_exhausted(), $should_exhausted);
            Ok(())
        }, $string).unwrap()
    }}
}

macro_rules! parse_longhand {
    ($name:ident, $s:expr) => {
        parse($name::parse, $s).unwrap()
    };
}

mod animation;
mod background;
mod basic_shape;
mod border;
mod box_;
mod column;
mod containment;
mod effects;
mod font;
mod image;
mod inherited_box;
mod inherited_text;
mod length;
mod mask;
mod outline;
mod position;
mod selectors;
mod supports;
mod text;
mod text_overflow;
mod transition_timing_function;
mod ui;
mod value;
