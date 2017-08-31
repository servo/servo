/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values.

use Atom;
use context::QuirksMode;
use euclid::Size2D;
use font_metrics::FontMetricsProvider;
use media_queries::Device;
#[cfg(feature = "gecko")]
use properties;
use properties::{ComputedValues, StyleBuilder};
#[cfg(feature = "servo")]
use servo_url::ServoUrl;
use std::f32;
use std::fmt;
#[cfg(feature = "servo")]
use std::sync::Arc;
use style_traits::ToCss;
use super::{CSSFloat, CSSInteger};
use super::generics::{GreaterThanOrEqualToOne, NonNegative};
use super::generics::grid::{TrackBreadth as GenericTrackBreadth, TrackSize as GenericTrackSize};
use super::generics::grid::GridTemplateComponent as GenericGridTemplateComponent;
use super::generics::grid::TrackList as GenericTrackList;
use super::specified;

pub use app_units::Au;
pub use properties::animated_properties::TransitionProperty;
#[cfg(feature = "gecko")]
pub use self::align::{AlignItems, AlignJustifyContent, AlignJustifySelf, JustifyItems};
pub use self::angle::Angle;
pub use self::background::BackgroundSize;
pub use self::border::{BorderImageSlice, BorderImageWidth, BorderImageSideWidth};
pub use self::border::{BorderRadius, BorderCornerRadius};
pub use self::color::{Color, RGBAColor};
pub use self::effects::{BoxShadow, Filter, SimpleShadow};
pub use self::flex::FlexBasis;
pub use self::image::{Gradient, GradientItem, Image, ImageLayer, LineDirection, MozImageRect};
#[cfg(feature = "gecko")]
pub use self::gecko::ScrollSnapPoint;
pub use self::rect::LengthOrNumberRect;
pub use super::{Auto, Either, None_};
pub use super::specified::BorderStyle;
pub use super::generics::grid::GridLine;
pub use self::length::{CalcLengthOrPercentage, Length, LengthOrNone, LengthOrNumber, LengthOrPercentage};
pub use self::length::{LengthOrPercentageOrAuto, LengthOrPercentageOrNone, MaxLength, MozLength};
pub use self::length::NonNegativeLengthOrPercentage;
pub use self::percentage::Percentage;
pub use self::position::Position;
pub use self::svg::{SVGLength, SVGOpacity, SVGPaint, SVGPaintKind, SVGStrokeDashArray, SVGWidth};
pub use self::text::{InitialLetter, LetterSpacing, LineHeight, WordSpacing};
pub use self::time::Time;
pub use self::transform::{TimingFunction, TransformOrigin};

#[cfg(feature = "gecko")]
pub mod align;
pub mod angle;
pub mod background;
pub mod basic_shape;
pub mod border;
pub mod color;
pub mod effects;
pub mod flex;
pub mod image;
#[cfg(feature = "gecko")]
pub mod gecko;
pub mod length;
pub mod percentage;
pub mod position;
pub mod rect;
pub mod svg;
pub mod text;
pub mod time;
pub mod transform;

/// A `Context` is all the data a specified value could ever need to compute
/// itself and be transformed to a computed value.
pub struct Context<'a> {
    /// Whether the current element is the root element.
    pub is_root_element: bool,

    /// Values accessed through this need to be in the properties "computed
    /// early": color, text-decoration, font-size, display, position, float,
    /// border-*-style, outline-style, font-family, writing-mode...
    pub builder: StyleBuilder<'a>,

    /// A cached computed system font value, for use by gecko.
    ///
    /// See properties/longhands/font.mako.rs
    #[cfg(feature = "gecko")]
    pub cached_system_font: Option<properties::longhands::system_font::ComputedSystemFont>,

    /// A dummy option for servo so initializing a computed::Context isn't
    /// painful.
    ///
    /// TODO(emilio): Make constructors for Context, and drop this.
    #[cfg(feature = "servo")]
    pub cached_system_font: Option<()>,

    /// A font metrics provider, used to access font metrics to implement
    /// font-relative units.
    pub font_metrics_provider: &'a FontMetricsProvider,

    /// Whether or not we are computing the media list in a media query
    pub in_media_query: bool,

    /// The quirks mode of this context.
    pub quirks_mode: QuirksMode,

    /// Whether this computation is being done for a SMIL animation.
    ///
    /// This is used to allow certain properties to generate out-of-range
    /// values, which SMIL allows.
    pub for_smil_animation: bool,
}

impl<'a> Context<'a> {
    /// Whether the current element is the root element.
    pub fn is_root_element(&self) -> bool {
        self.is_root_element
    }

