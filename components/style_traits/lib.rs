/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "style_traits"]
#![crate_type = "rlib"]
#![feature(core_intrinsics)]
#![feature(custom_derive)]
#![feature(custom_attribute)]
#![feature(plugin)]
#![plugin(plugins)]
#![plugin(serde_macros)]

extern crate euclid;
extern crate num;
extern crate util;
extern crate rustc_serialize;
extern crate selectors;
extern crate serde;
extern crate url;

#[macro_use]
extern crate cssparser;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use cssparser::{ToCss, Parser, Token, DeclarationParser,
                DeclarationListParser, AtRuleParser, parse_important, SourcePosition};
use euclid::size::{Size2D, TypedSize2D};
use euclid::scale_factor::ScaleFactor;
use selectors::parser::ParserContext as SelectorParserContext;
use std::{cmp, fmt};
use std::ops::Mul;
use std::ascii::AsciiExt;
use std::collections::hash_map::{Entry, HashMap};
use std::intrinsics;
use url::{Url, UrlParser};
use util::geometry::{Au, PagePx, ViewportPx};

#[macro_export]
macro_rules! define_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident ),+,) => {
        define_css_keyword_enum!($name: $( $css => $variant ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident ),+) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug, HeapSizeOf)]
        #[derive(Deserialize, Serialize)]
        pub enum $name {
            $( $variant ),+
        }

        impl $name {
            pub fn parse(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                                           $( $css => Ok($name::$variant) ),+
                                           _ => Err(())
                }
            }
        }

        impl ::cssparser::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                where W: ::std::fmt::Write {
                    match self {
                        $( &$name::$variant => dest.write_str($css) ),+
                    }
                }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewportConstraints {
    pub size: TypedSize2D<ViewportPx, f32>,

    pub initial_zoom: ScaleFactor<PagePx, ViewportPx, f32>,
    pub min_zoom: Option<ScaleFactor<PagePx, ViewportPx, f32>>,
    pub max_zoom: Option<ScaleFactor<PagePx, ViewportPx, f32>>,

    pub user_zoom: UserZoom,
    pub orientation: Orientation
}

impl ToCss for ViewportConstraints {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        try!(write!(dest, "@viewport {{"));
        try!(write!(dest, " width: {}px;", self.size.width.get()));
        try!(write!(dest, " height: {}px;", self.size.height.get()));
        try!(write!(dest, " zoom: {};", self.initial_zoom.get()));
        if let Some(min_zoom) = self.min_zoom {
            try!(write!(dest, " min-zoom: {};", min_zoom.get()));
        }
        if let Some(max_zoom) = self.max_zoom {
            try!(write!(dest, " max-zoom: {};", max_zoom.get()));
        }
        try!(write!(dest, " user-zoom: ")); try!(self.user_zoom.to_css(dest));
        try!(write!(dest, "; orientation: ")); try!(self.orientation.to_css(dest));
        write!(dest, "; }}")
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ViewportDescriptor {
    MinWidth(LengthOrPercentageOrAuto),
    MaxWidth(LengthOrPercentageOrAuto),

    MinHeight(LengthOrPercentageOrAuto),
    MaxHeight(LengthOrPercentageOrAuto),

    Zoom(Zoom),
    MinZoom(Zoom),
    MaxZoom(Zoom),

    UserZoom(UserZoom),
    Orientation(Orientation)
}

/// Zoom is a number | percentage | auto
/// See http://dev.w3.org/csswg/css-device-adapt/#descdef-viewport-zoom
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Zoom {
    Number(f32),
    Percentage(f32),
    Auto,
}

impl ToCss for Zoom {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match self {
            &Zoom::Number(number) => write!(dest, "{}", number),
            &Zoom::Percentage(percentage) => write!(dest, "{}%", percentage * 100.),
            &Zoom::Auto => write!(dest, "auto")
        }
    }
}

