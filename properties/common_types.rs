/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


pub type Float = f64;
pub type Integer = i64;


pub mod specified {
    use std::ascii::StrAsciiExt;
    use cssparser::*;
    use super::{Integer, Float};

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

    pub enum LengthOrPercentage {
        Length(Length),
        Percentage(Float),
    }
    impl LengthOrPercentage {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                              -> Option<LengthOrPercentage> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map_move(Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Some(Percentage(value.value)),
                &Number(ref value) if value.value == 0. =>  Some(Length(Au(0))),
                _ => None
            }
        }
        pub fn parse(input: &ComponentValue) -> Option<LengthOrPercentage> {
            LengthOrPercentage::parse_internal(input, /* negative_ok = */ true)
        }
        pub fn parse_non_negative(input: &ComponentValue) -> Option<LengthOrPercentage> {
            LengthOrPercentage::parse_internal(input, /* negative_ok = */ false)
        }
    }

    pub enum LengthOrPercentageOrAuto {
        Length_(Length),
        Percentage_(Float),
        Auto,
    }
    impl LengthOrPercentageOrAuto {
        #[inline]
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                     -> Option<LengthOrPercentageOrAuto> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map_move(Length_),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Some(Percentage_(value.value)),
                &Number(ref value) if value.value == 0. => Some(Length_(Au(0))),
                &Ident(ref value) if value.eq_ignore_ascii_case("auto") => Some(Auto),
                _ => None
            }
        }
        pub fn parse(input: &ComponentValue) -> Option<LengthOrPercentageOrAuto> {
            LengthOrPercentageOrAuto::parse_internal(input, /* negative_ok = */ true)
        }
        pub fn parse_non_negative(input: &ComponentValue) -> Option<LengthOrPercentageOrAuto> {
            LengthOrPercentageOrAuto::parse_internal(input, /* negative_ok = */ false)
        }
    }
}

pub mod computed {
    use super::*;
    struct Length(Integer);  // in application units
    impl Length {
        fn times(self, factor: Float) -> Length {
            Length(((*self as Float) * factor) as Integer)
        }

        pub fn compute(parent_font_size: Length, value: specified::Length) -> Length {
            match value {
                specified::Au(value) => Length(value),
                specified::Em(value) => parent_font_size.times(value),
                specified::Ex(value) => {
                    let x_height = 0.5;  // TODO: find that form the font
                    parent_font_size.times(value * x_height)
                },
            }
        }
    }
}
