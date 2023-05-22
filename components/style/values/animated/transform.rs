/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animated types for transform.
// There are still some implementation on Matrix3D in animated_properties.mako.rs
// because they still need mako to generate the code.

use super::animate_multiplicative_factor;
use super::{Animate, Procedure, ToAnimatedZero};
use crate::properties::animated_properties::ListAnimation;
use crate::values::computed::transform::Rotate as ComputedRotate;
use crate::values::computed::transform::Scale as ComputedScale;
use crate::values::computed::transform::Transform as ComputedTransform;
use crate::values::computed::transform::TransformOperation as ComputedTransformOperation;
use crate::values::computed::transform::Translate as ComputedTranslate;
use crate::values::computed::transform::{DirectionVector, Matrix, Matrix3D};
use crate::values::computed::Angle;
use crate::values::computed::{Length, LengthPercentage};
use crate::values::computed::{Number, Percentage};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::transform::{self, Transform, TransformOperation};
use crate::values::generics::transform::{Rotate, Scale, Translate};
use crate::values::CSSFloat;
use crate::Zero;
use std::cmp;

// ------------------------------------
// Animations for Matrix/Matrix3D.
// ------------------------------------
/// A 2d matrix for interpolation.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[allow(missing_docs)]
// FIXME: We use custom derive for ComputeSquaredDistance. However, If possible, we should convert
// the InnerMatrix2D into types with physical meaning. This custom derive computes the squared
// distance from each matrix item, and this makes the result different from that in Gecko if we
// have skew factor in the Matrix3D.
pub struct InnerMatrix2D {
    pub m11: CSSFloat,
    pub m12: CSSFloat,
    pub m21: CSSFloat,
    pub m22: CSSFloat,
}

impl Animate for InnerMatrix2D {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(InnerMatrix2D {
            m11: animate_multiplicative_factor(self.m11, other.m11, procedure)?,
            m12: self.m12.animate(&other.m12, procedure)?,
            m21: self.m21.animate(&other.m21, procedure)?,
            m22: animate_multiplicative_factor(self.m22, other.m22, procedure)?,
        })
    }
}

/// A 2d translation function.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
pub struct Translate2D(f32, f32);

/// A 2d scale function.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct Scale2D(f32, f32);

impl Animate for Scale2D {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Scale2D(
            animate_multiplicative_factor(self.0, other.0, procedure)?,
            animate_multiplicative_factor(self.1, other.1, procedure)?,
        ))
    }
}

/// A decomposed 2d matrix.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct MatrixDecomposed2D {
    /// The translation function.
    pub translate: Translate2D,
    /// The scale function.
    pub scale: Scale2D,
    /// The rotation angle.
    pub angle: f32,
    /// The inner matrix.
    pub matrix: InnerMatrix2D,
}

impl Animate for MatrixDecomposed2D {
    /// <https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-2d-matrix-values>
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        // If x-axis of one is flipped, and y-axis of the other,
        // convert to an unflipped rotation.
        let mut scale = self.scale;
        let mut angle = self.angle;
        let mut other_angle = other.angle;
        if (scale.0 < 0.0 && other.scale.1 < 0.0) || (scale.1 < 0.0 && other.scale.0 < 0.0) {
            scale.0 = -scale.0;
            scale.1 = -scale.1;
            angle += if angle < 0.0 { 180. } else { -180. };
        }

        // Don't rotate the long way around.
        if angle == 0.0 {
            angle = 360.
        }
        if other_angle == 0.0 {
            other_angle = 360.
        }

        if (angle - other_angle).abs() > 180. {
            if angle > other_angle {
                angle -= 360.
            } else {
                other_angle -= 360.
            }
        }

        // Interpolate all values.
        let translate = self.translate.animate(&other.translate, procedure)?;
        let scale = scale.animate(&other.scale, procedure)?;
        let angle = angle.animate(&other_angle, procedure)?;
        let matrix = self.matrix.animate(&other.matrix, procedure)?;

        Ok(MatrixDecomposed2D {
            translate,
            scale,
            angle,
            matrix,
        })
    }
}

impl ComputeSquaredDistance for MatrixDecomposed2D {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // Use Radian to compute the distance.
        const RAD_PER_DEG: f64 = std::f64::consts::PI / 180.0;
        let angle1 = self.angle as f64 * RAD_PER_DEG;
        let angle2 = other.angle as f64 * RAD_PER_DEG;
        Ok(self.translate.compute_squared_distance(&other.translate)? +
            self.scale.compute_squared_distance(&other.scale)? +
            angle1.compute_squared_distance(&angle2)? +
            self.matrix.compute_squared_distance(&other.matrix)?)
    }
}

impl From<Matrix3D> for MatrixDecomposed2D {
    /// Decompose a 2D matrix.
    /// <https://drafts.csswg.org/css-transforms/#decomposing-a-2d-matrix>
    fn from(matrix: Matrix3D) -> MatrixDecomposed2D {
        let mut row0x = matrix.m11;
        let mut row0y = matrix.m12;
        let mut row1x = matrix.m21;
        let mut row1y = matrix.m22;

        let translate = Translate2D(matrix.m41, matrix.m42);
        let mut scale = Scale2D(
            (row0x * row0x + row0y * row0y).sqrt(),
            (row1x * row1x + row1y * row1y).sqrt(),
        );

        // If determinant is negative, one axis was flipped.
        let determinant = row0x * row1y - row0y * row1x;
        if determinant < 0. {
            if row0x < row1y {
                scale.0 = -scale.0;
            } else {
                scale.1 = -scale.1;
            }
        }

        // Renormalize matrix to remove scale.
        if scale.0 != 0.0 {
            row0x *= 1. / scale.0;
            row0y *= 1. / scale.0;
        }
        if scale.1 != 0.0 {
            row1x *= 1. / scale.1;
            row1y *= 1. / scale.1;
        }

        // Compute rotation and renormalize matrix.
        let mut angle = row0y.atan2(row0x);
        if angle != 0.0 {
            let sn = -row0y;
            let cs = row0x;
            let m11 = row0x;
            let m12 = row0y;
            let m21 = row1x;
            let m22 = row1y;
            row0x = cs * m11 + sn * m21;
            row0y = cs * m12 + sn * m22;
            row1x = -sn * m11 + cs * m21;
            row1y = -sn * m12 + cs * m22;
        }

        let m = InnerMatrix2D {
            m11: row0x,
            m12: row0y,
            m21: row1x,
            m22: row1y,
        };

        // Convert into degrees because our rotation functions expect it.
        angle = angle.to_degrees();
        MatrixDecomposed2D {
            translate: translate,
            scale: scale,
            angle: angle,
            matrix: m,
        }
    }
}