impl Zoom {
    pub fn parse(input: &mut Parser) -> Result<Zoom, ()> {
        use cssparser::Token;

        match try!(input.next()) {
            Token::Percentage(ref value) if AllowedNumericType::NonNegative.is_ok(value.unit_value) =>
                Ok(Zoom::Percentage(value.unit_value)),
            Token::Number(ref value) if AllowedNumericType::NonNegative.is_ok(value.value) =>
                Ok(Zoom::Number(value.value)),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(Zoom::Auto),
            _ => Err(())
        }
    }

    #[inline]
    pub fn to_f32(&self) -> Option<f32> {
        match self {
            &Zoom::Number(number) => Some(number as f32),
            &Zoom::Percentage(percentage) => Some(percentage as f32),
            &Zoom::Auto => None
        }
    }
}

define_css_keyword_enum!(UserZoom:
                         "zoom" => Zoom,
                         "fixed" => Fixed);

define_css_keyword_enum!(Orientation:
                         "auto" => Auto,
                         "portrait" => Portrait,
                         "landscape" => Landscape);

#[derive(Debug, PartialEq)]
pub struct ViewportRule {
    pub declarations: Vec<ViewportDescriptorDeclaration>
}

impl ViewportRule {
    pub fn parse<'a>(input: &mut Parser, context: &'a ParserContext)
                     -> Result<ViewportRule, ()>
    {
        let parser = ViewportRuleParser { context: context };

        let mut errors = vec![];
        let valid_declarations = DeclarationListParser::new(input, parser)
            .filter_map(|result| {
                match result {
                    Ok(declarations) => Some(declarations),
                    Err(range) => {
                        errors.push(range);
                        None
                    }
                }
            })
            .flat_map(|declarations| declarations.into_iter())
            .collect::<Vec<_>>();

        for range in errors {
            let pos = range.start;
            let message = format!("Unsupported @viewport descriptor declaration: '{}'",
                                  input.slice(range));
            log_css_error(input, pos, &*message);
        }

        Ok(ViewportRule { declarations: valid_declarations.iter().cascade() })
    }
}

struct ViewportRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ViewportDescriptorDeclaration {
    pub origin: Origin,
    pub descriptor: ViewportDescriptor,
    pub important: bool
}

impl ViewportDescriptorDeclaration {
    pub fn new(origin: Origin,
               descriptor: ViewportDescriptor,
               important: bool) -> ViewportDescriptorDeclaration
    {
        ViewportDescriptorDeclaration {
            origin: origin,
            descriptor: descriptor,
            important: important
        }
    }
    fn higher_or_equal_precendence(&self, other: &ViewportDescriptorDeclaration) -> bool {
        let self_precedence = cascade_precendence(self.origin, self.important);
        let other_precedence = cascade_precendence(other.origin, other.important);

        self_precedence <= other_precedence
    }
}

/// Each style rule has an origin, which determines where it enters the cascade.
///
/// http://dev.w3.org/csswg/css-cascade/#cascading-origins
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Origin {
    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-ua
    UserAgent,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-author
    Author,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-user
    User,
}

/// Computes the cascade precedence as according to
/// http://dev.w3.org/csswg/css-cascade/#cascade-origin
fn cascade_precendence(origin: Origin, important: bool) -> u8 {
    match (origin, important) {
        (Origin::UserAgent, true) => 1,
        (Origin::User, true) => 2,
        (Origin::Author, true) => 3,
        (Origin::Author, false) => 4,
        (Origin::User, false) => 5,
        (Origin::UserAgent, false) => 6,
    }
}

pub type CSSFloat = f32;

#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
pub enum LengthOrPercentageOrAuto {
    Length(Length),
    Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
    Auto,
}

impl ToCss for LengthOrPercentageOrAuto {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &LengthOrPercentageOrAuto::Length(length) => length.to_css(dest),
            &LengthOrPercentageOrAuto::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
            &LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
        }
    }
}

