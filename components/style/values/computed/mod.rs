/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values.

use {Atom, Namespace};
use context::QuirksMode;
use euclid::Size2D;
use font_metrics::{FontMetricsProvider, get_metrics_provider_for_product};
use media_queries::Device;
#[cfg(feature = "gecko")]
use properties;
use properties::{ComputedValues, LonghandId, StyleBuilder};
use rule_cache::RuleCacheConditions;
#[cfg(feature = "servo")]
use servo_url::ServoUrl;
use std::{f32, fmt};
use std::cell::RefCell;
#[cfg(feature = "servo")]
use std::sync::Arc;
use style_traits::ToCss;
use style_traits::cursor::Cursor;
use super::{CSSFloat, CSSInteger};
use super::generics::{GreaterThanOrEqualToOne, NonNegative};
use super::generics::grid::{GridLine as GenericGridLine, TrackBreadth as GenericTrackBreadth};
use super::generics::grid::{TrackSize as GenericTrackSize, TrackList as GenericTrackList};
use super::generics::grid::GridTemplateComponent as GenericGridTemplateComponent;
use super::specified;

pub use app_units::Au;
pub use properties::animated_properties::TransitionProperty;
#[cfg(feature = "gecko")]
pub use self::align::{AlignItems, AlignJustifyContent, AlignJustifySelf, JustifyItems};
pub use self::angle::Angle;
pub use self::background::{BackgroundSize, BackgroundRepeat};
pub use self::border::{BorderImageSlice, BorderImageWidth, BorderImageSideWidth};
pub use self::border::{BorderRadius, BorderCornerRadius, BorderSpacing};
pub use self::font::{FontSize, FontSizeAdjust, FontSynthesis, FontWeight, FontVariantAlternates};
pub use self::font::{FontFamily, FontLanguageOverride, FontVariantSettings, FontVariantEastAsian};
pub use self::font::{FontVariantLigatures, FontVariantNumeric, FontFeatureSettings};
pub use self::font::{MozScriptLevel, MozScriptMinSize, MozScriptSizeMultiplier, XTextZoom, XLang};
pub use self::box_::{AnimationIterationCount, AnimationName, OverscrollBehavior, ScrollSnapType, VerticalAlign};
pub use self::color::{Color, ColorPropertyValue, RGBAColor};
pub use self::effects::{BoxShadow, Filter, SimpleShadow};
pub use self::flex::FlexBasis;
pub use self::image::{Gradient, GradientItem, Image, ImageLayer, LineDirection, MozImageRect};
#[cfg(feature = "gecko")]
pub use self::gecko::ScrollSnapPoint;
pub use self::rect::LengthOrNumberRect;
pub use super::{Auto, Either, None_};
pub use super::specified::{BorderStyle, TextDecorationLine};
pub use self::length::{CalcLengthOrPercentage, Length, LengthOrNone, LengthOrNumber, LengthOrPercentage};
pub use self::length::{LengthOrPercentageOrAuto, LengthOrPercentageOrNone, MaxLength, MozLength};
pub use self::length::{CSSPixelLength, NonNegativeLength, NonNegativeLengthOrPercentage};
pub use self::percentage::Percentage;
pub use self::position::{Position, GridAutoFlow};
pub use self::svg::{SVGLength, SVGOpacity, SVGPaint, SVGPaintKind, SVGStrokeDashArray, SVGWidth};
pub use self::table::XSpan;
pub use self::text::{InitialLetter, LetterSpacing, LineHeight, TextOverflow, WordSpacing};
pub use self::time::Time;
pub use self::transform::{TimingFunction, Transform, TransformOperation, TransformOrigin};
pub use self::ui::MozForceBrokenImageIcon;

