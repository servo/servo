/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tests for parsing and serialization of values/properties

use cssparser::Parser;
use euclid::size::TypedSize2D;
use media_queries::CSSErrorReporterTest;
use style::context::QuirksMode;
use style::font_metrics::ServoMetricsProvider;
use style::media_queries::{Device, MediaType};
use style::parser::{PARSING_MODE_DEFAULT, ParserContext};
use style::properties::{ComputedValues, StyleBuilder};
use style::stylesheets::{CssRuleType, Origin};
use style::values::computed::{Context, ToComputedValue};
use style_traits::ToCss;

fn parse<T, F: Fn(&ParserContext, &mut Parser) -> Result<T, ()>>(f: F, s: &str) -> Result<T, ()> {
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style),
                                     PARSING_MODE_DEFAULT,
                                     QuirksMode::NoQuirks);
    let mut parser = Parser::new(s);
    f(&context, &mut parser)
}

fn parse_entirely<T, F: Fn(&ParserContext, &mut Parser) -> Result<T, ()>>(f: F, s: &str) -> Result<T, ()> {
    parse(|context, parser| parser.parse_entirely(|p| f(context, p)), s)
}

fn assert_computed_serialization<C, F, T>(f: F, input: &str, output: &str)
    where F: Fn(&ParserContext, &mut Parser) -> Result<T, ()>,
          T: ToComputedValue<ComputedValue=C>, C: ToCss
{
    let viewport_size = TypedSize2D::new(0., 0.);
    let initial_style = ComputedValues::initial_values();
    let device = Device::new(MediaType::Screen, viewport_size);

    let context = Context {
        is_root_element: true,
        device: &device,
        inherited_style: initial_style,
        layout_parent_style: initial_style,
        style: StyleBuilder::for_derived_style(&initial_style),
        cached_system_font: None,
        font_metrics_provider: &ServoMetricsProvider,
        in_media_query: false,
        quirks_mode: QuirksMode::NoQuirks,
    };

    let parsed = parse(f, input).unwrap();
    let computed = parsed.to_computed_value(&context);
    let serialized = ToCss::to_css_string(&computed);
    assert_eq!(serialized, output);
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
mod transition_duration;
mod transition_property;
mod transition_timing_function;
mod ui;
mod value;