impl From<MatrixDecomposed2D> for Matrix3D {
    /// Recompose a 2D matrix.
    /// <https://drafts.csswg.org/css-transforms/#recomposing-to-a-2d-matrix>
    fn from(decomposed: MatrixDecomposed2D) -> Matrix3D {
        let mut computed_matrix = Matrix3D::identity();
        computed_matrix.m11 = decomposed.matrix.m11;
        computed_matrix.m12 = decomposed.matrix.m12;
        computed_matrix.m21 = decomposed.matrix.m21;
        computed_matrix.m22 = decomposed.matrix.m22;

        // Translate matrix.
        computed_matrix.m41 = decomposed.translate.0;
        computed_matrix.m42 = decomposed.translate.1;

        // Rotate matrix.
        let angle = decomposed.angle.to_radians();
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        let mut rotate_matrix = Matrix3D::identity();
        rotate_matrix.m11 = cos_angle;
        rotate_matrix.m12 = sin_angle;
        rotate_matrix.m21 = -sin_angle;
        rotate_matrix.m22 = cos_angle;

        // Multiplication of computed_matrix and rotate_matrix
        computed_matrix = rotate_matrix.multiply(&computed_matrix);

        // Scale matrix.
        computed_matrix.m11 *= decomposed.scale.0;
        computed_matrix.m12 *= decomposed.scale.0;
        computed_matrix.m21 *= decomposed.scale.1;
        computed_matrix.m22 *= decomposed.scale.1;
        computed_matrix
    }
}

impl Animate for Matrix {
    #[cfg(feature = "servo")]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let this = Matrix3D::from(*self);
        let other = Matrix3D::from(*other);
        let this = MatrixDecomposed2D::from(this);
        let other = MatrixDecomposed2D::from(other);
        Ok(Matrix3D::from(this.animate(&other, procedure)?).into_2d()?)
    }

    #[cfg(feature = "gecko")]
    // Gecko doesn't exactly follow the spec here; we use a different procedure
    // to match it
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let from = decompose_2d_matrix(&(*self).into());
        let to = decompose_2d_matrix(&(*other).into());
        match (from, to) {
            (Ok(from), Ok(to)) => Matrix3D::from(from.animate(&to, procedure)?).into_2d(),
            // Matrices can be undecomposable due to couple reasons, e.g.,
            // non-invertible matrices. In this case, we should report Err here,
            // and let the caller do the fallback procedure.
            _ => Err(()),
        }
    }
}

/// A 3d translation.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
pub struct Translate3D(pub f32, pub f32, pub f32);

/// A 3d scale function.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct Scale3D(pub f32, pub f32, pub f32);

impl Scale3D {
    /// Negate self.
    fn negate(&mut self) {
        self.0 *= -1.0;
        self.1 *= -1.0;
        self.2 *= -1.0;
    }
}

impl Animate for Scale3D {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Scale3D(
            animate_multiplicative_factor(self.0, other.0, procedure)?,
            animate_multiplicative_factor(self.1, other.1, procedure)?,
            animate_multiplicative_factor(self.2, other.2, procedure)?,
        ))
    }
}

/// A 3d skew function.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Animate, Clone, Copy, Debug)]
pub struct Skew(f32, f32, f32);

impl ComputeSquaredDistance for Skew {
    // We have to use atan() to convert the skew factors into skew angles, so implement
    // ComputeSquaredDistance manually.
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(self.0.atan().compute_squared_distance(&other.0.atan())? +
            self.1.atan().compute_squared_distance(&other.1.atan())? +
            self.2.atan().compute_squared_distance(&other.2.atan())?)
    }
}

/// A 3d perspective transformation.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct Perspective(pub f32, pub f32, pub f32, pub f32);

impl Animate for Perspective {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Perspective(
            self.0.animate(&other.0, procedure)?,
            self.1.animate(&other.1, procedure)?,
            self.2.animate(&other.2, procedure)?,
            animate_multiplicative_factor(self.3, other.3, procedure)?,
        ))
    }
}

/// A quaternion used to represent a rotation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct Quaternion(f64, f64, f64, f64);

impl Quaternion {
    /// Return a quaternion from a unit direction vector and angle (unit: radian).
    #[inline]
    fn from_direction_and_angle(vector: &DirectionVector, angle: f64) -> Self {
        debug_assert!(
            (vector.length() - 1.).abs() < 0.0001,
            "Only accept an unit direction vector to create a quaternion"
        );
        // Reference:
        // https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
        //
        // if the direction axis is (x, y, z) = xi + yj + zk,
        // and the angle is |theta|, this formula can be done using
        // an extension of Euler's formula:
        //   q = cos(theta/2) + (xi + yj + zk)(sin(theta/2))
        //     = cos(theta/2) +
        //       x*sin(theta/2)i + y*sin(theta/2)j + z*sin(theta/2)k
        Quaternion(
            vector.x as f64 * (angle / 2.).sin(),
            vector.y as f64 * (angle / 2.).sin(),
            vector.z as f64 * (angle / 2.).sin(),
            (angle / 2.).cos(),
        )
    }

    /// Calculate the dot product.
    #[inline]
    fn dot(&self, other: &Self) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2 + self.3 * other.3
    }

    /// Return the scaled quaternion by a factor.
    #[inline]
    fn scale(&self, factor: f64) -> Self {
        Quaternion(
            self.0 * factor,
            self.1 * factor,
            self.2 * factor,
            self.3 * factor,
        )
    }
}

impl Animate for Quaternion {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use std::f64;

        let (this_weight, other_weight) = procedure.weights();
        debug_assert!(
            // Doule EPSILON since both this_weight and other_weght have calculation errors
            // which are approximately equal to EPSILON.
            (this_weight + other_weight - 1.0f64).abs() <= f64::EPSILON * 2.0 ||
                other_weight == 1.0f64 ||
                other_weight == 0.0f64,
            "animate should only be used for interpolating or accumulating transforms"
        );

        // We take a specialized code path for accumulation (where other_weight
        // is 1).
        if let Procedure::Accumulate { .. } = procedure {
            debug_assert_eq!(other_weight, 1.0);
            if this_weight == 0.0 {
                return Ok(*other);
            }

            let clamped_w = self.3.min(1.0).max(-1.0);

            // Determine the scale factor.
            let mut theta = clamped_w.acos();
            let mut scale = if theta == 0.0 { 0.0 } else { 1.0 / theta.sin() };
            theta *= this_weight;
            scale *= theta.sin();

            // Scale the self matrix by this_weight.
            let mut scaled_self = *self;
            scaled_self.0 *= scale;
            scaled_self.1 *= scale;
            scaled_self.2 *= scale;
            scaled_self.3 = theta.cos();

            // Multiply scaled-self by other.
            let a = &scaled_self;
            let b = other;
            return Ok(Quaternion(
                a.3 * b.0 + a.0 * b.3 + a.1 * b.2 - a.2 * b.1,
                a.3 * b.1 - a.0 * b.2 + a.1 * b.3 + a.2 * b.0,
                a.3 * b.2 + a.0 * b.1 - a.1 * b.0 + a.2 * b.3,
                a.3 * b.3 - a.0 * b.0 - a.1 * b.1 - a.2 * b.2,
            ));
        }