#[cfg(feature = "gecko")]
pub mod align;
pub mod angle;
pub mod background;
pub mod basic_shape;
pub mod border;
#[path = "box.rs"]
pub mod box_;
pub mod color;
pub mod effects;
pub mod flex;
pub mod font;
pub mod image;
#[cfg(feature = "gecko")]
pub mod gecko;
pub mod length;
pub mod percentage;
pub mod position;
pub mod rect;
pub mod svg;
pub mod table;
pub mod text;
pub mod time;
pub mod transform;
pub mod ui;

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

    /// The property we are computing a value for, if it is a non-inherited
    /// property.  None if we are computed a value for an inherited property
    /// or not computing for a property at all (e.g. in a media query
    /// evaluation).
    pub for_non_inherited_property: Option<LonghandId>,

    /// The conditions to cache a rule node on the rule cache.
    ///
    /// FIXME(emilio): Drop the refcell.
    pub rule_cache_conditions: RefCell<&'a mut RuleCacheConditions>,
}

impl<'a> Context<'a> {
    /// Creates a suitable context for media query evaluation, in which
    /// font-relative units compute against the system_font, and executes `f`
    /// with it.
    pub fn for_media_query_evaluation<F, R>(
        device: &Device,
        quirks_mode: QuirksMode,
        f: F,
    ) -> R
    where
        F: FnOnce(&Context) -> R
    {
        let mut conditions = RuleCacheConditions::default();
        let default_values = device.default_computed_values();
        let provider = get_metrics_provider_for_product();

        let context = Context {
            is_root_element: false,
            builder: StyleBuilder::for_derived_style(device, default_values, None, None),
            font_metrics_provider: &provider,
            cached_system_font: None,
            in_media_query: true,
            quirks_mode,
            for_smil_animation: false,
            for_non_inherited_property: None,
            rule_cache_conditions: RefCell::new(&mut conditions),
        };

        f(&context)
    }

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
    pub fn maybe_zoom_text(&self, size: NonNegativeLength) -> NonNegativeLength {
        // We disable zoom for <svg:text> by unsetting the
        // -x-text-zoom property, which leads to a false value
        // in mAllowZoom
        if self.style().get_font().gecko.mAllowZoom {
            self.device().zoom_text(Au::from(size)).into()
        } else {
            size
        }
    }

    /// (Servo doesn't do text-zoom)
    #[cfg(feature = "servo")]
    pub fn maybe_zoom_text(&self, size: NonNegativeLength) -> NonNegativeLength {
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.values.len(), Some(self.values.len()))
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

impl<T> ToComputedValue for Box<T>
    where T: ToComputedValue
{
    type ComputedValue = Box<<T as ToComputedValue>::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        Box::new(T::to_computed_value(self, context))
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Box::new(T::from_computed_value(computed))
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

trivial_to_computed_value!(());
trivial_to_computed_value!(bool);
trivial_to_computed_value!(f32);
trivial_to_computed_value!(i32);
trivial_to_computed_value!(u8);
trivial_to_computed_value!(u16);
trivial_to_computed_value!(u32);
trivial_to_computed_value!(Atom);
trivial_to_computed_value!(BorderStyle);
trivial_to_computed_value!(Cursor);
trivial_to_computed_value!(Namespace);
trivial_to_computed_value!(String);

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
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
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
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, PartialEq)]
/// A computed cliprect for clip and image-region
pub struct ClipRect {
    pub top: Option<Length>,
    pub right: Option<Length>,
    pub bottom: Option<Length>,
    pub left: Option<Length>,
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
pub type TrackList = GenericTrackList<LengthOrPercentage, Integer>;

/// The computed value of a `<grid-line>`.
pub type GridLine = GenericGridLine<Integer>;

/// `<grid-template-rows> | <grid-template-columns>`
pub type GridTemplateComponent = GenericGridTemplateComponent<LengthOrPercentage, Integer>;

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

/// The computed value of a CSS `url()`, resolved relative to the stylesheet URL.
#[cfg(feature = "servo")]
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ComputedUrl {
    /// The `url()` was invalid or it wasn't specified by the user.
    Invalid(#[ignore_malloc_size_of = "Arc"] Arc<String>),
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
