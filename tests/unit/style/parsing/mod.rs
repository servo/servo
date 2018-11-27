/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tests for parsing and serialization of values/properties

use cssparser::{Parser, ParserInput};
use style::context::QuirksMode;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin};
use style_traits::{ParseError, ParsingMode};

fn parse<T, F>(f: F, s: &'static str) -> Result<T, ParseError<'static>>
where
    F: for<'t> Fn(&ParserContext, &mut Parser<'static, 't>) -> Result<T, ParseError<'static>>,
{
    let mut input = ParserInput::new(s);
    parse_input(f, &mut input)
}

fn parse_input<'i: 't, 't, T, F>(f: F, input: &'t mut ParserInput<'i>) -> Result<T, ParseError<'i>>
where
    F: Fn(&ParserContext, &mut Parser<'i, 't>) -> Result<T, ParseError<'i>>,
{
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(
        Origin::Author,
        &url,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        None,
        None,
    );
    let mut parser = Parser::new(input);
    f(&context, &mut parser)
}

fn parse_entirely<T, F>(f: F, s: &'static str) -> Result<T, ParseError<'static>>
where
    F: for<'t> Fn(&ParserContext, &mut Parser<'static, 't>) -> Result<T, ParseError<'static>>,
{
    let mut input = ParserInput::new(s);
    parse_entirely_input(f, &mut input)
}

fn parse_entirely_input<'i: 't, 't, T, F>(
    f: F,
    input: &'t mut ParserInput<'i>,
) -> Result<T, ParseError<'i>>
where
    F: Fn(&ParserContext, &mut Parser<'i, 't>) -> Result<T, ParseError<'i>>,
{
    parse_input(
        |context, parser| parser.parse_entirely(|p| f(context, p)),
        input,
    )
}

// This is a macro so that the file/line information
// is preserved in the panic
macro_rules! assert_roundtrip_with_context {
    ($fun:expr, $string:expr) => {
        assert_roundtrip_with_context!($fun, $string, $string);
    };
    ($fun:expr, $input:expr, $output:expr) => {{
        let mut input = ::cssparser::ParserInput::new($input);
        let serialized = super::parse_input(
            |context, i| {
                let parsed = $fun(context, i).expect(&format!("Failed to parse {}", $input));
                let serialized = ToCss::to_css_string(&parsed);
                assert_eq!(serialized, $output);
                Ok(serialized)
            },
            &mut input,
        )
        .unwrap();

        let mut input = ::cssparser::ParserInput::new(&serialized);
        let unwrapped = super::parse_input(
            |context, i| {
                let re_parsed =
                    $fun(context, i).expect(&format!("Failed to parse serialization {}", $input));
                let re_serialized = ToCss::to_css_string(&re_parsed);
                assert_eq!(serialized, re_serialized);
                Ok(())
            },
            &mut input,
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
        let mut input = ParserInput::new($input);
        let mut parser = Parser::new(&mut input);
        let parsed = $fun(&mut parser).expect(&format!("Failed to parse {}", $input));
        let serialized = ToCss::to_css_string(&parsed);
        assert_eq!(serialized, $output);

        let mut input = ParserInput::new(&serialized);
        let mut parser = Parser::new(&mut input);
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

mod animation;
mod background;
mod border;
mod box_;
mod column;
mod effects;
mod image;
mod inherited_text;
mod outline;
mod position;
mod selectors;
mod supports;
mod text_overflow;
mod transition_duration;
mod transition_timing_function;
