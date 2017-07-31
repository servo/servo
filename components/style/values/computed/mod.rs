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
use std::f32;
use std::f64;
use std::f64::consts::PI;
use std::fmt;
use style_traits::ToCss;
use super::{CSSFloat, CSSInteger, RGBA};
use super::generics::grid::{TrackBreadth as GenericTrackBreadth, TrackSize as GenericTrackSize};
use super::generics::grid::GridTemplateComponent as GenericGridTemplateComponent;
use super::generics::grid::TrackList as GenericTrackList;
use super::specified;

pub use app_units::Au;
pub use properties::animated_properties::TransitionProperty;
#[cfg(feature = "gecko")]
pub use self::align::{AlignItems, AlignJustifyContent, AlignJustifySelf, JustifyItems};
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
pub use super::specified::{BorderStyle, UrlOrNone};
pub use super::generics::grid::GridLine;
pub use super::specified::url::SpecifiedUrl;
pub use self::length::{CalcLengthOrPercentage, Length, LengthOrNone, LengthOrNumber, LengthOrPercentage};
pub use self::length::{LengthOrPercentageOrAuto, LengthOrPercentageOrNone, MaxLength, MozLength, Percentage};
pub use self::position::Position;
pub use self::text::{InitialLetter, LetterSpacing, LineHeight, WordSpacing};
pub use self::transform::{TimingFunction, TransformOrigin};

#[cfg(feature = "gecko")]
pub mod align;
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
pub mod position;
pub mod rect;
pub mod text;
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

    /// The current viewport size.
    pub fn viewport_size(&self) -> Size2D<Au> {
        self.builder.device.au_viewport_size()
    }

    /// The default computed style we're getting our reset style from.
    pub fn default_style(&self) -> &ComputedValues {
        self.builder.default_style()
    }

    /// The current style.
    pub fn style(&self) -> &StyleBuilder {
        &self.builder
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

/// A computed `<angle>` value.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub enum Angle {
    /// An angle with degree unit
    Degree(CSSFloat),
    /// An angle with gradian unit
    Gradian(CSSFloat),
    /// An angle with radian unit
    Radian(CSSFloat),
    /// An angle with turn unit
    Turn(CSSFloat),
}

impl Angle {
    /// Construct a computed `Angle` value from a radian amount.
    pub fn from_radians(radians: CSSFloat) -> Self {
        Angle::Radian(radians)
    }

    /// Return the amount of radians this angle represents.
    #[inline]
    pub fn radians(&self) -> CSSFloat {
        self.radians64().min(f32::MAX as f64).max(f32::MIN as f64) as f32
    }

    /// Return the amount of radians this angle represents as a 64-bit float.
    /// Gecko stores angles as singles, but does this computation using doubles.
    /// See nsCSSValue::GetAngleValueInRadians.
    /// This is significant enough to mess up rounding to the nearest
    /// quarter-turn for 225 degrees, for example.
    #[inline]
    pub fn radians64(&self) -> f64 {
        const RAD_PER_DEG: f64 = PI / 180.0;
        const RAD_PER_GRAD: f64 = PI / 200.0;
        const RAD_PER_TURN: f64 = PI * 2.0;

        let radians = match *self {
            Angle::Degree(val) => val as f64 * RAD_PER_DEG,
            Angle::Gradian(val) => val as f64 * RAD_PER_GRAD,
            Angle::Turn(val) => val as f64 * RAD_PER_TURN,
            Angle::Radian(val) => val as f64,
        };
        radians.min(f64::MAX).max(f64::MIN)
    }

    /// Returns an angle that represents a rotation of zero radians.
    pub fn zero() -> Self {
        Angle::Radian(0.0)
    }
}

impl ToCss for Angle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            Angle::Degree(val) => write!(dest, "{}deg", val),
            Angle::Gradian(val) => write!(dest, "{}grad", val),
            Angle::Radian(val) => write!(dest, "{}rad", val),
            Angle::Turn(val) => write!(dest, "{}turn", val),
        }
    }
}

/// A computed `<time>` value.
#[derive(Clone, PartialEq, PartialOrd, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub struct Time {
    seconds: CSSFloat,
}

impl Time {
    /// Construct a computed `Time` value from a seconds amount.
    pub fn from_seconds(seconds: CSSFloat) -> Self {
        Time {
            seconds: seconds,
        }
    }

    /// Construct a computed `Time` value that represents zero seconds.
    pub fn zero() -> Self {
        Self::from_seconds(0.0)
    }

    /// Return the amount of seconds this time represents.
    #[inline]
    pub fn seconds(&self) -> CSSFloat {
        self.seconds
    }
}

impl ToCss for Time {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        write!(dest, "{}s", self.seconds())
    }
}

impl ComputedValueAsSpecified for specified::BorderStyle {}

/// A `<number>` value.
pub type Number = CSSFloat;

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToCss)]
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

/// Computed SVG Paint value
pub type SVGPaint = ::values::generics::SVGPaint<RGBA>;
/// Computed SVG Paint Kind value
pub type SVGPaintKind = ::values::generics::SVGPaintKind<RGBA>;

impl Default for SVGPaint {
    fn default() -> Self {
        SVGPaint {
            kind: ::values::generics::SVGPaintKind::None,
            fallback: None,
        }
    }
}

impl SVGPaint {
    /// Opaque black color
    pub fn black() -> Self {
        let rgba = RGBA::from_floats(0., 0., 0., 1.);
        SVGPaint {
            kind: ::values::generics::SVGPaintKind::Color(rgba),
            fallback: None,
        }
    }
}

/// <length> | <percentage> | <number>
pub type LengthOrPercentageOrNumber = Either<Number, LengthOrPercentage>;

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
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