        // Straight from gfxQuaternion::Slerp.
        //
        // Dot product, clamped between -1 and 1.
        let dot = (self.0 * other.0 + self.1 * other.1 + self.2 * other.2 + self.3 * other.3)
            .min(1.0)
            .max(-1.0);

        if dot.abs() == 1.0 {
            return Ok(*self);
        }

        let theta = dot.acos();
        let rsintheta = 1.0 / (1.0 - dot * dot).sqrt();

        let right_weight = (other_weight * theta).sin() * rsintheta;
        let left_weight = (other_weight * theta).cos() - dot * right_weight;

        let left = self.scale(left_weight);
        let right = other.scale(right_weight);

        Ok(Quaternion(
            left.0 + right.0,
            left.1 + right.1,
            left.2 + right.2,
            left.3 + right.3,
        ))
    }
}

impl ComputeSquaredDistance for Quaternion {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // Use quaternion vectors to get the angle difference. Both q1 and q2 are unit vectors,
        // so we can get their angle difference by:
        // cos(theta/2) = (q1 dot q2) / (|q1| * |q2|) = q1 dot q2.
        let distance = self.dot(other).max(-1.0).min(1.0).acos() * 2.0;
        Ok(SquaredDistance::from_sqrt(distance))
    }
}

/// A decomposed 3d matrix.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct MatrixDecomposed3D {
    /// A translation function.
    pub translate: Translate3D,
    /// A scale function.
    pub scale: Scale3D,
    /// The skew component of the transformation.
    pub skew: Skew,
    /// The perspective component of the transformation.
    pub perspective: Perspective,
    /// The quaternion used to represent the rotation.
    pub quaternion: Quaternion,
}

impl From<MatrixDecomposed3D> for Matrix3D {
    /// Recompose a 3D matrix.
    /// <https://drafts.csswg.org/css-transforms/#recomposing-to-a-3d-matrix>
    fn from(decomposed: MatrixDecomposed3D) -> Matrix3D {
        let mut matrix = Matrix3D::identity();

        // Apply perspective
        matrix.set_perspective(&decomposed.perspective);

        // Apply translation
        matrix.apply_translate(&decomposed.translate);

        // Apply rotation
        {
            let x = decomposed.quaternion.0;
            let y = decomposed.quaternion.1;
            let z = decomposed.quaternion.2;
            let w = decomposed.quaternion.3;

            // Construct a composite rotation matrix from the quaternion values
            // rotationMatrix is a identity 4x4 matrix initially
            let mut rotation_matrix = Matrix3D::identity();
            rotation_matrix.m11 = 1.0 - 2.0 * (y * y + z * z) as f32;
            rotation_matrix.m12 = 2.0 * (x * y + z * w) as f32;
            rotation_matrix.m13 = 2.0 * (x * z - y * w) as f32;
            rotation_matrix.m21 = 2.0 * (x * y - z * w) as f32;
            rotation_matrix.m22 = 1.0 - 2.0 * (x * x + z * z) as f32;
            rotation_matrix.m23 = 2.0 * (y * z + x * w) as f32;
            rotation_matrix.m31 = 2.0 * (x * z + y * w) as f32;
            rotation_matrix.m32 = 2.0 * (y * z - x * w) as f32;
            rotation_matrix.m33 = 1.0 - 2.0 * (x * x + y * y) as f32;

            matrix = rotation_matrix.multiply(&matrix);
        }

        // Apply skew
        {
            let mut temp = Matrix3D::identity();
            if decomposed.skew.2 != 0.0 {
                temp.m32 = decomposed.skew.2;
                matrix = temp.multiply(&matrix);
                temp.m32 = 0.0;
            }

            if decomposed.skew.1 != 0.0 {
                temp.m31 = decomposed.skew.1;
                matrix = temp.multiply(&matrix);
                temp.m31 = 0.0;
            }

            if decomposed.skew.0 != 0.0 {
                temp.m21 = decomposed.skew.0;
                matrix = temp.multiply(&matrix);
            }
        }

        // Apply scale
        matrix.apply_scale(&decomposed.scale);

        matrix
    }
}

