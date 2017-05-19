/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for the handling of [images].
//!
//! [images]: https://drafts.csswg.org/css-images/#image-values

use Atom;
use cssparser::serialize_identifier;
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;
use values::computed::{Context, ToComputedValue};
use values::specified::url::SpecifiedUrl;

/// An [image].
///
/// [image]: https://drafts.csswg.org/css-images/#image-values
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Image<Gradient, ImageRect> {
    /// A `<url()>` image.
    Url(SpecifiedUrl),
    /// A `<gradient>` image.
    Gradient(Gradient),
    /// A `-moz-image-rect` image
    Rect(ImageRect),
    /// A `-moz-element(# <element-id>)`
    Element(Atom),
}

/// A CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
#[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Gradient<LineDirection, Length, LengthOrPercentage, Position, Color> {
    /// Gradients can be linear or radial.
    pub kind: GradientKind<LineDirection, Length, LengthOrPercentage, Position>,
    /// The color stops and interpolation hints.
    pub items: Vec<GradientItem<Color, LengthOrPercentage>>,
    /// True if this is a repeating gradient.
    pub repeating: bool,
    /// Compatibility mode.
    pub compat_mode: CompatMode,
}

#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// Whether we used the modern notation or the compatibility `-webkit` prefix.
pub enum CompatMode {
    /// Modern syntax.
    Modern,
    /// `-webkit` prefix.
    WebKit,
}

/// A gradient kind.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum GradientKind<LineDirection, Length, LengthOrPercentage, Position> {
    /// A linear gradient.
    Linear(LineDirection),
    /// A radial gradient.
    Radial(EndingShape<Length, LengthOrPercentage>, Position),
}

/// A radial gradient's ending shape.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum EndingShape<Length, LengthOrPercentage> {
    /// A circular gradient.
    Circle(Circle<Length>),
    /// An elliptic gradient.
    Ellipse(Ellipse<LengthOrPercentage>),
}

/// A circle shape.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Circle<Length> {
    /// A circle radius.
    Radius(Length),
    /// A circle extent.
    Extent(ShapeExtent),
}

/// An ellipse shape.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Ellipse<LengthOrPercentage> {
    /// An ellipse pair of radii.
    Radii(LengthOrPercentage, LengthOrPercentage),
    /// An ellipse extent.
    Extent(ShapeExtent),
}

/// https://drafts.csswg.org/css-images/#typedef-extent-keyword
define_css_keyword_enum!(ShapeExtent:
    "closest-side" => ClosestSide,
    "farthest-side" => FarthestSide,
    "closest-corner" => ClosestCorner,
    "farthest-corner" => FarthestCorner,
    "contain" => Contain,
    "cover" => Cover
);
no_viewport_percentage!(ShapeExtent);

/// A gradient item.
/// https://drafts.csswg.org/css-images-4/#color-stop-syntax
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum GradientItem<Color, LengthOrPercentage> {
    /// A color stop.
    ColorStop(ColorStop<Color, LengthOrPercentage>),
    /// An interpolation hint.
    InterpolationHint(LengthOrPercentage),
}

/// A color stop.
/// https://drafts.csswg.org/css-images/#typedef-color-stop-list
#[derive(Clone, Copy, HasViewportPercentage, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ColorStop<Color, LengthOrPercentage> {
    /// The color of this stop.
    pub color: Color,
    /// The position of this stop.
    pub position: Option<LengthOrPercentage>,
}

/// Values for `moz-image-rect`.
///
/// `-moz-image-rect(<uri>, top, right, bottom, left);`
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct ImageRect<NumberOrPercentage> {
    pub url: SpecifiedUrl,
    pub top: NumberOrPercentage,
    pub bottom: NumberOrPercentage,
    pub right: NumberOrPercentage,
    pub left: NumberOrPercentage,
}

