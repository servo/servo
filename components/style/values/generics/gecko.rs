/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for legacy Gecko-only properties that should probably be
//! un-shipped at some point in the future.

/// A generic value for scroll snap points.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum ScrollSnapPoint<LengthPercentage> {
    /// `none`
    None,
    /// `repeat(<length-or-percentage>)`
    #[css(function)]
    Repeat(LengthPercentage),
}

impl<L> ScrollSnapPoint<L> {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        ScrollSnapPoint::None
    }

    /// Returns the repeat argument, if any.
    #[inline]
    pub fn repeated(&self) -> Option<&L> {
        match *self {
            ScrollSnapPoint::None => None,
            ScrollSnapPoint::Repeat(ref length) => Some(length),
        }
    }
}