/// Decompose a 3D matrix.
/// https://drafts.csswg.org/css-transforms-2/#decomposing-a-3d-matrix
/// http://www.realtimerendering.com/resources/GraphicsGems/gemsii/unmatrix.c
fn decompose_3d_matrix(mut matrix: Matrix3D) -> Result<MatrixDecomposed3D, ()> {
    // Combine 2 point.
    let combine = |a: [f32; 3], b: [f32; 3], ascl: f32, bscl: f32| {
        [
            (ascl * a[0]) + (bscl * b[0]),
            (ascl * a[1]) + (bscl * b[1]),
            (ascl * a[2]) + (bscl * b[2]),
        ]
    };
    // Dot product.
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    // Cross product.
    let cross = |row1: [f32; 3], row2: [f32; 3]| {
        [
            row1[1] * row2[2] - row1[2] * row2[1],
            row1[2] * row2[0] - row1[0] * row2[2],
            row1[0] * row2[1] - row1[1] * row2[0],
        ]
    };

    if matrix.m44 == 0.0 {
        return Err(());
    }

    let scaling_factor = matrix.m44;

    // Normalize the matrix.
    matrix.scale_by_factor(1.0 / scaling_factor);

    // perspective_matrix is used to solve for perspective, but it also provides
    // an easy way to test for singularity of the upper 3x3 component.
    let mut perspective_matrix = matrix;

    perspective_matrix.m14 = 0.0;
    perspective_matrix.m24 = 0.0;
    perspective_matrix.m34 = 0.0;
    perspective_matrix.m44 = 1.0;

    if perspective_matrix.determinant() == 0.0 {
        return Err(());
    }

    // First, isolate perspective.
    let perspective = if matrix.m14 != 0.0 || matrix.m24 != 0.0 || matrix.m34 != 0.0 {
        let right_hand_side: [f32; 4] = [matrix.m14, matrix.m24, matrix.m34, matrix.m44];

        perspective_matrix = perspective_matrix.inverse().unwrap().transpose();
        let perspective = perspective_matrix.pre_mul_point4(&right_hand_side);
        // NOTE(emilio): Even though the reference algorithm clears the
        // fourth column here (matrix.m14..matrix.m44), they're not used below
        // so it's not really needed.
        Perspective(
            perspective[0],
            perspective[1],
            perspective[2],
            perspective[3],
        )
    } else {
        Perspective(0.0, 0.0, 0.0, 1.0)
    };

    // Next take care of translation (easy).
    let translate = Translate3D(matrix.m41, matrix.m42, matrix.m43);

    // Now get scale and shear. 'row' is a 3 element array of 3 component vectors
    let mut row = matrix.get_matrix_3x3_part();

    // Compute X scale factor and normalize first row.
    let row0len = (row[0][0] * row[0][0] + row[0][1] * row[0][1] + row[0][2] * row[0][2]).sqrt();
    let mut scale = Scale3D(row0len, 0.0, 0.0);
    row[0] = [
        row[0][0] / row0len,
        row[0][1] / row0len,
        row[0][2] / row0len,
    ];

    // Compute XY shear factor and make 2nd row orthogonal to 1st.
    let mut skew = Skew(dot(row[0], row[1]), 0.0, 0.0);
    row[1] = combine(row[1], row[0], 1.0, -skew.0);

    // Now, compute Y scale and normalize 2nd row.
    let row1len = (row[1][0] * row[1][0] + row[1][1] * row[1][1] + row[1][2] * row[1][2]).sqrt();
    scale.1 = row1len;
    row[1] = [
        row[1][0] / row1len,
        row[1][1] / row1len,
        row[1][2] / row1len,
    ];
    skew.0 /= scale.1;

    // Compute XZ and YZ shears, orthogonalize 3rd row
    skew.1 = dot(row[0], row[2]);
    row[2] = combine(row[2], row[0], 1.0, -skew.1);
    skew.2 = dot(row[1], row[2]);
    row[2] = combine(row[2], row[1], 1.0, -skew.2);

    // Next, get Z scale and normalize 3rd row.
    let row2len = (row[2][0] * row[2][0] + row[2][1] * row[2][1] + row[2][2] * row[2][2]).sqrt();
    scale.2 = row2len;
    row[2] = [
        row[2][0] / row2len,
        row[2][1] / row2len,
        row[2][2] / row2len,
    ];
    skew.1 /= scale.2;
    skew.2 /= scale.2;

    // At this point, the matrix (in rows) is orthonormal.
    // Check for a coordinate system flip.  If the determinant
    // is -1, then negate the matrix and the scaling factors.
    if dot(row[0], cross(row[1], row[2])) < 0.0 {
        scale.negate();
        for i in 0..3 {
            row[i][0] *= -1.0;
            row[i][1] *= -1.0;
            row[i][2] *= -1.0;
        }
    }

    // Now, get the rotations out.
    let mut quaternion = Quaternion(
        0.5 * ((1.0 + row[0][0] - row[1][1] - row[2][2]).max(0.0) as f64).sqrt(),
        0.5 * ((1.0 - row[0][0] + row[1][1] - row[2][2]).max(0.0) as f64).sqrt(),
        0.5 * ((1.0 - row[0][0] - row[1][1] + row[2][2]).max(0.0) as f64).sqrt(),
        0.5 * ((1.0 + row[0][0] + row[1][1] + row[2][2]).max(0.0) as f64).sqrt(),
    );

    if row[2][1] > row[1][2] {
        quaternion.0 = -quaternion.0
    }
    if row[0][2] > row[2][0] {
        quaternion.1 = -quaternion.1
    }
    if row[1][0] > row[0][1] {
        quaternion.2 = -quaternion.2
    }

    Ok(MatrixDecomposed3D {
        translate,
        scale,
        skew,
        perspective,
        quaternion,
    })
}

/// Decompose a 2D matrix for Gecko.
// Use the algorithm from nsStyleTransformMatrix::Decompose2DMatrix() in Gecko.
#[cfg(feature = "gecko")]
fn decompose_2d_matrix(matrix: &Matrix3D) -> Result<MatrixDecomposed3D, ()> {
    // The index is column-major, so the equivalent transform matrix is:
    // | m11 m21  0 m41 |  =>  | m11 m21 | and translate(m41, m42)
    // | m12 m22  0 m42 |      | m12 m22 |
    // |   0   0  1   0 |
    // |   0   0  0   1 |
    let (mut m11, mut m12) = (matrix.m11, matrix.m12);
    let (mut m21, mut m22) = (matrix.m21, matrix.m22);
    // Check if this is a singular matrix.
    if m11 * m22 == m12 * m21 {
        return Err(());
    }

    let mut scale_x = (m11 * m11 + m12 * m12).sqrt();
    m11 /= scale_x;
    m12 /= scale_x;

    let mut shear_xy = m11 * m21 + m12 * m22;
    m21 -= m11 * shear_xy;
    m22 -= m12 * shear_xy;

    let scale_y = (m21 * m21 + m22 * m22).sqrt();
    m21 /= scale_y;
    m22 /= scale_y;
    shear_xy /= scale_y;

    let determinant = m11 * m22 - m12 * m21;
    // Determinant should now be 1 or -1.
    if 0.99 > determinant.abs() || determinant.abs() > 1.01 {
        return Err(());
    }

    if determinant < 0. {
        m11 = -m11;
        m12 = -m12;
        shear_xy = -shear_xy;
        scale_x = -scale_x;
    }

    Ok(MatrixDecomposed3D {
        translate: Translate3D(matrix.m41, matrix.m42, 0.),
        scale: Scale3D(scale_x, scale_y, 1.),
        skew: Skew(shear_xy, 0., 0.),
        perspective: Perspective(0., 0., 0., 1.),
        quaternion: Quaternion::from_direction_and_angle(
            &DirectionVector::new(0., 0., 1.),
            m12.atan2(m11) as f64,
        ),
    })
}

impl Animate for Matrix3D {
    #[cfg(feature = "servo")]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.is_3d() || other.is_3d() {
            let decomposed_from = decompose_3d_matrix(*self);
            let decomposed_to = decompose_3d_matrix(*other);
            match (decomposed_from, decomposed_to) {
                (Ok(this), Ok(other)) => Ok(Matrix3D::from(this.animate(&other, procedure)?)),
                // Matrices can be undecomposable due to couple reasons, e.g.,
                // non-invertible matrices. In this case, we should report Err
                // here, and let the caller do the fallback procedure.
                _ => Err(()),
            }
        } else {
            let this = MatrixDecomposed2D::from(*self);
            let other = MatrixDecomposed2D::from(*other);
            Ok(Matrix3D::from(this.animate(&other, procedure)?))
        }
    }

    #[cfg(feature = "gecko")]
    // Gecko doesn't exactly follow the spec here; we use a different procedure
    // to match it
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (from, to) = if self.is_3d() || other.is_3d() {
            (decompose_3d_matrix(*self), decompose_3d_matrix(*other))
        } else {
            (decompose_2d_matrix(self), decompose_2d_matrix(other))
        };
        match (from, to) {
            (Ok(from), Ok(to)) => Ok(Matrix3D::from(from.animate(&to, procedure)?)),
            // Matrices can be undecomposable due to couple reasons, e.g.,
            // non-invertible matrices. In this case, we should report Err here,
            // and let the caller do the fallback procedure.
            _ => Err(()),
        }
    }
}

