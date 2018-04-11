/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic values for pointing properties.

/// A generic value for the `caret-color` property.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         ToAnimatedValue, ToAnimatedZero, ToComputedValue, ToCss)]
pub enum CaretColor<Color> {
    /// An explicit color.
    Color(Color),
    /// The keyword `auto`.
    Auto,
}