    /// The current device.
    pub fn device(&self) -> &Device {
        self.builder.device
    }

    /// The current viewport size, used to resolve viewport units.
    pub fn viewport_size_for_viewport_unit_resolution(&self) -> Size2D<Au> {
        self.builder.device.au_viewport_size_for_viewport_unit_resolution()
    }

    /// The default computed style we're getting our reset style from.
    pub fn default_style(&self) -> &ComputedValues {
        self.builder.default_style()
    }

    /// The current style.
    pub fn style(&self) -> &StyleBuilder {
        &self.builder
    }

    /// Apply text-zoom if enabled.
    #[cfg(feature = "gecko")]
    pub fn maybe_zoom_text(&self, size: NonNegativeAu) -> NonNegativeAu {
        // We disable zoom for <svg:text> by unsetting the
        // -x-text-zoom property, which leads to a false value
        // in mAllowZoom
        if self.style().get_font().gecko.mAllowZoom {
            self.device().zoom_text(size.0).into()
        } else {
            size
        }
    }

    /// (Servo doesn't do text-zoom)
    #[cfg(feature = "servo")]
    pub fn maybe_zoom_text(&self, size: NonNegativeAu) -> NonNegativeAu {
        size
    }
}

/// An iterator over a slice of computed values
#[derive(Clone)]
pub struct ComputedVecIter<'a, 'cx, 'cx_a: 'cx, S: ToComputedValue + 'a> {
    cx: &'cx Context<'cx_a>,
    values: &'a [S],
}

impl<'a, 'cx, 'cx_a: 'cx, S: ToComputedValue + 'a> ComputedVecIter<'a, 'cx, 'cx_a, S> {
    /// Construct an iterator from a slice of specified values and a context
    pub fn new(cx: &'cx Context<'cx_a>, values: &'a [S]) -> Self {
        ComputedVecIter {
            cx: cx,
            values: values,
        }
    }
}

impl<'a, 'cx, 'cx_a: 'cx, S: ToComputedValue + 'a> ExactSizeIterator for ComputedVecIter<'a, 'cx, 'cx_a, S> {
    fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'a, 'cx, 'cx_a: 'cx, S: ToComputedValue + 'a> Iterator for ComputedVecIter<'a, 'cx, 'cx_a, S> {
    type Item = S::ComputedValue;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((next, rest)) = self.values.split_first() {
            let ret = next.to_computed_value(self.cx);
            self.values = rest;
            Some(ret)
        } else {
            None
        }
    }
}

/// A trait to represent the conversion between computed and specified values.
///
/// This trait is derivable with `#[derive(ToComputedValue)]`. The derived
/// implementation just calls `ToComputedValue::to_computed_value` on each field
/// of the passed value, or `Clone::clone` if the field is annotated with
/// `#[compute(clone)]`.
pub trait ToComputedValue {
    /// The computed value type we're going to be converted to.
    type ComputedValue;

    /// Convert a specified value to a computed value, using itself and the data
    /// inside the `Context`.
    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue;

    #[inline]
    /// Convert a computed value to specified value form.
    ///
    /// This will be used for recascading during animation.
    /// Such from_computed_valued values should recompute to the same value.
    fn from_computed_value(computed: &Self::ComputedValue) -> Self;
}

impl<A, B> ToComputedValue for (A, B)
    where A: ToComputedValue, B: ToComputedValue,
{
    type ComputedValue = (
        <A as ToComputedValue>::ComputedValue,
        <B as ToComputedValue>::ComputedValue,
    );

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        (self.0.to_computed_value(context), self.1.to_computed_value(context))
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        (A::from_computed_value(&computed.0), B::from_computed_value(&computed.1))
    }
}

impl<T> ToComputedValue for Option<T>
    where T: ToComputedValue
{
    type ComputedValue = Option<<T as ToComputedValue>::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.as_ref().map(|item| item.to_computed_value(context))
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed.as_ref().map(T::from_computed_value)
    }
}

impl<T> ToComputedValue for Size2D<T>
    where T: ToComputedValue
{
    type ComputedValue = Size2D<<T as ToComputedValue>::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        Size2D::new(
            self.width.to_computed_value(context),
            self.height.to_computed_value(context),
        )
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Size2D::new(
            T::from_computed_value(&computed.width),
            T::from_computed_value(&computed.height),
        )
    }
}

impl<T> ToComputedValue for Vec<T>
    where T: ToComputedValue
{
    type ComputedValue = Vec<<T as ToComputedValue>::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.iter().map(|item| item.to_computed_value(context)).collect()
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed.iter().map(T::from_computed_value).collect()
    }
}