impl ComputeSquaredDistance for Matrix3D {
    #[inline]
    #[cfg(feature = "servo")]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.is_3d() || other.is_3d() {
            let from = decompose_3d_matrix(*self)?;
            let to = decompose_3d_matrix(*other)?;
            from.compute_squared_distance(&to)
        } else {
            let from = MatrixDecomposed2D::from(*self);
            let to = MatrixDecomposed2D::from(*other);
            from.compute_squared_distance(&to)
        }
    }

    #[inline]
    #[cfg(feature = "gecko")]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let (from, to) = if self.is_3d() || other.is_3d() {
            (decompose_3d_matrix(*self)?, decompose_3d_matrix(*other)?)
        } else {
            (decompose_2d_matrix(self)?, decompose_2d_matrix(other)?)
        };
        from.compute_squared_distance(&to)
    }
}

// ------------------------------------
// Animation for Transform list.
// ------------------------------------
fn is_matched_operation(
    first: &ComputedTransformOperation,
    second: &ComputedTransformOperation,
) -> bool {
    match (first, second) {
        (&TransformOperation::Matrix(..), &TransformOperation::Matrix(..)) |
        (&TransformOperation::Matrix3D(..), &TransformOperation::Matrix3D(..)) |
        (&TransformOperation::Skew(..), &TransformOperation::Skew(..)) |
        (&TransformOperation::SkewX(..), &TransformOperation::SkewX(..)) |
        (&TransformOperation::SkewY(..), &TransformOperation::SkewY(..)) |
        (&TransformOperation::Rotate(..), &TransformOperation::Rotate(..)) |
        (&TransformOperation::Rotate3D(..), &TransformOperation::Rotate3D(..)) |
        (&TransformOperation::RotateX(..), &TransformOperation::RotateX(..)) |
        (&TransformOperation::RotateY(..), &TransformOperation::RotateY(..)) |
        (&TransformOperation::RotateZ(..), &TransformOperation::RotateZ(..)) |
        (&TransformOperation::Perspective(..), &TransformOperation::Perspective(..)) => true,
        // Match functions that have the same primitive transform function
        (a, b) if a.is_translate() && b.is_translate() => true,
        (a, b) if a.is_scale() && b.is_scale() => true,
        (a, b) if a.is_rotate() && b.is_rotate() => true,
        // InterpolateMatrix and AccumulateMatrix are for mismatched transforms
        _ => false,
    }
}

/// <https://drafts.csswg.org/css-transforms/#interpolation-of-transforms>
impl Animate for ComputedTransform {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use std::borrow::Cow;

        // Addition for transforms simply means appending to the list of
        // transform functions. This is different to how we handle the other
        // animation procedures so we treat it separately here rather than
        // handling it in TransformOperation.
        if procedure == Procedure::Add {
            let result = self.0.iter().chain(&*other.0).cloned().collect();
            return Ok(Transform(result));
        }

        let this = Cow::Borrowed(&self.0);
        let other = Cow::Borrowed(&other.0);

        // Interpolate the common prefix
        let mut result = this
            .iter()
            .zip(other.iter())
            .take_while(|(this, other)| is_matched_operation(this, other))
            .map(|(this, other)| this.animate(other, procedure))
            .collect::<Result<Vec<_>, _>>()?;

        // Deal with the remainders
        let this_remainder = if this.len() > result.len() {
            Some(&this[result.len()..])
        } else {
            None
        };
        let other_remainder = if other.len() > result.len() {
            Some(&other[result.len()..])
        } else {
            None
        };

