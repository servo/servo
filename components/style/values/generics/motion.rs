/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS Motion Path.

use crate::values::animated::ToAnimatedZero;
use crate::values::generics::position::{GenericPosition, GenericPositionOrAuto};
use crate::values::specified::SVGPathData;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// The <size> in ray() function.
///
/// https://drafts.fxtf.org/motion-1/#valdef-offsetpath-size
#[allow(missing_docs)]
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

/// The `ray()` function, `ray( [ <angle> && <size> && contain? && [at <position>]? ] )`
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
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericRayFunction<Angle, Position> {
    /// The bearing angle with `0deg` pointing up and positive angles
    /// representing clockwise rotation.
    pub angle: Angle,
    /// Decide the path length used when `offset-distance` is expressed
    /// as a percentage.
    pub size: RaySize,
    /// Clamp `offset-distance` so that the box is entirely contained
    /// within the path.
    #[animation(constant)]
    pub contain: bool,
    /// The "at <position>" part. If omitted, we use auto to represent it.
    pub position: GenericPositionOrAuto<Position>,
}

pub use self::GenericRayFunction as RayFunction;

impl<Angle, Position> ToCss for RayFunction<Angle, Position>
where
    Angle: ToCss,
    Position: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.angle.to_css(dest)?;

        if !matches!(self.size, RaySize::ClosestSide) {
            dest.write_char(' ')?;
            self.size.to_css(dest)?;
        }

        if self.contain {
            dest.write_str(" contain")?;
        }

        if !matches!(self.position, GenericPositionOrAuto::Auto) {
            dest.write_str(" at ")?;
            self.position.to_css(dest)?;
        }

        Ok(())
    }
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
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericOffsetPath<RayFunction> {
    // We could merge SVGPathData into ShapeSource, so we could reuse them. However,
    // we don't want to support other value for offset-path, so use SVGPathData only for now.
    /// Path value for path(<string>).
    #[css(function)]
    Path(SVGPathData),
    /// ray() function, which defines a path in the polar coordinate system.
    /// Use Box<> to make sure the size of offset-path is not too large.
    #[css(function)]
    Ray(Box<RayFunction>),
    /// None value.
    #[animation(error)]
    None,
    // Bug 1186329: Implement <basic-shape>, <geometry-box>, and <url>.
}

pub use self::GenericOffsetPath as OffsetPath;

impl<Ray> OffsetPath<Ray> {
    /// Return None.
    #[inline]
    pub fn none() -> Self {
        OffsetPath::None
    }
}

impl<Ray> ToAnimatedZero for OffsetPath<Ray> {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
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
