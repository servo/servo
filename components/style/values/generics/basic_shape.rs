/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the [`basic-shape`](https://drafts.csswg.org/css-shapes/#typedef-basic-shape)
//! types that are generic over their `ToCss` implementations.

use euclid::size::Size2D;
use std::fmt;
use style_traits::{HasViewportPercentage, ToCss};
use values::computed::ComputedValueAsSpecified;
use values::generics::BorderRadiusSize;
use values::generics::position::Position;
use values::generics::rect::Rect;
use values::specified::url::SpecifiedUrl;

/// A clipping shape, for `clip-path`.
pub type ClippingShape<BasicShape> = ShapeSource<BasicShape, GeometryBox>;

/// https://drafts.fxtf.org/css-masking-1/#typedef-geometry-box
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GeometryBox {
    FillBox,
    StrokeBox,
    ViewBox,
    ShapeBox(ShapeBox),
}
impl ComputedValueAsSpecified for GeometryBox {}

/// A float area shape, for `shape-outside`.
pub type FloatAreaShape<BasicShape> = ShapeSource<BasicShape, ShapeBox>;

// https://drafts.csswg.org/css-shapes-1/#typedef-shape-box
define_css_keyword_enum!(ShapeBox:
    "margin-box" => MarginBox,
    "border-box" => BorderBox,
    "padding-box" => PaddingBox,
    "content-box" => ContentBox
);
add_impls_for_keyword_enum!(ShapeBox);

/// A shape source, for some reference box.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToComputedValue)]
pub enum ShapeSource<BasicShape, ReferenceBox> {
    Url(SpecifiedUrl),
    Shape(BasicShape, Option<ReferenceBox>),
    Box(ReferenceBox),
    None,
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToComputedValue)]
pub enum BasicShape<H, V, LengthOrPercentage> {
    Inset(InsetRect<LengthOrPercentage>),
    Circle(Circle<H, V, LengthOrPercentage>),
    Ellipse(Ellipse<H, V, LengthOrPercentage>),
    Polygon(Polygon<LengthOrPercentage>),
}

/// https://drafts.csswg.org/css-shapes/#funcdef-inset
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToComputedValue)]
pub struct InsetRect<LengthOrPercentage> {
    pub rect: Rect<LengthOrPercentage>,
    pub round: Option<BorderRadius<LengthOrPercentage>>,
}

/// A generic type used for `border-radius`, `outline-radius` and `inset()` values.
///
/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToComputedValue)]
pub struct BorderRadius<LengthOrPercentage> {
    /// The top left radius.
    pub top_left: BorderRadiusSize<LengthOrPercentage>,
    /// The top right radius.
    pub top_right: BorderRadiusSize<LengthOrPercentage>,
    /// The bottom right radius.
    pub bottom_right: BorderRadiusSize<LengthOrPercentage>,
    /// The bottom left radius.
    pub bottom_left: BorderRadiusSize<LengthOrPercentage>,
}

/// https://drafts.csswg.org/css-shapes/#funcdef-circle
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToComputedValue)]
pub struct Circle<H, V, LengthOrPercentage> {
    pub position: Position<H, V>,
    pub radius: ShapeRadius<LengthOrPercentage>,
}

/// https://drafts.csswg.org/css-shapes/#funcdef-ellipse
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToComputedValue)]
pub struct Ellipse<H, V, LengthOrPercentage> {
    pub position: Position<H, V>,
    pub semiaxis_x: ShapeRadius<LengthOrPercentage>,
    pub semiaxis_y: ShapeRadius<LengthOrPercentage>,
}

/// https://drafts.csswg.org/css-shapes/#typedef-shape-radius
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToComputedValue)]
pub enum ShapeRadius<LengthOrPercentage> {
    Length(LengthOrPercentage),
    ClosestSide,
    FarthestSide,
}

#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToComputedValue)]
/// A generic type for representing the `polygon()` function
///
/// https://drafts.csswg.org/css-shapes/#funcdef-polygon
pub struct Polygon<LengthOrPercentage> {
    /// The filling rule for a polygon.
    pub fill: FillRule,
    /// A collection of (x, y) coordinates to draw the polygon.
    pub coordinates: Vec<(LengthOrPercentage, LengthOrPercentage)>,
}

