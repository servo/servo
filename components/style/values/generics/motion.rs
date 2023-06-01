/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS Motion Path.

use crate::values::generics::position::GenericPosition;
use crate::values::specified::SVGPathData;

/// The <size> in ray() function.
///
/// https://drafts.fxtf.org/motion-1/#valdef-offsetpath-size
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    Parse,
    PartialEq,
    Serialize,
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

impl RaySize {
    /// Returns true if it is the default value.
    #[inline]
    pub fn is_default(&self) -> bool {
        *self == RaySize::ClosestSide
    }
}

/// The `ray()` function, `ray( [ <angle> && <size> && contain? ] )`
///
/// https://drafts.fxtf.org/motion-1/#valdef-offsetpath-ray
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
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
    #[css(skip_if = "RaySize::is_default")]
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
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
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

/// The offset-position property, which specifies the offset starting position that is used by the
/// <offset-path> functions if they don’t specify their own starting position.
///
/// https://drafts.fxtf.org/motion-1/#offset-position-property
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    Parse,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericOffsetPosition<H, V> {
    /// The element does not have an offset starting position.
    Normal,
    /// The offset starting position is the top-left corner of the box.
    Auto,
    /// The offset starting position is the result of using the <position> to position a 0x0 object
    /// area within the box’s containing block.
    Position(
        #[css(field_bound)]
        #[parse(field_bound)]
        GenericPosition<H, V>,
    ),
}

pub use self::GenericOffsetPosition as OffsetPosition;

impl<H, V> OffsetPosition<H, V> {
    /// Returns the initial value, auto.
    #[inline]
    pub fn auto() -> Self {
        Self::Auto
    }
}