impl<T> ToComputedValue for Box<[T]>
    where T: ToComputedValue
{
    type ComputedValue = Box<[<T as ToComputedValue>::ComputedValue]>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.iter().map(|item| item.to_computed_value(context)).collect::<Vec<_>>().into_boxed_slice()
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed.iter().map(T::from_computed_value).collect::<Vec<_>>().into_boxed_slice()
    }
}

/// A marker trait to represent that the specified value is also the computed
/// value.
pub trait ComputedValueAsSpecified {}

impl<T> ToComputedValue for T
    where T: ComputedValueAsSpecified + Clone,
{
    type ComputedValue = T;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> T {
        self.clone()
    }

    #[inline]
    fn from_computed_value(computed: &T) -> Self {
        computed.clone()
    }
}

impl ComputedValueAsSpecified for Atom {}
impl ComputedValueAsSpecified for bool {}
impl ComputedValueAsSpecified for f32 {}

impl ComputedValueAsSpecified for specified::BorderStyle {}

/// A `<number>` value.
pub type Number = CSSFloat;

/// A wrapper of Number, but the value >= 0.
pub type NonNegativeNumber = NonNegative<CSSFloat>;

impl From<CSSFloat> for NonNegativeNumber {
    #[inline]
    fn from(number: CSSFloat) -> NonNegativeNumber {
        NonNegative::<CSSFloat>(number)
    }
}

impl From<NonNegativeNumber> for CSSFloat {
    #[inline]
    fn from(number: NonNegativeNumber) -> CSSFloat {
        number.0
    }
}

/// A wrapper of Number, but the value >= 1.
pub type GreaterThanOrEqualToOneNumber = GreaterThanOrEqualToOne<CSSFloat>;

impl From<CSSFloat> for GreaterThanOrEqualToOneNumber {
    #[inline]
    fn from(number: CSSFloat) -> GreaterThanOrEqualToOneNumber {
        GreaterThanOrEqualToOne::<CSSFloat>(number)
    }
}

impl From<GreaterThanOrEqualToOneNumber> for CSSFloat {
    #[inline]
    fn from(number: GreaterThanOrEqualToOneNumber) -> CSSFloat {
        number.0
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, PartialEq, ToCss)]
pub enum NumberOrPercentage {
    Percentage(Percentage),
    Number(Number),
}

impl ToComputedValue for specified::NumberOrPercentage {
    type ComputedValue = NumberOrPercentage;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> NumberOrPercentage {
        match *self {
            specified::NumberOrPercentage::Percentage(percentage) =>
                NumberOrPercentage::Percentage(percentage.to_computed_value(context)),
            specified::NumberOrPercentage::Number(number) =>
                NumberOrPercentage::Number(number.to_computed_value(context)),
        }
    }
    #[inline]
    fn from_computed_value(computed: &NumberOrPercentage) -> Self {
        match *computed {
            NumberOrPercentage::Percentage(percentage) =>
                specified::NumberOrPercentage::Percentage(ToComputedValue::from_computed_value(&percentage)),
            NumberOrPercentage::Number(number) =>
                specified::NumberOrPercentage::Number(ToComputedValue::from_computed_value(&number)),
        }
    }
}

/// A type used for opacity.
pub type Opacity = CSSFloat;

/// A `<integer>` value.
pub type Integer = CSSInteger;

/// <integer> | auto
pub type IntegerOrAuto = Either<CSSInteger, Auto>;

impl IntegerOrAuto {
    /// Returns the integer value if it is an integer, otherwise return
    /// the given value.
    pub fn integer_or(&self, auto_value: CSSInteger) -> CSSInteger {
        match *self {
            Either::First(n) => n,
            Either::Second(Auto) => auto_value,
        }
    }
}

/// A wrapper of Integer, but only accept a value >= 1.
pub type PositiveInteger = GreaterThanOrEqualToOne<CSSInteger>;

impl From<CSSInteger> for PositiveInteger {
    #[inline]
    fn from(int: CSSInteger) -> PositiveInteger {
        GreaterThanOrEqualToOne::<CSSInteger>(int)
    }
}

/// PositiveInteger | auto
pub type PositiveIntegerOrAuto = Either<PositiveInteger, Auto>;

/// <length> | <percentage> | <number>
pub type LengthOrPercentageOrNumber = Either<Number, LengthOrPercentage>;

