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
pub enum Image<G, N> {
    /// A `<url()>` image.
    Url(SpecifiedUrl),
    /// A `<gradient>` image.
    Gradient(G),
    /// A `-moz-image-rect` image
    Rect(ImageRect<N>),
    /// A `-moz-element(# <element-id>)`
    Element(Atom),
}

/// A CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Gradient<K, C, L> {
    /// Gradients can be linear or radial.
    pub kind: K,
    /// The color stops and interpolation hints.
    pub items: Vec<GradientItem<C, L>>,
    /// True if this is a repeating gradient.
    pub repeating: bool,
    /// Compatibility mode.
    pub compat_mode: CompatMode,
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// Whether we used the modern notation or the compatibility `-webkit` prefix.
pub enum CompatMode {
    /// Modern syntax.
    Modern,
    /// `-webkit` prefix.
    WebKit,
}

/// A gradient item.
/// https://drafts.csswg.org/css-images-4/#color-stop-syntax
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum GradientItem<C, L> {
    /// A color stop.
    ColorStop(ColorStop<C, L>),
    /// An interpolation hint.
    InterpolationHint(L),
}

/// A color stop.
/// https://drafts.csswg.org/css-images/#typedef-color-stop-list
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ColorStop<C, L> {
    /// The color of this stop.
    pub color: C,
    /// The position of this stop.
    pub position: Option<L>,
}

/// Values for `moz-image-rect`.
///
/// `-moz-image-rect(<uri>, top, right, bottom, left);`
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct ImageRect<C> {
    pub url: SpecifiedUrl,
    pub top: C,
    pub bottom: C,
    pub right: C,
    pub left: C,
}

impl<G, N> fmt::Debug for Image<G, N>
    where G: fmt::Debug, N: fmt::Debug,
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

impl<G, N> ToCss for Image<G, N>
    where G: ToCss, N: ToCss,
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

impl<G, N> HasViewportPercentage for Image<G, N>
    where G: HasViewportPercentage
{
    fn has_viewport_percentage(&self) -> bool {
        if let Image::Gradient(ref gradient) = *self {
            gradient.has_viewport_percentage()
        } else {
            false
        }
    }
}

impl<G, N> ToComputedValue for Image<G, N>
    where G: ToComputedValue, N: ToComputedValue,
{
    type ComputedValue = Image<<G as ToComputedValue>::ComputedValue,
                               <N as ToComputedValue>::ComputedValue>;

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

impl<K, C, L> HasViewportPercentage for Gradient<K, C, L>
    where K: HasViewportPercentage, L: HasViewportPercentage,
{
    fn has_viewport_percentage(&self) -> bool {
        self.kind.has_viewport_percentage() ||
        self.items.iter().any(|i| i.has_viewport_percentage())
    }
}

impl<K, C, L> ToComputedValue for Gradient<K, C, L>
    where K: ToComputedValue, C: ToComputedValue, L: ToComputedValue,
{
    type ComputedValue = Gradient<<K as ToComputedValue>::ComputedValue,
                                  <C as ToComputedValue>::ComputedValue,
                                  <L as ToComputedValue>::ComputedValue>;

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

impl<C, L> HasViewportPercentage for GradientItem<C, L>
    where L: HasViewportPercentage,
{
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            GradientItem::ColorStop(ref stop) => stop.has_viewport_percentage(),
            GradientItem::InterpolationHint(ref hint) => hint.has_viewport_percentage(),
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
        try!(self.color.to_css(dest));
        if let Some(ref position) = self.position {
            try!(dest.write_str(" "));
            try!(position.to_css(dest));
        }
        Ok(())
    }
}

impl<C, L> HasViewportPercentage for ColorStop<C, L>
    where L: HasViewportPercentage,
{
    fn has_viewport_percentage(&self) -> bool {
        self.position.as_ref().map_or(false, HasViewportPercentage::has_viewport_percentage)
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
            url: self.url.clone(),
            top: self.top.to_computed_value(context),
            right: self.right.to_computed_value(context),
            bottom: self.bottom.to_computed_value(context),
            left: self.left.to_computed_value(context),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        ImageRect {
            url: computed.url.clone(),
            top: ToComputedValue::from_computed_value(&computed.top),
            right: ToComputedValue::from_computed_value(&computed.right),
            bottom: ToComputedValue::from_computed_value(&computed.bottom),
            left: ToComputedValue::from_computed_value(&computed.left),
        }
    }
}
