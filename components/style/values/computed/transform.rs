/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to transformations.

use euclid::{Transform3D, Vector3D};
use num_traits::Zero;
use super::{CSSFloat, Either};
use values::animated::ToAnimatedZero;
use values::computed::{Angle, Integer, Length, LengthOrPercentage, Number, Percentage};
use values::computed::{LengthOrNumber, LengthOrPercentageOrNumber};
use values::generics::transform::{self, Matrix as GenericMatrix, Matrix3D as GenericMatrix3D};
use values::generics::transform::{Transform as GenericTransform, TransformOperation as GenericTransformOperation};
use values::generics::transform::TimingFunction as GenericTimingFunction;
use values::generics::transform::TransformOrigin as GenericTransformOrigin;

/// A single operation in a computed CSS `transform`
pub type TransformOperation = GenericTransformOperation<
    Angle,
    Number,
    Length,
    Integer,
    LengthOrNumber,
    LengthOrPercentage,
    LengthOrPercentageOrNumber,
>;
/// A computed CSS `transform`
pub type Transform = GenericTransform<TransformOperation>;

/// The computed value of a CSS `<transform-origin>`
pub type TransformOrigin = GenericTransformOrigin<LengthOrPercentage, LengthOrPercentage, Length>;

/// A computed timing function.
pub type TimingFunction = GenericTimingFunction<u32, Number>;

/// A vector to represent the direction vector (rotate axis) for Rotate3D.
pub type DirectionVector = Vector3D<CSSFloat>;

impl TransformOrigin {
    /// Returns the initial computed value for `transform-origin`.
    #[inline]
    pub fn initial_value() -> Self {
        Self::new(
            LengthOrPercentage::Percentage(Percentage(0.5)),
            LengthOrPercentage::Percentage(Percentage(0.5)),
            Length::new(0.),
        )
    }
}

/// computed value of matrix3d()
pub type Matrix3D = GenericMatrix3D<Number>;
/// computed value of matrix3d() in -moz-transform
pub type PrefixedMatrix3D = GenericMatrix3D<Number, LengthOrPercentageOrNumber, LengthOrNumber>;
/// computed value of matrix()
pub type Matrix = GenericMatrix<Number>;
/// computed value of matrix() in -moz-transform
pub type PrefixedMatrix = GenericMatrix<Number, LengthOrPercentageOrNumber>;

// we rustfmt_skip here because we want the matrices to look like
// matrices instead of being split across lines
#[cfg_attr(rustfmt, rustfmt_skip)]
impl Matrix3D {
    #[inline]
    /// Get an identity matrix
    pub fn identity() -> Self {
        Self {
            m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
            m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
            m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
            m41: 0., m42: 0., m43: 0., m44: 1.0
        }
    }

