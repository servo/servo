/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values that are related to motion path.

use crate::values::computed::Angle;
use crate::values::generics::motion::GenericOffsetPath;
use crate::Zero;

/// The computed value of `offset-path`.
pub type OffsetPath = GenericOffsetPath<Angle>;

#[inline]
fn is_auto_zero_angle(auto: &bool, angle: &Angle) -> bool {
    *auto && angle.is_zero()
}

/// A computed offset-rotate.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    ToAnimatedZero,
    ToCss,
    ToResolvedValue,
)]
#[repr(C)]
pub struct OffsetRotate {
    /// If auto is false, this is a fixed angle which indicates a
    /// constant clockwise rotation transformation applied to it by this
    /// specified rotation angle. Otherwise, the angle will be added to
    /// the angle of the direction in layout.
    #[animation(constant)]
    #[css(represents_keyword)]
    pub auto: bool,
    /// The angle value.
    #[css(contextual_skip_if = "is_auto_zero_angle")]
    pub angle: Angle,
}

impl OffsetRotate {
    /// Returns "auto 0deg".
    #[inline]
    pub fn auto() -> Self {
        OffsetRotate {
            auto: true,
            angle: Zero::zero(),
        }
    }
}
