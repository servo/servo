/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the [`basic-shape`](https://drafts.csswg.org/css-shapes/#typedef-basic-shape)
//! types that are generic over their `ToCss` implementations.

use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::border::GenericBorderRadius;
use crate::values::generics::position::GenericPosition;
use crate::values::generics::rect::Rect;
use crate::values::specified::SVGPathData;
use crate::Zero;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A clipping shape, for `clip-path`.
pub type ClippingShape<BasicShape, Url> = ShapeSource<BasicShape, GeometryBox, Url>;

/// <https://drafts.fxtf.org/css-masking-1/#typedef-geometry-box>
#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum GeometryBox {
    FillBox,
    StrokeBox,
    ViewBox,
    ShapeBox(ShapeBox),
}

/// A float area shape, for `shape-outside`.
pub type FloatAreaShape<BasicShape, Image> = ShapeSource<BasicShape, ShapeBox, Image>;

/// https://drafts.csswg.org/css-shapes-1/#typedef-shape-box
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Animate,
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum ShapeBox {
    MarginBox,
    BorderBox,
    PaddingBox,
    ContentBox,
}

/// A shape source, for some reference box.
#[allow(missing_docs)]
#[animation(no_bound(ImageOrUrl))]
#[derive(
    Animate,
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum ShapeSource<BasicShape, ReferenceBox, ImageOrUrl> {
    #[animation(error)]
    ImageOrUrl(ImageOrUrl),
    Shape(Box<BasicShape>, Option<ReferenceBox>),
    #[animation(error)]
    Box(ReferenceBox),
    #[css(function)]
    Path(Path),
    #[animation(error)]
    None,
}

#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericBasicShape<H, V, LengthPercentage, NonNegativeLengthPercentage> {
    Inset(
        #[css(field_bound)]
        #[shmem(field_bound)]
        InsetRect<LengthPercentage, NonNegativeLengthPercentage>,
    ),
    Circle(
        #[css(field_bound)]
        #[shmem(field_bound)]
        Circle<H, V, NonNegativeLengthPercentage>,
    ),
    Ellipse(
        #[css(field_bound)]
        #[shmem(field_bound)]
        Ellipse<H, V, NonNegativeLengthPercentage>,
    ),
    Polygon(GenericPolygon<LengthPercentage>),
}

pub use self::GenericBasicShape as BasicShape;

/// <https://drafts.csswg.org/css-shapes/#funcdef-inset>
#[allow(missing_docs)]
#[css(function = "inset")]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct InsetRect<LengthPercentage, NonNegativeLengthPercentage> {
    pub rect: Rect<LengthPercentage>,
    #[shmem(field_bound)]
    pub round: GenericBorderRadius<NonNegativeLengthPercentage>,
}

/// <https://drafts.csswg.org/css-shapes/#funcdef-circle>
#[allow(missing_docs)]
#[css(function)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct Circle<H, V, NonNegativeLengthPercentage> {
    pub position: GenericPosition<H, V>,
    pub radius: GenericShapeRadius<NonNegativeLengthPercentage>,
}

/// <https://drafts.csswg.org/css-shapes/#funcdef-ellipse>
#[allow(missing_docs)]
#[css(function)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct Ellipse<H, V, NonNegativeLengthPercentage> {
    pub position: GenericPosition<H, V>,
    pub semiaxis_x: GenericShapeRadius<NonNegativeLengthPercentage>,
    pub semiaxis_y: GenericShapeRadius<NonNegativeLengthPercentage>,
}

/// <https://drafts.csswg.org/css-shapes/#typedef-shape-radius>
#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericShapeRadius<NonNegativeLengthPercentage> {
    Length(NonNegativeLengthPercentage),
    #[animation(error)]
    ClosestSide,
    #[animation(error)]
    FarthestSide,
}

pub use self::GenericShapeRadius as ShapeRadius;

/// A generic type for representing the `polygon()` function
///
/// <https://drafts.csswg.org/css-shapes/#funcdef-polygon>
#[css(comma, function = "polygon")]
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericPolygon<LengthPercentage> {
    /// The filling rule for a polygon.
    #[css(skip_if = "fill_is_default")]
    pub fill: FillRule,
    /// A collection of (x, y) coordinates to draw the polygon.
    #[css(iterable)]
    pub coordinates: crate::OwnedSlice<PolygonCoord<LengthPercentage>>,
}

pub use self::GenericPolygon as Polygon;

/// Coordinates for Polygon.
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct PolygonCoord<LengthPercentage>(pub LengthPercentage, pub LengthPercentage);

// https://drafts.csswg.org/css-shapes/#typedef-fill-rule
// NOTE: Basic shapes spec says that these are the only two values, however
// https://www.w3.org/TR/SVG/painting.html#FillRuleProperty
// says that it can also be `inherit`
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum FillRule {
    Nonzero,
    Evenodd,
}

/// The path function defined in css-shape-2.
///
/// https://drafts.csswg.org/css-shapes-2/#funcdef-path
#[css(comma)]
#[derive(
    Animate,
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub struct Path {
    /// The filling rule for the svg path.
    #[css(skip_if = "fill_is_default")]
    #[animation(constant)]
    pub fill: FillRule,
    /// The svg path data.
    pub path: SVGPathData,
}

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
            ) if this_box == other_box => this.compute_squared_distance(other),
            (&ShapeSource::Path(ref this), &ShapeSource::Path(ref other))
                if this.fill == other.fill =>
            {
                this.path.compute_squared_distance(&other.path)
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

impl<Length, NonNegativeLength> ToCss for InsetRect<Length, NonNegativeLength>
where
    Length: ToCss + PartialEq,
    NonNegativeLength: ToCss + PartialEq + Zero,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("inset(")?;
        self.rect.to_css(dest)?;
        if !self.round.is_zero() {
            dest.write_str(" round ")?;
            self.round.to_css(dest)?;
        }
        dest.write_str(")")
    }
}

impl<L> Default for ShapeRadius<L> {
    #[inline]
    fn default() -> Self {
        ShapeRadius::ClosestSide
    }
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
        let coordinates = self
            .coordinates
            .iter()
            .zip(other.coordinates.iter())
            .map(|(this, other)| {
                Ok(PolygonCoord(
                    this.0.animate(&other.0, procedure)?,
                    this.1.animate(&other.1, procedure)?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into();
        Ok(Polygon {
            fill: self.fill,
            coordinates,
        })
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
        self.coordinates
            .iter()
            .zip(other.coordinates.iter())
            .map(|(this, other)| {
                let d1 = this.0.compute_squared_distance(&other.0)?;
                let d2 = this.1.compute_squared_distance(&other.1)?;
                Ok(d1 + d2)
            })
            .sum()
    }
}

impl Default for FillRule {
    #[inline]
    fn default() -> Self {
        FillRule::Nonzero
    }
}

#[inline]
fn fill_is_default(fill: &FillRule) -> bool {
    *fill == FillRule::default()
}
