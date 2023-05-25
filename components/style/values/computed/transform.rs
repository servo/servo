/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to transformations.

use super::CSSFloat;
use crate::values::animated::transform::{Perspective, Scale3D, Translate3D};
use crate::values::animated::ToAnimatedZero;
use crate::values::computed::{Angle, Integer, Length, LengthPercentage, Number, Percentage};
use crate::values::generics::transform as generic;
use crate::Zero;
use euclid::default::{Transform3D, Vector3D};

pub use crate::values::generics::transform::TransformStyle;

/// A single operation in a computed CSS `transform`
pub type TransformOperation =
    generic::GenericTransformOperation<Angle, Number, Length, Integer, LengthPercentage>;
/// A computed CSS `transform`
pub type Transform = generic::GenericTransform<TransformOperation>;

/// The computed value of a CSS `<transform-origin>`
pub type TransformOrigin =
    generic::GenericTransformOrigin<LengthPercentage, LengthPercentage, Length>;

/// The computed value of the `perspective()` transform function.
pub type PerspectiveFunction = generic::PerspectiveFunction<Length>;

/// A vector to represent the direction vector (rotate axis) for Rotate3D.
pub type DirectionVector = Vector3D<CSSFloat>;

impl TransformOrigin {
    /// Returns the initial computed value for `transform-origin`.
    #[inline]
    pub fn initial_value() -> Self {
        Self::new(
            LengthPercentage::new_percent(Percentage(0.5)),
            LengthPercentage::new_percent(Percentage(0.5)),
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
    /// Get an identity matrix
    #[inline]
    pub fn identity() -> Self {
        Self {
            m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
            m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
            m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
            m41: 0., m42: 0., m43: 0., m44: 1.0
        }
    }

    /// Convert to a 2D Matrix
    #[inline]
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

    /// Return true if this has 3D components.
    #[inline]
    pub fn is_3d(&self) -> bool {
        self.m13 != 0.0 || self.m14 != 0.0 ||
        self.m23 != 0.0 || self.m24 != 0.0 ||
        self.m31 != 0.0 || self.m32 != 0.0 ||
        self.m33 != 1.0 || self.m34 != 0.0 ||
        self.m43 != 0.0 || self.m44 != 1.0
    }

    /// Return determinant value.
    #[inline]
    pub fn determinant(&self) -> CSSFloat {
        self.m14 * self.m23 * self.m32 * self.m41 -
        self.m13 * self.m24 * self.m32 * self.m41 -
        self.m14 * self.m22 * self.m33 * self.m41 +
        self.m12 * self.m24 * self.m33 * self.m41 +
        self.m13 * self.m22 * self.m34 * self.m41 -
        self.m12 * self.m23 * self.m34 * self.m41 -
        self.m14 * self.m23 * self.m31 * self.m42 +
        self.m13 * self.m24 * self.m31 * self.m42 +
        self.m14 * self.m21 * self.m33 * self.m42 -
        self.m11 * self.m24 * self.m33 * self.m42 -
        self.m13 * self.m21 * self.m34 * self.m42 +
        self.m11 * self.m23 * self.m34 * self.m42 +
        self.m14 * self.m22 * self.m31 * self.m43 -
        self.m12 * self.m24 * self.m31 * self.m43 -
        self.m14 * self.m21 * self.m32 * self.m43 +
        self.m11 * self.m24 * self.m32 * self.m43 +
        self.m12 * self.m21 * self.m34 * self.m43 -
        self.m11 * self.m22 * self.m34 * self.m43 -
        self.m13 * self.m22 * self.m31 * self.m44 +
        self.m12 * self.m23 * self.m31 * self.m44 +
        self.m13 * self.m21 * self.m32 * self.m44 -
        self.m11 * self.m23 * self.m32 * self.m44 -
        self.m12 * self.m21 * self.m33 * self.m44 +
        self.m11 * self.m22 * self.m33 * self.m44
    }

    /// Transpose a matrix.
    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            m11: self.m11, m12: self.m21, m13: self.m31, m14: self.m41,
            m21: self.m12, m22: self.m22, m23: self.m32, m24: self.m42,
            m31: self.m13, m32: self.m23, m33: self.m33, m34: self.m43,
            m41: self.m14, m42: self.m24, m43: self.m34, m44: self.m44,
        }
    }

