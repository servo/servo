/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to flexbox.

/// A generic value for the `flex-basis` property.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub enum GenericFlexBasis<S> {
    /// `content`
    Content,
    /// `<width>`
    Size(S),
}

pub use self::GenericFlexBasis as FlexBasis;
