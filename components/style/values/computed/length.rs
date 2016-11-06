/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use ordered_float::NotNaN;
use std::fmt;
use super::{Number, ToComputedValue, Context};
use values::{CSSFloat, LocalToCss, specified};

pub use cssparser::Color as CSSColor;
pub use super::image::{EndingShape as GradientShape, Gradient, GradientKind, Image};
pub use super::image::{LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use values::specified::{Angle, BorderStyle, Time, UrlExtraData, UrlOrNone};

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct CalcLengthOrPercentage {
    pub length: Option<Au>,
    pub percentage: Option<CSSFloat>,
}

impl CalcLengthOrPercentage {
    #[inline]
    pub fn length(&self) -> Au {
        self.length.unwrap_or(Au(0))
    }

    #[inline]
    pub fn percentage(&self) -> CSSFloat {
        self.percentage.unwrap_or(0.)
    }
}

impl From<LengthOrPercentage> for CalcLengthOrPercentage {
    fn from(len: LengthOrPercentage) -> CalcLengthOrPercentage {
        match len {
            LengthOrPercentage::Percentage(this) => {
                CalcLengthOrPercentage {
                    length: None,
                    percentage: Some(this),
                }
            }
            LengthOrPercentage::Length(this) => {
                CalcLengthOrPercentage {
                    length: Some(this),
                    percentage: None,
                }
            }
            LengthOrPercentage::Calc(this) => {
                this
            }
        }
    }
}

impl From<LengthOrPercentageOrAuto> for Option<CalcLengthOrPercentage> {
    fn from(len: LengthOrPercentageOrAuto) -> Option<CalcLengthOrPercentage> {
        match len {
            LengthOrPercentageOrAuto::Percentage(this) => {
                Some(CalcLengthOrPercentage {
                    length: None,
                    percentage: Some(this),
                })
            }
            LengthOrPercentageOrAuto::Length(this) => {
                Some(CalcLengthOrPercentage {
                    length: Some(this),
                    percentage: None,
                })
            }
            LengthOrPercentageOrAuto::Calc(this) => {
                Some(this)
            }
            LengthOrPercentageOrAuto::Auto => {
                None
            }
        }
    }
}

impl ::cssparser::ToCss for CalcLengthOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match (self.length, self.percentage) {
            (None, Some(p)) => write!(dest, "{}%", p * 100.),
            (Some(l), None) => write!(dest, "{}px", Au::to_px(l)),
            (Some(l), Some(p)) => write!(dest, "calc({}px + {}%)", Au::to_px(l), p * 100.),
            _ => unreachable!()
        }
    }
}

impl ToComputedValue for specified::CalcLengthOrPercentage {
    type ComputedValue = CalcLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> CalcLengthOrPercentage {
        self.compute_from_viewport_and_font_size(context.viewport_size(),
                                                 context.style().get_font().clone_font_size(),
                                                 context.style().root_font_size())

    }