    /// Return inverse matrix.
    pub fn inverse(&self) -> Result<Matrix3D, ()> {
        let mut det = self.determinant();

        if det == 0.0 {
            return Err(());
        }

        det = 1.0 / det;
        let x = Matrix3D {
            m11: det *
            (self.m23 * self.m34 * self.m42 - self.m24 * self.m33 * self.m42 +
             self.m24 * self.m32 * self.m43 - self.m22 * self.m34 * self.m43 -
             self.m23 * self.m32 * self.m44 + self.m22 * self.m33 * self.m44),
            m12: det *
            (self.m14 * self.m33 * self.m42 - self.m13 * self.m34 * self.m42 -
             self.m14 * self.m32 * self.m43 + self.m12 * self.m34 * self.m43 +
             self.m13 * self.m32 * self.m44 - self.m12 * self.m33 * self.m44),
            m13: det *
            (self.m13 * self.m24 * self.m42 - self.m14 * self.m23 * self.m42 +
             self.m14 * self.m22 * self.m43 - self.m12 * self.m24 * self.m43 -
             self.m13 * self.m22 * self.m44 + self.m12 * self.m23 * self.m44),
            m14: det *
            (self.m14 * self.m23 * self.m32 - self.m13 * self.m24 * self.m32 -
             self.m14 * self.m22 * self.m33 + self.m12 * self.m24 * self.m33 +
             self.m13 * self.m22 * self.m34 - self.m12 * self.m23 * self.m34),
            m21: det *
            (self.m24 * self.m33 * self.m41 - self.m23 * self.m34 * self.m41 -
             self.m24 * self.m31 * self.m43 + self.m21 * self.m34 * self.m43 +
             self.m23 * self.m31 * self.m44 - self.m21 * self.m33 * self.m44),
            m22: det *
            (self.m13 * self.m34 * self.m41 - self.m14 * self.m33 * self.m41 +
             self.m14 * self.m31 * self.m43 - self.m11 * self.m34 * self.m43 -
             self.m13 * self.m31 * self.m44 + self.m11 * self.m33 * self.m44),
            m23: det *
            (self.m14 * self.m23 * self.m41 - self.m13 * self.m24 * self.m41 -
             self.m14 * self.m21 * self.m43 + self.m11 * self.m24 * self.m43 +
             self.m13 * self.m21 * self.m44 - self.m11 * self.m23 * self.m44),
            m24: det *
            (self.m13 * self.m24 * self.m31 - self.m14 * self.m23 * self.m31 +
             self.m14 * self.m21 * self.m33 - self.m11 * self.m24 * self.m33 -
             self.m13 * self.m21 * self.m34 + self.m11 * self.m23 * self.m34),
            m31: det *
            (self.m22 * self.m34 * self.m41 - self.m24 * self.m32 * self.m41 +
             self.m24 * self.m31 * self.m42 - self.m21 * self.m34 * self.m42 -
             self.m22 * self.m31 * self.m44 + self.m21 * self.m32 * self.m44),
            m32: det *
            (self.m14 * self.m32 * self.m41 - self.m12 * self.m34 * self.m41 -
             self.m14 * self.m31 * self.m42 + self.m11 * self.m34 * self.m42 +
             self.m12 * self.m31 * self.m44 - self.m11 * self.m32 * self.m44),
            m33: det *
            (self.m12 * self.m24 * self.m41 - self.m14 * self.m22 * self.m41 +
             self.m14 * self.m21 * self.m42 - self.m11 * self.m24 * self.m42 -
             self.m12 * self.m21 * self.m44 + self.m11 * self.m22 * self.m44),
            m34: det *
            (self.m14 * self.m22 * self.m31 - self.m12 * self.m24 * self.m31 -
             self.m14 * self.m21 * self.m32 + self.m11 * self.m24 * self.m32 +
             self.m12 * self.m21 * self.m34 - self.m11 * self.m22 * self.m34),
            m41: det *
            (self.m23 * self.m32 * self.m41 - self.m22 * self.m33 * self.m41 -
             self.m23 * self.m31 * self.m42 + self.m21 * self.m33 * self.m42 +
             self.m22 * self.m31 * self.m43 - self.m21 * self.m32 * self.m43),
            m42: det *
            (self.m12 * self.m33 * self.m41 - self.m13 * self.m32 * self.m41 +
             self.m13 * self.m31 * self.m42 - self.m11 * self.m33 * self.m42 -
             self.m12 * self.m31 * self.m43 + self.m11 * self.m32 * self.m43),
            m43: det *
            (self.m13 * self.m22 * self.m41 - self.m12 * self.m23 * self.m41 -
             self.m13 * self.m21 * self.m42 + self.m11 * self.m23 * self.m42 +
             self.m12 * self.m21 * self.m43 - self.m11 * self.m22 * self.m43),
            m44: det *
            (self.m12 * self.m23 * self.m31 - self.m13 * self.m22 * self.m31 +
             self.m13 * self.m21 * self.m32 - self.m11 * self.m23 * self.m32 -
             self.m12 * self.m21 * self.m33 + self.m11 * self.m22 * self.m33),
        };

        Ok(x)
    }

