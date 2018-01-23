/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the [`basic-shape`](https://drafts.csswg.org/css-shapes/#typedef-basic-shape)
//! types that are generic over their `ToCss` implementations.

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::animated::{Animate, Procedure, ToAnimatedZero};
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::border::BorderRadius;
use values::generics::position::Position;
use values::generics::rect::Rect;

/// A clipping shape, for `clip-path`.
pub type ClippingShape<BasicShape, Url> = ShapeSource<BasicShape, GeometryBox, Url>;

/// <https://drafts.fxtf.org/css-masking-1/#typedef-geometry-box>
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub enum GeometryBox {
    FillBox,
    StrokeBox,
    ViewBox,
    ShapeBox(ShapeBox),
}

/// A float area shape, for `shape-outside`.
pub type FloatAreaShape<BasicShape, Image> = ShapeSource<BasicShape, ShapeBox, Image>;

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
#[derive(Animate, Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub enum ShapeSource<BasicShape, ReferenceBox, ImageOrUrl> {
    #[animation(error)]
    ImageOrUrl(ImageOrUrl),
    Shape(
        BasicShape,
        #[animation(constant)]
        Option<ReferenceBox>,
    ),
    #[animation(error)]
    Box(ReferenceBox),
    #[animation(error)]
    None,
}

#[allow(missing_docs)]
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq)]
#[derive(ToComputedValue, ToCss)]
pub enum BasicShape<H, V, LengthOrPercentage> {
    Inset(InsetRect<LengthOrPercentage>),
    Circle(Circle<H, V, LengthOrPercentage>),
    Ellipse(Ellipse<H, V, LengthOrPercentage>),
    Polygon(Polygon<LengthOrPercentage>),
}

/// <https://drafts.csswg.org/css-shapes/#funcdef-inset>
#[allow(missing_docs)]
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct InsetRect<LengthOrPercentage> {
    pub rect: Rect<LengthOrPercentage>,
    pub round: Option<BorderRadius<LengthOrPercentage>>,
}

/// <https://drafts.csswg.org/css-shapes/#funcdef-circle>
#[allow(missing_docs)]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Circle<H, V, LengthOrPercentage> {
    pub position: Position<H, V>,
    pub radius: ShapeRadius<LengthOrPercentage>,
}

/// <https://drafts.csswg.org/css-shapes/#funcdef-ellipse>
#[allow(missing_docs)]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Ellipse<H, V, LengthOrPercentage> {
    pub position: Position<H, V>,
    pub semiaxis_x: ShapeRadius<LengthOrPercentage>,
    pub semiaxis_y: ShapeRadius<LengthOrPercentage>,
}

/// <https://drafts.csswg.org/css-shapes/#typedef-shape-radius>
#[allow(missing_docs)]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq)]
#[derive(ToComputedValue, ToCss)]
pub enum ShapeRadius<LengthOrPercentage> {
    Length(LengthOrPercentage),
    #[animation(error)]
    ClosestSide,
    #[animation(error)]
    FarthestSide,
}

/// A generic type for representing the `polygon()` function
///
/// <https://drafts.csswg.org/css-shapes/#funcdef-polygon>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
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

// FIXME(nox): Implement ComputeSquaredDistance for T types and stop
// using PartialEq here, this will let us derive this impl.
impl<B, T, U> ComputeSquaredDistance for ShapeSource<B, T, U>
where
    B: ComputeSquaredDistance,
    T: PartialEq,
{
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (
                &ShapeSource::Shape(ref this, ref this_box),
                &ShapeSource::Shape(ref other, ref other_box),
            ) if this_box == other_box => {
                this.compute_squared_distance(other)
            },
            _ => Err(()),
        }
    }
}

impl<B, T, U> ToAnimatedZero for ShapeSource<B, T, U> {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

impl<L> ToCss for InsetRect<L>
    where L: ToCss + PartialEq
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("inset(")?;
        self.rect.to_css(dest)?;
        if let Some(ref radius) = self.round {
            dest.write_str(" round ")?;
            radius.to_css(dest)?;
        }
        dest.write_str(")")
    }
}

impl<L> Default for ShapeRadius<L> {
    #[inline]
    fn default() -> Self { ShapeRadius::ClosestSide }
}

impl<L> Animate for Polygon<L>
where
    L: Animate,
{
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.fill != other.fill {
            return Err(());
        }
        if self.coordinates.len() != other.coordinates.len() {
            return Err(());
        }
        let coordinates = self.coordinates.iter().zip(other.coordinates.iter()).map(|(this, other)| {
            Ok((
                this.0.animate(&other.0, procedure)?,
                this.1.animate(&other.1, procedure)?,
            ))
        }).collect::<Result<Vec<_>, _>>()?;
        Ok(Polygon { fill: self.fill, coordinates })
    }
}

impl<L> ComputeSquaredDistance for Polygon<L>
where
    L: ComputeSquaredDistance,
{
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.fill != other.fill {
            return Err(());
        }
        if self.coordinates.len() != other.coordinates.len() {
            return Err(());
        }
        self.coordinates.iter().zip(other.coordinates.iter()).map(|(this, other)| {
            Ok(
                this.0.compute_squared_distance(&other.0)? +
                this.1.compute_squared_distance(&other.1)?,
            )
        }).sum()
    }
}

impl<L: ToCss> ToCss for Polygon<L> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
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