    #[inline]
    fn from_computed_value(computed: &CalcLengthOrPercentage) -> Self {
        specified::CalcLengthOrPercentage {
            absolute: computed.length,
            percentage: computed.percentage.map(specified::Percentage),
            ..Default::default()
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentage {
    Length(Au),
    Percentage(CSSFloat),
    Calc(CalcLengthOrPercentage),
}

impl LengthOrPercentage {
    #[inline]
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(Au(0))
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    ///
    /// (Returns false for calc() values, even if ones that may resolve to zero.)
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        use self::LengthOrPercentage::*;
        match *self {
            Length(Au(0)) | Percentage(0.0) => true,
            Length(_) | Percentage(_) | Calc(_) => false
        }
    }

    pub fn to_hash_key(&self) -> (Au, NotNaN<f32>) {
        use self::LengthOrPercentage::*;
        match *self {
            Length(l) => (l, NotNaN::new(0.0).unwrap()),
            Percentage(p) => (Au(0), NotNaN::new(p).unwrap()),
            Calc(c) => (c.length(), NotNaN::new(c.percentage()).unwrap()),
        }
    }
}

impl fmt::Debug for LengthOrPercentage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentage::Length(length) => write!(f, "{:?}", length),
            LengthOrPercentage::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
            LengthOrPercentage::Calc(calc) => write!(f, "{:?}", calc),
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentage {
    type ComputedValue = LengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> LengthOrPercentage {
        match *self {
            specified::LengthOrPercentage::Length(value) => {
                LengthOrPercentage::Length(value.to_computed_value(context))
            }
            specified::LengthOrPercentage::Percentage(value) => {
                LengthOrPercentage::Percentage(value.0)
            }
            specified::LengthOrPercentage::Calc(calc) => {
                LengthOrPercentage::Calc(calc.to_computed_value(context))
            }
        }
    }

    fn from_computed_value(computed: &LengthOrPercentage) -> Self {
        match *computed {
            LengthOrPercentage::Length(value) => {
                specified::LengthOrPercentage::Length(
                    ToComputedValue::from_computed_value(&value)
                )
            }
            LengthOrPercentage::Percentage(value) => {
                specified::LengthOrPercentage::Percentage(specified::Percentage(value))
            }
            LengthOrPercentage::Calc(calc) => {
                specified::LengthOrPercentage::Calc(
                    ToComputedValue::from_computed_value(&calc)
                )
            }
        }
    }
}

impl ::cssparser::ToCss for LengthOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentage::Length(length) => length.to_css(dest),
            LengthOrPercentage::Percentage(percentage)
            => write!(dest, "{}%", percentage * 100.),
            LengthOrPercentage::Calc(calc) => calc.to_css(dest),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrAuto {
    Length(Au),
    Percentage(CSSFloat),
    Auto,
    Calc(CalcLengthOrPercentage),
}

impl LengthOrPercentageOrAuto {
    /// Returns true if the computed value is absolute 0 or 0%.
    ///
    /// (Returns false for calc() values, even if ones that may resolve to zero.)
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        use self::LengthOrPercentageOrAuto::*;
        match *self {
            Length(Au(0)) | Percentage(0.0) => true,
            Length(_) | Percentage(_) | Calc(_) | Auto => false
        }
    }
}

impl fmt::Debug for LengthOrPercentageOrAuto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentageOrAuto::Length(length) => write!(f, "{:?}", length),
            LengthOrPercentageOrAuto::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
            LengthOrPercentageOrAuto::Auto => write!(f, "auto"),
            LengthOrPercentageOrAuto::Calc(calc) => write!(f, "{:?}", calc),
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentageOrAuto {
    type ComputedValue = LengthOrPercentageOrAuto;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrAuto {
        match *self {
            specified::LengthOrPercentageOrAuto::Length(value) => {
                LengthOrPercentageOrAuto::Length(value.to_computed_value(context))
            }
            specified::LengthOrPercentageOrAuto::Percentage(value) => {
                LengthOrPercentageOrAuto::Percentage(value.0)
            }
            specified::LengthOrPercentageOrAuto::Auto => {
                LengthOrPercentageOrAuto::Auto
            }
            specified::LengthOrPercentageOrAuto::Calc(calc) => {
                LengthOrPercentageOrAuto::Calc(calc.to_computed_value(context))
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrAuto) -> Self {
        match *computed {
            LengthOrPercentageOrAuto::Auto => specified::LengthOrPercentageOrAuto::Auto,
            LengthOrPercentageOrAuto::Length(value) => {
                specified::LengthOrPercentageOrAuto::Length(
                    ToComputedValue::from_computed_value(&value)
                )
            }
            LengthOrPercentageOrAuto::Percentage(value) => {
                specified::LengthOrPercentageOrAuto::Percentage(specified::Percentage(value))
            }
            LengthOrPercentageOrAuto::Calc(calc) => {
                specified::LengthOrPercentageOrAuto::Calc(
                    ToComputedValue::from_computed_value(&calc)
                )
            }
        }
    }
}

impl ::cssparser::ToCss for LengthOrPercentageOrAuto {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrAuto::Length(length) => length.to_css(dest),
            LengthOrPercentageOrAuto::Percentage(percentage)
            => write!(dest, "{}%", percentage * 100.),
            LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
            LengthOrPercentageOrAuto::Calc(calc) => calc.to_css(dest),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrAutoOrContent {
    Length(Au),
    Percentage(CSSFloat),
    Calc(CalcLengthOrPercentage),
    Auto,
    Content
}

impl fmt::Debug for LengthOrPercentageOrAutoOrContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentageOrAutoOrContent::Length(length) => write!(f, "{:?}", length),
            LengthOrPercentageOrAutoOrContent::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
            LengthOrPercentageOrAutoOrContent::Calc(calc) => write!(f, "{:?}", calc),
            LengthOrPercentageOrAutoOrContent::Auto => write!(f, "auto"),
            LengthOrPercentageOrAutoOrContent::Content => write!(f, "content")
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentageOrAutoOrContent {
    type ComputedValue = LengthOrPercentageOrAutoOrContent;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrAutoOrContent {
        match *self {
            specified::LengthOrPercentageOrAutoOrContent::Length(value) => {
                LengthOrPercentageOrAutoOrContent::Length(value.to_computed_value(context))
            },
            specified::LengthOrPercentageOrAutoOrContent::Percentage(value) => {
                LengthOrPercentageOrAutoOrContent::Percentage(value.0)
            },
            specified::LengthOrPercentageOrAutoOrContent::Calc(calc) => {
                LengthOrPercentageOrAutoOrContent::Calc(calc.to_computed_value(context))
            },
            specified::LengthOrPercentageOrAutoOrContent::Auto => {
                LengthOrPercentageOrAutoOrContent::Auto
            },
            specified::LengthOrPercentageOrAutoOrContent::Content => {
                LengthOrPercentageOrAutoOrContent::Content
            }
        }
    }


    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrAutoOrContent) -> Self {
        match *computed {
            LengthOrPercentageOrAutoOrContent::Auto => {
                specified::LengthOrPercentageOrAutoOrContent::Auto
            }
            LengthOrPercentageOrAutoOrContent::Content => {
                specified::LengthOrPercentageOrAutoOrContent::Content
            }
            LengthOrPercentageOrAutoOrContent::Length(value) => {
                specified::LengthOrPercentageOrAutoOrContent::Length(
                    ToComputedValue::from_computed_value(&value)
                )
            }
            LengthOrPercentageOrAutoOrContent::Percentage(value) => {
                specified::LengthOrPercentageOrAutoOrContent::Percentage(specified::Percentage(value))
            }
            LengthOrPercentageOrAutoOrContent::Calc(calc) => {
                specified::LengthOrPercentageOrAutoOrContent::Calc(
                    ToComputedValue::from_computed_value(&calc)
                )
            }
        }
    }
}

impl ::cssparser::ToCss for LengthOrPercentageOrAutoOrContent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrAutoOrContent::Length(length) => length.to_css(dest),
            LengthOrPercentageOrAutoOrContent::Percentage(percentage)
            => write!(dest, "{}%", percentage * 100.),
            LengthOrPercentageOrAutoOrContent::Calc(calc) => calc.to_css(dest),
            LengthOrPercentageOrAutoOrContent::Auto => dest.write_str("auto"),
            LengthOrPercentageOrAutoOrContent::Content => dest.write_str("content")
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrNone {
    Length(Au),
    Percentage(CSSFloat),
    Calc(CalcLengthOrPercentage),
    None,
}

impl fmt::Debug for LengthOrPercentageOrNone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentageOrNone::Length(length) => write!(f, "{:?}", length),
            LengthOrPercentageOrNone::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
            LengthOrPercentageOrNone::Calc(calc) => write!(f, "{:?}", calc),
            LengthOrPercentageOrNone::None => write!(f, "none"),
        }
    }
}