    /// Multiply `pin * self`.
    #[inline]
    pub fn pre_mul_point4(&self, pin: &[f32; 4]) -> [f32; 4] {
        [
            pin[0] * self.m11 + pin[1] * self.m21 + pin[2] * self.m31 + pin[3] * self.m41,
            pin[0] * self.m12 + pin[1] * self.m22 + pin[2] * self.m32 + pin[3] * self.m42,
            pin[0] * self.m13 + pin[1] * self.m23 + pin[2] * self.m33 + pin[3] * self.m43,
            pin[0] * self.m14 + pin[1] * self.m24 + pin[2] * self.m34 + pin[3] * self.m44,
        ]
    }

    /// Return the multiplication of two 4x4 matrices.
    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        Matrix3D {
            m11: self.m11 * other.m11 + self.m12 * other.m21 +
                 self.m13 * other.m31 + self.m14 * other.m41,
            m12: self.m11 * other.m12 + self.m12 * other.m22 +
                 self.m13 * other.m32 + self.m14 * other.m42,
            m13: self.m11 * other.m13 + self.m12 * other.m23 +
                 self.m13 * other.m33 + self.m14 * other.m43,
            m14: self.m11 * other.m14 + self.m12 * other.m24 +
                 self.m13 * other.m34 + self.m14 * other.m44,
            m21: self.m21 * other.m11 + self.m22 * other.m21 +
                 self.m23 * other.m31 + self.m24 * other.m41,
            m22: self.m21 * other.m12 + self.m22 * other.m22 +
                 self.m23 * other.m32 + self.m24 * other.m42,
            m23: self.m21 * other.m13 + self.m22 * other.m23 +
                 self.m23 * other.m33 + self.m24 * other.m43,
            m24: self.m21 * other.m14 + self.m22 * other.m24 +
                 self.m23 * other.m34 + self.m24 * other.m44,
            m31: self.m31 * other.m11 + self.m32 * other.m21 +
                 self.m33 * other.m31 + self.m34 * other.m41,
            m32: self.m31 * other.m12 + self.m32 * other.m22 +
                 self.m33 * other.m32 + self.m34 * other.m42,
            m33: self.m31 * other.m13 + self.m32 * other.m23 +
                 self.m33 * other.m33 + self.m34 * other.m43,
            m34: self.m31 * other.m14 + self.m32 * other.m24 +
                 self.m33 * other.m34 + self.m34 * other.m44,
            m41: self.m41 * other.m11 + self.m42 * other.m21 +
                 self.m43 * other.m31 + self.m44 * other.m41,
            m42: self.m41 * other.m12 + self.m42 * other.m22 +
                 self.m43 * other.m32 + self.m44 * other.m42,
            m43: self.m41 * other.m13 + self.m42 * other.m23 +
                 self.m43 * other.m33 + self.m44 * other.m43,
            m44: self.m41 * other.m14 + self.m42 * other.m24 +
                 self.m43 * other.m34 + self.m44 * other.m44,
        }
    }

    /// Scale the matrix by a factor.
    #[inline]
    pub fn scale_by_factor(&mut self, scaling_factor: CSSFloat) {
        self.m11 *= scaling_factor;
        self.m12 *= scaling_factor;
        self.m13 *= scaling_factor;
        self.m14 *= scaling_factor;
        self.m21 *= scaling_factor;
        self.m22 *= scaling_factor;
        self.m23 *= scaling_factor;
        self.m24 *= scaling_factor;
        self.m31 *= scaling_factor;
        self.m32 *= scaling_factor;
        self.m33 *= scaling_factor;
        self.m34 *= scaling_factor;
        self.m41 *= scaling_factor;
        self.m42 *= scaling_factor;
        self.m43 *= scaling_factor;
        self.m44 *= scaling_factor;
    }

    /// Return the matrix 3x3 part (top-left corner).
    /// This is used by retrieving the scale and shear factors
    /// during decomposing a 3d matrix.
    #[inline]
    pub fn get_matrix_3x3_part(&self) -> [[f32; 3]; 3] {
        [
            [ self.m11, self.m12, self.m13 ],
            [ self.m21, self.m22, self.m23 ],
            [ self.m31, self.m32, self.m33 ],
        ]
    }

    /// Set perspective on the matrix.
    #[inline]
    pub fn set_perspective(&mut self, perspective: &Perspective) {
        self.m14 = perspective.0;
        self.m24 = perspective.1;
        self.m34 = perspective.2;
        self.m44 = perspective.3;
    }

    /// Apply translate on the matrix.
    #[inline]
    pub fn apply_translate(&mut self, translate: &Translate3D) {
        self.m41 += translate.0 * self.m11 + translate.1 * self.m21 + translate.2 * self.m31;
        self.m42 += translate.0 * self.m12 + translate.1 * self.m22 + translate.2 * self.m32;
        self.m43 += translate.0 * self.m13 + translate.1 * self.m23 + translate.2 * self.m33;
        self.m44 += translate.0 * self.m14 + translate.1 * self.m24 + translate.2 * self.m34;
    }

    /// Apply scale on the matrix.
    #[inline]
    pub fn apply_scale(&mut self, scale: &Scale3D) {
        self.m11 *= scale.0;
        self.m12 *= scale.0;
        self.m13 *= scale.0;
        self.m14 *= scale.0;
        self.m21 *= scale.1;
        self.m22 *= scale.1;
        self.m23 *= scale.1;
        self.m24 *= scale.1;
        self.m31 *= scale.2;
        self.m32 *= scale.2;
        self.m33 *= scale.2;
        self.m34 *= scale.2;
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
            generic::TransformOperation::TranslateX(ref x) => {
                generic::TransformOperation::Translate3D(
                    x.clone(),
                    LengthPercentage::zero(),
                    Length::zero(),
                )
            },
            generic::TransformOperation::Translate(ref x, ref y) => {
                generic::TransformOperation::Translate3D(x.clone(), y.clone(), Length::zero())
            },
            generic::TransformOperation::TranslateY(ref y) => {
                generic::TransformOperation::Translate3D(
                    LengthPercentage::zero(),
                    y.clone(),
                    Length::zero(),
                )
            },
            generic::TransformOperation::TranslateZ(ref z) => {
                generic::TransformOperation::Translate3D(
                    LengthPercentage::zero(),
                    LengthPercentage::zero(),
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
            },
            generic::TransformOperation::RotateX(ref angle) => {
                generic::TransformOperation::Rotate3D(1., 0., 0., angle.clone())
            },
            generic::TransformOperation::RotateY(ref angle) => {
                generic::TransformOperation::Rotate3D(0., 1., 0., angle.clone())
            },
            _ => unreachable!(),
        }
    }

    /// Convert to a Scale3D.
    ///
    /// Must be called on a Scale function
    pub fn to_scale_3d(&self) -> Self {
        match *self {
            generic::TransformOperation::Scale3D(..) => self.clone(),
            generic::TransformOperation::Scale(x, y) => {
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
                Ok(generic::TransformOperation::Scale(1.0, 1.0))
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
            generic::TransformOperation::Perspective(_) => Ok(
                generic::TransformOperation::Perspective(generic::PerspectiveFunction::None)
            ),
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
        Ok(generic::Transform(
            self.0
                .iter()
                .map(|op| op.to_animated_zero())
                .collect::<Result<crate::OwnedSlice<_>, _>>()?,
        ))
    }
}

/// A computed CSS `rotate`
pub type Rotate = generic::GenericRotate<Number, Angle>;

/// A computed CSS `translate`
pub type Translate = generic::GenericTranslate<LengthPercentage, Length>;

/// A computed CSS `scale`
pub type Scale = generic::GenericScale<Number>;
