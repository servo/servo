/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tests for parsing and serialization of values/properties

use cssparser::Parser;
use style::context::QuirksMode;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin};
use style_traits::{ParseError, ParsingMode};
use url::Url;

fn parse<'a, T, F>(f: F, s: &'a str) -> Result<T, ParseError>
where
    F: for<'t> Fn(&ParserContext, &mut Parser<'a>) -> Result<T, ParseError>,
{
    let url_data = Url::parse("http://localhost").unwrap().into();
    let context = ParserContext::new(
        Origin::Author,
        &url_data,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        /* namespaces = */ Default::default(),
        None,
        None,
        /* attr_taint = */ Default::default(),
    );
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
        let serialized = super::parse(
            |context, i| {
                let parsed = $fun(context, i).expect(&format!("Failed to parse {}", $input));
                let serialized = ToCss::to_css_string(&parsed);
                assert_eq!(serialized, $output);
                Ok(serialized)
            },
            $input,
        )
        .unwrap();

        let unwrapped = super::parse(
            |context, i| {
                let re_parsed =
                    $fun(context, i).expect(&format!("Failed to parse serialization {}", $input));
                let re_serialized = ToCss::to_css_string(&re_parsed);
                assert_eq!(serialized, re_serialized);
                Ok(())
            },
            &serialized,
        )
        .unwrap();
        unwrapped
    }};
}

macro_rules! assert_roundtrip {
    ($fun:expr, $string:expr) => {
        assert_roundtrip!($fun, $string, $string);
    };
    ($fun:expr, $input:expr, $output:expr) => {
        let mut parser = Parser::new($input);
        let parsed = $fun(&mut parser).expect(&format!("Failed to parse {}", $input));
        let serialized = ToCss::to_css_string(&parsed);
        assert_eq!(serialized, $output);

        let mut parser = Parser::new(&serialized);
        let re_parsed =
            $fun(&mut parser).expect(&format!("Failed to parse serialization {}", $input));
        let re_serialized = ToCss::to_css_string(&re_parsed);
        assert_eq!(serialized, re_serialized)
    };
}

macro_rules! assert_parser_exhausted {
    ($fun:expr, $string:expr, $should_exhausted:expr) => {{
        parse(
            |context, input| {
                let parsed = $fun(context, input);
                assert_eq!(parsed.is_ok(), true);
                assert_eq!(input.is_exhausted(), $should_exhausted);
                Ok(())
            },
            $string,
        )
        .unwrap()
    }};
}

macro_rules! parse_longhand {
    ($name:ident, $s:expr) => {
        parse($name::parse, $s).unwrap()
    };
}

mod background;
mod border;
mod box_;
mod column;
mod effects;
mod image;
mod inherited_text;
mod outline;
mod selectors;
mod supports;
mod text_overflow;
mod transition_duration;
mod transition_timing_function;