    /// Convert to a 2D Matrix
    pub fn into_2d(self) -> Result<Matrix, ()> {
        if self.m13 == 0. && self.m23 == 0. &&
           self.m31 == 0. && self.m32 == 0. &&
           self.m33 == 1. && self.m34 == 0. &&
           self.m14 == 0. && self.m24 == 0. &&
           self.m43 == 0. && self.m44 == 1. {
            Ok(Matrix {
                a: self.m11, c: self.m21, e: self.m41,
                b: self.m12, d: self.m22, f: self.m42,
            })
        } else {
            Err(())
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl PrefixedMatrix3D {
    #[inline]
    /// Get an identity matrix
    pub fn identity() -> Self {
        Self {
            m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
            m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
            m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
            m41: Either::First(0.), m42: Either::First(0.),
            m43: Either::First(Length::new(0.)), m44: 1.0
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Matrix {
    #[inline]
    /// Get an identity matrix
    pub fn identity() -> Self {
        Self {
            a: 1., c: 0., /* 0      0*/
            b: 0., d: 1., /* 0      0*/
            /* 0      0      1      0 */
            e: 0., f: 0., /* 0      1 */
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl From<Matrix> for Matrix3D {
    fn from(m: Matrix) -> Self {
        Self {
            m11: m.a, m12: m.b, m13: 0.0, m14: 0.0,
            m21: m.c, m22: m.d, m23: 0.0, m24: 0.0,
            m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
            m41: m.e, m42: m.f, m43: 0.0, m44: 1.0
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl PrefixedMatrix {
    #[inline]
    /// Get an identity matrix
    pub fn identity() -> Self {
        Self {
            a: 1.,                    c: 0., /*            0      0 */
            b: 0.,                    d: 1., /*            0      0 */
            /* 0                      0                    1      0 */
            e: Either::First(0.), f: Either::First(0.), /* 0      1 */
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl From<Transform3D<CSSFloat>> for Matrix3D {
    #[inline]
    fn from(m: Transform3D<CSSFloat>) -> Self {
        Matrix3D {
            m11: m.m11, m12: m.m12, m13: m.m13, m14: m.m14,
            m21: m.m21, m22: m.m22, m23: m.m23, m24: m.m24,
            m31: m.m31, m32: m.m32, m33: m.m33, m34: m.m34,
            m41: m.m41, m42: m.m42, m43: m.m43, m44: m.m44
        }
    }
}

impl TransformOperation {
    /// Convert to a Translate3D.
    ///
    /// Must be called on a Translate function
    pub fn to_translate_3d(&self) -> Self {
        match *self {
            GenericTransformOperation::Translate3D(..) => self.clone(),
            GenericTransformOperation::TranslateX(ref x) |
            GenericTransformOperation::Translate(ref x, None) => {
                GenericTransformOperation::Translate3D(x.clone(), LengthOrPercentage::zero(), Length::zero())
            },
            GenericTransformOperation::Translate(ref x, Some(ref y)) => {
                GenericTransformOperation::Translate3D(x.clone(), y.clone(), Length::zero())
            },
            GenericTransformOperation::TranslateY(ref y) => {
                GenericTransformOperation::Translate3D(LengthOrPercentage::zero(), y.clone(), Length::zero())
            },
            GenericTransformOperation::TranslateZ(ref z) => {
                GenericTransformOperation::Translate3D(
                    LengthOrPercentage::zero(),
                    LengthOrPercentage::zero(),
                    z.clone(),
                )
            },
            _ => unreachable!(),
        }
    }
    /// Convert to a Scale3D.
    ///
    /// Must be called on a Scale function
    pub fn to_scale_3d(&self) -> Self {
        match *self {
            GenericTransformOperation::Scale3D(..) => self.clone(),
            GenericTransformOperation::Scale(s, None) => GenericTransformOperation::Scale3D(s, s, 1.),
            GenericTransformOperation::Scale(x, Some(y)) => GenericTransformOperation::Scale3D(x, y, 1.),
            GenericTransformOperation::ScaleX(x) => GenericTransformOperation::Scale3D(x, 1., 1.),
            GenericTransformOperation::ScaleY(y) => GenericTransformOperation::Scale3D(1., y, 1.),
            GenericTransformOperation::ScaleZ(z) => GenericTransformOperation::Scale3D(1., 1., z),
            _ => unreachable!(),
        }
    }
}

/// Build an equivalent 'identity transform function list' based
/// on an existing transform list.
/// http://dev.w3.org/csswg/css-transforms/#none-transform-animation
impl ToAnimatedZero for TransformOperation {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            GenericTransformOperation::Matrix3D(..) => Ok(GenericTransformOperation::Matrix3D(Matrix3D::identity())),
            GenericTransformOperation::PrefixedMatrix3D(..) => {
                Ok(GenericTransformOperation::PrefixedMatrix3D(
                    PrefixedMatrix3D::identity(),
                ))
            },
            GenericTransformOperation::Matrix(..) => Ok(GenericTransformOperation::Matrix(Matrix::identity())),
            GenericTransformOperation::PrefixedMatrix(..) => {
                Ok(GenericTransformOperation::PrefixedMatrix(
                    PrefixedMatrix::identity(),
                ))
            },
            GenericTransformOperation::Skew(sx, sy) => {
                Ok(GenericTransformOperation::Skew(
                    sx.to_animated_zero()?,
                    sy.to_animated_zero()?,
                ))
            },
            GenericTransformOperation::SkewX(s) => Ok(GenericTransformOperation::SkewX(s.to_animated_zero()?)),
            GenericTransformOperation::SkewY(s) => Ok(GenericTransformOperation::SkewY(s.to_animated_zero()?)),
            GenericTransformOperation::Translate3D(ref tx, ref ty, ref tz) => {
                Ok(GenericTransformOperation::Translate3D(
                    tx.to_animated_zero()?,
                    ty.to_animated_zero()?,
                    tz.to_animated_zero()?,
                ))
            },
            GenericTransformOperation::Translate(ref tx, ref ty) => {
                Ok(GenericTransformOperation::Translate(
                    tx.to_animated_zero()?,
                    ty.to_animated_zero()?,
                ))
            },
            GenericTransformOperation::TranslateX(ref t) => {
                Ok(GenericTransformOperation::TranslateX(t.to_animated_zero()?))
            },
            GenericTransformOperation::TranslateY(ref t) => {
                Ok(GenericTransformOperation::TranslateY(t.to_animated_zero()?))
            },
            GenericTransformOperation::TranslateZ(ref t) => {
                Ok(GenericTransformOperation::TranslateZ(t.to_animated_zero()?))
            },
            GenericTransformOperation::Scale3D(..) => Ok(GenericTransformOperation::Scale3D(1.0, 1.0, 1.0)),
            GenericTransformOperation::Scale(_, _) => Ok(GenericTransformOperation::Scale(1.0, Some(1.0))),
            GenericTransformOperation::ScaleX(..) => Ok(GenericTransformOperation::ScaleX(1.0)),
            GenericTransformOperation::ScaleY(..) => Ok(GenericTransformOperation::ScaleY(1.0)),
            GenericTransformOperation::ScaleZ(..) => Ok(GenericTransformOperation::ScaleZ(1.0)),
            GenericTransformOperation::Rotate3D(x, y, z, a) => {
                let (x, y, z, _) = transform::get_normalized_vector_and_angle(x, y, z, a);
                Ok(GenericTransformOperation::Rotate3D(x, y, z, Angle::zero()))
            },
            GenericTransformOperation::RotateX(_) => Ok(GenericTransformOperation::RotateX(Angle::zero())),
            GenericTransformOperation::RotateY(_) => Ok(GenericTransformOperation::RotateY(Angle::zero())),
            GenericTransformOperation::RotateZ(_) => Ok(GenericTransformOperation::RotateZ(Angle::zero())),
            GenericTransformOperation::Rotate(_) => Ok(GenericTransformOperation::Rotate(Angle::zero())),
            GenericTransformOperation::Perspective(..) |
            GenericTransformOperation::AccumulateMatrix {
                ..
            } |
            GenericTransformOperation::InterpolateMatrix {
                ..
            } => {
                // Perspective: We convert a perspective function into an equivalent
                //     ComputedMatrix, and then decompose/interpolate/recompose these matrices.
                // AccumulateMatrix/InterpolateMatrix: We do interpolation on
                //     AccumulateMatrix/InterpolateMatrix by reading it as a ComputedMatrix
                //     (with layout information), and then do matrix interpolation.
                //
                // Therefore, we use an identity matrix to represent the identity transform list.
                // http://dev.w3.org/csswg/css-transforms/#identity-transform-function
                Ok(GenericTransformOperation::Matrix3D(Matrix3D::identity()))
            },
        }
    }
}


impl ToAnimatedZero for Transform {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(GenericTransform(self.0
            .iter()
            .map(|op| op.to_animated_zero())
            .collect::<Result<Vec<_>, _>>()?))
    }
}
