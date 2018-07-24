/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are related to transformations.

use app_units::Au;
use euclid::{self, Rect, Transform3D};
use num_traits::Zero;
use values::{computed, CSSFloat};
use values::computed::length::Length as ComputedLength;
use values::computed::length::LengthOrPercentage as ComputedLengthOrPercentage;
use values::specified::length::Length as SpecifiedLength;
use values::specified::length::LengthOrPercentage as SpecifiedLengthOrPercentage;

/// A generic 2D transformation matrix.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
#[css(comma, function)]
pub struct Matrix<T> {
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
    pub e: T,
    pub f: T,
}

#[allow(missing_docs)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[css(comma, function = "matrix3d")]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub struct Matrix3D<T> {
    pub m11: T, pub m12: T, pub m13: T, pub m14: T,
    pub m21: T, pub m22: T, pub m23: T, pub m24: T,
    pub m31: T, pub m32: T, pub m33: T, pub m34: T,
    pub m41: T, pub m42: T, pub m43: T, pub m44: T,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl<T: Into<f64>> From<Matrix<T>> for Transform3D<f64> {
    #[inline]
    fn from(m: Matrix<T>) -> Self {
        Transform3D::row_major(
            m.a.into(), m.b.into(), 0.0, 0.0,
            m.c.into(), m.d.into(), 0.0, 0.0,
            0.0,        0.0,        1.0, 0.0,
            m.e.into(), m.f.into(), 0.0, 1.0,
        )
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl<T: Into<f64>> From<Matrix3D<T>> for Transform3D<f64> {
    #[inline]
    fn from(m: Matrix3D<T>) -> Self {
        Transform3D::row_major(
            m.m11.into(), m.m12.into(), m.m13.into(), m.m14.into(),
            m.m21.into(), m.m22.into(), m.m23.into(), m.m24.into(),
            m.m31.into(), m.m32.into(), m.m33.into(), m.m34.into(),
            m.m41.into(), m.m42.into(), m.m43.into(), m.m44.into(),
        )
    }
}

/// A generic transform origin.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf,
         PartialEq, SpecifiedValueInfo, ToAnimatedZero, ToComputedValue, ToCss)]
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
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToCss)]
#[value_info(ty = "TIMING_FUNCTION")]
pub enum TimingFunction<Integer, Number> {
    /// `linear | ease | ease-in | ease-out | ease-in-out`
    Keyword(TimingKeyword),
    /// `cubic-bezier(<number>, <number>, <number>, <number>)`
    #[allow(missing_docs)]
    #[css(comma, function)]
    CubicBezier {
        x1: Number,
        y1: Number,
        x2: Number,
        y2: Number,
    },
    /// `step-start | step-end | steps(<integer>, [ start | end ]?)`
    #[css(comma, function)]
    #[value_info(other_values = "step-start,step-end")]
    Steps(Integer, #[css(skip_if = "is_end")] StepPosition),
    /// `frames(<integer>)`
    #[css(comma, function)]
    Frames(Integer),
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
pub enum TimingKeyword {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToComputedValue, ToCss)]
pub enum StepPosition {
    Start,
    End,
}

#[inline]
fn is_end(position: &StepPosition) -> bool {
    *position == StepPosition::End
}

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

#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
/// A single operation in the list of a `transform` value
pub enum TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage> {
    /// Represents a 2D 2x3 matrix.
    Matrix(Matrix<Number>),
    /// Represents a 3D 4x4 matrix.
    Matrix3D(Matrix3D<Number>),
    /// A 2D skew.
    ///
    /// If the second angle is not provided it is assumed zero.
    ///
    /// Syntax can be skew(angle) or skew(angle, angle)
    #[css(comma, function)]
    Skew(Angle, Option<Angle>),
    /// skewX(angle)
    #[css(function = "skewX")]
    SkewX(Angle),
    /// skewY(angle)
    #[css(function = "skewY")]
    SkewY(Angle),
    /// translate(x, y) or translate(x)
    #[css(comma, function)]
    Translate(LengthOrPercentage, Option<LengthOrPercentage>),
    /// translateX(x)
    #[css(function = "translateX")]
    TranslateX(LengthOrPercentage),
    /// translateY(y)
    #[css(function = "translateY")]
    TranslateY(LengthOrPercentage),
    /// translateZ(z)
    #[css(function = "translateZ")]
    TranslateZ(Length),
    /// translate3d(x, y, z)
    #[css(comma, function = "translate3d")]
    Translate3D(LengthOrPercentage, LengthOrPercentage, Length),
    /// A 2D scaling factor.
    ///
    /// `scale(2)` is parsed as `Scale(Number::new(2.0), None)` and is equivalent to
    /// writing `scale(2, 2)` (`Scale(Number::new(2.0), Some(Number::new(2.0)))`).
    ///
    /// Negative values are allowed and flip the element.
    ///
    /// Syntax can be scale(factor) or scale(factor, factor)
    #[css(comma, function)]
    Scale(Number, Option<Number>),
    /// scaleX(factor)
    #[css(function = "scaleX")]
    ScaleX(Number),
    /// scaleY(factor)
    #[css(function = "scaleY")]
    ScaleY(Number),
    /// scaleZ(factor)
    #[css(function = "scaleZ")]
    ScaleZ(Number),
    /// scale3D(factorX, factorY, factorZ)
    #[css(comma, function = "scale3d")]
    Scale3D(Number, Number, Number),
    /// Describes a 2D Rotation.
    ///
    /// In a 3D scene `rotate(angle)` is equivalent to `rotateZ(angle)`.
    #[css(function)]
    Rotate(Angle),
    /// Rotation in 3D space around the x-axis.
    #[css(function = "rotateX")]
    RotateX(Angle),
    /// Rotation in 3D space around the y-axis.
    #[css(function = "rotateY")]
    RotateY(Angle),
    /// Rotation in 3D space around the z-axis.
    #[css(function = "rotateZ")]
    RotateZ(Angle),
    /// Rotation in 3D space.
    ///
    /// Generalization of rotateX, rotateY and rotateZ.
    #[css(comma, function = "rotate3d")]
    Rotate3D(Number, Number, Number, Angle),
    /// Specifies a perspective projection matrix.
    ///
    /// Part of CSS Transform Module Level 2 and defined at
    /// [§ 13.1. 3D Transform Function](https://drafts.csswg.org/css-transforms-2/#funcdef-perspective).
    ///
    /// The value must be greater than or equal to zero.
    #[css(function)]
    Perspective(Length),
    /// A intermediate type for interpolation of mismatched transform lists.
    #[allow(missing_docs)]
    #[css(comma, function = "interpolatematrix")]
    InterpolateMatrix {
        from_list:
            Transform<TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage>>,
        to_list: Transform<TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage>>,
        progress: computed::Percentage,
    },
    /// A intermediate type for accumulation of mismatched transform lists.
    #[allow(missing_docs)]
    #[css(comma, function = "accumulatematrix")]
    AccumulateMatrix {
        from_list:
            Transform<TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage>>,
        to_list: Transform<TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage>>,
        count: Integer,
    },
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
/// A value of the `transform` property
pub struct Transform<T>(#[css(if_empty = "none", iterable)] pub Vec<T>);

impl<Angle, Number, Length, Integer, LengthOrPercentage>
    TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage>
{
    /// Check if it is any rotate function.
    pub fn is_rotate(&self) -> bool {
        use self::TransformOperation::*;
        matches!(
            *self,
            Rotate(..) |
            Rotate3D(..) |
            RotateX(..) |
            RotateY(..) |
            RotateZ(..)
        )
    }

    /// Check if it is any translate function
    pub fn is_translate(&self) -> bool {
        use self::TransformOperation::*;
        match *self {
            Translate(..) | Translate3D(..) | TranslateX(..) | TranslateY(..) | TranslateZ(..) => {
                true
            },
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

/// Convert a length type into the absolute lengths.
pub trait ToAbsoluteLength {
    /// Returns the absolute length as pixel value.
    fn to_pixel_length(&self, containing_len: Option<Au>) -> Result<CSSFloat, ()>;
}

impl ToAbsoluteLength for SpecifiedLength {
    // This returns Err(()) if there is any relative length or percentage. We use this when
    // parsing a transform list of DOMMatrix because we want to return a DOM Exception
    // if there is relative length.
    #[inline]
    fn to_pixel_length(&self, _containing_len: Option<Au>) -> Result<CSSFloat, ()> {
        match *self {
            SpecifiedLength::NoCalc(len) => len.to_computed_pixel_length_without_context(),
            SpecifiedLength::Calc(ref calc) => calc.to_computed_pixel_length_without_context(),
        }
    }
}

impl ToAbsoluteLength for SpecifiedLengthOrPercentage {
    // This returns Err(()) if there is any relative length or percentage. We use this when
    // parsing a transform list of DOMMatrix because we want to return a DOM Exception
    // if there is relative length.
    #[inline]
    fn to_pixel_length(&self, _containing_len: Option<Au>) -> Result<CSSFloat, ()> {
        use self::SpecifiedLengthOrPercentage::*;
        match *self {
            Length(len) => len.to_computed_pixel_length_without_context(),
            Calc(ref calc) => calc.to_computed_pixel_length_without_context(),
            _ => Err(()),
        }
    }
}

impl ToAbsoluteLength for ComputedLength {
    #[inline]
    fn to_pixel_length(&self, _containing_len: Option<Au>) -> Result<CSSFloat, ()> {
        Ok(self.px())
    }
}

impl ToAbsoluteLength for ComputedLengthOrPercentage {
    #[inline]
    fn to_pixel_length(&self, containing_len: Option<Au>) -> Result<CSSFloat, ()> {
        let extract_pixel_length = |lop: &ComputedLengthOrPercentage| match *lop {
            ComputedLengthOrPercentage::Length(px) => px.px(),
            ComputedLengthOrPercentage::Percentage(_) => 0.,
            ComputedLengthOrPercentage::Calc(calc) => calc.length().px(),
        };

        match containing_len {
            Some(relative_len) => Ok(self.to_pixel_length(relative_len).px()),
            // If we don't have reference box, we cannot resolve the used value,
            // so only retrieve the length part. This will be used for computing
            // distance without any layout info.
            None => Ok(extract_pixel_length(self)),
        }
    }
}

/// Support the conversion to a 3d matrix.
pub trait ToMatrix {
    /// Check if it is a 3d transform function.
    fn is_3d(&self) -> bool;

    /// Return the equivalent 3d matrix.
    fn to_3d_matrix(&self, reference_box: Option<&Rect<Au>>) -> Result<Transform3D<f64>, ()>;
}

impl<Angle, Number, Length, Integer, LoP> ToMatrix
    for TransformOperation<Angle, Number, Length, Integer, LoP>
where
    Angle: Copy + AsRef<computed::angle::Angle>,
    Number: Copy + Into<f32> + Into<f64>,
    Length: ToAbsoluteLength,
    LoP: ToAbsoluteLength,
{
    #[inline]
    fn is_3d(&self) -> bool {
        use self::TransformOperation::*;
        match *self {
            Translate3D(..) | TranslateZ(..) | Rotate3D(..) | RotateX(..) | RotateY(..) |
            RotateZ(..) | Scale3D(..) | ScaleZ(..) | Perspective(..) | Matrix3D(..) => true,
            _ => false,
        }
    }

    /// If |reference_box| is None, we will drop the percent part from translate because
    /// we cannot resolve it without the layout info, for computed TransformOperation.
    /// However, for specified TransformOperation, we will return Err(()) if there is any relative
    /// lengths because the only caller, DOMMatrix, doesn't accept relative lengths.
    #[inline]
    fn to_3d_matrix(&self, reference_box: Option<&Rect<Au>>) -> Result<Transform3D<f64>, ()> {
        use self::TransformOperation::*;
        use std::f64;

        const TWO_PI: f64 = 2.0f64 * f64::consts::PI;
        let reference_width = reference_box.map(|v| v.size.width);
        let reference_height = reference_box.map(|v| v.size.height);
        let matrix = match *self {
            Rotate3D(ax, ay, az, theta) => {
                let theta = TWO_PI - theta.as_ref().radians64();
                let (ax, ay, az, theta) =
                    get_normalized_vector_and_angle(ax.into(), ay.into(), az.into(), theta);
                Transform3D::create_rotation(
                    ax as f64,
                    ay as f64,
                    az as f64,
                    euclid::Angle::radians(theta),
                )
            },
            RotateX(theta) => {
                let theta = euclid::Angle::radians(TWO_PI - theta.as_ref().radians64());
                Transform3D::create_rotation(1., 0., 0., theta)
            },
            RotateY(theta) => {
                let theta = euclid::Angle::radians(TWO_PI - theta.as_ref().radians64());
                Transform3D::create_rotation(0., 1., 0., theta)
            },
            RotateZ(theta) | Rotate(theta) => {
                let theta = euclid::Angle::radians(TWO_PI - theta.as_ref().radians64());
                Transform3D::create_rotation(0., 0., 1., theta)
            },
            Perspective(ref d) => {
                let m = create_perspective_matrix(d.to_pixel_length(None)?);
                m.cast()
            },
            Scale3D(sx, sy, sz) => Transform3D::create_scale(sx.into(), sy.into(), sz.into()),
            Scale(sx, sy) => Transform3D::create_scale(sx.into(), sy.unwrap_or(sx).into(), 1.),
            ScaleX(s) => Transform3D::create_scale(s.into(), 1., 1.),
            ScaleY(s) => Transform3D::create_scale(1., s.into(), 1.),
            ScaleZ(s) => Transform3D::create_scale(1., 1., s.into()),
            Translate3D(ref tx, ref ty, ref tz) => {
                let tx = tx.to_pixel_length(reference_width)? as f64;
                let ty = ty.to_pixel_length(reference_height)? as f64;
                Transform3D::create_translation(tx, ty, tz.to_pixel_length(None)? as f64)
            },
            Translate(ref tx, Some(ref ty)) => {
                let tx = tx.to_pixel_length(reference_width)? as f64;
                let ty = ty.to_pixel_length(reference_height)? as f64;
                Transform3D::create_translation(tx, ty, 0.)
            },
            TranslateX(ref t) | Translate(ref t, None) => {
                let t = t.to_pixel_length(reference_width)? as f64;
                Transform3D::create_translation(t, 0., 0.)
            },
            TranslateY(ref t) => {
                let t = t.to_pixel_length(reference_height)? as f64;
                Transform3D::create_translation(0., t, 0.)
            },
            TranslateZ(ref z) => {
                Transform3D::create_translation(0., 0., z.to_pixel_length(None)? as f64)
            },
            Skew(theta_x, theta_y) => Transform3D::create_skew(
                euclid::Angle::radians(theta_x.as_ref().radians64()),
                euclid::Angle::radians(theta_y.map_or(0., |a| a.as_ref().radians64())),
            ),
            SkewX(theta) => Transform3D::create_skew(
                euclid::Angle::radians(theta.as_ref().radians64()),
                euclid::Angle::radians(0.),
            ),
            SkewY(theta) => Transform3D::create_skew(
                euclid::Angle::radians(0.),
                euclid::Angle::radians(theta.as_ref().radians64()),
            ),
            Matrix3D(m) => m.into(),
            Matrix(m) => m.into(),
            InterpolateMatrix { .. } | AccumulateMatrix { .. } => {
                // TODO: Convert InterpolateMatrix/AccumulateMatrix into a valid Transform3D by
                // the reference box and do interpolation on these two Transform3D matrices.
                // Both Gecko and Servo don't support this for computing distance, and Servo
                // doesn't support animations on InterpolateMatrix/AccumulateMatrix, so
                // return an identity matrix.
                // Note: DOMMatrix doesn't go into this arm.
                Transform3D::identity()
            },
        };
        Ok(matrix)
    }
}

impl<T> Transform<T> {
    /// `none`
    pub fn none() -> Self {
        Transform(vec![])
    }
}

impl<T: ToMatrix> Transform<T> {
    /// Return the equivalent 3d matrix of this transform list.
    /// We return a pair: the first one is the transform matrix, and the second one
    /// indicates if there is any 3d transform function in this transform list.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn to_transform_3d_matrix(
        &self,
        reference_box: Option<&Rect<Au>>
    ) -> Result<(Transform3D<CSSFloat>, bool), ()> {
        let cast_3d_transform = |m: Transform3D<f64>| -> Transform3D<CSSFloat> {
            use std::{f32, f64};
            let cast = |v: f64| { v.min(f32::MAX as f64).max(f32::MIN as f64) as f32 };
            Transform3D::row_major(
                cast(m.m11), cast(m.m12), cast(m.m13), cast(m.m14),
                cast(m.m21), cast(m.m22), cast(m.m23), cast(m.m24),
                cast(m.m31), cast(m.m32), cast(m.m33), cast(m.m34),
                cast(m.m41), cast(m.m42), cast(m.m43), cast(m.m44),
            )
        };

        // We intentionally use Transform3D<f64> during computation to avoid error propagation
        // because using f32 to compute triangle functions (e.g. in create_rotation()) is not
        // accurate enough. In Gecko, we also use "double" to compute the triangle functions.
        // Therefore, let's use Transform3D<f64> during matrix computation and cast it into f32
        // in the end.
        let mut transform = Transform3D::<f64>::identity();
        let mut contain_3d = false;

        for operation in &self.0 {
            let matrix = operation.to_3d_matrix(reference_box)?;
            contain_3d |= operation.is_3d();
            transform = transform.pre_mul(&matrix);
        }

        Ok((cast_3d_transform(transform), contain_3d))
    }
}

/// Return the transform matrix from a perspective length.
#[inline]
pub fn create_perspective_matrix(d: CSSFloat) -> Transform3D<CSSFloat> {
    // TODO(gw): The transforms spec says that perspective length must
    // be positive. However, there is some confusion between the spec
    // and browser implementations as to handling the case of 0 for the
    // perspective value. Until the spec bug is resolved, at least ensure
    // that a provided perspective value of <= 0.0 doesn't cause panics
    // and behaves as it does in other browsers.
    // See https://lists.w3.org/Archives/Public/www-style/2016Jan/0020.html for more details.
    if d <= 0.0 {
        Transform3D::identity()
    } else {
        Transform3D::create_perspective(d)
    }
}

/// Return the normalized direction vector and its angle for Rotate3D.
pub fn get_normalized_vector_and_angle<T: Zero>(
    x: CSSFloat,
    y: CSSFloat,
    z: CSSFloat,
    angle: T,
) -> (CSSFloat, CSSFloat, CSSFloat, T) {
    use euclid::approxeq::ApproxEq;
    use values::computed::transform::DirectionVector;
    let vector = DirectionVector::new(x, y, z);
    if vector.square_length().approx_eq(&f32::zero()) {
        // https://www.w3.org/TR/css-transforms-1/#funcdef-rotate3d
        // A direction vector that cannot be normalized, such as [0, 0, 0], will cause the
        // rotation to not be applied, so we use identity matrix (i.e. rotate3d(0, 0, 1, 0)).
        (0., 0., 1., T::zero())
    } else {
        let vector = vector.robust_normalize();
        (vector.x, vector.y, vector.z, angle)
    }
}

#[derive(Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedZero, ToComputedValue, ToCss)]
/// A value of the `Rotate` property
///
/// <https://drafts.csswg.org/css-transforms-2/#individual-transforms>
pub enum Rotate<Number, Angle> {
    /// 'none'
    None,
    /// '<angle>'
    Rotate(Angle),
    /// '<number>{3} <angle>'
    Rotate3D(Number, Number, Number, Angle),
}

#[derive(Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedZero, ToComputedValue, ToCss)]
/// A value of the `Scale` property
///
/// <https://drafts.csswg.org/css-transforms-2/#individual-transforms>
pub enum Scale<Number> {
    /// 'none'
    None,
    /// '<number>'
    ScaleX(Number),
    /// '<number>{2}'
    Scale(Number, Number),
    /// '<number>{3}'
    Scale3D(Number, Number, Number),
}

#[derive(Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedZero, ToComputedValue, ToCss)]
/// A value of the `Translate` property
///
/// <https://drafts.csswg.org/css-transforms-2/#individual-transforms>
pub enum Translate<LengthOrPercentage, Length> {
    /// 'none'
    None,
    /// '<length-percentage>'
    TranslateX(LengthOrPercentage),
    /// '<length-percentage> <length-percentage>'
    Translate(LengthOrPercentage, LengthOrPercentage),
    /// '<length-percentage> <length-percentage> <length>'
    Translate3D(LengthOrPercentage, LengthOrPercentage, Length),
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub enum TransformStyle {
    #[cfg(feature = "servo")]
    Auto,
    Flat,
    #[css(keyword = "preserve-3d")]
    Preserve3d,
}
