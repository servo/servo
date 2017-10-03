/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are related to transformations.

use std::fmt;
use style_traits::ToCss;
use values::{computed, specified, CSSFloat};

/// A generic 2D transformation matrix.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
#[css(comma, function)]
pub struct Matrix<T, U = T> {
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
    pub e: U,
    pub f: U,
}

/// A generic transform origin.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
#[derive(MallocSizeOf, PartialEq, ToAnimatedZero, ToComputedValue, ToCss)]
pub struct TransformOrigin<H, V, Depth> {
    /// The horizontal origin.
    pub horizontal: H,
    /// The vertical origin.
    pub vertical: V,
    /// The depth.
    pub depth: Depth,
}

/// A generic timing function.
///
/// <https://drafts.csswg.org/css-timing-1/#single-timing-function-production>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub enum TimingFunction<Integer, Number> {
    /// `linear | ease | ease-in | ease-out | ease-in-out`
    Keyword(TimingKeyword),
    /// `cubic-bezier(<number>, <number>, <number>, <number>)`
    #[allow(missing_docs)]
    CubicBezier { x1: Number, y1: Number, x2: Number, y2: Number },
    /// `step-start | step-end | steps(<integer>, [ start | end ]?)`
    Steps(Integer, StepPosition),
    /// `frames(<integer>)`
    Frames(Integer),
}

define_css_keyword_enum! { TimingKeyword:
    "linear" => Linear,
    "ease" => Ease,
    "ease-in" => EaseIn,
    "ease-out" => EaseOut,
    "ease-in-out" => EaseInOut,
}
add_impls_for_keyword_enum!(TimingKeyword);

define_css_keyword_enum! { StepPosition:
    "start" => Start,
    "end" => End,
}
add_impls_for_keyword_enum!(StepPosition);

impl<H, V, D> TransformOrigin<H, V, D> {
    /// Returns a new transform origin.
    pub fn new(horizontal: H, vertical: V, depth: D) -> Self {
        Self {
            horizontal: horizontal,
            vertical: vertical,
            depth: depth,
        }
    }
}

impl<Integer, Number> TimingFunction<Integer, Number> {
    /// `ease`
    #[inline]
    pub fn ease() -> Self {
        TimingFunction::Keyword(TimingKeyword::Ease)
    }
}

impl<Integer, Number> ToCss for TimingFunction<Integer, Number>
where
    Integer: ToCss,
    Number: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            TimingFunction::Keyword(keyword) => keyword.to_css(dest),
            TimingFunction::CubicBezier { ref x1, ref y1, ref x2, ref y2 } => {
                dest.write_str("cubic-bezier(")?;
                x1.to_css(dest)?;
                dest.write_str(", ")?;
                y1.to_css(dest)?;
                dest.write_str(", ")?;
                x2.to_css(dest)?;
                dest.write_str(", ")?;
                y2.to_css(dest)?;
                dest.write_str(")")
            },
            TimingFunction::Steps(ref intervals, position) => {
                dest.write_str("steps(")?;
                intervals.to_css(dest)?;
                if position != StepPosition::End {
                    dest.write_str(", ")?;
                    position.to_css(dest)?;
                }
                dest.write_str(")")
            },
            TimingFunction::Frames(ref frames) => {
                dest.write_str("frames(")?;
                frames.to_css(dest)?;
                dest.write_str(")")
            },
        }
    }
}

