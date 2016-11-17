/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use ordered_float::NotNaN;
use std::fmt;
use style_traits::ToCss;
use super::{Number, ToComputedValue, Context};
use values::{CSSFloat, Either, None_, specified};

pub use cssparser::Color as CSSColor;
pub use super::image::{EndingShape as GradientShape, Gradient, GradientKind, Image};
pub use super::image::{LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use values::specified::{Angle, BorderStyle, Time, UrlOrNone};

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct CalcLengthOrPercentage {
    pub length: Au,
    pub percentage: Option<CSSFloat>,
}

impl CalcLengthOrPercentage {
    #[inline]
    pub fn length(&self) -> Au {
        self.length
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
                    length: Au(0),
                    percentage: Some(this),
                }
            }
            LengthOrPercentage::Length(this) => {
                CalcLengthOrPercentage {
                    length: this,
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
                    length: Au(0),
                    percentage: Some(this),
                })
            }
            LengthOrPercentageOrAuto::Length(this) => {
                Some(CalcLengthOrPercentage {
                    length: this,
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

impl ToCss for CalcLengthOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match (self.length, self.percentage) {
            (l, Some(p)) if l == Au(0) => write!(dest, "{}%", p * 100.),
            (l, Some(p)) => write!(dest, "calc({}px + {}%)", Au::to_px(l), p * 100.),
            (l, None) => write!(dest, "{}px", Au::to_px(l)),
        }
    }
}

impl ToComputedValue for specified::CalcLengthOrPercentage {
    type ComputedValue = CalcLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> CalcLengthOrPercentage {
        let mut length = Au(0);

        if let Some(absolute) = self.absolute {
            length += absolute;
        }

        for val in &[self.vw, self.vh, self.vmin, self.vmax] {
            if let Some(val) = *val {
                length += val.to_computed_value(context.viewport_size());
            }
        }

        for val in &[self.ch, self.em, self.ex, self.rem] {
            if let Some(val) = *val {
                length += val.to_computed_value(context, /* use inherited */ false);
            }
        }

        CalcLengthOrPercentage {
            length: length,
            percentage: self.percentage.map(|p| p.0),
        }
    }

    #[inline]
    fn from_computed_value(computed: &CalcLengthOrPercentage) -> Self {
        specified::CalcLengthOrPercentage {
            absolute: Some(computed.length),
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

impl ToCss for LengthOrPercentage {
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

impl ToCss for LengthOrPercentageOrAuto {
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

impl ToCss for LengthOrPercentageOrAutoOrContent {
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

impl ToCss for LengthOrPercentageOrNone {
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

pub type LengthOrNone = Either<Length, None_>;

pub type LengthOrNumber = Either<Length, Number>;

pub type Length = Au;
