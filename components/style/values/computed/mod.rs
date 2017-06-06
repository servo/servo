/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values.

use Atom;
use context::QuirksMode;
use euclid::size::Size2D;
use font_metrics::FontMetricsProvider;
use media_queries::Device;
#[cfg(feature = "gecko")]
use properties;
use properties::{ComputedValues, StyleBuilder};
use std::f32;
use std::f32::consts::PI;
use std::fmt;
use style_traits::ToCss;
use super::{CSSFloat, CSSInteger, RGBA};
use super::generics::grid::{TrackBreadth as GenericTrackBreadth, TrackSize as GenericTrackSize};
use super::generics::grid::TrackList as GenericTrackList;
use super::specified;

pub use app_units::Au;
pub use cssparser::Color as CSSColor;
pub use self::background::BackgroundSize;
pub use self::border::{BorderImageSlice, BorderImageWidth, BorderImageSideWidth};
pub use self::border::{BorderRadius, BorderCornerRadius};
pub use self::image::{Gradient, GradientItem, ImageLayer, LineDirection, Image, ImageRect};
pub use self::rect::LengthOrNumberRect;
pub use super::{Auto, Either, None_};
#[cfg(feature = "gecko")]
pub use super::specified::{AlignItems, AlignJustifyContent, AlignJustifySelf, JustifyItems};
pub use super::specified::{BorderStyle, Percentage, UrlOrNone};
pub use super::generics::grid::GridLine;
pub use super::specified::url::SpecifiedUrl;
pub use self::length::{CalcLengthOrPercentage, Length, LengthOrNumber, LengthOrPercentage, LengthOrPercentageOrAuto};
pub use self::length::{LengthOrPercentageOrAutoOrContent, LengthOrPercentageOrNone, LengthOrNone};
pub use self::length::{MaxLength, MozLength};
pub use self::position::Position;
pub use self::text::{LetterSpacing, LineHeight, WordSpacing};
pub use self::transform::{TimingFunction, TransformOrigin};

pub mod background;
pub mod basic_shape;
pub mod border;
pub mod image;
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

    /// The Device holds the viewport and other external state.
    pub device: &'a Device,

    /// The style we're inheriting from.
    pub inherited_style: &'a ComputedValues,

    /// The style of the layout parent node. This will almost always be
    /// `inherited_style`, except when `display: contents` is at play, in which
    /// case it's the style of the last ancestor with a `display` value that
    /// isn't `contents`.
    pub layout_parent_style: &'a ComputedValues,

    /// Values accessed through this need to be in the properties "computed
    /// early": color, text-decoration, font-size, display, position, float,
    /// border-*-style, outline-style, font-family, writing-mode...
    pub style: StyleBuilder<'a>,

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
}

