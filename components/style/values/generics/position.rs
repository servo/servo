/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS handling of specified and computed values of
//! [`position`](https://drafts.csswg.org/css-backgrounds-3/#position)

/// A generic type for representing a CSS [position](https://drafts.csswg.org/css-values/#position).
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf,
         PartialEq, SpecifiedValueInfo, ToAnimatedZero, ToComputedValue)]
pub struct Position<H, V> {
    /// The horizontal component of position.
    pub horizontal: H,
    /// The vertical component of position.
    pub vertical: V,
}

impl<H, V> Position<H, V> {
    /// Returns a new position.
    pub fn new(horizontal: H, vertical: V) -> Self {
        Self {
            horizontal: horizontal,
            vertical: vertical,
        }
    }
}

/// A generic value for the `z-index` property.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf,
         PartialEq, SpecifiedValueInfo, ToAnimatedZero, ToComputedValue, ToCss)]
pub enum ZIndex<Integer> {
    /// An integer value.
    Integer(Integer),
    /// The keyword `auto`.
    Auto,
}

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
