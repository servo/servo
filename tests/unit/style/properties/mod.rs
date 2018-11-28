/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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

macro_rules! assert_roundtrip_with_context {
    ($fun:expr, $string:expr) => {
        assert_roundtrip_with_context!($fun, $string, $string);
    };
    ($fun:expr, $input:expr, $output:expr) => {{
        let serialized = parse(
            |context, i| {
                let parsed = $fun(context, i).expect(&format!("Failed to parse {}", $input));
                let serialized = ToCss::to_css_string(&parsed);
                assert_eq!(serialized, $output);
                Ok(serialized)
            },
            $input,
        )
        .unwrap();

        let mut input = ::cssparser::ParserInput::new(&serialized);
        let unwrapped = parse_input(
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

mod scaffolding;
mod serialization;
