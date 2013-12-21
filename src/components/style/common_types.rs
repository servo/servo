/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use servo_util::geometry::Au;

pub type CSSFloat = f64;


pub mod specified {
    use std::ascii::StrAsciiExt;
    use cssparser::ast;
    use cssparser::ast::*;
    use super::{Au, CSSFloat};
    pub use CSSColor = cssparser::Color;

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
        fn parse_internal(input: &ComponentValue, negative_ok: bool) -> Option<Length> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()),
                &Number(ref value) if value.value == 0. =>  Some(Au_(Au(0))),
                _ => None
            }
        }
        #[allow(dead_code)]
        pub fn parse(input: &ComponentValue) -> Option<Length> {
            Length::parse_internal(input, /* negative_ok = */ true)
        }
        pub fn parse_non_negative(input: &ComponentValue) -> Option<Length> {
            Length::parse_internal(input, /* negative_ok = */ false)
        }
        pub fn parse_dimension(value: CSSFloat, unit: &str) -> Option<Length> {
            // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
            let unit_lower = unit.to_ascii_lower(); 
            match unit_lower.as_slice() {
                "px" => Some(Length::from_px(value)),
                "in" => Some(Au_(Au((value * AU_PER_IN) as i32))),
                "cm" => Some(Au_(Au((value * AU_PER_CM) as i32))),
                "mm" => Some(Au_(Au((value * AU_PER_MM) as i32))),
                "pt" => Some(Au_(Au((value * AU_PER_PT) as i32))),
                "pc" => Some(Au_(Au((value * AU_PER_PC) as i32))),
                "em" => Some(Em(value)),
                "ex" => Some(Ex(value)),
                _ => None
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
                              -> Option<LengthOrPercentage> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LP_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Some(LP_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. =>  Some(LP_Length(Au_(Au(0)))),
                _ => None
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &ComponentValue) -> Option<LengthOrPercentage> {
            LengthOrPercentage::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Option<LengthOrPercentage> {
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
                     -> Option<LengthOrPercentageOrAuto> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LPA_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Some(LPA_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Some(LPA_Length(Au_(Au(0)))),
                &Ident(ref value) if value.eq_ignore_ascii_case("auto") => Some(LPA_Auto),
                _ => None
            }
        }
        #[inline]
        pub fn parse(input: &ComponentValue) -> Option<LengthOrPercentageOrAuto> {
            LengthOrPercentageOrAuto::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Option<LengthOrPercentageOrAuto> {
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
                     -> Option<LengthOrPercentageOrNone> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LPN_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Some(LPN_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Some(LPN_Length(Au_(Au(0)))),
                &Ident(ref value) if value.eq_ignore_ascii_case("none") => Some(LPN_None),
                _ => None
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &ComponentValue) -> Option<LengthOrPercentageOrNone> {
            LengthOrPercentageOrNone::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Option<LengthOrPercentageOrNone> {
            LengthOrPercentageOrNone::parse_internal(input, /* negative_ok = */ false)
        }
    }
}

pub mod computed {
    use cssparser;
    pub use CSSColor = cssparser::Color;
    pub use compute_CSSColor = super::super::longhands::computed_as_specified;
    use super::*;
    use super::super::longhands;
    pub use servo_util::geometry::Au;

    pub struct Context {
        current_color: cssparser::RGBA,
        font_size: Au,
        font_weight: longhands::font_weight::computed_value::T,
        position: longhands::position::SpecifiedValue,
        float: longhands::float::SpecifiedValue,
        is_root_element: bool,
        has_border_top: bool,
        has_border_right: bool,
        has_border_bottom: bool,
        has_border_left: bool,
        // TODO, as needed: root font size, viewport size, etc.
    }

    pub fn compute_Au(value: specified::Length, context: &Context) -> Au {
        match value {
            specified::Au_(value) => value,
            specified::Em(value) => context.font_size.scale_by(value),
            specified::Ex(value) => {
                let x_height = 0.5;  // TODO: find that from the font
                context.font_size.scale_by(value * x_height)
            },
        }
    }

    #[deriving(Eq, Clone)]
    pub enum LengthOrPercentage {
        LP_Length(Au),
        LP_Percentage(CSSFloat),
    }
    pub fn compute_LengthOrPercentage(value: specified::LengthOrPercentage, context: &Context)
                                   -> LengthOrPercentage {
        match value {
            specified::LP_Length(value) => LP_Length(compute_Au(value, context)),
            specified::LP_Percentage(value) => LP_Percentage(value),
        }
    }

    #[deriving(Eq, Clone)]
    pub enum LengthOrPercentageOrAuto {
        LPA_Length(Au),
        LPA_Percentage(CSSFloat),
        LPA_Auto,
    }
    pub fn compute_LengthOrPercentageOrAuto(value: specified::LengthOrPercentageOrAuto,
                                            context: &Context) -> LengthOrPercentageOrAuto {
        match value {
            specified::LPA_Length(value) => LPA_Length(compute_Au(value, context)),
            specified::LPA_Percentage(value) => LPA_Percentage(value),
            specified::LPA_Auto => LPA_Auto,
        }
    }

    #[deriving(Eq, Clone)]
    pub enum LengthOrPercentageOrNone {
        LPN_Length(Au),
        LPN_Percentage(CSSFloat),
        LPN_None,
    }
    pub fn compute_LengthOrPercentageOrNone(value: specified::LengthOrPercentageOrNone,
                                            context: &Context) -> LengthOrPercentageOrNone {
        match value {
            specified::LPN_Length(value) => LPN_Length(compute_Au(value, context)),
            specified::LPN_Percentage(value) => LPN_Percentage(value),
            specified::LPN_None => LPN_None,
        }
    }
}