impl<G, R> fmt::Debug for Image<G, R>
    where G: fmt::Debug, R: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Image::Url(ref url) => url.to_css(f),
            Image::Gradient(ref grad) => grad.fmt(f),
            Image::Rect(ref rect) => rect.fmt(f),
            Image::Element(ref selector) => {
                f.write_str("-moz-element(#")?;
                serialize_identifier(&selector.to_string(), f)?;
                f.write_str(")")
            },
        }
    }
}

impl<G, R> ToCss for Image<G, R>
    where G: ToCss, R: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Image::Url(ref url) => url.to_css(dest),
            Image::Gradient(ref gradient) => gradient.to_css(dest),
            Image::Rect(ref rect) => rect.to_css(dest),
            Image::Element(ref selector) => {
                dest.write_str("-moz-element(#")?;
                serialize_identifier(&selector.to_string(), dest)?;
                dest.write_str(")")
            },
        }
    }
}

impl<G, R> HasViewportPercentage for Image<G, R>
    where G: HasViewportPercentage
{
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            Image::Gradient(ref gradient) => gradient.has_viewport_percentage(),
            _ => false,
        }
    }
}

impl<G, R> ToComputedValue for Image<G, R>
    where G: ToComputedValue, R: ToComputedValue,
{
    type ComputedValue = Image<<G as ToComputedValue>::ComputedValue,
                               <R as ToComputedValue>::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            Image::Url(ref url) => {
                Image::Url(url.clone())
            },
            Image::Gradient(ref gradient) => {
                Image::Gradient(gradient.to_computed_value(context))
            },
            Image::Rect(ref rect) => {
                Image::Rect(rect.to_computed_value(context))
            },
            Image::Element(ref selector) => {
                Image::Element(selector.clone())
            }
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            Image::Url(ref url) => {
                Image::Url(url.clone())
            },
            Image::Gradient(ref gradient) => {
                Image::Gradient(ToComputedValue::from_computed_value(gradient))
            },
            Image::Rect(ref rect) => {
                Image::Rect(ToComputedValue::from_computed_value(rect))
            },
            Image::Element(ref selector) => {
                Image::Element(selector.clone())
            },
        }
    }
}

