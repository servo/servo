/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are related to transformations.

use std::fmt;
use style_traits::ToCss;
use values::{computed, CSSFloat};

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

#[allow(missing_docs)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Matrix3D<T, U = T, V = T> {
    pub m11: T, pub m12: T, pub m13: T, pub m14: T,
    pub m21: T, pub m22: T, pub m23: T, pub m24: T,
    pub m31: T, pub m32: T, pub m33: T, pub m34: T,
    pub m41: U, pub m42: U, pub m43: V, pub m44: T,
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
    CubicBezier {
        x1: Number,
        y1: Number,
        x2: Number,
        y2: Number,
    },
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
            TimingFunction::CubicBezier {
                ref x1,
                ref y1,
                ref x2,
                ref y2,
            } => {
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

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
#[derive(ToComputedValue)]
/// A single operation in the list of a `transform` value
pub enum TransformOperation<Angle, Number, Length, Integer, LengthOrNumber, LengthOrPercentage, LoPoNumber> {
    /// Represents a 2D 2x3 matrix.
    Matrix(Matrix<Number>),
    /// Represents a 3D 4x4 matrix with percentage and length values.
    /// For `moz-transform`.
    PrefixedMatrix(Matrix<Number, LoPoNumber>),
    /// Represents a 3D 4x4 matrix.
    #[allow(missing_docs)]
    Matrix3D(Matrix3D<Number>),
    /// Represents a 3D 4x4 matrix with percentage and length values.
    /// For `moz-transform`.
    #[allow(missing_docs)]
    PrefixedMatrix3D(Matrix3D<Number, LoPoNumber, LengthOrNumber>),
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
    Perspective(Length),
    /// A intermediate type for interpolation of mismatched transform lists.
    #[allow(missing_docs)]
    InterpolateMatrix {
        #[compute(ignore_bound)]
        from_list: Transform<
            TransformOperation<
                Angle,
                Number,
                Length,
                Integer,
                LengthOrNumber,
                LengthOrPercentage,
                LoPoNumber,
            >,
        >,
        #[compute(ignore_bound)]
        to_list: Transform<
            TransformOperation<
                Angle,
                Number,
                Length,
                Integer,
                LengthOrNumber,
                LengthOrPercentage,
                LoPoNumber,
            >,
        >,
        #[compute(clone)]
        progress: computed::Percentage,
    },
    /// A intermediate type for accumulation of mismatched transform lists.
    #[allow(missing_docs)]
    AccumulateMatrix {
        #[compute(ignore_bound)]
        from_list: Transform<
            TransformOperation<
                Angle,
                Number,
                Length,
                Integer,
                LengthOrNumber,
                LengthOrPercentage,
                LoPoNumber,
            >,
        >,
        #[compute(ignore_bound)]
        to_list: Transform<
            TransformOperation<
                Angle,
                Number,
                Length,
                Integer,
                LengthOrNumber,
                LengthOrPercentage,
                LoPoNumber,
            >,
        >,
        count: Integer,
    },
}

#[derive(Animate, ToComputedValue)]
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
/// A value of the `transform` property
pub struct Transform<T>(pub Vec<T>);

impl<Angle, Number, Length, Integer, LengthOrNumber, LengthOrPercentage, LoPoNumber>
    TransformOperation<Angle, Number, Length, Integer, LengthOrNumber, LengthOrPercentage, LoPoNumber> {
    /// Check if it is any translate function
    pub fn is_translate(&self) -> bool {
        use self::TransformOperation::*;
        match *self {
            Translate(..) | Translate3D(..) | TranslateX(..) | TranslateY(..) | TranslateZ(..) => true,
            _ => false,
        }
    }

    /// Check if it is any scale function
    pub fn is_scale(&self) -> bool {
        use self::TransformOperation::*;
        match *self {
            Scale(..) | Scale3D(..) | ScaleX(..) | ScaleY(..) | ScaleZ(..) => true,
            _ => false,
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl<Angle: ToCss + Copy, Number: ToCss + Copy, Length: ToCss,
     Integer: ToCss + Copy, LengthOrNumber: ToCss, LengthOrPercentage: ToCss, LoPoNumber: ToCss>
    ToCss for
    TransformOperation<Angle, Number, Length, Integer, LengthOrNumber, LengthOrPercentage, LoPoNumber> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TransformOperation::Matrix(ref m) => m.to_css(dest),
            TransformOperation::PrefixedMatrix(ref m) => m.to_css(dest),
            TransformOperation::Matrix3D(Matrix3D {
                m11, m12, m13, m14,
                m21, m22, m23, m24,
                m31, m32, m33, m34,
                m41, m42, m43, m44,
            }) => {
                serialize_function!(dest, matrix3d(
                    m11, m12, m13, m14,
                    m21, m22, m23, m24,
                    m31, m32, m33, m34,
                    m41, m42, m43, m44,
                ))
            }
            TransformOperation::PrefixedMatrix3D(Matrix3D {
                m11, m12, m13, m14,
                m21, m22, m23, m24,
                m31, m32, m33, m34,
                ref m41, ref m42, ref m43, m44,
            }) => {
                serialize_function!(dest, matrix3d(
                    m11, m12, m13, m14,
                    m21, m22, m23, m24,
                    m31, m32, m33, m34,
                    m41, m42, m43, m44,
                ))
            }
            TransformOperation::Skew(ax, None) => {
                serialize_function!(dest, skew(ax))
            }
            TransformOperation::Skew(ax, Some(ay)) => {
                serialize_function!(dest, skew(ax, ay))
            }
            TransformOperation::SkewX(angle) => {
                serialize_function!(dest, skewX(angle))
            }
            TransformOperation::SkewY(angle) => {
                serialize_function!(dest, skewY(angle))
            }
            TransformOperation::Translate(ref tx, None) => {
                serialize_function!(dest, translate(tx))
            }
            TransformOperation::Translate(ref tx, Some(ref ty)) => {
                serialize_function!(dest, translate(tx, ty))
            }
            TransformOperation::TranslateX(ref tx) => {
                serialize_function!(dest, translateX(tx))
            }
            TransformOperation::TranslateY(ref ty) => {
                serialize_function!(dest, translateY(ty))
            }
            TransformOperation::TranslateZ(ref tz) => {
                serialize_function!(dest, translateZ(tz))
            }
            TransformOperation::Translate3D(ref tx, ref ty, ref tz) => {
                serialize_function!(dest, translate3d(tx, ty, tz))
            }
            TransformOperation::Scale(factor, None) => {
                serialize_function!(dest, scale(factor))
            }
            TransformOperation::Scale(sx, Some(sy)) => {
                serialize_function!(dest, scale(sx, sy))
            }
            TransformOperation::ScaleX(sx) => {
                serialize_function!(dest, scaleX(sx))
            }
            TransformOperation::ScaleY(sy) => {
                serialize_function!(dest, scaleY(sy))
            }
            TransformOperation::ScaleZ(sz) => {
                serialize_function!(dest, scaleZ(sz))
            }
            TransformOperation::Scale3D(sx, sy, sz) => {
                serialize_function!(dest, scale3d(sx, sy, sz))
            }
            TransformOperation::Rotate(theta) => {
                serialize_function!(dest, rotate(theta))
            }
            TransformOperation::RotateX(theta) => {
                serialize_function!(dest, rotateX(theta))
            }
            TransformOperation::RotateY(theta) => {
                serialize_function!(dest, rotateY(theta))
            }
            TransformOperation::RotateZ(theta) => {
                serialize_function!(dest, rotateZ(theta))
            }
            TransformOperation::Rotate3D(x, y, z, theta) => {
                serialize_function!(dest, rotate3d(x, y, z, theta))
            }
            TransformOperation::Perspective(ref length) => {
                serialize_function!(dest, perspective(length))
            }
            TransformOperation::InterpolateMatrix { ref from_list, ref to_list, progress } => {
                serialize_function!(dest, interpolatematrix(from_list, to_list, progress))
            }
            TransformOperation::AccumulateMatrix { ref from_list, ref to_list, count } => {
                serialize_function!(dest, accumulatematrix(from_list, to_list, count))
            }
        }
    }
}

impl<T: ToCss> ToCss for Transform<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        if self.0.is_empty() {
            return dest.write_str("none");
        }

        let mut first = true;
        for operation in &self.0 {
            if !first {
                dest.write_str(" ")?;
            }
            first = false;
            operation.to_css(dest)?
        }
        Ok(())
    }
}