impl LengthOrPercentageOrAuto {
    fn parse_internal(input: &mut Parser, context: &AllowedNumericType)
                      -> Result<LengthOrPercentageOrAuto, ()>
    {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit).map(LengthOrPercentageOrAuto::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentageOrAuto::Percentage(value.unit_value)),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentageOrAuto::Length(Length::Absolute(Au(0)))),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(LengthOrPercentageOrAuto::Auto),
            _ => Err(())
        }
    }
    #[inline]
    pub fn parse(input: &mut Parser) -> Result<LengthOrPercentageOrAuto, ()> {
        LengthOrPercentageOrAuto::parse_internal(input, &AllowedNumericType::All)
    }
    #[inline]
    pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentageOrAuto, ()> {
        LengthOrPercentageOrAuto::parse_internal(input, &AllowedNumericType::NonNegative)
    }
}

#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
pub enum LengthOrPercentage {
    Length(Length),
    Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
}

impl ToCss for LengthOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &LengthOrPercentage::Length(length) => length.to_css(dest),
            &LengthOrPercentage::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
        }
    }
}
impl LengthOrPercentage {
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(Length::Absolute(Au(0)))
    }

    fn parse_internal(input: &mut Parser, context: &AllowedNumericType)
                      -> Result<LengthOrPercentage, ()>
    {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit).map(LengthOrPercentage::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentage::Percentage(value.unit_value)),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentage::Length(Length::Absolute(Au(0)))),
            _ => Err(())
        }
    }
    #[allow(dead_code)]
    #[inline]
    pub fn parse(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
        LengthOrPercentage::parse_internal(input, &AllowedNumericType::All)
    }
    #[inline]
    pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
        LengthOrPercentage::parse_internal(input, &AllowedNumericType::NonNegative)
    }
}

#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
pub enum Length {
    Absolute(Au),  // application units
    FontRelative(FontRelativeLength),
    ViewportPercentage(ViewportPercentageLength),

    /// HTML5 "character width", as defined in HTML5 § 14.5.4.
    ///
    /// This cannot be specified by the user directly and is only generated by
    /// `Stylist::synthesize_rules_for_legacy_attributes()`.
    ServoCharacterWidth(CharacterWidth),
}

impl ToCss for Length {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &Length::Absolute(length) => write!(dest, "{}px", length.to_f32_px()),
            &Length::FontRelative(length) => length.to_css(dest),
            &Length::ViewportPercentage(length) => length.to_css(dest),
            &Length::ServoCharacterWidth(_)
                => panic!("internal CSS values should never be serialized"),
        }
    }
}

impl Mul<CSSFloat> for Length {
    type Output = Length;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> Length {
        match self {
            Length::Absolute(Au(v)) => Length::Absolute(Au(((v as f32) * scalar) as i32)),
            Length::FontRelative(v) => Length::FontRelative(v * scalar),
            Length::ViewportPercentage(v) => Length::ViewportPercentage(v * scalar),
            Length::ServoCharacterWidth(_) => panic!("Can't multiply ServoCharacterWidth!"),
        }
    }
}