impl<D, L, LoP, P, C> ToCss for Gradient<D, L, LoP, P, C>
    where D: LineDirection, L: ToCss, LoP: ToCss, P: ToCss, C: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.compat_mode == CompatMode::WebKit {
            dest.write_str("-webkit-")?;
        }
        if self.repeating {
            dest.write_str("repeating-")?;
        }
        dest.write_str(self.kind.label())?;
        dest.write_str("-gradient(")?;
        let mut skip_comma = match self.kind {
            GradientKind::Linear(ref direction) if direction.points_downwards() => true,
            GradientKind::Linear(ref direction) => {
                direction.to_css(dest, self.compat_mode)?;
                false
            },
            GradientKind::Radial(ref shape, ref position) => {
                let omit_shape = match *shape {
                    EndingShape::Ellipse(Ellipse::Extent(ShapeExtent::Cover)) |
                    EndingShape::Ellipse(Ellipse::Extent(ShapeExtent::FarthestCorner)) => {
                        true
                    },
                    _ => false,
                };
                if self.compat_mode == CompatMode::Modern {
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
        for item in &self.items {
            if !skip_comma {
                dest.write_str(", ")?;
            }
            skip_comma = false;
            item.to_css(dest)?;
        }
        dest.write_str(")")
    }
}

impl<D, L, LoP, P, C> ToComputedValue for Gradient<D, L, LoP, P, C>
    where D: ToComputedValue,
          L: ToComputedValue,
          LoP: ToComputedValue,
          P: ToComputedValue,
          C: ToComputedValue,
{
    type ComputedValue = Gradient<<D as ToComputedValue>::ComputedValue,
                                  <L as ToComputedValue>::ComputedValue,
                                  <LoP as ToComputedValue>::ComputedValue,
                                  <P as ToComputedValue>::ComputedValue,
                                  <C as ToComputedValue>::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        Gradient {
            kind: self.kind.to_computed_value(context),
            items: self.items.iter().map(|s| s.to_computed_value(context)).collect(),
            repeating: self.repeating,
            compat_mode: self.compat_mode,
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Gradient {
            kind: ToComputedValue::from_computed_value(&computed.kind),
            items: computed.items.iter().map(ToComputedValue::from_computed_value).collect(),
            repeating: computed.repeating,
            compat_mode: computed.compat_mode,
        }
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

impl<D, L, LoP, P> ToComputedValue for GradientKind<D, L, LoP, P>
    where D: ToComputedValue,
          L: ToComputedValue,
          LoP: ToComputedValue,
          P: ToComputedValue,
{
    type ComputedValue = GradientKind<<D as ToComputedValue>::ComputedValue,
                                      <L as ToComputedValue>::ComputedValue,
                                      <LoP as ToComputedValue>::ComputedValue,
                                      <P as ToComputedValue>::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            GradientKind::Linear(ref direction) => {
                GradientKind::Linear(direction.to_computed_value(context))
            },
            GradientKind::Radial(ref shape, ref position) => {
                GradientKind::Radial(shape.to_computed_value(context), position.to_computed_value(context))
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GradientKind::Linear(ref direction) => {
                GradientKind::Linear(ToComputedValue::from_computed_value(direction))
            },
            GradientKind::Radial(ref shape, ref position) => {
                GradientKind::Radial(
                    ToComputedValue::from_computed_value(shape),
                    ToComputedValue::from_computed_value(position),
                )
            }
        }
    }
}

/// The direction of a linear gradient.
pub trait LineDirection {
    /// Whether this direction points towards, and thus can be omitted.
    fn points_downwards(&self) -> bool;

    /// Serialises this direction according to the compatibility mode.
    fn to_css<W>(&self, dest: &mut W, compat_mode: CompatMode) -> fmt::Result
        where W: fmt::Write;
}

impl<L, LoP> ToCss for EndingShape<L, LoP>
    where L: ToCss, LoP: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            EndingShape::Circle(Circle::Extent(ShapeExtent::FarthestCorner)) |
            EndingShape::Circle(Circle::Extent(ShapeExtent::Cover)) => {
                dest.write_str("circle")
            },
            EndingShape::Circle(Circle::Extent(keyword)) => {
                dest.write_str("circle ")?;
                keyword.to_css(dest)
            },
            EndingShape::Circle(Circle::Radius(ref length)) => {
                length.to_css(dest)
            },
            EndingShape::Ellipse(Ellipse::Extent(keyword)) => {
                keyword.to_css(dest)
            },
            EndingShape::Ellipse(Ellipse::Radii(ref x, ref y)) => {
                x.to_css(dest)?;
                dest.write_str(" ")?;
                y.to_css(dest)
            },
        }
    }
}

impl<L, LoP> ToComputedValue for EndingShape<L, LoP>
    where L: ToComputedValue, LoP: ToComputedValue,
{
    type ComputedValue = EndingShape<<L as ToComputedValue>::ComputedValue,
                                     <LoP as ToComputedValue>::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            EndingShape::Circle(Circle::Radius(ref length)) => {
                EndingShape::Circle(Circle::Radius(length.to_computed_value(context)))
            },
            EndingShape::Circle(Circle::Extent(extent)) => {
                EndingShape::Circle(Circle::Extent(extent))
            },
            EndingShape::Ellipse(Ellipse::Radii(ref x, ref y)) => {
                EndingShape::Ellipse(Ellipse::Radii(
                    x.to_computed_value(context),
                    y.to_computed_value(context),
                ))
            },
            EndingShape::Ellipse(Ellipse::Extent(extent)) => {
                EndingShape::Ellipse(Ellipse::Extent(extent))
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            EndingShape::Circle(Circle::Radius(ref length)) => {
                EndingShape::Circle(Circle::Radius(ToComputedValue::from_computed_value(length)))
            },
            EndingShape::Circle(Circle::Extent(extent)) => {
                EndingShape::Circle(Circle::Extent(extent))
            },
            EndingShape::Ellipse(Ellipse::Radii(ref x, ref y)) => {
                EndingShape::Ellipse(Ellipse::Radii(
                    ToComputedValue::from_computed_value(x),
                    ToComputedValue::from_computed_value(y),
                ))
            },
            EndingShape::Ellipse(Ellipse::Extent(extent)) => {
                EndingShape::Ellipse(Ellipse::Extent(extent))
            },
        }
    }
}

