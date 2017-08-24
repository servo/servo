/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to transformations.

use app_units::Au;
use euclid::{Rect, Transform3D, Vector3D};
use properties::longhands::transform::computed_value::{ComputedOperation, ComputedMatrix};
use properties::longhands::transform::computed_value::T as TransformList;
use std::f32;
use super::CSSFloat;
use values::computed::{Angle, Length, LengthOrPercentage, Number, Percentage};
use values::generics::transform::TimingFunction as GenericTimingFunction;
use values::generics::transform::TransformOrigin as GenericTransformOrigin;

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
            Length::from_px(0),
        )
    }
}

impl From<ComputedMatrix> for Transform3D<CSSFloat> {
    #[inline]
    fn from(m: ComputedMatrix) -> Self {
        Transform3D::row_major(
            m.m11, m.m12, m.m13, m.m14,
            m.m21, m.m22, m.m23, m.m24,
            m.m31, m.m32, m.m33, m.m34,
            m.m41, m.m42, m.m43, m.m44)
    }
}

impl From<Transform3D<CSSFloat>> for ComputedMatrix {
    #[inline]
    fn from(m: Transform3D<CSSFloat>) -> Self {
        ComputedMatrix {
            m11: m.m11, m12: m.m12, m13: m.m13, m14: m.m14,
            m21: m.m21, m22: m.m22, m23: m.m23, m24: m.m24,
            m31: m.m31, m32: m.m32, m33: m.m33, m34: m.m34,
            m41: m.m41, m42: m.m42, m43: m.m43, m44: m.m44
        }
    }
}

impl TransformList {
    /// Return the equivalent 3d matrix of this transform list.
    /// If |reference_box| is None, we will drop the percent part from translate because
    /// we can resolve it without the layout info.
    pub fn to_transform_3d_matrix(&self, reference_box: Option<&Rect<Au>>)
                                  -> Option<Transform3D<CSSFloat>> {
        let mut transform = Transform3D::identity();
        let list = match self.0.as_ref() {
            Some(list) => list,
            None => return None,
        };

        let extract_pixel_length = |lop: &LengthOrPercentage| {
            match *lop {
                LengthOrPercentage::Length(au) => au.to_f32_px(),
                LengthOrPercentage::Percentage(_) => 0.,
                LengthOrPercentage::Calc(calc) => calc.length().to_f32_px(),
            }
        };

        for operation in list {
            let matrix = match *operation {
                ComputedOperation::Rotate(ax, ay, az, theta) => {
                    let theta = Angle::from_radians(2.0f32 * f32::consts::PI - theta.radians());
                    let (ax, ay, az, theta) =
                        Self::get_normalized_vector_and_angle(ax, ay, az, theta);
                    Transform3D::create_rotation(ax, ay, az, theta.into())
                }
                ComputedOperation::Perspective(d) => {
                    Self::create_perspective_matrix(d)
                }
                ComputedOperation::Scale(sx, sy, sz) => {
                    Transform3D::create_scale(sx, sy, sz)
                }
                ComputedOperation::Translate(tx, ty, tz) => {
                    let (tx, ty) = match reference_box {
                        Some(relative_border_box) => {
                            (tx.to_used_value(relative_border_box.size.width).to_f32_px(),
                             ty.to_used_value(relative_border_box.size.height).to_f32_px())
                        },
                        None => {
                            // If we don't have reference box, we cannot resolve the used value,
                            // so only retrieve the length part. This will be used for computing
                            // distance without any layout info.
                            (extract_pixel_length(&tx), extract_pixel_length(&ty))
                        }
                    };
                    let tz = tz.to_f32_px();
                    Transform3D::create_translation(tx, ty, tz)
                }
                ComputedOperation::Matrix(m) => {
                    m.into()
                }
                ComputedOperation::MatrixWithPercents(_) => {
                    // `-moz-transform` is not implemented in Servo yet.
                    unreachable!()
                }
                ComputedOperation::Skew(theta_x, theta_y) => {
                    Transform3D::create_skew(theta_x.into(), theta_y.into())
                }
                ComputedOperation::InterpolateMatrix { .. } |
                ComputedOperation::AccumulateMatrix { .. } => {
                    // TODO: Convert InterpolateMatrix/AccmulateMatrix into a valid Transform3D by
                    // the reference box and do interpolation on these two Transform3D matrices.
                    // Both Gecko and Servo don't support this for computing distance, and Servo
                    // doesn't support animations on InterpolateMatrix/AccumulateMatrix, so
                    // return None.
                    return None;
                }
            };

            transform = transform.pre_mul(&matrix);
        }

        Some(transform)
    }

    /// Return the transform matrix from a perspective length.
    #[inline]
    pub fn create_perspective_matrix(d: Au) -> Transform3D<f32> {
        // TODO(gw): The transforms spec says that perspective length must
        // be positive. However, there is some confusion between the spec
        // and browser implementations as to handling the case of 0 for the
        // perspective value. Until the spec bug is resolved, at least ensure
        // that a provided perspective value of <= 0.0 doesn't cause panics
        // and behaves as it does in other browsers.
        // See https://lists.w3.org/Archives/Public/www-style/2016Jan/0020.html for more details.
        let d = d.to_f32_px();
        if d <= 0.0 {
            Transform3D::identity()
        } else {
            Transform3D::create_perspective(d)
        }
    }

    /// Return the normalized direction vector and its angle for Rotate3D.
    pub fn get_normalized_vector_and_angle(x: f32, y: f32, z: f32, angle: Angle)
                                           -> (f32, f32, f32, Angle) {
        use euclid::approxeq::ApproxEq;
        use euclid::num::Zero;
        let vector = DirectionVector::new(x, y, z);
        if vector.square_length().approx_eq(&f32::zero()) {
            // https://www.w3.org/TR/css-transforms-1/#funcdef-rotate3d
            // A direction vector that cannot be normalized, such as [0, 0, 0], will cause the
            // rotation to not be applied, so we use identity matrix (i.e. rotate3d(0, 0, 1, 0)).
            (0., 0., 1., Angle::zero())
        } else {
            let vector = vector.normalize();
            (vector.x, vector.y, vector.z, angle)
        }
    }
}