const AU_PER_PX: CSSFloat = 60.;
const AU_PER_IN: CSSFloat = AU_PER_PX * 96.;
const AU_PER_CM: CSSFloat = AU_PER_IN / 2.54;
const AU_PER_MM: CSSFloat = AU_PER_IN / 25.4;
const AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
const AU_PER_PC: CSSFloat = AU_PER_PT * 12.;
impl Length {
    #[inline]
    fn parse_internal(input: &mut Parser, context: &AllowedNumericType) -> Result<Length, ()> {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit),
            Token::Number(ref value) if value.value == 0. =>
                Ok(Length::Absolute(Au(0))),
            _ => Err(())
        }
    }
    #[allow(dead_code)]
    pub fn parse(input: &mut Parser) -> Result<Length, ()> {
        Length::parse_internal(input, &AllowedNumericType::All)
    }
    pub fn parse_non_negative(input: &mut Parser) -> Result<Length, ()> {
        Length::parse_internal(input, &AllowedNumericType::NonNegative)
    }
    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Length, ()> {
        match_ignore_ascii_case! { unit,
                                   "px" => Ok(Length::from_px(value)),
                                   "in" => Ok(Length::Absolute(Au((value * AU_PER_IN) as i32))),
                                   "cm" => Ok(Length::Absolute(Au((value * AU_PER_CM) as i32))),
                                   "mm" => Ok(Length::Absolute(Au((value * AU_PER_MM) as i32))),
                                   "pt" => Ok(Length::Absolute(Au((value * AU_PER_PT) as i32))),
                                   "pc" => Ok(Length::Absolute(Au((value * AU_PER_PC) as i32))),
                                   // font-relative
                                   "em" => Ok(Length::FontRelative(FontRelativeLength::Em(value))),
                                   "ex" => Ok(Length::FontRelative(FontRelativeLength::Ex(value))),
                                   "ch" => Ok(Length::FontRelative(FontRelativeLength::Ch(value))),
                                   "rem" => Ok(Length::FontRelative(FontRelativeLength::Rem(value))),
                                   // viewport percentages
                                   "vw" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vw(value))),
                                   "vh" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vh(value))),
                                   "vmin" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vmin(value))),
                                   "vmax" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vmax(value)))
                                   _ => Err(())
        }
    }
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> Length {
        Length::Absolute(Au((px_value * AU_PER_PX) as i32))
    }
}


#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
pub enum FontRelativeLength {
    Em(CSSFloat),
    Ex(CSSFloat),
    Ch(CSSFloat),
    Rem(CSSFloat)
}

impl ToCss for FontRelativeLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &FontRelativeLength::Em(length) => write!(dest, "{}em", length),
            &FontRelativeLength::Ex(length) => write!(dest, "{}ex", length),
            &FontRelativeLength::Ch(length) => write!(dest, "{}ch", length),
            &FontRelativeLength::Rem(length) => write!(dest, "{}rem", length)
        }
    }
}

impl Mul<CSSFloat> for FontRelativeLength {
    type Output = FontRelativeLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> FontRelativeLength {
        match self {
            FontRelativeLength::Em(v) => FontRelativeLength::Em(v * scalar),
            FontRelativeLength::Ex(v) => FontRelativeLength::Ex(v * scalar),
            FontRelativeLength::Ch(v) => FontRelativeLength::Ch(v * scalar),
            FontRelativeLength::Rem(v) => FontRelativeLength::Rem(v * scalar),
        }
    }
}

impl FontRelativeLength {
    pub fn to_computed_value(&self,
                             reference_font_size: Au,
                             root_font_size: Au)
                             -> Au
    {
        match self {
            &FontRelativeLength::Em(length) => reference_font_size.scale_by(length),
            &FontRelativeLength::Ex(length) | &FontRelativeLength::Ch(length) => {
                // https://github.com/servo/servo/issues/7462
                let em_factor = 0.5;
                reference_font_size.scale_by(length * em_factor)
            },
            &FontRelativeLength::Rem(length) => root_font_size.scale_by(length)
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
pub enum ViewportPercentageLength {
    Vw(CSSFloat),
    Vh(CSSFloat),
    Vmin(CSSFloat),
    Vmax(CSSFloat)
}

impl ToCss for ViewportPercentageLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &ViewportPercentageLength::Vw(length) => write!(dest, "{}vw", length),
            &ViewportPercentageLength::Vh(length) => write!(dest, "{}vh", length),
            &ViewportPercentageLength::Vmin(length) => write!(dest, "{}vmin", length),
            &ViewportPercentageLength::Vmax(length) => write!(dest, "{}vmax", length)
        }
    }
}
impl Mul<CSSFloat> for ViewportPercentageLength {
    type Output = ViewportPercentageLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> ViewportPercentageLength {
        match self {
            ViewportPercentageLength::Vw(v) => ViewportPercentageLength::Vw(v * scalar),
            ViewportPercentageLength::Vh(v) => ViewportPercentageLength::Vh(v * scalar),
            ViewportPercentageLength::Vmin(v) => ViewportPercentageLength::Vmin(v * scalar),
            ViewportPercentageLength::Vmax(v) => ViewportPercentageLength::Vmax(v * scalar),
        }
    }
}

