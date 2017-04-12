/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the [`basic-shape`](https://drafts.csswg.org/css-shapes/#typedef-basic-shape)
//! types that are generic over their `ToCss` implementations.

use euclid::size::Size2D;
use properties::shorthands::serialize_four_sides;
use std::fmt;
use style_traits::ToCss;
use values::computed::{Context, ToComputedValue};
use values::generics::BorderRadiusSize;

/// A generic type used for `border-radius`, `outline-radius` and `inset()` values.
///
/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct BorderRadius<L> {
    pub top_left: BorderRadiusSize<L>,
    pub top_right: BorderRadiusSize<L>,
    pub bottom_right: BorderRadiusSize<L>,
    pub bottom_left: BorderRadiusSize<L>,
}

/// Serialization helper for types of longhands like `border-radius` and `outline-radius`
pub fn serialize_radius_values<L, W>(dest: &mut W, top_left: &Size2D<L>,
                                     top_right: &Size2D<L>, bottom_right: &Size2D<L>,
                                     bottom_left: &Size2D<L>) -> fmt::Result
    where L: ToCss + PartialEq, W: fmt::Write
{
    if top_left.width == top_left.height && top_right.width == top_right.height &&
       bottom_right.width == bottom_right.height && bottom_left.width == bottom_left.height {
        serialize_four_sides(dest, &top_left.width, &top_right.width,
                             &bottom_right.width, &bottom_left.width)
    } else {
        serialize_four_sides(dest, &top_left.width, &top_right.width,
                             &bottom_right.width, &bottom_left.width)?;
        dest.write_str(" / ")?;
        serialize_four_sides(dest, &top_left.height, &top_right.height,
                             &bottom_right.height, &bottom_left.height)
    }
}

impl<L: ToCss + PartialEq> ToCss for BorderRadius<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_radius_values(dest, &self.top_left.0, &self.top_right.0,
                                &self.bottom_right.0, &self.bottom_left.0)
    }
}

impl<L: ToComputedValue> ToComputedValue for BorderRadius<L> {
    type ComputedValue = BorderRadius<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        BorderRadius {
            top_left: self.top_left.to_computed_value(cx),
            top_right: self.top_right.to_computed_value(cx),
            bottom_right: self.bottom_right.to_computed_value(cx),
            bottom_left: self.bottom_left.to_computed_value(cx),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        BorderRadius {
            top_left: ToComputedValue::from_computed_value(&computed.top_left),
            top_right: ToComputedValue::from_computed_value(&computed.top_right),
            bottom_right: ToComputedValue::from_computed_value(&computed.bottom_right),
            bottom_left: ToComputedValue::from_computed_value(&computed.bottom_left),
        }
    }
}

/// https://drafts.csswg.org/css-shapes/#typedef-shape-radius
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum ShapeRadius<L> {
    Length(L),
    ClosestSide,
    FarthestSide,
}

impl<L> Default for ShapeRadius<L> {
    #[inline]
    fn default() -> Self { ShapeRadius::ClosestSide }
}

impl<L: ToCss> ToCss for ShapeRadius<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ShapeRadius::Length(ref lop) => lop.to_css(dest),
            ShapeRadius::ClosestSide => dest.write_str("closest-side"),
            ShapeRadius::FarthestSide => dest.write_str("farthest-side"),
        }
    }
}

impl<L: ToComputedValue> ToComputedValue for ShapeRadius<L> {
    type ComputedValue = ShapeRadius<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            ShapeRadius::Length(ref lop) => ShapeRadius::Length(lop.to_computed_value(cx)),
            ShapeRadius::ClosestSide => ShapeRadius::ClosestSide,
            ShapeRadius::FarthestSide => ShapeRadius::FarthestSide,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            ShapeRadius::Length(ref lop) => ShapeRadius::Length(ToComputedValue::from_computed_value(lop)),
            ShapeRadius::ClosestSide => ShapeRadius::ClosestSide,
            ShapeRadius::FarthestSide => ShapeRadius::FarthestSide,
        }
    }
}
