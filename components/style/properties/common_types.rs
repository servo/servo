/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_camel_case_types)]

use url::{Url, UrlParser};

pub use servo_util::geometry::Au;

pub type CSSFloat = f64;

pub static DEFAULT_LINE_HEIGHT: CSSFloat = 1.14;

pub mod specified {
    use std::ascii::StrAsciiExt;
    use cssparser::ast;
    use cssparser::ast::*;
    use super::{Au, CSSFloat};
    pub use cssparser::Color as CSSColor;

    #[deriving(Clone)]
    pub enum Length {
        Au_(Au),  // application units
        Em(CSSFloat),
        Ex(CSSFloat),
        // XXX uncomment when supported:
//        Ch(CSSFloat),
//        Rem(CSSFloat),
//        Vw(CSSFloat),
//        Vh(CSSFloat),
//        Vmin(CSSFloat),
//        Vmax(CSSFloat),
    }
    static AU_PER_PX: CSSFloat = 60.;
    static AU_PER_IN: CSSFloat = AU_PER_PX * 96.;
    static AU_PER_CM: CSSFloat = AU_PER_IN / 2.54;
    static AU_PER_MM: CSSFloat = AU_PER_IN / 25.4;
    static AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
    static AU_PER_PC: CSSFloat = AU_PER_PT * 12.;
    impl Length {
        #[inline]
        fn parse_internal(input: &ComponentValue, negative_ok: bool) -> Result<Length, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()),
                &Number(ref value) if value.value == 0. =>  Ok(Au_(Au(0))),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        pub fn parse(input: &ComponentValue) -> Result<Length, ()> {
            Length::parse_internal(input, /* negative_ok = */ true)
        }
        pub fn parse_non_negative(input: &ComponentValue) -> Result<Length, ()> {
            Length::parse_internal(input, /* negative_ok = */ false)
        }
        pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Length, ()> {
            match unit.to_ascii_lower().as_slice() {
                "px" => Ok(Length::from_px(value)),
                "in" => Ok(Au_(Au((value * AU_PER_IN) as i32))),
                "cm" => Ok(Au_(Au((value * AU_PER_CM) as i32))),
                "mm" => Ok(Au_(Au((value * AU_PER_MM) as i32))),
                "pt" => Ok(Au_(Au((value * AU_PER_PT) as i32))),
                "pc" => Ok(Au_(Au((value * AU_PER_PC) as i32))),
                "em" => Ok(Em(value)),
                "ex" => Ok(Ex(value)),
                _ => Err(())
            }
        }
        #[inline]
        pub fn from_px(px_value: CSSFloat) -> Length {
            Au_(Au((px_value * AU_PER_PX) as i32))
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentage {
        LP_Length(Length),
        LP_Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
    }
    impl LengthOrPercentage {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                              -> Result<LengthOrPercentage, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LP_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Ok(LP_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. =>  Ok(LP_Length(Au_(Au(0)))),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &ComponentValue) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, /* negative_ok = */ false)
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentageOrAuto {
        LPA_Length(Length),
        LPA_Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        LPA_Auto,
    }
    impl LengthOrPercentageOrAuto {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                     -> Result<LengthOrPercentageOrAuto, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LPA_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Ok(LPA_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Ok(LPA_Length(Au_(Au(0)))),
                &Ident(ref value) if value.as_slice().eq_ignore_ascii_case("auto") => Ok(LPA_Auto),
                _ => Err(())
            }
        }
        #[inline]
        pub fn parse(input: &ComponentValue) -> Result<LengthOrPercentageOrAuto, ()> {
            LengthOrPercentageOrAuto::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Result<LengthOrPercentageOrAuto, ()> {
            LengthOrPercentageOrAuto::parse_internal(input, /* negative_ok = */ false)
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentageOrNone {
        LPN_Length(Length),
        LPN_Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        LPN_None,
    }
    impl LengthOrPercentageOrNone {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                     -> Result<LengthOrPercentageOrNone, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LPN_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Ok(LPN_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Ok(LPN_Length(Au_(Au(0)))),
                &Ident(ref value) if value.as_slice().eq_ignore_ascii_case("none") => Ok(LPN_None),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &ComponentValue) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, /* negative_ok = */ false)
        }
    }

    // http://dev.w3.org/csswg/css2/colors.html#propdef-background-position
    #[deriving(Clone)]
    pub enum PositionComponent {
        Pos_Length(Length),
        Pos_Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        Pos_Center,
        Pos_Left,
        Pos_Right,
        Pos_Top,
        Pos_Bottom,
    }
    impl PositionComponent {
        pub fn parse(input: &ComponentValue) -> Result<PositionComponent, ()> {
            match input {
                &Dimension(ref value, ref unit) =>
                    Length::parse_dimension(value.value, unit.as_slice()).map(Pos_Length),
                &ast::Percentage(ref value) => Ok(Pos_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Ok(Pos_Length(Au_(Au(0)))),
                &Ident(ref value) => {
                    if value.as_slice().eq_ignore_ascii_case("center") { Ok(Pos_Center) }
                    else if value.as_slice().eq_ignore_ascii_case("left") { Ok(Pos_Left) }
                    else if value.as_slice().eq_ignore_ascii_case("right") { Ok(Pos_Right) }
                    else if value.as_slice().eq_ignore_ascii_case("top") { Ok(Pos_Top) }
                    else if value.as_slice().eq_ignore_ascii_case("bottom") { Ok(Pos_Bottom) }
                    else { Err(()) }
                }
                _ => Err(())
            }
        }
        #[inline]
        pub fn to_length_or_percentage(self) -> LengthOrPercentage {
            match self {
                Pos_Length(x) => LP_Length(x),
                Pos_Percentage(x) => LP_Percentage(x),
                Pos_Center => LP_Percentage(0.5),
                Pos_Left | Pos_Top => LP_Percentage(0.0),
                Pos_Right | Pos_Bottom => LP_Percentage(1.0),
            }
        }
    }
}

pub mod computed {
    pub use cssparser::Color as CSSColor;
    pub use super::super::longhands::computed_as_specified as compute_CSSColor;
    use super::*;
    use super::super::longhands;

    pub struct Context {
        pub inherited_font_weight: longhands::font_weight::computed_value::T,
        pub inherited_font_size: longhands::font_size::computed_value::T,
        pub inherited_text_decorations_in_effect: longhands::_servo_text_decorations_in_effect::T,
        pub inherited_height: longhands::height::T,
        pub color: longhands::color::computed_value::T,
        pub text_decoration: longhands::text_decoration::computed_value::T,
        pub font_size: longhands::font_size::computed_value::T,
        pub display: longhands::display::computed_value::T,
        pub positioned: bool,
        pub floated: bool,
        pub border_top_present: bool,
        pub border_right_present: bool,
        pub border_bottom_present: bool,
        pub border_left_present: bool,
        pub is_root_element: bool,
        // TODO, as needed: root font size, viewport size, etc.
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn compute_Au(value: specified::Length, context: &Context) -> Au {
        compute_Au_with_font_size(value, context.font_size)
    }

    /// A special version of `compute_Au` used for `font-size`.
    #[allow(non_snake_case)]
    #[inline]
    pub fn compute_Au_with_font_size(value: specified::Length, reference_font_size: Au) -> Au {
        match value {
            specified::Au_(value) => value,
            specified::Em(value) => reference_font_size.scale_by(value),
            specified::Ex(value) => {
                let x_height = 0.5;  // TODO: find that from the font
                reference_font_size.scale_by(value * x_height)
            },
        }
    }

    #[deriving(PartialEq, Clone)]
    pub enum LengthOrPercentage {
        LP_Length(Au),
        LP_Percentage(CSSFloat),
    }
    #[allow(non_snake_case)]
    pub fn compute_LengthOrPercentage(value: specified::LengthOrPercentage, context: &Context)
                                   -> LengthOrPercentage {
        match value {
            specified::LP_Length(value) => LP_Length(compute_Au(value, context)),
            specified::LP_Percentage(value) => LP_Percentage(value),
        }
    }

    #[deriving(PartialEq, Clone)]
    pub enum LengthOrPercentageOrAuto {
        LPA_Length(Au),
        LPA_Percentage(CSSFloat),
        LPA_Auto,
    }
    #[allow(non_snake_case)]
    pub fn compute_LengthOrPercentageOrAuto(value: specified::LengthOrPercentageOrAuto,
                                            context: &Context) -> LengthOrPercentageOrAuto {
        match value {
            specified::LPA_Length(value) => LPA_Length(compute_Au(value, context)),
            specified::LPA_Percentage(value) => LPA_Percentage(value),
            specified::LPA_Auto => LPA_Auto,
        }
    }

    #[deriving(PartialEq, Clone)]
    pub enum LengthOrPercentageOrNone {
        LPN_Length(Au),
        LPN_Percentage(CSSFloat),
        LPN_None,
    }
    #[allow(non_snake_case)]
    pub fn compute_LengthOrPercentageOrNone(value: specified::LengthOrPercentageOrNone,
                                            context: &Context) -> LengthOrPercentageOrNone {
        match value {
            specified::LPN_Length(value) => LPN_Length(compute_Au(value, context)),
            specified::LPN_Percentage(value) => LPN_Percentage(value),
            specified::LPN_None => LPN_None,
        }
    }
}

pub fn parse_url(input: &str, base_url: &Url) -> Url {
    UrlParser::new().base_url(base_url).parse(input)
        .unwrap_or_else(|_| Url::parse("about:invalid").unwrap())
}
