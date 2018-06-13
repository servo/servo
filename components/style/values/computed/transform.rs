/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to transformations.

use euclid::{Transform3D, Vector3D};
use num_traits::Zero;
use super::CSSFloat;
use values::animated::ToAnimatedZero;
use values::computed::{Angle, Integer, Length, LengthOrPercentage, Number, Percentage};
use values::generics::transform as generic;

pub use values::generics::transform::TransformStyle;

/// A single operation in a computed CSS `transform`
pub type TransformOperation =
    generic::TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage>;
/// A computed CSS `transform`
pub type Transform = generic::Transform<TransformOperation>;

/// The computed value of a CSS `<transform-origin>`
pub type TransformOrigin = generic::TransformOrigin<LengthOrPercentage, LengthOrPercentage, Length>;

/// A computed timing function.
pub type TimingFunction = generic::TimingFunction<u32, Number>;

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
pub type Matrix3D = generic::Matrix3D<Number>;
/// computed value of matrix()
pub type Matrix = generic::Matrix<Number>;

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
            generic::TransformOperation::Translate3D(..) => self.clone(),
            generic::TransformOperation::TranslateX(ref x) |
            generic::TransformOperation::Translate(ref x, None) => {
                generic::TransformOperation::Translate3D(
                    x.clone(),
                    LengthOrPercentage::zero(),
                    Length::zero(),
                )
            },
            generic::TransformOperation::Translate(ref x, Some(ref y)) => {
                generic::TransformOperation::Translate3D(x.clone(), y.clone(), Length::zero())
            },
            generic::TransformOperation::TranslateY(ref y) => {
                generic::TransformOperation::Translate3D(
                    LengthOrPercentage::zero(),
                    y.clone(),
                    Length::zero(),
                )
            },
            generic::TransformOperation::TranslateZ(ref z) => {
                generic::TransformOperation::Translate3D(
                    LengthOrPercentage::zero(),
                    LengthOrPercentage::zero(),
                    z.clone(),
                )
            },
            _ => unreachable!(),
        }
    }

    /// Convert to a Rotate3D.
    ///
    /// Must be called on a Rotate function.
    pub fn to_rotate_3d(&self) -> Self {
        match *self {
            generic::TransformOperation::Rotate3D(..) => self.clone(),
            generic::TransformOperation::RotateZ(ref angle) |
            generic::TransformOperation::Rotate(ref angle) => {
                generic::TransformOperation::Rotate3D(0., 0., 1., angle.clone())
            }
            generic::TransformOperation::RotateX(ref angle) => {
                generic::TransformOperation::Rotate3D(1., 0., 0., angle.clone())
            }
            generic::TransformOperation::RotateY(ref angle) => {
                generic::TransformOperation::Rotate3D(0., 1., 0., angle.clone())
            }
            _ => unreachable!(),
        }
    }

    /// Convert to a Scale3D.
    ///
    /// Must be called on a Scale function
    pub fn to_scale_3d(&self) -> Self {
        match *self {
            generic::TransformOperation::Scale3D(..) => self.clone(),
            generic::TransformOperation::Scale(s, None) => {
                generic::TransformOperation::Scale3D(s, s, 1.)
            },
            generic::TransformOperation::Scale(x, Some(y)) => {
                generic::TransformOperation::Scale3D(x, y, 1.)
            },
            generic::TransformOperation::ScaleX(x) => {
                generic::TransformOperation::Scale3D(x, 1., 1.)
            },
            generic::TransformOperation::ScaleY(y) => {
                generic::TransformOperation::Scale3D(1., y, 1.)
            },
            generic::TransformOperation::ScaleZ(z) => {
                generic::TransformOperation::Scale3D(1., 1., z)
            },
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
            generic::TransformOperation::Matrix3D(..) => {
                Ok(generic::TransformOperation::Matrix3D(Matrix3D::identity()))
            },
            generic::TransformOperation::Matrix(..) => {
                Ok(generic::TransformOperation::Matrix(Matrix::identity()))
            },
            generic::TransformOperation::Skew(sx, sy) => Ok(generic::TransformOperation::Skew(
                sx.to_animated_zero()?,
                sy.to_animated_zero()?,
            )),
            generic::TransformOperation::SkewX(s) => {
                Ok(generic::TransformOperation::SkewX(s.to_animated_zero()?))
            },
            generic::TransformOperation::SkewY(s) => {
                Ok(generic::TransformOperation::SkewY(s.to_animated_zero()?))
            },
            generic::TransformOperation::Translate3D(ref tx, ref ty, ref tz) => {
                Ok(generic::TransformOperation::Translate3D(
                    tx.to_animated_zero()?,
                    ty.to_animated_zero()?,
                    tz.to_animated_zero()?,
                ))
            },
            generic::TransformOperation::Translate(ref tx, ref ty) => {
                Ok(generic::TransformOperation::Translate(
                    tx.to_animated_zero()?,
                    ty.to_animated_zero()?,
                ))
            },
            generic::TransformOperation::TranslateX(ref t) => Ok(
                generic::TransformOperation::TranslateX(t.to_animated_zero()?),
            ),
            generic::TransformOperation::TranslateY(ref t) => Ok(
                generic::TransformOperation::TranslateY(t.to_animated_zero()?),
            ),
            generic::TransformOperation::TranslateZ(ref t) => Ok(
                generic::TransformOperation::TranslateZ(t.to_animated_zero()?),
            ),
            generic::TransformOperation::Scale3D(..) => {
                Ok(generic::TransformOperation::Scale3D(1.0, 1.0, 1.0))
            },
            generic::TransformOperation::Scale(_, _) => {
                Ok(generic::TransformOperation::Scale(1.0, Some(1.0)))
            },
            generic::TransformOperation::ScaleX(..) => Ok(generic::TransformOperation::ScaleX(1.0)),
            generic::TransformOperation::ScaleY(..) => Ok(generic::TransformOperation::ScaleY(1.0)),
            generic::TransformOperation::ScaleZ(..) => Ok(generic::TransformOperation::ScaleZ(1.0)),
            generic::TransformOperation::Rotate3D(x, y, z, a) => {
                let (x, y, z, _) = generic::get_normalized_vector_and_angle(x, y, z, a);
                Ok(generic::TransformOperation::Rotate3D(
                    x,
                    y,
                    z,
                    Angle::zero(),
                ))
            },
            generic::TransformOperation::RotateX(_) => {
                Ok(generic::TransformOperation::RotateX(Angle::zero()))
            },
            generic::TransformOperation::RotateY(_) => {
                Ok(generic::TransformOperation::RotateY(Angle::zero()))
            },
            generic::TransformOperation::RotateZ(_) => {
                Ok(generic::TransformOperation::RotateZ(Angle::zero()))
            },
            generic::TransformOperation::Rotate(_) => {
                Ok(generic::TransformOperation::Rotate(Angle::zero()))
            },
            generic::TransformOperation::Perspective(ref l) => {
                Ok(generic::TransformOperation::Perspective(l.to_animated_zero()?))
            },
            generic::TransformOperation::AccumulateMatrix { .. } |
            generic::TransformOperation::InterpolateMatrix { .. } => {
                // AccumulateMatrix/InterpolateMatrix: We do interpolation on
                //     AccumulateMatrix/InterpolateMatrix by reading it as a ComputedMatrix
                //     (with layout information), and then do matrix interpolation.
                //
                // Therefore, we use an identity matrix to represent the identity transform list.
                // http://dev.w3.org/csswg/css-transforms/#identity-transform-function
                Ok(generic::TransformOperation::Matrix3D(Matrix3D::identity()))
            },
        }
    }
}

