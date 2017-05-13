/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types that share their serialization implementations
//! for both specified and computed values.

use euclid::size::Size2D;
use std::fmt;
use style_traits::ToCss;
use super::HasViewportPercentage;
use super::computed::{Context, ToComputedValue};

pub use self::basic_shape::serialize_radius_values;

pub mod basic_shape;
pub mod position;

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A type for representing CSS `widthh` and `height` values.
pub struct BorderRadiusSize<L>(pub Size2D<L>);

impl<L> HasViewportPercentage for BorderRadiusSize<L> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool { false }
}

impl<L: Clone> From<L> for BorderRadiusSize<L> {
    fn from(other: L) -> Self {
        Self::new(other.clone(), other)
    }
}

impl<L> BorderRadiusSize<L> {
    #[inline]
    /// Create a new `BorderRadiusSize` for an area of given width and height.
    pub fn new(width: L, height: L) -> BorderRadiusSize<L> {
        BorderRadiusSize(Size2D::new(width, height))
    }
}

impl<L: Clone> BorderRadiusSize<L> {
    #[inline]
    /// Create a new `BorderRadiusSize` for a circle of given radius.
    pub fn circle(radius: L) -> BorderRadiusSize<L> {
        BorderRadiusSize(Size2D::new(radius.clone(), radius))
    }
}

impl<L: ToCss> ToCss for BorderRadiusSize<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.width.to_css(dest)?;
        dest.write_str(" ")?;
        self.0.height.to_css(dest)
    }
}

impl<L: ToComputedValue> ToComputedValue for BorderRadiusSize<L> {
    type ComputedValue = BorderRadiusSize<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let w = self.0.width.to_computed_value(context);
        let h = self.0.height.to_computed_value(context);
        BorderRadiusSize(Size2D::new(w, h))
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        let w = ToComputedValue::from_computed_value(&computed.0.width);
        let h = ToComputedValue::from_computed_value(&computed.0.height);
        BorderRadiusSize(Size2D::new(w, h))
    }
}
