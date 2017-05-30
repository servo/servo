/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are related to transformations.

use std::fmt;
use style_traits::ToCss;

/// A generic transform origin.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
pub struct TransformOrigin<H, V, Depth> {
    /// The horizontal origin.
    pub horizontal: H,
    /// The vertical origin.
    pub vertical: V,
    /// The depth.
    pub depth: Depth,
}

impl<H, V, D> TransformOrigin<H, V, D> {
    /// Returns a new transform origin.
    pub fn new(horizontal: H, vertical: V, depth: D) -> Self {
        Self {
            horizontal: horizontal,
            vertical: vertical,
            depth: depth,
        }
    }
}

impl<H, V, D> ToCss for TransformOrigin<H, V, D>
    where H: ToCss, V: ToCss, D: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        self.horizontal.to_css(dest)?;
        dest.write_str(" ")?;
        self.vertical.to_css(dest)?;
        dest.write_str(" ")?;
        self.depth.to_css(dest)
    }
}
