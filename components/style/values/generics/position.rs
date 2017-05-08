/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS handling of specified and computed values of
//! [`position`](https://drafts.csswg.org/css-backgrounds-3/#position)

use values::HasViewportPercentage;
use values::computed::{Context, ToComputedValue};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A generic type for representing a CSS [position](https://drafts.csswg.org/css-values/#position).
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

impl<H: HasViewportPercentage, V: HasViewportPercentage> HasViewportPercentage for Position<H, V> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.horizontal.has_viewport_percentage() || self.vertical.has_viewport_percentage()
    }
}

impl<H: ToComputedValue, V: ToComputedValue> ToComputedValue for Position<H, V> {
    type ComputedValue = Position<<H as ToComputedValue>::ComputedValue,
                                  <V as ToComputedValue>::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        Position {
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self {
            horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
            vertical: ToComputedValue::from_computed_value(&computed.vertical),
        }
    }
}