/// NonNegativeLengthOrPercentage | NonNegativeNumber
pub type NonNegativeLengthOrPercentageOrNumber = Either<NonNegativeNumber, NonNegativeLengthOrPercentage>;

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, Eq, PartialEq)]
/// A computed cliprect for clip and image-region
pub struct ClipRect {
    pub top: Option<Au>,
    pub right: Option<Au>,
    pub bottom: Option<Au>,
    pub left: Option<Au>,
}

impl ToCss for ClipRect {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("rect(")?;
        if let Some(top) = self.top {
            top.to_css(dest)?;
            dest.write_str(", ")?;
        } else {
            dest.write_str("auto, ")?;
        }

        if let Some(right) = self.right {
            right.to_css(dest)?;
            dest.write_str(", ")?;
        } else {
            dest.write_str("auto, ")?;
        }

        if let Some(bottom) = self.bottom {
            bottom.to_css(dest)?;
            dest.write_str(", ")?;
        } else {
            dest.write_str("auto, ")?;
        }

        if let Some(left) = self.left {
            left.to_css(dest)?;
        } else {
            dest.write_str("auto")?;
        }
        dest.write_str(")")
    }
}

/// rect(...) | auto
pub type ClipRectOrAuto = Either<ClipRect, Auto>;

/// The computed value of a grid `<track-breadth>`
pub type TrackBreadth = GenericTrackBreadth<LengthOrPercentage>;

/// The computed value of a grid `<track-size>`
pub type TrackSize = GenericTrackSize<LengthOrPercentage>;

/// The computed value of a grid `<track-list>`
/// (could also be `<auto-track-list>` or `<explicit-track-list>`)
pub type TrackList = GenericTrackList<LengthOrPercentage>;

/// `<grid-template-rows> | <grid-template-columns>`
pub type GridTemplateComponent = GenericGridTemplateComponent<LengthOrPercentage>;

impl ClipRectOrAuto {
    /// Return an auto (default for clip-rect and image-region) value
    pub fn auto() -> Self {
        Either::Second(Auto)
    }

    /// Check if it is auto
    pub fn is_auto(&self) -> bool {
        match *self {
            Either::Second(_) => true,
            _ => false
        }
    }
}

/// <color> | auto
pub type ColorOrAuto = Either<Color, Auto>;

/// A wrapper of Au, but the value >= 0.
pub type NonNegativeAu = NonNegative<Au>;

impl NonNegativeAu {
    /// Return a zero value.
    #[inline]
    pub fn zero() -> Self {
        NonNegative::<Au>(Au(0))
    }

    /// Return a NonNegativeAu from pixel.
    #[inline]
    pub fn from_px(px: i32) -> Self {
        NonNegative::<Au>(Au::from_px(::std::cmp::max(px, 0)))
    }

    /// Get the inner value of |NonNegativeAu.0|.
    #[inline]
    pub fn value(self) -> i32 {
        (self.0).0
    }

    /// Scale this NonNegativeAu.
    #[inline]
    pub fn scale_by(self, factor: f32) -> Self {
        // scale this by zero if factor is negative.
        NonNegative::<Au>(self.0.scale_by(factor.max(0.)))
    }
}

impl From<Au> for NonNegativeAu {
    #[inline]
    fn from(au: Au) -> NonNegativeAu {
        NonNegative::<Au>(au)
    }
}

/// The computed value of a CSS `url()`, resolved relative to the stylesheet URL.
#[cfg(feature = "servo")]
#[derive(Clone, Debug, Deserialize, HeapSizeOf, PartialEq, Serialize)]
pub enum ComputedUrl {
    /// The `url()` was invalid or it wasn't specified by the user.
    Invalid(Arc<String>),
    /// The resolved `url()` relative to the stylesheet URL.
    Valid(ServoUrl),
}

/// TODO: Properly build ComputedUrl for gecko
#[cfg(feature = "gecko")]
pub type ComputedUrl = specified::url::SpecifiedUrl;

#[cfg(feature = "servo")]
impl ComputedUrl {
    /// Returns the resolved url if it was valid.
    pub fn url(&self) -> Option<&ServoUrl> {
        match *self {
            ComputedUrl::Valid(ref url) => Some(url),
            _ => None,
        }
    }
}

#[cfg(feature = "servo")]
impl ToCss for ComputedUrl {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let string = match *self {
            ComputedUrl::Valid(ref url) => url.as_str(),
            ComputedUrl::Invalid(ref invalid_string) => invalid_string,
        };

        dest.write_str("url(")?;
        string.to_css(dest)?;
        dest.write_str(")")
    }
}

/// <url> | <none>
pub type UrlOrNone = Either<ComputedUrl, None_>;