impl<'a> Context<'a> {
    /// Whether the current element is the root element.
    pub fn is_root_element(&self) -> bool { self.is_root_element }
    /// The current viewport size.
    pub fn viewport_size(&self) -> Size2D<Au> { self.device.au_viewport_size() }
    /// The style we're inheriting from.
    pub fn inherited_style(&self) -> &ComputedValues { &self.inherited_style }
    /// The current style. Note that only "eager" properties should be accessed
    /// from here, see the comment in the member.
    pub fn style(&self) -> &StyleBuilder { &self.style }
    /// A mutable reference to the current style.
    pub fn mutate_style(&mut self) -> &mut StyleBuilder<'a> { &mut self.style }
    /// Get a mutable reference to the current style as well as the device
    pub fn mutate_style_with_device(&mut self) -> (&mut StyleBuilder<'a>, &Device) { (&mut self.style, &self.device) }
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
        const RAD_PER_DEG: CSSFloat = PI / 180.0;
        const RAD_PER_GRAD: CSSFloat = PI / 200.0;
        const RAD_PER_TURN: CSSFloat = PI * 2.0;

        let radians = match *self {
            Angle::Degree(val) => val * RAD_PER_DEG,
            Angle::Gradian(val) => val * RAD_PER_GRAD,
            Angle::Turn(val) => val * RAD_PER_TURN,
            Angle::Radian(val) => val,
        };
        radians.min(f32::MAX).max(f32::MIN)
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

impl ToComputedValue for specified::Color {
    type ComputedValue = RGBA;

    #[cfg(not(feature = "gecko"))]
    fn to_computed_value(&self, context: &Context) -> RGBA {
        match *self {
            specified::Color::RGBA(rgba) => rgba,
            specified::Color::CurrentColor => context.inherited_style.get_color().clone_color(),
        }
    }

    #[cfg(feature = "gecko")]
    fn to_computed_value(&self, context: &Context) -> RGBA {
        use gecko::values::convert_nscolor_to_rgba as to_rgba;
        // It's safe to access the nsPresContext immutably during style computation.
        let pres_context = unsafe { &*context.device.pres_context };
        match *self {
            specified::Color::RGBA(rgba) => rgba,
            specified::Color::System(system) => to_rgba(system.to_computed_value(context)),
            specified::Color::CurrentColor => context.inherited_style.get_color().clone_color(),
            specified::Color::MozDefaultColor => to_rgba(pres_context.mDefaultColor),
            specified::Color::MozDefaultBackgroundColor => to_rgba(pres_context.mBackgroundColor),
            specified::Color::MozHyperlinktext => to_rgba(pres_context.mLinkColor),
            specified::Color::MozActiveHyperlinktext => to_rgba(pres_context.mActiveLinkColor),
            specified::Color::MozVisitedHyperlinktext => to_rgba(pres_context.mVisitedLinkColor),
            specified::Color::InheritFromBodyQuirk => {
                use dom::TElement;
                use gecko::wrapper::GeckoElement;
                use gecko_bindings::bindings::Gecko_GetBody;
                let body = unsafe {
                    Gecko_GetBody(pres_context)
                };
                if let Some(body) = body {
                    let wrap = GeckoElement(body);
                    let borrow = wrap.borrow_data();
                    borrow.as_ref().unwrap()
                          .styles().primary.values()
                          .get_color()
                          .clone_color()
                } else {
                    to_rgba(pres_context.mDefaultColor)
                }
            },
        }
    }

    fn from_computed_value(computed: &RGBA) -> Self {
        specified::Color::RGBA(*computed)
    }
}

impl ToComputedValue for specified::CSSColor {
    type ComputedValue = CSSColor;

    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn to_computed_value(&self, _context: &Context) -> CSSColor {
        self.parsed
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn to_computed_value(&self, context: &Context) -> CSSColor {
        match self.parsed {
            specified::Color::RGBA(rgba) => CSSColor::RGBA(rgba),
            specified::Color::CurrentColor => CSSColor::CurrentColor,
            // Resolve non-standard -moz keywords to RGBA:
            non_standard => CSSColor::RGBA(non_standard.to_computed_value(context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &CSSColor) -> Self {
        (match *computed {
            CSSColor::RGBA(rgba) => specified::Color::RGBA(rgba),
            CSSColor::CurrentColor => specified::Color::CurrentColor,
        }).into()
    }
}

#[cfg(feature = "gecko")]
impl ToComputedValue for specified::JustifyItems {
    type ComputedValue = JustifyItems;

    // https://drafts.csswg.org/css-align/#valdef-justify-items-auto
    fn to_computed_value(&self, context: &Context) -> JustifyItems {
        use values::specified::align;
        // If the inherited value of `justify-items` includes the `legacy` keyword, `auto` computes
        // to the inherited value.
        if self.0 == align::ALIGN_AUTO {
            let inherited = context.inherited_style.get_position().clone_justify_items();
            if inherited.0.contains(align::ALIGN_LEGACY) {
                return inherited
            }
        }
        return *self
    }

    #[inline]
    fn from_computed_value(computed: &JustifyItems) -> Self {
        *computed
    }
}

#[cfg(feature = "gecko")]
impl ComputedValueAsSpecified for specified::AlignItems {}
#[cfg(feature = "gecko")]
impl ComputedValueAsSpecified for specified::AlignJustifyContent {}
#[cfg(feature = "gecko")]
impl ComputedValueAsSpecified for specified::AlignJustifySelf {}
impl ComputedValueAsSpecified for specified::BorderStyle {}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Shadow {
    pub offset_x: Au,
    pub offset_y: Au,
    pub blur_radius: Au,
    pub spread_radius: Au,
    pub color: CSSColor,
    pub inset: bool,
}

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
pub type SVGPaint = ::values::generics::SVGPaint<CSSColor>;
/// Computed SVG Paint Kind value
pub type SVGPaintKind = ::values::generics::SVGPaintKind<CSSColor>;

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
            kind: ::values::generics::SVGPaintKind::Color(CSSColor::RGBA(rgba)),
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
        try!(dest.write_str("rect("));
        if let Some(top) = self.top {
            try!(top.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        if let Some(right) = self.right {
            try!(right.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        if let Some(bottom) = self.bottom {
            try!(bottom.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        if let Some(left) = self.left {
            try!(left.to_css(dest));
        } else {
            try!(dest.write_str("auto"));
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
pub type TrackList = GenericTrackList<TrackSize>;

/// `<track-list> | none`
pub type TrackListOrNone = Either<TrackList, None_>;

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
pub type ColorOrAuto = Either<CSSColor, Auto>;