impl ToComputedValue for specified::LengthOrPercentageOrNone {
    type ComputedValue = LengthOrPercentageOrNone;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrNone {
        match *self {
            specified::LengthOrPercentageOrNone::Length(value) => {
                LengthOrPercentageOrNone::Length(value.to_computed_value(context))
            }
            specified::LengthOrPercentageOrNone::Percentage(value) => {
                LengthOrPercentageOrNone::Percentage(value.0)
            }
            specified::LengthOrPercentageOrNone::Calc(calc) => {
                LengthOrPercentageOrNone::Calc(calc.to_computed_value(context))
            }
            specified::LengthOrPercentageOrNone::None => {
                LengthOrPercentageOrNone::None
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrNone) -> Self {
        match *computed {
            LengthOrPercentageOrNone::None => specified::LengthOrPercentageOrNone::None,
            LengthOrPercentageOrNone::Length(value) => {
                specified::LengthOrPercentageOrNone::Length(
                    ToComputedValue::from_computed_value(&value)
                )
            }
            LengthOrPercentageOrNone::Percentage(value) => {
                specified::LengthOrPercentageOrNone::Percentage(specified::Percentage(value))
            }
            LengthOrPercentageOrNone::Calc(calc) => {
                specified::LengthOrPercentageOrNone::Calc(
                    ToComputedValue::from_computed_value(&calc)
                )
            }
        }
    }
}

impl ::cssparser::ToCss for LengthOrPercentageOrNone {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrNone::Length(length) => length.to_css(dest),
            LengthOrPercentageOrNone::Percentage(percentage) =>
                write!(dest, "{}%", percentage * 100.),
            LengthOrPercentageOrNone::Calc(calc) => calc.to_css(dest),
            LengthOrPercentageOrNone::None => dest.write_str("none"),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrNone {
    Length(Au),
    None,
}

impl fmt::Debug for LengthOrNone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrNone::Length(length) => write!(f, "{:?}", length),
            LengthOrNone::None => write!(f, "none"),
        }
    }
}