        match (this_remainder, other_remainder) {
            // If there is a remainder from *both* lists we must have had mismatched functions.
            // => Add the remainders to a suitable ___Matrix function.
            (Some(this_remainder), Some(other_remainder)) => match procedure {
                Procedure::Add => {
                    debug_assert!(false, "Should have already dealt with add by the point");
                    return Err(());
                },
                Procedure::Interpolate { progress } => {
                    result.push(TransformOperation::InterpolateMatrix {
                        from_list: Transform(this_remainder.to_vec().into()),
                        to_list: Transform(other_remainder.to_vec().into()),
                        progress: Percentage(progress as f32),
                    });
                },
                Procedure::Accumulate { count } => {
                    result.push(TransformOperation::AccumulateMatrix {
                        from_list: Transform(this_remainder.to_vec().into()),
                        to_list: Transform(other_remainder.to_vec().into()),
                        count: cmp::min(count, i32::max_value() as u64) as i32,
                    });
                },
            },
            // If there is a remainder from just one list, then one list must be shorter but
            // completely match the type of the corresponding functions in the longer list.
            // => Interpolate the remainder with identity transforms.
            (Some(remainder), None) | (None, Some(remainder)) => {
                let fill_right = this_remainder.is_some();
                result.append(
                    &mut remainder
                        .iter()
                        .map(|transform| {
                            let identity = transform.to_animated_zero().unwrap();

                            match transform {
                                // We can't interpolate/accumulate ___Matrix types directly with a
                                // matrix. Instead we need to wrap it in another ___Matrix type.
                                TransformOperation::AccumulateMatrix { .. } |
                                TransformOperation::InterpolateMatrix { .. } => {
                                    let transform_list = Transform(vec![transform.clone()].into());
                                    let identity_list = Transform(vec![identity].into());
                                    let (from_list, to_list) = if fill_right {
                                        (transform_list, identity_list)
                                    } else {
                                        (identity_list, transform_list)
                                    };

                                    match procedure {
                                        Procedure::Add => Err(()),
                                        Procedure::Interpolate { progress } => {
                                            Ok(TransformOperation::InterpolateMatrix {
                                                from_list,
                                                to_list,
                                                progress: Percentage(progress as f32),
                                            })
                                        },
                                        Procedure::Accumulate { count } => {
                                            Ok(TransformOperation::AccumulateMatrix {
                                                from_list,
                                                to_list,
                                                count: cmp::min(count, i32::max_value() as u64)
                                                    as i32,
                                            })
                                        },
                                    }
                                },
                                _ => {
                                    let (lhs, rhs) = if fill_right {
                                        (transform, &identity)
                                    } else {
                                        (&identity, transform)
                                    };
                                    lhs.animate(rhs, procedure)
                                },
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
            },
            (None, None) => {},
        }

        Ok(Transform(result.into()))
    }
}

impl ComputeSquaredDistance for ComputedTransform {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let squared_dist = self.0.squared_distance_with_zero(&other.0);

        // Roll back to matrix interpolation if there is any Err(()) in the
        // transform lists, such as mismatched transform functions.
        if squared_dist.is_err() {
            let matrix1: Matrix3D = self.to_transform_3d_matrix(None)?.0.into();
            let matrix2: Matrix3D = other.to_transform_3d_matrix(None)?.0.into();
            return matrix1.compute_squared_distance(&matrix2);
        }

        squared_dist
    }
}

/// <http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms>
impl Animate for ComputedTransformOperation {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&TransformOperation::Matrix3D(ref this), &TransformOperation::Matrix3D(ref other)) => {
                Ok(TransformOperation::Matrix3D(
                    this.animate(other, procedure)?,
                ))
            },
            (&TransformOperation::Matrix(ref this), &TransformOperation::Matrix(ref other)) => {
                Ok(TransformOperation::Matrix(this.animate(other, procedure)?))
            },
            (
                &TransformOperation::Skew(ref fx, ref fy),
                &TransformOperation::Skew(ref tx, ref ty),
            ) => Ok(TransformOperation::Skew(
                fx.animate(tx, procedure)?,
                fy.animate(ty, procedure)?,
            )),
            (&TransformOperation::SkewX(ref f), &TransformOperation::SkewX(ref t)) => {
                Ok(TransformOperation::SkewX(f.animate(t, procedure)?))
            },
            (&TransformOperation::SkewY(ref f), &TransformOperation::SkewY(ref t)) => {
                Ok(TransformOperation::SkewY(f.animate(t, procedure)?))
            },
            (
                &TransformOperation::Translate3D(ref fx, ref fy, ref fz),
                &TransformOperation::Translate3D(ref tx, ref ty, ref tz),
            ) => Ok(TransformOperation::Translate3D(
                fx.animate(tx, procedure)?,
                fy.animate(ty, procedure)?,
                fz.animate(tz, procedure)?,
            )),
            (
                &TransformOperation::Translate(ref fx, ref fy),
                &TransformOperation::Translate(ref tx, ref ty),
            ) => Ok(TransformOperation::Translate(
                fx.animate(tx, procedure)?,
                fy.animate(ty, procedure)?,
            )),
            (&TransformOperation::TranslateX(ref f), &TransformOperation::TranslateX(ref t)) => {
                Ok(TransformOperation::TranslateX(f.animate(t, procedure)?))
            },
            (&TransformOperation::TranslateY(ref f), &TransformOperation::TranslateY(ref t)) => {
                Ok(TransformOperation::TranslateY(f.animate(t, procedure)?))
            },
            (&TransformOperation::TranslateZ(ref f), &TransformOperation::TranslateZ(ref t)) => {
                Ok(TransformOperation::TranslateZ(f.animate(t, procedure)?))
            },
            (
                &TransformOperation::Scale3D(ref fx, ref fy, ref fz),
                &TransformOperation::Scale3D(ref tx, ref ty, ref tz),
            ) => Ok(TransformOperation::Scale3D(
                animate_multiplicative_factor(*fx, *tx, procedure)?,
                animate_multiplicative_factor(*fy, *ty, procedure)?,
                animate_multiplicative_factor(*fz, *tz, procedure)?,
            )),
            (&TransformOperation::ScaleX(ref f), &TransformOperation::ScaleX(ref t)) => Ok(
                TransformOperation::ScaleX(animate_multiplicative_factor(*f, *t, procedure)?),
            ),
            (&TransformOperation::ScaleY(ref f), &TransformOperation::ScaleY(ref t)) => Ok(
                TransformOperation::ScaleY(animate_multiplicative_factor(*f, *t, procedure)?),
            ),
            (&TransformOperation::ScaleZ(ref f), &TransformOperation::ScaleZ(ref t)) => Ok(
                TransformOperation::ScaleZ(animate_multiplicative_factor(*f, *t, procedure)?),
            ),
            (
                &TransformOperation::Scale(ref fx, ref fy),
                &TransformOperation::Scale(ref tx, ref ty),
            ) => Ok(TransformOperation::Scale(
                animate_multiplicative_factor(*fx, *tx, procedure)?,
                animate_multiplicative_factor(*fy, *ty, procedure)?,
            )),
            (
                &TransformOperation::Rotate3D(fx, fy, fz, fa),
                &TransformOperation::Rotate3D(tx, ty, tz, ta),
            ) => {
                let animated = Rotate::Rotate3D(fx, fy, fz, fa)
                    .animate(&Rotate::Rotate3D(tx, ty, tz, ta), procedure)?;
                let (fx, fy, fz, fa) = ComputedRotate::resolve(&animated);
                Ok(TransformOperation::Rotate3D(fx, fy, fz, fa))
            },
            (&TransformOperation::RotateX(fa), &TransformOperation::RotateX(ta)) => {
                Ok(TransformOperation::RotateX(fa.animate(&ta, procedure)?))
            },
            (&TransformOperation::RotateY(fa), &TransformOperation::RotateY(ta)) => {
                Ok(TransformOperation::RotateY(fa.animate(&ta, procedure)?))
            },
            (&TransformOperation::RotateZ(fa), &TransformOperation::RotateZ(ta)) => {
                Ok(TransformOperation::RotateZ(fa.animate(&ta, procedure)?))
            },
            (&TransformOperation::Rotate(fa), &TransformOperation::Rotate(ta)) => {
                Ok(TransformOperation::Rotate(fa.animate(&ta, procedure)?))
            },
            (&TransformOperation::Rotate(fa), &TransformOperation::RotateZ(ta)) => {
                Ok(TransformOperation::Rotate(fa.animate(&ta, procedure)?))
            },
            (&TransformOperation::RotateZ(fa), &TransformOperation::Rotate(ta)) => {
                Ok(TransformOperation::Rotate(fa.animate(&ta, procedure)?))
            },
            (
                &TransformOperation::Perspective(ref fd),
                &TransformOperation::Perspective(ref td),
            ) => {
                use crate::values::computed::CSSPixelLength;
                use crate::values::generics::transform::create_perspective_matrix;

                // From https://drafts.csswg.org/css-transforms-2/#interpolation-of-transform-functions:
                //
                //    The transform functions matrix(), matrix3d() and
                //    perspective() get converted into 4x4 matrices first and
                //    interpolated as defined in section Interpolation of
                //    Matrices afterwards.
                //
                let from = create_perspective_matrix(fd.px());
                let to = create_perspective_matrix(td.px());

                let interpolated = Matrix3D::from(from).animate(&Matrix3D::from(to), procedure)?;

                let decomposed = decompose_3d_matrix(interpolated)?;
                let perspective_z = decomposed.perspective.2;
                // Clamp results outside of the -1 to 0 range so that we get perspective
                // function values between 1 and infinity.
                let used_value = if perspective_z >= 0. {
                    std::f32::INFINITY
                } else if perspective_z <= -1. {
                    1.
                } else {
                    -1. / perspective_z
                };
                Ok(TransformOperation::Perspective(CSSPixelLength::new(
                    used_value,
                )))
            },
            _ if self.is_translate() && other.is_translate() => self
                .to_translate_3d()
                .animate(&other.to_translate_3d(), procedure),
            _ if self.is_scale() && other.is_scale() => {
                self.to_scale_3d().animate(&other.to_scale_3d(), procedure)
            },
            _ if self.is_rotate() && other.is_rotate() => self
                .to_rotate_3d()
                .animate(&other.to_rotate_3d(), procedure),
            _ => Err(()),
        }
    }
}