impl TimingKeyword {
    /// Returns the keyword as a quadruplet of Bezier point coordinates
    /// `(x1, y1, x2, y2)`.
    #[inline]
    pub fn to_bezier(self) -> (CSSFloat, CSSFloat, CSSFloat, CSSFloat) {
        match self {
            TimingKeyword::Linear => (0., 0., 1., 1.),
            TimingKeyword::Ease => (0.25, 0.1, 0.25, 1.),
            TimingKeyword::EaseIn => (0.42, 0., 1., 1.),
            TimingKeyword::EaseOut => (0., 0., 0.58, 1.),
            TimingKeyword::EaseInOut => (0.42, 0., 0.58, 1.),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A single operation in the list of a `transform` value
pub enum TransformOperation<Angle, Number, Length, LengthOrNumber, LengthOrPercentage, LoPoNumber> {
    /// Represents a 2D 2x3 matrix.
    Matrix(Matrix<Number>),
    /// Represents a 3D 4x4 matrix with percentage and length values.
    /// For `moz-transform`.
    PrefixedMatrix(Matrix<Number, LoPoNumber>),
    /// Represents a 3D 4x4 matrix.
    #[allow(missing_docs)]
    Matrix3D {
        m11: Number, m12: Number, m13: Number, m14: Number,
        m21: Number, m22: Number, m23: Number, m24: Number,
        m31: Number, m32: Number, m33: Number, m34: Number,
        m41: Number, m42: Number, m43: Number, m44: Number,
    },
    /// Represents a 3D 4x4 matrix with percentage and length values.
    /// For `moz-transform`.
    #[allow(missing_docs)]
    PrefixedMatrix3D {
        m11: Number,     m12: Number,     m13: Number,         m14: Number,
        m21: Number,     m22: Number,     m23: Number,         m24: Number,
        m31: Number,     m32: Number,     m33: Number,         m34: Number,
        m41: LoPoNumber, m42: LoPoNumber, m43: LengthOrNumber, m44: Number,
    },
    /// A 2D skew.
    ///
    /// If the second angle is not provided it is assumed zero.
    ///
    /// Syntax can be skew(angle) or skew(angle, angle)
    Skew(Angle, Option<Angle>),
    /// skewX(angle)
    SkewX(Angle),
    /// skewY(angle)
    SkewY(Angle),
    /// translate(x, y) or translate(x)
    Translate(LengthOrPercentage, Option<LengthOrPercentage>),
    /// translateX(x)
    TranslateX(LengthOrPercentage),
    /// translateY(y)
    TranslateY(LengthOrPercentage),
    /// translateZ(z)
    TranslateZ(Length),
    /// translate3d(x, y, z)
    Translate3D(LengthOrPercentage, LengthOrPercentage, Length),
    /// A 2D scaling factor.
    ///
    /// `scale(2)` is parsed as `Scale(Number::new(2.0), None)` and is equivalent to
    /// writing `scale(2, 2)` (`Scale(Number::new(2.0), Some(Number::new(2.0)))`).
    ///
    /// Negative values are allowed and flip the element.
    ///
    /// Syntax can be scale(factor) or scale(factor, factor)
    Scale(Number, Option<Number>),
    /// scaleX(factor)
    ScaleX(Number),
    /// scaleY(factor)
    ScaleY(Number),
    /// scaleZ(factor)
    ScaleZ(Number),
    /// scale3D(factorX, factorY, factorZ)
    Scale3D(Number, Number, Number),
    /// Describes a 2D Rotation.
    ///
    /// In a 3D scene `rotate(angle)` is equivalent to `rotateZ(angle)`.
    Rotate(Angle),
    /// Rotation in 3D space around the x-axis.
    RotateX(Angle),
    /// Rotation in 3D space around the y-axis.
    RotateY(Angle),
    /// Rotation in 3D space around the z-axis.
    RotateZ(Angle),
    /// Rotation in 3D space.
    ///
    /// Generalization of rotateX, rotateY and rotateZ.
    Rotate3D(Number, Number, Number, Angle),
    /// Specifies a perspective projection matrix.
    ///
    /// Part of CSS Transform Module Level 2 and defined at
    /// [§ 13.1. 3D Transform Function](https://drafts.csswg.org/css-transforms-2/#funcdef-perspective).
    ///
    /// The value must be greater than or equal to zero.
    Perspective(specified::Length),
    /// A intermediate type for interpolation of mismatched transform lists.
    #[allow(missing_docs)]
    InterpolateMatrix { from_list: Transform<TransformOperation<Angle, Number, Length, LengthOrNumber, LengthOrPercentage, LoPoNumber>>,
                        to_list: Transform<TransformOperation<Angle, Number, Length, LengthOrNumber, LengthOrPercentage, LoPoNumber>>,
                        progress: computed::Percentage },
    /// A intermediate type for accumulation of mismatched transform lists.
    #[allow(missing_docs)]
    AccumulateMatrix { from_list: Transform<TransformOperation<Angle, Number, Length, LengthOrNumber, LengthOrPercentage, LoPoNumber>>,
                       to_list: Transform<TransformOperation<Angle, Number, Length, LengthOrNumber, LengthOrPercentage, LoPoNumber>>,
                       count: specified::Integer },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A value of the `transform` property
pub struct Transform<T>(Vec<T>);