impl ToComputedValue for specified::LengthOrNone {
    type ComputedValue = LengthOrNone;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrNone {
        match *self {
            specified::LengthOrNone::Length(specified::Length::Calc(calc, range)) => {
                LengthOrNone::Length(range.clamp(calc.to_computed_value(context).length()))
            }
            specified::LengthOrNone::Length(value) => {
                LengthOrNone::Length(value.to_computed_value(context))
            }
            specified::LengthOrNone::None => {
                LengthOrNone::None
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &LengthOrNone) -> Self {
        match *computed {
            LengthOrNone::Length(au) => {
                specified::LengthOrNone::Length(ToComputedValue::from_computed_value(&au))
            }
            LengthOrNone::None => {
                specified::LengthOrNone::None
            }
        }
    }
}

impl ::cssparser::ToCss for LengthOrNone {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrNone::Length(length) => length.to_css(dest),
            LengthOrNone::None => dest.write_str("none"),
        }
    }
}

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrNumber {
    Length(Length),
    Number(Number),
}

impl fmt::Debug for LengthOrNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrNumber::Length(length) => write!(f, "{:?}", length),
            LengthOrNumber::Number(number) => write!(f, "{:?}", number),
        }
    }
}

impl ToComputedValue for specified::LengthOrNumber {
    type ComputedValue = LengthOrNumber;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrNumber {
        match *self {
            specified::LengthOrNumber::Length(len) =>
                LengthOrNumber::Length(len.to_computed_value(context)),
            specified::LengthOrNumber::Number(number) =>
                LengthOrNumber::Number(number.to_computed_value(context)),
        }
    }
    #[inline]
    fn from_computed_value(computed: &LengthOrNumber) -> Self {
        match *computed {
            LengthOrNumber::Length(len) =>
                specified::LengthOrNumber::Length(ToComputedValue::from_computed_value(&len)),
            LengthOrNumber::Number(number) =>
                specified::LengthOrNumber::Number(ToComputedValue::from_computed_value(&number)),
        }
    }
}

impl ::cssparser::ToCss for LengthOrNumber {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrNumber::Length(len) => len.to_css(dest),
            LengthOrNumber::Number(number) => number.to_css(dest),
        }
    }
}

pub type Length = Au;