// This might not be the most useful definition of distance. It might be better, for example,
// to trace the distance travelled by a point as its transform is interpolated between the two
// lists. That, however, proves to be quite complicated so we take a simple approach for now.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=1318591#c0.
impl ComputeSquaredDistance for ComputedTransformOperation {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&TransformOperation::Matrix3D(ref this), &TransformOperation::Matrix3D(ref other)) => {
                this.compute_squared_distance(other)
            },
            (&TransformOperation::Matrix(ref this), &TransformOperation::Matrix(ref other)) => {
                let this: Matrix3D = (*this).into();
                let other: Matrix3D = (*other).into();
                this.compute_squared_distance(&other)
            },
            (
                &TransformOperation::Skew(ref fx, ref fy),
                &TransformOperation::Skew(ref tx, ref ty),
            ) => Ok(fx.compute_squared_distance(&tx)? + fy.compute_squared_distance(&ty)?),
            (&TransformOperation::SkewX(ref f), &TransformOperation::SkewX(ref t)) |
            (&TransformOperation::SkewY(ref f), &TransformOperation::SkewY(ref t)) => {
                f.compute_squared_distance(&t)
            },
            (
                &TransformOperation::Translate3D(ref fx, ref fy, ref fz),
                &TransformOperation::Translate3D(ref tx, ref ty, ref tz),
            ) => {
                // For translate, We don't want to require doing layout in order
                // to calculate the result, so drop the percentage part.
                //
                // However, dropping percentage makes us impossible to compute
                // the distance for the percentage-percentage case, but Gecko
                // uses the same formula, so it's fine for now.
                let basis = Length::new(0.);
                let fx = fx.resolve(basis).px();
                let fy = fy.resolve(basis).px();
                let tx = tx.resolve(basis).px();
                let ty = ty.resolve(basis).px();

                Ok(fx.compute_squared_distance(&tx)? +
                    fy.compute_squared_distance(&ty)? +
                    fz.compute_squared_distance(&tz)?)
            },
            (
                &TransformOperation::Scale3D(ref fx, ref fy, ref fz),
                &TransformOperation::Scale3D(ref tx, ref ty, ref tz),
            ) => Ok(fx.compute_squared_distance(&tx)? +
                fy.compute_squared_distance(&ty)? +
                fz.compute_squared_distance(&tz)?),
            (
                &TransformOperation::Rotate3D(fx, fy, fz, fa),
                &TransformOperation::Rotate3D(tx, ty, tz, ta),
            ) => Rotate::Rotate3D(fx, fy, fz, fa)
                .compute_squared_distance(&Rotate::Rotate3D(tx, ty, tz, ta)),
            (&TransformOperation::RotateX(fa), &TransformOperation::RotateX(ta)) |
            (&TransformOperation::RotateY(fa), &TransformOperation::RotateY(ta)) |
            (&TransformOperation::RotateZ(fa), &TransformOperation::RotateZ(ta)) |
            (&TransformOperation::Rotate(fa), &TransformOperation::Rotate(ta)) => {
                fa.compute_squared_distance(&ta)
            },
            (
                &TransformOperation::Perspective(ref fd),
                &TransformOperation::Perspective(ref td),
            ) => fd.compute_squared_distance(td),
            (&TransformOperation::Perspective(ref p), &TransformOperation::Matrix3D(ref m)) |
            (&TransformOperation::Matrix3D(ref m), &TransformOperation::Perspective(ref p)) => {
                // FIXME(emilio): Is this right? Why interpolating this with
                // Perspective but not with anything else?
                let mut p_matrix = Matrix3D::identity();
                if p.px() >= 0. {
                    p_matrix.m34 = -1. / p.px().max(1.);
                }
                p_matrix.compute_squared_distance(&m)
            },
            // Gecko cross-interpolates amongst all translate and all scale
            // functions (See ToPrimitive in layout/style/StyleAnimationValue.cpp)
            // without falling back to InterpolateMatrix
            _ if self.is_translate() && other.is_translate() => self
                .to_translate_3d()
                .compute_squared_distance(&other.to_translate_3d()),
            _ if self.is_scale() && other.is_scale() => self
                .to_scale_3d()
                .compute_squared_distance(&other.to_scale_3d()),
            _ if self.is_rotate() && other.is_rotate() => self
                .to_rotate_3d()
                .compute_squared_distance(&other.to_rotate_3d()),
            _ => Err(()),
        }
    }
}

// ------------------------------------
// Individual transforms.
// ------------------------------------
/// <https://drafts.csswg.org/css-transforms-2/#propdef-rotate>
impl ComputedRotate {
    fn resolve(&self) -> (Number, Number, Number, Angle) {
        // According to the spec:
        // https://drafts.csswg.org/css-transforms-2/#individual-transforms
        //
        // If the axis is unspecified, it defaults to "0 0 1"
        match *self {
            Rotate::None => (0., 0., 1., Angle::zero()),
            Rotate::Rotate3D(rx, ry, rz, angle) => (rx, ry, rz, angle),
            Rotate::Rotate(angle) => (0., 0., 1., angle),
        }
    }
}