// https://drafts.csswg.org/css-shapes/#typedef-fill-rule
// NOTE: Basic shapes spec says that these are the only two values, however
// https://www.w3.org/TR/SVG/painting.html#FillRuleProperty
// says that it can also be `inherit`
define_css_keyword_enum!(FillRule:
    "nonzero" => NonZero,
    "evenodd" => EvenOdd
);
add_impls_for_keyword_enum!(FillRule);

impl<B, T> HasViewportPercentage for ShapeSource<B, T> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool { false }
}

impl<B: ToCss, T: ToCss> ToCss for ShapeSource<B, T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ShapeSource::Url(ref url) => url.to_css(dest),
            ShapeSource::Shape(ref shape, Some(ref ref_box)) => {
                shape.to_css(dest)?;
                dest.write_str(" ")?;
                ref_box.to_css(dest)
            },
            ShapeSource::Shape(ref shape, None) => shape.to_css(dest),
            ShapeSource::Box(ref val) => val.to_css(dest),
            ShapeSource::None => dest.write_str("none"),
        }
    }
}

impl ToCss for GeometryBox {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            GeometryBox::FillBox => dest.write_str("fill-box"),
            GeometryBox::StrokeBox => dest.write_str("stroke-box"),
            GeometryBox::ViewBox => dest.write_str("view-box"),
            GeometryBox::ShapeBox(s) => s.to_css(dest),
        }
    }
}

impl<H, V, L> ToCss for BasicShape<H, V, L>
    where H: ToCss,
          V: ToCss,
          L: PartialEq + ToCss,
          Circle<H, V, L>: ToCss,
          Ellipse<H, V, L>: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            BasicShape::Inset(ref rect) => rect.to_css(dest),
            BasicShape::Circle(ref circle) => circle.to_css(dest),
            BasicShape::Ellipse(ref ellipse) => ellipse.to_css(dest),
            BasicShape::Polygon(ref polygon) => polygon.to_css(dest),
        }
    }
}

impl<L> ToCss for InsetRect<L>
    where L: ToCss + PartialEq
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("inset(")?;
        self.rect.to_css(dest)?;
        if let Some(ref radius) = self.round {
            dest.write_str(" round ")?;
            radius.to_css(dest)?;
        }
        dest.write_str(")")
    }
}

impl<L: ToCss + PartialEq> ToCss for BorderRadius<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_radius_values(dest, &self.top_left.0, &self.top_right.0,
                                &self.bottom_right.0, &self.bottom_left.0)
    }
}

/// Serialization helper for types of longhands like `border-radius` and `outline-radius`
pub fn serialize_radius_values<L, W>(dest: &mut W, top_left: &Size2D<L>,
                                     top_right: &Size2D<L>, bottom_right: &Size2D<L>,
                                     bottom_left: &Size2D<L>) -> fmt::Result
    where L: ToCss + PartialEq, W: fmt::Write
{
    Rect::new(&top_left.width, &top_right.width, &bottom_right.width, &bottom_left.width).to_css(dest)?;
    if
        top_left.width != top_left.height ||
        top_right.width != top_right.height ||
        bottom_right.width != bottom_right.height ||
        bottom_left.width != bottom_left.height
    {
        dest.write_str(" / ")?;
        Rect::new(&top_left.height, &top_right.height, &bottom_right.height, &bottom_left.height).to_css(dest)?;
    }
    Ok(())
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

impl<L: ToCss> ToCss for Polygon<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("polygon(")?;
        if self.fill != FillRule::default() {
            self.fill.to_css(dest)?;
            dest.write_str(", ")?;
        }

        for (i, coord) in self.coordinates.iter().enumerate() {
            if i > 0 {
                dest.write_str(", ")?;
            }

            coord.0.to_css(dest)?;
            dest.write_str(" ")?;
            coord.1.to_css(dest)?;
        }

        dest.write_str(")")
    }
}

impl Default for FillRule {
    #[inline]
    fn default() -> Self { FillRule::NonZero }
}
