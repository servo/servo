/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS handling of specified and computed values of
//! [`position`](https://drafts.csswg.org/css-backgrounds-3/#position)

use crate::values::animated::ToAnimatedZero;
use crate::values::generics::ratio::Ratio;

/// A generic type for representing a CSS [position](https://drafts.csswg.org/css-values/#position).
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericPosition<H, V> {
    /// The horizontal component of position.
    pub horizontal: H,
    /// The vertical component of position.
    pub vertical: V,
}

impl<H, V> PositionComponent for Position<H, V>
where
    H: PositionComponent,
    V: PositionComponent,
{
    #[inline]
    fn is_center(&self) -> bool {
        self.horizontal.is_center() && self.vertical.is_center()
    }
}

pub use self::GenericPosition as Position;

impl<H, V> Position<H, V> {
    /// Returns a new position.
    pub fn new(horizontal: H, vertical: V) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

/// Implements a method that checks if the position is centered.
pub trait PositionComponent {
    /// Returns if the position component is 50% or center.
    /// For pixel lengths, it always returns false.
    fn is_center(&self) -> bool;
}

/// A generic type for representing an `Auto | <position>`.
/// This is used by <offset-anchor> for now.
/// https://drafts.fxtf.org/motion-1/#offset-anchor-property
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
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericPositionOrAuto<Pos> {
    /// The <position> value.
    Position(Pos),
    /// The keyword `auto`.
    Auto,
}

pub use self::GenericPositionOrAuto as PositionOrAuto;

impl<Pos> PositionOrAuto<Pos> {
    /// Return `auto`.
    #[inline]
    pub fn auto() -> Self {
        PositionOrAuto::Auto
    }
}

/// A generic value for the `z-index` property.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericZIndex<I> {
    /// An integer value.
    Integer(I),
    /// The keyword `auto`.
    Auto,
}

pub use self::GenericZIndex as ZIndex;

impl<Integer> ZIndex<Integer> {
    /// Returns `auto`
    #[inline]
    pub fn auto() -> Self {
        ZIndex::Auto
    }

    /// Returns whether `self` is `auto`.
    #[inline]
    pub fn is_auto(self) -> bool {
        matches!(self, ZIndex::Auto)
    }

    /// Returns the integer value if it is an integer, or `auto`.
    #[inline]
    pub fn integer_or(self, auto: Integer) -> Integer {
        match self {
            ZIndex::Integer(n) => n,
            ZIndex::Auto => auto,
        }
    }
}

/// Ratio or None.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum PreferredRatio<N> {
    /// Without specified ratio
    #[css(skip)]
    None,
    /// With specified ratio
    Ratio(
        #[animation(field_bound)]
        #[css(field_bound)]
        #[distance(field_bound)]
        Ratio<N>,
    ),
}

/// A generic value for the `aspect-ratio` property, the value is `auto || <ratio>`.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericAspectRatio<N> {
    /// Specifiy auto or not.
    #[animation(constant)]
    #[css(represents_keyword)]
    pub auto: bool,
    /// The preferred aspect-ratio value.
    #[animation(field_bound)]
    #[css(field_bound)]
    #[distance(field_bound)]
    pub ratio: PreferredRatio<N>,
}

pub use self::GenericAspectRatio as AspectRatio;

impl<N> AspectRatio<N> {
    /// Returns `auto`
    #[inline]
    pub fn auto() -> Self {
        AspectRatio {
            auto: true,
            ratio: PreferredRatio::None,
        }
    }
}

impl<N> ToAnimatedZero for AspectRatio<N> {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}