impl Animate for ComputedRotate {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&Rotate::None, &Rotate::None) => Ok(Rotate::None),
            (&Rotate::Rotate3D(fx, fy, fz, fa), &Rotate::None) => {
                // We always normalize direction vector for rotate3d() first, so we should also
                // apply the same rule for rotate property. In other words, we promote none into
                // a 3d rotate, and normalize both direction vector first, and then do
                // interpolation.
                let (fx, fy, fz, fa) = transform::get_normalized_vector_and_angle(fx, fy, fz, fa);
                Ok(Rotate::Rotate3D(
                    fx,
                    fy,
                    fz,
                    fa.animate(&Angle::zero(), procedure)?,
                ))
            },
            (&Rotate::None, &Rotate::Rotate3D(tx, ty, tz, ta)) => {
                // Normalize direction vector first.
                let (tx, ty, tz, ta) = transform::get_normalized_vector_and_angle(tx, ty, tz, ta);
                Ok(Rotate::Rotate3D(
                    tx,
                    ty,
                    tz,
                    Angle::zero().animate(&ta, procedure)?,
                ))
            },
            (&Rotate::Rotate3D(_, ..), _) | (_, &Rotate::Rotate3D(_, ..)) => {
                let (from, to) = (self.resolve(), other.resolve());
                let (mut fx, mut fy, mut fz, fa) =
                    transform::get_normalized_vector_and_angle(from.0, from.1, from.2, from.3);
                let (mut tx, mut ty, mut tz, ta) =
                    transform::get_normalized_vector_and_angle(to.0, to.1, to.2, to.3);

                if fa == Angle::from_degrees(0.) {
                    fx = tx;
                    fy = ty;
                    fz = tz;
                } else if ta == Angle::from_degrees(0.) {
                    tx = fx;
                    ty = fy;
                    tz = fz;
                }

                if (fx, fy, fz) == (tx, ty, tz) {
                    return Ok(Rotate::Rotate3D(fx, fy, fz, fa.animate(&ta, procedure)?));
                }

                let fv = DirectionVector::new(fx, fy, fz);
                let tv = DirectionVector::new(tx, ty, tz);
                let fq = Quaternion::from_direction_and_angle(&fv, fa.radians64());
                let tq = Quaternion::from_direction_and_angle(&tv, ta.radians64());

                let rq = Quaternion::animate(&fq, &tq, procedure)?;
                let (x, y, z, angle) = transform::get_normalized_vector_and_angle(
                    rq.0 as f32,
                    rq.1 as f32,
                    rq.2 as f32,
                    rq.3.acos() as f32 * 2.0,
                );

                Ok(Rotate::Rotate3D(x, y, z, Angle::from_radians(angle)))
            },
            (&Rotate::Rotate(_), _) | (_, &Rotate::Rotate(_)) => {
                // If this is a 2D rotation, we just animate the <angle>
                let (from, to) = (self.resolve().3, other.resolve().3);
                Ok(Rotate::Rotate(from.animate(&to, procedure)?))
            },
        }
    }
}

impl ComputeSquaredDistance for ComputedRotate {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&Rotate::None, &Rotate::None) => Ok(SquaredDistance::from_sqrt(0.)),
            (&Rotate::Rotate3D(_, _, _, a), &Rotate::None) |
            (&Rotate::None, &Rotate::Rotate3D(_, _, _, a)) => {
                a.compute_squared_distance(&Angle::zero())
            },
            (&Rotate::Rotate3D(_, ..), _) | (_, &Rotate::Rotate3D(_, ..)) => {
                let (from, to) = (self.resolve(), other.resolve());
                let (mut fx, mut fy, mut fz, angle1) =
                    transform::get_normalized_vector_and_angle(from.0, from.1, from.2, from.3);
                let (mut tx, mut ty, mut tz, angle2) =
                    transform::get_normalized_vector_and_angle(to.0, to.1, to.2, to.3);

                if angle1 == Angle::zero() {
                    fx = tx;
                    fy = ty;
                    fz = tz;
                } else if angle2 == Angle::zero() {
                    tx = fx;
                    ty = fy;
                    tz = fz;
                }

                if (fx, fy, fz) == (tx, ty, tz) {
                    angle1.compute_squared_distance(&angle2)
                } else {
                    let v1 = DirectionVector::new(fx, fy, fz);
                    let v2 = DirectionVector::new(tx, ty, tz);
                    let q1 = Quaternion::from_direction_and_angle(&v1, angle1.radians64());
                    let q2 = Quaternion::from_direction_and_angle(&v2, angle2.radians64());
                    q1.compute_squared_distance(&q2)
                }
            },
            (&Rotate::Rotate(_), _) | (_, &Rotate::Rotate(_)) => self
                .resolve()
                .3
                .compute_squared_distance(&other.resolve().3),
        }
    }
}

/// <https://drafts.csswg.org/css-transforms-2/#propdef-translate>
impl ComputedTranslate {
    fn resolve(&self) -> (LengthPercentage, LengthPercentage, Length) {
        // According to the spec:
        // https://drafts.csswg.org/css-transforms-2/#individual-transforms
        //
        // Unspecified translations default to 0px
        match *self {
            Translate::None => (
                LengthPercentage::zero(),
                LengthPercentage::zero(),
                Length::zero(),
            ),
            Translate::Translate(ref tx, ref ty, ref tz) => (tx.clone(), ty.clone(), tz.clone()),
        }
    }
}

impl Animate for ComputedTranslate {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&Translate::None, &Translate::None) => Ok(Translate::None),
            (&Translate::Translate(_, ..), _) | (_, &Translate::Translate(_, ..)) => {
                let (from, to) = (self.resolve(), other.resolve());
                Ok(Translate::Translate(
                    from.0.animate(&to.0, procedure)?,
                    from.1.animate(&to.1, procedure)?,
                    from.2.animate(&to.2, procedure)?,
                ))
            },
        }
    }
}

impl ComputeSquaredDistance for ComputedTranslate {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let (from, to) = (self.resolve(), other.resolve());
        Ok(from.0.compute_squared_distance(&to.0)? +
            from.1.compute_squared_distance(&to.1)? +
            from.2.compute_squared_distance(&to.2)?)
    }
}

/// <https://drafts.csswg.org/css-transforms-2/#propdef-scale>
impl ComputedScale {
    fn resolve(&self) -> (Number, Number, Number) {
        // According to the spec:
        // https://drafts.csswg.org/css-transforms-2/#individual-transforms
        //
        // Unspecified scales default to 1
        match *self {
            Scale::None => (1.0, 1.0, 1.0),
            Scale::Scale(sx, sy, sz) => (sx, sy, sz),
        }
    }
}

impl Animate for ComputedScale {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&Scale::None, &Scale::None) => Ok(Scale::None),
            (&Scale::Scale(_, ..), _) | (_, &Scale::Scale(_, ..)) => {
                let (from, to) = (self.resolve(), other.resolve());
                // For transform lists, we add by appending to the list of
                // transform functions. However, ComputedScale cannot be
                // simply concatenated, so we have to calculate the additive
                // result here.
                if procedure == Procedure::Add {
                    // scale(x1,y1,z1)*scale(x2,y2,z2) = scale(x1*x2, y1*y2, z1*z2)
                    return Ok(Scale::Scale(from.0 * to.0, from.1 * to.1, from.2 * to.2));
                }
                Ok(Scale::Scale(
                    animate_multiplicative_factor(from.0, to.0, procedure)?,
                    animate_multiplicative_factor(from.1, to.1, procedure)?,
                    animate_multiplicative_factor(from.2, to.2, procedure)?,
                ))
            },
        }
    }
}

impl ComputeSquaredDistance for ComputedScale {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let (from, to) = (self.resolve(), other.resolve());
        Ok(from.0.compute_squared_distance(&to.0)? +
            from.1.compute_squared_distance(&to.1)? +
            from.2.compute_squared_distance(&to.2)?)
    }
}
