/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to flexbox.

/// A generic value for the `flex-basis` property.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, PartialEq)]
#[derive(ToAnimatedValue, ToAnimatedZero, ToComputedValue, ToCss)]
pub enum FlexBasis<Width> {
    /// `content`
    Content,
    /// `<width>`
    Width(Width),
}
