/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for the handling of [images].
//!
//! [images]: https://drafts.csswg.org/css-images/#image-values

use crate::custom_properties;
use crate::values::serialize_atom_identifier;
use crate::Atom;
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// An <image> | <none> (for background-image, for example).
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum GenericImageLayer<Image> {
    /// The `none` value.
    None,
    /// The `<image>` value.
    Image(Image),
}

pub use self::GenericImageLayer as ImageLayer;

impl<I> ImageLayer<I> {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        ImageLayer::None
    }
}

/// An [image].
///
/// [image]: https://drafts.csswg.org/css-images/#image-values
#[derive(
    Clone, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem,
)]
#[repr(C, u8)]
pub enum GenericImage<Gradient, MozImageRect, ImageUrl> {
    /// A `<url()>` image.
    Url(ImageUrl),
    /// A `<gradient>` image.  Gradients are rather large, and not nearly as
    /// common as urls, so we box them here to keep the size of this enum sane.
    Gradient(Box<Gradient>),
    /// A `-moz-image-rect` image.  Also fairly large and rare.
    Rect(Box<MozImageRect>),
    /// A `-moz-element(# <element-id>)`
    #[css(function = "-moz-element")]
    Element(Atom),
    /// A paint worklet image.
    /// <https://drafts.css-houdini.org/css-paint-api/>
    #[cfg(feature = "servo")]
    PaintWorklet(PaintWorklet),
}

pub use self::GenericImage as Image;

/// A CSS gradient.
/// <https://drafts.csswg.org/css-images/#gradients>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem)]
#[repr(C)]
pub struct GenericGradient<
    LineDirection,
    LengthPercentage,
    NonNegativeLength,
    NonNegativeLengthPercentage,
    Position,
    Color,
> {
    /// Gradients can be linear or radial.
    pub kind: GenericGradientKind<
        LineDirection,
        NonNegativeLength,
        NonNegativeLengthPercentage,
        Position,
    >,
    /// The color stops and interpolation hints.
    pub items: crate::OwnedSlice<GenericGradientItem<Color, LengthPercentage>>,
    /// True if this is a repeating gradient.
    pub repeating: bool,
    /// Compatibility mode.
    pub compat_mode: GradientCompatMode,
}

pub use self::GenericGradient as Gradient;

#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem)]
#[repr(u8)]
/// Whether we used the modern notation or the compatibility `-webkit`, `-moz` prefixes.
pub enum GradientCompatMode {
    /// Modern syntax.
    Modern,
    /// `-webkit` prefix.
    WebKit,
    /// `-moz` prefix
    Moz,
}

/// A gradient kind.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem)]
#[repr(C, u8)]
pub enum GenericGradientKind<
    LineDirection,
    NonNegativeLength,
    NonNegativeLengthPercentage,
    Position,
> {
    /// A linear gradient.
    Linear(LineDirection),
    /// A radial gradient.
    Radial(
        GenericEndingShape<NonNegativeLength, NonNegativeLengthPercentage>,
        Position,
    ),
}

pub use self::GenericGradientKind as GradientKind;

/// A radial gradient's ending shape.
#[derive(
    Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss, ToResolvedValue, ToShmem,
)]
#[repr(C, u8)]
pub enum GenericEndingShape<NonNegativeLength, NonNegativeLengthPercentage> {
    /// A circular gradient.
    Circle(GenericCircle<NonNegativeLength>),
    /// An elliptic gradient.
    Ellipse(GenericEllipse<NonNegativeLengthPercentage>),
}

pub use self::GenericEndingShape as EndingShape;

/// A circle shape.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem)]
#[repr(C, u8)]
pub enum GenericCircle<NonNegativeLength> {
    /// A circle radius.
    Radius(NonNegativeLength),
    /// A circle extent.
    Extent(ShapeExtent),
}

pub use self::GenericCircle as Circle;

/// An ellipse shape.
#[derive(
    Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss, ToResolvedValue, ToShmem,
)]
#[repr(C, u8)]
pub enum GenericEllipse<NonNegativeLengthPercentage> {
    /// An ellipse pair of radii.
    Radii(NonNegativeLengthPercentage, NonNegativeLengthPercentage),
    /// An ellipse extent.
    Extent(ShapeExtent),
}

pub use self::GenericEllipse as Ellipse;

/// <https://drafts.csswg.org/css-images/#typedef-extent-keyword>
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
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum ShapeExtent {
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    FarthestCorner,
    Contain,
    Cover,
}

/// A gradient item.
/// <https://drafts.csswg.org/css-images-4/#color-stop-syntax>
#[derive(
    Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss, ToResolvedValue, ToShmem,
)]
#[repr(C, u8)]
pub enum GenericGradientItem<Color, LengthPercentage> {
    /// A simple color stop, without position.
    SimpleColorStop(Color),
    /// A complex color stop, with a position.
    ComplexColorStop {
        /// The color for the stop.
        color: Color,
        /// The position for the stop.
        position: LengthPercentage,
    },
    /// An interpolation hint.
    InterpolationHint(LengthPercentage),
}

pub use self::GenericGradientItem as GradientItem;

/// A color stop.
/// <https://drafts.csswg.org/css-images/#typedef-color-stop-list>
#[derive(
    Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss, ToResolvedValue, ToShmem,
)]
pub struct ColorStop<Color, LengthPercentage> {
    /// The color of this stop.
    pub color: Color,
    /// The position of this stop.
    pub position: Option<LengthPercentage>,
}

