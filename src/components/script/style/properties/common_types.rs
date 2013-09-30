/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub type Float = f64;
pub type Integer = i64;


pub mod specified {
    use std::ascii::StrAsciiExt;
    use cssparser::*;
    use super::{Integer, Float};
    pub use CSSColor = cssparser::Color;

    #[deriving(Clone)]
    pub enum Length {
        Au(Integer),  // application units
        Em(Float),
        Ex(Float),
        // XXX uncomment when supported:
//        Ch(Float),
//        Rem(Float),
//        Vw(Float),
//        Vh(Float),
//        Vmin(Float),
//        Vmax(Float),
    }
    static AU_PER_PX: Float = 60.;
    static AU_PER_IN: Float = AU_PER_PX * 96.;
    static AU_PER_CM: Float = AU_PER_IN / 2.54;
    static AU_PER_MM: Float = AU_PER_IN / 25.4;
    static AU_PER_PT: Float = AU_PER_IN / 72.;
    static AU_PER_PC: Float = AU_PER_PT * 12.;
    impl Length {
        #[inline]
        fn parse_internal(input: &ComponentValue, negative_ok: bool) -> Option<Length> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()),
                &Number(ref value) if value.value == 0. =>  Some(Au(0)),
                _ => None
            }
        }
        pub fn parse(input: &ComponentValue) -> Option<Length> {
            Length::parse_internal(input, /* negative_ok = */ true)
        }
        pub fn parse_non_negative(input: &ComponentValue) -> Option<Length> {
            Length::parse_internal(input, /* negative_ok = */ false)
        }
        pub fn parse_dimension(value: Float, unit: &str) -> Option<Length> {
            match unit.to_ascii_lower().as_slice() {
                "px" => Some(Length::from_px(value)),
                "in" => Some(Au((value * AU_PER_IN) as Integer)),
                "cm" => Some(Au((value * AU_PER_CM) as Integer)),
                "mm" => Some(Au((value * AU_PER_MM) as Integer)),
                "pt" => Some(Au((value * AU_PER_PT) as Integer)),
                "pc" => Some(Au((value * AU_PER_PC) as Integer)),
                "em" => Some(Em(value)),
                "ex" => Some(Ex(value)),
                _ => None
            }
        }
        #[inline]
        pub fn from_px(px_value: Float) -> Length {
            Au((px_value * AU_PER_PX) as Integer)
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentage {
        LP_Length(Length),
        LP_Percentage(Float),
    }
    impl LengthOrPercentage {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                              -> Option<LengthOrPercentage> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map_move(LP_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Some(LP_Percentage(value.value)),
                &Number(ref value) if value.value == 0. =>  Some(LP_Length(Au(0))),
                _ => None
            }
        }
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
        LPA_Percentage(Float),
        LPA_Auto,
    }
    impl LengthOrPercentageOrAuto {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                     -> Option<LengthOrPercentageOrAuto> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map_move(LPA_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Some(LPA_Percentage(value.value)),
                &Number(ref value) if value.value == 0. => Some(LPA_Length(Au(0))),
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
}

pub mod computed {
    use cssparser;
    pub use CSSColor = cssparser::Color;
    pub use compute_CSSColor = super::super::longhands::computed_as_specified;
    use super::*;
    use super::super::longhands;
    pub struct Context {
        current_color: cssparser::RGBA,
        font_size: Length,
        font_weight: longhands::font_weight::ComputedValue,
        position: longhands::position::SpecifiedValue,
        float: longhands::float::SpecifiedValue,
        is_root_element: bool,
        has_border_top: bool,
        has_border_right: bool,
        has_border_bottom: bool,
        has_border_left: bool,
        // TODO, as needed: root font size, viewport size, etc.
    }
    #[deriving(Clone)]
    pub struct Length(Integer);  // in application units
    impl Length {
        pub fn times(self, factor: Float) -> Length {
            Length(((*self as Float) * factor) as Integer)
        }
    }

    pub fn compute_Length(value: specified::Length, context: &Context) -> Length {
        match value {
            specified::Au(value) => Length(value),
            specified::Em(value) => context.font_size.times(value),
            specified::Ex(value) => {
                let x_height = 0.5;  // TODO: find that from the font
                context.font_size.times(value * x_height)
            },
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentage {
        LP_Length(Length),
        LP_Percentage(Float),
    }
    pub fn compute_LengthOrPercentage(value: specified::LengthOrPercentage, context: &Context)
                                   -> LengthOrPercentage {
        match value {
            specified::LP_Length(value) => LP_Length(compute_Length(value, context)),
            specified::LP_Percentage(value) => LP_Percentage(value),
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentageOrAuto {
        LPA_Length(Length),
        LPA_Percentage(Float),
        LPA_Auto,
    }
    pub fn compute_LengthOrPercentageOrAuto(value: specified::LengthOrPercentageOrAuto,
                                            context: &Context) -> LengthOrPercentageOrAuto {
        match value {
            specified::LPA_Length(value) => LPA_Length(compute_Length(value, context)),
            specified::LPA_Percentage(value) => LPA_Percentage(value),
            specified::LPA_Auto => LPA_Auto,
        }
    }
}