impl<C, L> ToCss for GradientItem<C, L>
    where C: ToCss, L: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            GradientItem::ColorStop(ref stop) => stop.to_css(dest),
            GradientItem::InterpolationHint(ref hint) => hint.to_css(dest),
        }
    }
}

impl<C, L> ToComputedValue for GradientItem<C, L>
    where C: ToComputedValue, L: ToComputedValue,
{
    type ComputedValue = GradientItem<<C as ToComputedValue>::ComputedValue,
                                      <L as ToComputedValue>::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            GradientItem::ColorStop(ref stop) => {
                GradientItem::ColorStop(stop.to_computed_value(context))
            },
            GradientItem::InterpolationHint(ref hint) => {
                GradientItem::InterpolationHint(hint.to_computed_value(context))
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GradientItem::ColorStop(ref stop) => {
                GradientItem::ColorStop(ToComputedValue::from_computed_value(stop))
            },
            GradientItem::InterpolationHint(ref hint) => {
                GradientItem::InterpolationHint(ToComputedValue::from_computed_value(hint))
            },
        }
    }
}

impl<C, L> fmt::Debug for ColorStop<C, L>
    where C: fmt::Debug, L: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.color)?;
        if let Some(ref pos) = self.position {
            write!(f, " {:?}", pos)?;
        }
        Ok(())
    }
}

impl<C, L> ToCss for ColorStop<C, L>
    where C: ToCss, L: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.color.to_css(dest)?;
        if let Some(ref position) = self.position {
            dest.write_str(" ")?;
            position.to_css(dest)?;
        }
        Ok(())
    }
}

impl<C, L> ToComputedValue for ColorStop<C, L>
    where C: ToComputedValue, L: ToComputedValue,
{
    type ComputedValue = ColorStop<<C as ToComputedValue>::ComputedValue,
                                   <L as ToComputedValue>::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ColorStop {
            color: self.color.to_computed_value(context),
            position: self.position.as_ref().map(|p| p.to_computed_value(context)),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        ColorStop {
            color: ToComputedValue::from_computed_value(&computed.color),
            position: computed.position.as_ref().map(ToComputedValue::from_computed_value),
        }
    }
}

impl<C> ToCss for ImageRect<C>
    where C: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("-moz-image-rect(")?;
        self.url.to_css(dest)?;
        dest.write_str(", ")?;
        self.top.to_css(dest)?;
        dest.write_str(", ")?;
        self.right.to_css(dest)?;
        dest.write_str(", ")?;
        self.bottom.to_css(dest)?;
        dest.write_str(", ")?;
        self.left.to_css(dest)?;
        dest.write_str(")")
    }
}

impl<C> ToComputedValue for ImageRect<C>
    where C: ToComputedValue,
{
    type ComputedValue = ImageRect<<C as ToComputedValue>::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ImageRect {
            url: self.url.to_computed_value(context),
            top: self.top.to_computed_value(context),
            right: self.right.to_computed_value(context),
            bottom: self.bottom.to_computed_value(context),
            left: self.left.to_computed_value(context),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        ImageRect {
            url: ToComputedValue::from_computed_value(&computed.url),
            top: ToComputedValue::from_computed_value(&computed.top),
            right: ToComputedValue::from_computed_value(&computed.right),
            bottom: ToComputedValue::from_computed_value(&computed.bottom),
            left: ToComputedValue::from_computed_value(&computed.left),
        }
    }
}