impl ViewportPercentageLength {
    pub fn to_computed_value(&self, viewport_size: Size2D<Au>) -> Au {
        macro_rules! to_unit {
            ($viewport_dimension:expr) => {
                $viewport_dimension.to_f32_px() / 100.0
            }
        }

        let value = match self {
            &ViewportPercentageLength::Vw(length) =>
                length * to_unit!(viewport_size.width),
            &ViewportPercentageLength::Vh(length) =>
                length * to_unit!(viewport_size.height),
            &ViewportPercentageLength::Vmin(length) =>
                length * to_unit!(cmp::min(viewport_size.width, viewport_size.height)),
            &ViewportPercentageLength::Vmax(length) =>
                length * to_unit!(cmp::max(viewport_size.width, viewport_size.height)),
        };
        Au::from_f32_px(value)
    }
}

#[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
pub struct CharacterWidth(pub i32);

impl CharacterWidth {
    pub fn to_computed_value(&self, reference_font_size: Au) -> Au {
        // This applies the *converting a character width to pixels* algorithm as specified
        // in HTML5 § 14.5.4.
        //
        // TODO(pcwalton): Find these from the font.
        let average_advance = reference_font_size.scale_by(0.5);
        let max_advance = reference_font_size;
        average_advance.scale_by(self.0 as CSSFloat - 1.0) + max_advance
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllowedNumericType {
    All,
    NonNegative
}

impl AllowedNumericType {
    #[inline]
    pub fn is_ok(&self, value: f32) -> bool {
        match self {
            &AllowedNumericType::All => true,
            &AllowedNumericType::NonNegative => value >= 0.,
        }
    }
}


pub struct ParserContext<'a> {
    pub stylesheet_origin: Origin,
    pub base_url: &'a Url,
    pub selector_context: SelectorParserContext,
}

impl<'a> ParserContext<'a> {
    pub fn new(stylesheet_origin: Origin, base_url: &'a Url) -> ParserContext<'a> {
        let mut selector_context = SelectorParserContext::new();
        selector_context.in_user_agent_stylesheet = stylesheet_origin == Origin::UserAgent;
        ParserContext {
            stylesheet_origin: stylesheet_origin,
            base_url: base_url,
            selector_context: selector_context,
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

fn parse_shorthand(input: &mut Parser) -> Result<[LengthOrPercentageOrAuto; 2], ()> {
    let min = try!(LengthOrPercentageOrAuto::parse_non_negative(input));
    match input.try(|input| LengthOrPercentageOrAuto::parse_non_negative(input)) {
        Err(()) => Ok([min.clone(), min]),
        Ok(max) => Ok([min, max])
    }
}

impl<'a, 'b> AtRuleParser for ViewportRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = Vec<ViewportDescriptorDeclaration>;
}

impl<'a, 'b> DeclarationParser for ViewportRuleParser<'a, 'b> {
    type Declaration = Vec<ViewportDescriptorDeclaration>;

    fn parse_value(&self, name: &str, input: &mut Parser) -> Result<Vec<ViewportDescriptorDeclaration>, ()> {
        macro_rules! declaration {
            ($declaration:ident($parse:path)) => {
                declaration!($declaration(value: try!($parse(input)),
                                          important: input.try(parse_important).is_ok()))
            };
            ($declaration:ident(value: $value:expr, important: $important:expr)) => {
                ViewportDescriptorDeclaration::new(
                    self.context.stylesheet_origin,
                    ViewportDescriptor::$declaration($value),
                    $important)
            }
        }

        macro_rules! ok {
            ($declaration:ident($parse:path)) => {
                Ok(vec![declaration!($declaration($parse))])
            };
            (shorthand -> [$min:ident, $max:ident]) => {{
                let shorthand = try!(parse_shorthand(input));
                let important = input.try(parse_important).is_ok();

                Ok(vec![declaration!($min(value: shorthand[0], important: important)),
                        declaration!($max(value: shorthand[1], important: important))])
            }}
        }

        match name {
            n if n.eq_ignore_ascii_case("min-width") =>
                ok!(MinWidth(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("max-width") =>
                ok!(MaxWidth(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("width") =>
                ok!(shorthand -> [MinWidth, MaxWidth]),

            n if n.eq_ignore_ascii_case("min-height") =>
                ok!(MinHeight(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("max-height") =>
                ok!(MaxHeight(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("height") =>
                ok!(shorthand -> [MinHeight, MaxHeight]),

            n if n.eq_ignore_ascii_case("zoom") =>
                ok!(Zoom(Zoom::parse)),
            n if n.eq_ignore_ascii_case("min-zoom") =>
                ok!(MinZoom(Zoom::parse)),
            n if n.eq_ignore_ascii_case("max-zoom") =>
                ok!(MaxZoom(Zoom::parse)),

            n if n.eq_ignore_ascii_case("user-zoom") =>
                ok!(UserZoom(UserZoom::parse)),
            n if n.eq_ignore_ascii_case("orientation") =>
                ok!(Orientation(Orientation::parse)),

            _ => Err(()),
        }
    }
}

pub trait ViewportRuleCascade: Iterator + Sized {
    fn cascade(self) -> ViewportRule;
}

impl<'a, I> ViewportRuleCascade for I
    where I: Iterator<Item=&'a ViewportRule>
{
    #[inline]
    fn cascade(self) -> ViewportRule {
        ViewportRule {
            declarations: self.flat_map(|r| r.declarations.iter()).cascade()
        }
    }
}

trait ViewportDescriptorDeclarationCascade: Iterator + Sized {
    fn cascade(self) -> Vec<ViewportDescriptorDeclaration>;
}

fn cascade<'a, I>(iter: I) -> Vec<ViewportDescriptorDeclaration>
    where I: Iterator<Item=&'a ViewportDescriptorDeclaration>
{
    let mut declarations: HashMap<u64, (usize, &'a ViewportDescriptorDeclaration)> = HashMap::new();

    // index is used to reconstruct order of appearance after all declarations
    // have been added to the map
    let mut index = 0;
    for declaration in iter {
        let descriptor = unsafe {
            intrinsics::discriminant_value(&declaration.descriptor)
        };

        match declarations.entry(descriptor) {
            Entry::Occupied(mut entry) => {
                if declaration.higher_or_equal_precendence(entry.get().1) {
                    entry.insert((index, declaration));
                    index += 1;
                }
            }
            Entry::Vacant(entry) => {
                entry.insert((index, declaration));
                index += 1;
            }
        }
    }

    // convert to a list and sort the descriptors by order of appearance
    let mut declarations: Vec<_> = declarations.into_iter().map(|kv| kv.1).collect();
    declarations.sort_by(|a, b| a.0.cmp(&b.0));
    declarations.into_iter().map(|id| *id.1).collect::<Vec<_>>()
}

impl<'a, I> ViewportDescriptorDeclarationCascade for I
    where I: Iterator<Item=&'a ViewportDescriptorDeclaration>
{
    #[inline]
    fn cascade(self) -> Vec<ViewportDescriptorDeclaration> {
        cascade(self)
    }
}