impl<Color, LengthPercentage> ColorStop<Color, LengthPercentage> {
    /// Convert the color stop into an appropriate `GradientItem`.
    #[inline]
    pub fn into_item(self) -> GradientItem<Color, LengthPercentage> {
        match self.position {
            Some(position) => GradientItem::ComplexColorStop {
                color: self.color,
                position,
            },
            None => GradientItem::SimpleColorStop(self.color),
        }
    }
}

/// Specified values for a paint worklet.
/// <https://drafts.css-houdini.org/css-paint-api/>
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToComputedValue, ToResolvedValue, ToShmem)]
pub struct PaintWorklet {
    /// The name the worklet was registered with.
    pub name: Atom,
    /// The arguments for the worklet.
    /// TODO: store a parsed representation of the arguments.
    #[cfg_attr(feature = "servo", ignore_malloc_size_of = "Arc")]
    pub arguments: Vec<Arc<custom_properties::SpecifiedValue>>,
}

impl ::style_traits::SpecifiedValueInfo for PaintWorklet {}

impl ToCss for PaintWorklet {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("paint(")?;
        serialize_atom_identifier(&self.name, dest)?;
        for argument in &self.arguments {
            dest.write_str(", ")?;
            argument.to_css(dest)?;
        }
        dest.write_str(")")
    }
}

/// Values for `moz-image-rect`.
///
/// `-moz-image-rect(<uri>, top, right, bottom, left);`
#[allow(missing_docs)]
#[css(comma, function)]
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub struct MozImageRect<NumberOrPercentage, MozImageRectUrl> {
    pub url: MozImageRectUrl,
    pub top: NumberOrPercentage,
    pub right: NumberOrPercentage,
    pub bottom: NumberOrPercentage,
    pub left: NumberOrPercentage,
}

impl<G, R, U> fmt::Debug for Image<G, R, U>
where
    G: ToCss,
    R: ToCss,
    U: ToCss,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_css(&mut CssWriter::new(f))
    }
}

impl<G, R, U> ToCss for Image<G, R, U>
where
    G: ToCss,
    R: ToCss,
    U: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            Image::Url(ref url) => url.to_css(dest),
            Image::Gradient(ref gradient) => gradient.to_css(dest),
            Image::Rect(ref rect) => rect.to_css(dest),
            #[cfg(feature = "servo")]
            Image::PaintWorklet(ref paint_worklet) => paint_worklet.to_css(dest),
            Image::Element(ref selector) => {
                dest.write_str("-moz-element(#")?;
                serialize_atom_identifier(selector, dest)?;
                dest.write_str(")")
            },
        }
    }
}

impl<D, LP, NL, NLP, P, C> ToCss for Gradient<D, LP, NL, NLP, P, C>
where
    D: LineDirection,
    LP: ToCss,
    NL: ToCss,
    NLP: ToCss,
    P: ToCss,
    C: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self.compat_mode {
            GradientCompatMode::WebKit => dest.write_str("-webkit-")?,
            GradientCompatMode::Moz => dest.write_str("-moz-")?,
            _ => {},
        }

        if self.repeating {
            dest.write_str("repeating-")?;
        }
        dest.write_str(self.kind.label())?;
        dest.write_str("-gradient(")?;
        let mut skip_comma = match self.kind {
            GradientKind::Linear(ref direction) if direction.points_downwards(self.compat_mode) => {
                true
            },
            GradientKind::Linear(ref direction) => {
                direction.to_css(dest, self.compat_mode)?;
                false
            },
            GradientKind::Radial(ref shape, ref position) => {
                let omit_shape = match *shape {
                    EndingShape::Ellipse(Ellipse::Extent(ShapeExtent::Cover)) |
                    EndingShape::Ellipse(Ellipse::Extent(ShapeExtent::FarthestCorner)) => true,
                    _ => false,
                };
                if self.compat_mode == GradientCompatMode::Modern {
                    if !omit_shape {
                        shape.to_css(dest)?;
                        dest.write_str(" ")?;
                    }
                    dest.write_str("at ")?;
                    position.to_css(dest)?;
                } else {
                    position.to_css(dest)?;
                    if !omit_shape {
                        dest.write_str(", ")?;
                        shape.to_css(dest)?;
                    }
                }
                false
            },
        };
        for item in &*self.items {
            if !skip_comma {
                dest.write_str(", ")?;
            }
            skip_comma = false;
            item.to_css(dest)?;
        }
        dest.write_str(")")
    }
}

impl<D, L, LoP, P> GradientKind<D, L, LoP, P> {
    fn label(&self) -> &str {
        match *self {
            GradientKind::Linear(..) => "linear",
            GradientKind::Radial(..) => "radial",
        }
    }
}

/// The direction of a linear gradient.
pub trait LineDirection {
    /// Whether this direction points towards, and thus can be omitted.
    fn points_downwards(&self, compat_mode: GradientCompatMode) -> bool;

    /// Serialises this direction according to the compatibility mode.
    fn to_css<W>(&self, dest: &mut CssWriter<W>, compat_mode: GradientCompatMode) -> fmt::Result
    where
        W: Write;
}

impl<L> ToCss for Circle<L>
where
    L: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            Circle::Extent(ShapeExtent::FarthestCorner) | Circle::Extent(ShapeExtent::Cover) => {
                dest.write_str("circle")
            },
            Circle::Extent(keyword) => {
                dest.write_str("circle ")?;
                keyword.to_css(dest)
            },
            Circle::Radius(ref length) => length.to_css(dest),
        }
    }
}
