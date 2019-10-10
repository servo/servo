/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS Motion Path.

use crate::values::specified::SVGPathData;

/// The <size> in ray() function.
///
/// https://drafts.fxtf.org/motion-1/#valdef-offsetpath-size
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum RaySize {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    FarthestCorner,
    Sides,
}

/// The `ray()` function, `ray( [ <angle> && <size> && contain? ] )`
///
/// https://drafts.fxtf.org/motion-1/#valdef-offsetpath-ray
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct RayFunction<Angle> {
    /// The bearing angle with `0deg` pointing up and positive angles
    /// representing clockwise rotation.
    pub angle: Angle,
    /// Decide the path length used when `offset-distance` is expressed
    /// as a percentage.
    #[animation(constant)]
    pub size: RaySize,
    /// Clamp `offset-distance` so that the box is entirely contained
    /// within the path.
    #[animation(constant)]
    #[css(represents_keyword)]
    pub contain: bool,
}

/// The offset-path value.
///
/// https://drafts.fxtf.org/motion-1/#offset-path-property
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericOffsetPath<Angle> {
    // We could merge SVGPathData into ShapeSource, so we could reuse them. However,
    // we don't want to support other value for offset-path, so use SVGPathData only for now.
    /// Path value for path(<string>).
    #[css(function)]
    Path(SVGPathData),
    /// ray() function, which defines a path in the polar coordinate system.
    #[css(function)]
    Ray(RayFunction<Angle>),
    /// None value.
    #[animation(error)]
    None,
    // Bug 1186329: Implement <basic-shape>, <geometry-box>, and <url>.
}

pub use self::GenericOffsetPath as OffsetPath;

impl<Angle> OffsetPath<Angle> {
    /// Return None.
    #[inline]
    pub fn none() -> Self {
        OffsetPath::None
    }
}