impl ToAnimatedZero for Transform {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(generic::Transform(self.0
            .iter()
            .map(|op| op.to_animated_zero())
            .collect::<Result<Vec<_>, _>>()?))
    }
}

/// A computed CSS `rotate`
pub type Rotate = generic::Rotate<Number, Angle>;

impl Rotate {
    /// Convert TransformOperation to Rotate.
    pub fn to_transform_operation(&self) -> Option<TransformOperation> {
        match *self {
            generic::Rotate::None => None,
            generic::Rotate::Rotate(angle) => Some(generic::TransformOperation::Rotate(angle)),
            generic::Rotate::Rotate3D(rx, ry, rz, angle) => {
                Some(generic::TransformOperation::Rotate3D(rx, ry, rz, angle))
            },
        }
    }

    /// Convert Rotate to TransformOperation.
    pub fn from_transform_operation(operation: &TransformOperation) -> Rotate {
        match *operation {
            generic::TransformOperation::Rotate(angle) => generic::Rotate::Rotate(angle),
            generic::TransformOperation::Rotate3D(rx, ry, rz, angle) => {
                generic::Rotate::Rotate3D(rx, ry, rz, angle)
            },
            _ => unreachable!("Found unexpected value for rotate property"),
        }
    }
}

/// A computed CSS `translate`
pub type Translate = generic::Translate<LengthOrPercentage, Length>;

impl Translate {
    /// Convert TransformOperation to Translate.
    pub fn to_transform_operation(&self) -> Option<TransformOperation> {
        match *self {
            generic::Translate::None => None,
            generic::Translate::TranslateX(tx) => Some(generic::TransformOperation::TranslateX(tx)),
            generic::Translate::Translate(tx, ty) => {
                Some(generic::TransformOperation::Translate(tx, Some(ty)))
            },
            generic::Translate::Translate3D(tx, ty, tz) => {
                Some(generic::TransformOperation::Translate3D(tx, ty, tz))
            },
        }
    }

    /// Convert Translate to TransformOperation.
    pub fn from_transform_operation(operation: &TransformOperation) -> Translate {
        match *operation {
            generic::TransformOperation::TranslateX(tx) => generic::Translate::TranslateX(tx),
            generic::TransformOperation::Translate(tx, Some(ty)) => {
                generic::Translate::Translate(tx, ty)
            },
            generic::TransformOperation::Translate3D(tx, ty, tz) => {
                generic::Translate::Translate3D(tx, ty, tz)
            },
            _ => unreachable!("Found unexpected value for translate"),
        }
    }
}

/// A computed CSS `scale`
pub type Scale = generic::Scale<Number>;

impl Scale {
    /// Convert TransformOperation to Scale.
    pub fn to_transform_operation(&self) -> Option<TransformOperation> {
        match *self {
            generic::Scale::None => None,
            generic::Scale::ScaleX(sx) => Some(generic::TransformOperation::ScaleX(sx)),
            generic::Scale::Scale(sx, sy) => Some(generic::TransformOperation::Scale(sx, Some(sy))),
            generic::Scale::Scale3D(sx, sy, sz) => {
                Some(generic::TransformOperation::Scale3D(sx, sy, sz))
            },
        }
    }

    /// Convert Scale to TransformOperation.
    pub fn from_transform_operation(operation: &TransformOperation) -> Scale {
        match *operation {
            generic::TransformOperation::ScaleX(sx) => generic::Scale::ScaleX(sx),
            generic::TransformOperation::Scale(sx, Some(sy)) => generic::Scale::Scale(sx, sy),
            generic::TransformOperation::Scale3D(sx, sy, sz) => generic::Scale::Scale3D(sx, sy, sz),
            _ => unreachable!("Found unexpected value for scale"),
        }
    }
}
