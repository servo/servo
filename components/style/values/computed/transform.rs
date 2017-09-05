/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to transformations.

use app_units::Au;
use euclid::{Rect, Transform3D, Vector3D};
use properties::longhands::transform::computed_value::{ComputedOperation, ComputedMatrix};
use properties::longhands::transform::computed_value::T as TransformList;
use std::f32;
use super::{Context, CSSFloat, Either, ToComputedValue};
use values::animated::{Animate, ToAnimatedZero, Procedure};
use values::computed::{Angle, CalcLengthOrPercentage, Length, LengthOrPercentage, Number};
use values::computed::Percentage;
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::transform::TimingFunction as GenericTimingFunction;
use values::generics::transform::TransformOrigin as GenericTransformOrigin;
use values::specified;

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
                ComputedOperation::Translate(ref tx, ref ty, ref tz) => {
                    let (tx, ty) = match reference_box {
                        Some(relative_border_box) => {
                            (tx.to_used_value(relative_border_box.size.width).to_f32_px(),
                             ty.to_used_value(relative_border_box.size.height).to_f32_px())
                        },
                        None => {
                            // If we don't have reference box, we cannot resolve the used value,
                            // so only retrieve the length part. This will be used for computing
                            // distance without any layout info.
                            (tx.extract_pixel_length(), ty.extract_pixel_length())
                        }
                    };
                    Transform3D::create_translation(tx, ty, *tz)
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
                    // TODO: Convert InterpolateMatrix/AccumulateMatrix into a valid Transform3D by
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
    pub fn create_perspective_matrix(d: CSSFloat) -> Transform3D<f32> {
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

/// The computed value of TransformLengthOrPercentage. We treat computed values of transform
/// as the specified values, but need to convert the relative lengths into absolute lengths.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[animate(fallback = "Self::animate_fallback")]
#[derive(Animate, Clone, Copy, Debug, PartialEq, ToAnimatedZero, ToCss)]
pub enum TransformLengthOrPercentage {
    /// Length type. Use pixel value.
    Length(CSSFloat),
    /// Percentage type.
    Percentage(Percentage),
    // FIXME(boris): specified::CalcLengthOrPercentage and computed::CalcLengthOrPercentage store
    // the absolute values as Au, so in order to fix the rounding issue, we probably need to
    // revise them or write different ones.
    /// Calc expression type.
    Calc(CalcLengthOrPercentage),
}

impl TransformLengthOrPercentage {
    /// Returns a `zero` length.
    #[inline]
    pub fn zero() -> Self {
        TransformLengthOrPercentage::Length(0.)
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    ///
    /// (Returns false for calc() values, even if ones that may resolve to zero.)
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        match *self {
            TransformLengthOrPercentage::Length(l) => l == 0.0,
            TransformLengthOrPercentage::Percentage(p) => p.0 == 0.0,
            TransformLengthOrPercentage::Calc(_) => false,
        }
    }

    /// Return the pixel value if any.
    #[inline]
    pub fn extract_pixel_length(&self) -> CSSFloat {
        match *self {
            TransformLengthOrPercentage::Length(l) => l,
            TransformLengthOrPercentage::Percentage(_) => 0.,
            TransformLengthOrPercentage::Calc(calc) => calc.length().to_f32_px(),
        }
    }

    /// Returns the used value.
    #[inline]
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        match *self {
            TransformLengthOrPercentage::Length(length) => Au::from_f32_px(length),
            TransformLengthOrPercentage::Percentage(p) => containing_length.scale_by(p.0),
            TransformLengthOrPercentage::Calc(ref calc) => {
                calc.to_used_value(Some(containing_length)).unwrap()
            },
        }
    }

    /// Used for Animate custom-derive.
    /// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
    fn animate_fallback(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        // Special handling for zero values since these should not require calc().
        if self.is_definitely_zero() {
            return other.to_animated_zero()?.animate(other, procedure);
        }
        if other.is_definitely_zero() {
            return self.animate(&self.to_animated_zero()?, procedure);
        }

        let this = CalcLengthOrPercentage::from(*self);
        let other = CalcLengthOrPercentage::from(*other);
        Ok(TransformLengthOrPercentage::Calc(this.animate(&other, procedure)?))
    }
}

impl ToComputedValue for specified::TransformLengthOrPercentage {
    type ComputedValue = TransformLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match self.0 {
            specified::LengthOrPercentage::Length(ref value) => {
                TransformLengthOrPercentage::Length(value.to_px(context))
            },
            specified::LengthOrPercentage::Percentage(value) => {
                TransformLengthOrPercentage::Percentage(value)
            }
            specified::LengthOrPercentage::Calc(ref calc) => {
                TransformLengthOrPercentage::Calc(calc.to_computed_value(context))
            }
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        use values::specified::NoCalcLength;

        let lop = match *computed {
            TransformLengthOrPercentage::Length(value) => {
                specified::LengthOrPercentage::Length(NoCalcLength::from_px(value))
            }
            TransformLengthOrPercentage::Percentage(value) => {
                specified::LengthOrPercentage::Percentage(value)
            }
            TransformLengthOrPercentage::Calc(ref calc) => {
                specified::LengthOrPercentage::Calc(
                    Box::new(ToComputedValue::from_computed_value(calc))
                )
            }
        };
        specified::TransformLengthOrPercentage(lop)
    }
}

impl ComputeSquaredDistance for TransformLengthOrPercentage {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // We don't want to require doing layout in order to calculate the result, so
        // drop the percentage part. However, dropping percentage makes us impossible to
        // compute the distance for the percentage-percentage case, but Gecko uses the
        // same formula, so it's fine for now.
        let p1 = self.extract_pixel_length();
        let p2 = other.extract_pixel_length();
        p1.compute_squared_distance(&p2)
    }
}

impl From<TransformLengthOrPercentage> for CalcLengthOrPercentage {
    #[inline]
    fn from(lop: TransformLengthOrPercentage) -> CalcLengthOrPercentage {
        match lop {
            TransformLengthOrPercentage::Calc(this) => this,
            TransformLengthOrPercentage::Length(this) => {
                CalcLengthOrPercentage::new(Au::from_f32_px(this), None)
            },
            TransformLengthOrPercentage::Percentage(this) => {
                CalcLengthOrPercentage::new(Au(0), Some(this))
            },
        }
    }
}

/// The computed value of TransformLength.
pub type TransformLength = CSSFloat;

impl ToComputedValue for specified::TransformLength {
    type ComputedValue = TransformLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match self.0 {
            specified::Length::NoCalc(l) => l.to_px(context),
            specified::Length::Calc(ref calc) => {
                calc.to_computed_value(context).length().to_f32_px()
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        specified::TransformLength(specified::Length::from_px(*computed))
    }
}

/// The computed value of TransformLengthOrPercentageOrNumber.
pub type TransformLengthOrPercentageOrNumber = Either<Number, TransformLengthOrPercentage>;

/// The computed value of TransformLengthOrNumber.
pub type TransformLengthOrNumber = Either<Number, TransformLength>;
