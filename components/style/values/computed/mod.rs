/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed values.

use self::transform::DirectionVector;
use super::animated::ToAnimatedValue;
use super::generics::grid::GridTemplateComponent as GenericGridTemplateComponent;
use super::generics::grid::ImplicitGridTracks as GenericImplicitGridTracks;
use super::generics::grid::{GenericGridLine, GenericTrackBreadth};
use super::generics::grid::{GenericTrackSize, TrackList as GenericTrackList};
use super::generics::transform::IsParallelTo;
use super::generics::{self, GreaterThanOrEqualToOne, NonNegative, ZeroToOne};
use super::specified;
use super::{CSSFloat, CSSInteger};
use crate::context::QuirksMode;
use crate::font_metrics::{get_metrics_provider_for_product, FontMetricsProvider};
use crate::media_queries::Device;
#[cfg(feature = "gecko")]
use crate::properties;
use crate::properties::{ComputedValues, LonghandId, StyleBuilder};
use crate::rule_cache::RuleCacheConditions;
use crate::{ArcSlice, Atom};
use euclid::default::Size2D;
use servo_arc::Arc;
use std::cell::RefCell;
use std::cmp;
use std::f32;

#[cfg(feature = "gecko")]
pub use self::align::{AlignContent, AlignItems, JustifyContent, JustifyItems, SelfAlignment};
#[cfg(feature = "gecko")]
pub use self::align::{AlignSelf, JustifySelf};
pub use self::angle::Angle;
pub use self::background::{BackgroundRepeat, BackgroundSize};
pub use self::basic_shape::FillRule;
pub use self::border::{BorderCornerRadius, BorderRadius, BorderSpacing};
pub use self::border::{BorderImageRepeat, BorderImageSideWidth};
pub use self::border::{BorderImageSlice, BorderImageWidth};
pub use self::box_::{AnimationIterationCount, AnimationName, Contain};
pub use self::box_::{Appearance, BreakBetween, BreakWithin, Clear, Float};
pub use self::box_::{Display, Overflow, OverflowAnchor, TransitionProperty};
pub use self::box_::{OverflowClipBox, OverscrollBehavior, Perspective, Resize};
pub use self::box_::{ScrollSnapAlign, ScrollSnapAxis, ScrollSnapStrictness, ScrollSnapType};
pub use self::box_::{TouchAction, VerticalAlign, WillChange};
pub use self::color::{Color, ColorOrAuto, ColorPropertyValue};
pub use self::column::ColumnCount;
pub use self::counters::{Content, ContentItem, CounterIncrement, CounterSetOrReset};
pub use self::easing::TimingFunction;
pub use self::effects::{BoxShadow, Filter, SimpleShadow};
pub use self::flex::FlexBasis;
pub use self::font::{FontFamily, FontLanguageOverride, FontStyle};
pub use self::font::{FontFeatureSettings, FontVariantLigatures, FontVariantNumeric};
pub use self::font::{FontSize, FontSizeAdjust, FontStretch, FontSynthesis};
pub use self::font::{FontVariantAlternates, FontWeight};
pub use self::font::{FontVariantEastAsian, FontVariationSettings};
pub use self::font::{MozScriptLevel, MozScriptMinSize, MozScriptSizeMultiplier, XLang, XTextZoom};
pub use self::image::{Gradient, Image, LineDirection, MozImageRect};
pub use self::length::{CSSPixelLength, ExtremumLength, NonNegativeLength};
pub use self::length::{Length, LengthOrNumber, LengthPercentage, NonNegativeLengthOrNumber};
pub use self::length::{LengthOrAuto, LengthPercentageOrAuto, MaxSize, Size};
pub use self::length::{NonNegativeLengthPercentage, NonNegativeLengthPercentageOrAuto};
#[cfg(feature = "gecko")]
pub use self::list::ListStyleType;
pub use self::list::MozListReversed;
pub use self::list::Quotes;
pub use self::motion::{OffsetPath, OffsetRotate};
pub use self::outline::OutlineStyle;
pub use self::percentage::{NonNegativePercentage, Percentage};
pub use self::position::{GridAutoFlow, GridTemplateAreas, Position, PositionOrAuto, ZIndex};
pub use self::rect::NonNegativeLengthOrNumberRect;
pub use self::resolution::Resolution;
pub use self::svg::MozContextProperties;
pub use self::svg::{SVGLength, SVGOpacity, SVGPaint, SVGPaintKind};
pub use self::svg::{SVGPaintOrder, SVGStrokeDashArray, SVGWidth};
pub use self::text::TextUnderlinePosition;
pub use self::text::{InitialLetter, LetterSpacing, LineBreak, LineHeight};
pub use self::text::{OverflowWrap, TextOverflow, WordBreak, WordSpacing};
pub use self::text::{TextAlign, TextAlignLast, TextEmphasisPosition, TextEmphasisStyle};
pub use self::text::{TextDecorationLength, TextDecorationSkipInk};
pub use self::time::Time;
pub use self::transform::{Rotate, Scale, Transform, TransformOperation};
pub use self::transform::{TransformOrigin, TransformStyle, Translate};
#[cfg(feature = "gecko")]
pub use self::ui::CursorImage;
pub use self::ui::{Cursor, MozForceBrokenImageIcon, UserSelect};
pub use super::specified::TextTransform;
pub use super::specified::{BorderStyle, TextDecorationLine};
pub use super::{Auto, Either, None_};
pub use app_units::Au;

#[cfg(feature = "gecko")]
pub mod align;
pub mod angle;
pub mod background;
pub mod basic_shape;
pub mod border;
#[path = "box.rs"]
pub mod box_;
pub mod color;
pub mod column;
pub mod counters;
pub mod easing;
pub mod effects;
pub mod flex;
pub mod font;
pub mod image;
pub mod length;
pub mod length_percentage;
pub mod list;
pub mod motion;
pub mod outline;
pub mod percentage;
pub mod position;
pub mod rect;
pub mod resolution;
pub mod svg;
pub mod text;
pub mod time;
pub mod transform;
pub mod ui;
pub mod url;

/// A `Context` is all the data a specified value could ever need to compute
/// itself and be transformed to a computed value.
pub struct Context<'a> {
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
    pub font_metrics_provider: &'a dyn FontMetricsProvider,

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
    pub fn for_media_query_evaluation<F, R>(device: &Device, quirks_mode: QuirksMode, f: F) -> R
    where
        F: FnOnce(&Context) -> R,
    {
        let mut conditions = RuleCacheConditions::default();
        let provider = get_metrics_provider_for_product();

        let context = Context {
            builder: StyleBuilder::for_inheritance(device, None, None),
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

    /// The current device.
    pub fn device(&self) -> &Device {
        self.builder.device
    }

    /// The current viewport size, used to resolve viewport units.
    pub fn viewport_size_for_viewport_unit_resolution(&self) -> Size2D<Au> {
        self.builder
            .device
            .au_viewport_size_for_viewport_unit_resolution()
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
    pub fn maybe_zoom_text(&self, size: CSSPixelLength) -> CSSPixelLength {
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
    pub fn maybe_zoom_text(&self, size: CSSPixelLength) -> CSSPixelLength {
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

impl<'a, 'cx, 'cx_a: 'cx, S: ToComputedValue + 'a> ExactSizeIterator
    for ComputedVecIter<'a, 'cx, 'cx_a, S>
{
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
/// of the passed value. The deriving code assumes that if the type isn't
/// generic, then the trait can be implemented as simple `Clone::clone` calls,
/// this means that a manual implementation with `ComputedValue = Self` is bogus
/// if it returns anything else than a clone.
pub trait ToComputedValue {
    /// The computed value type we're going to be converted to.
    type ComputedValue;

    /// Convert a specified value to a computed value, using itself and the data
    /// inside the `Context`.
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue;

    /// Convert a computed value to specified value form.
    ///
    /// This will be used for recascading during animation.
    /// Such from_computed_valued values should recompute to the same value.
    fn from_computed_value(computed: &Self::ComputedValue) -> Self;
}

impl<A, B> ToComputedValue for (A, B)
where
    A: ToComputedValue,
    B: ToComputedValue,
{
    type ComputedValue = (
        <A as ToComputedValue>::ComputedValue,
        <B as ToComputedValue>::ComputedValue,
    );

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        (
            self.0.to_computed_value(context),
            self.1.to_computed_value(context),
        )
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        (
            A::from_computed_value(&computed.0),
            B::from_computed_value(&computed.1),
        )
    }
}

impl<T> ToComputedValue for Option<T>
where
    T: ToComputedValue,
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
where
    T: ToComputedValue,
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
where
    T: ToComputedValue,
{
    type ComputedValue = Vec<<T as ToComputedValue>::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.iter()
            .map(|item| item.to_computed_value(context))
            .collect()
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed.iter().map(T::from_computed_value).collect()
    }
}

impl<T> ToComputedValue for Box<T>
where
    T: ToComputedValue,
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
where
    T: ToComputedValue,
{
    type ComputedValue = Box<[<T as ToComputedValue>::ComputedValue]>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.iter()
            .map(|item| item.to_computed_value(context))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed
            .iter()
            .map(T::from_computed_value)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

impl<T> ToComputedValue for crate::OwnedSlice<T>
where
    T: ToComputedValue,
{
    type ComputedValue = crate::OwnedSlice<<T as ToComputedValue>::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.iter()
            .map(|item| item.to_computed_value(context))
            .collect()
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed.iter().map(T::from_computed_value).collect()
    }
}

// NOTE(emilio): This is implementable more generically, but it's unlikely
// what you want there, as it forces you to have an extra allocation.
//
// We could do that if needed, ideally with specialization for the case where
// ComputedValue = T. But we don't need it for now.
impl<T> ToComputedValue for Arc<T>
where
    T: ToComputedValue<ComputedValue = T>,
{
    type ComputedValue = Self;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self {
        self.clone()
    }

    #[inline]
    fn from_computed_value(computed: &Self) -> Self {
        computed.clone()
    }
}

// Same caveat as above applies.
impl<T> ToComputedValue for ArcSlice<T>
where
    T: ToComputedValue<ComputedValue = T>,
{
    type ComputedValue = Self;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self {
        self.clone()
    }

    #[inline]
    fn from_computed_value(computed: &Self) -> Self {
        computed.clone()
    }
}

trivial_to_computed_value!(());
trivial_to_computed_value!(bool);
trivial_to_computed_value!(f32);
trivial_to_computed_value!(i32);
trivial_to_computed_value!(u8);
trivial_to_computed_value!(u16);
trivial_to_computed_value!(u32);
trivial_to_computed_value!(usize);
trivial_to_computed_value!(Atom);
#[cfg(feature = "servo")]
trivial_to_computed_value!(html5ever::Namespace);
#[cfg(feature = "servo")]
trivial_to_computed_value!(html5ever::Prefix);
trivial_to_computed_value!(String);
trivial_to_computed_value!(Box<str>);
trivial_to_computed_value!(crate::OwnedStr);
trivial_to_computed_value!(style_traits::values::specified::AllowedNumericType);

#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    ToAnimatedZero,
    ToCss,
    ToResolvedValue,
)]
#[repr(C, u8)]
pub enum AngleOrPercentage {
    Percentage(Percentage),
    Angle(Angle),
}

impl ToComputedValue for specified::AngleOrPercentage {
    type ComputedValue = AngleOrPercentage;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> AngleOrPercentage {
        match *self {
            specified::AngleOrPercentage::Percentage(percentage) => {
                AngleOrPercentage::Percentage(percentage.to_computed_value(context))
            },
            specified::AngleOrPercentage::Angle(angle) => {
                AngleOrPercentage::Angle(angle.to_computed_value(context))
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &AngleOrPercentage) -> Self {
        match *computed {
            AngleOrPercentage::Percentage(percentage) => specified::AngleOrPercentage::Percentage(
                ToComputedValue::from_computed_value(&percentage),
            ),
            AngleOrPercentage::Angle(angle) => {
                specified::AngleOrPercentage::Angle(ToComputedValue::from_computed_value(&angle))
            },
        }
    }
}

/// A `<number>` value.
pub type Number = CSSFloat;

impl IsParallelTo for (Number, Number, Number) {
    fn is_parallel_to(&self, vector: &DirectionVector) -> bool {
        use euclid::approxeq::ApproxEq;
        // If a and b is parallel, the angle between them is 0deg, so
        // a x b = |a|*|b|*sin(0)*n = 0 * n, |a x b| == 0.
        let self_vector = DirectionVector::new(self.0, self.1, self.2);
        self_vector
            .cross(*vector)
            .square_length()
            .approx_eq(&0.0f32)
    }
}

/// A wrapper of Number, but the value >= 0.
pub type NonNegativeNumber = NonNegative<CSSFloat>;

impl ToAnimatedValue for NonNegativeNumber {
    type AnimatedValue = CSSFloat;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.max(0.).into()
    }
}

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

/// A wrapper of Number, but the value between 0 and 1
pub type ZeroToOneNumber = ZeroToOne<CSSFloat>;

impl ToAnimatedValue for ZeroToOneNumber {
    type AnimatedValue = CSSFloat;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        Self(animated.max(0.).min(1.))
    }
}

impl From<CSSFloat> for ZeroToOneNumber {
    #[inline]
    fn from(number: CSSFloat) -> Self {
        Self(number)
    }
}

/// A wrapper of Number, but the value >= 1.
pub type GreaterThanOrEqualToOneNumber = GreaterThanOrEqualToOne<CSSFloat>;

impl ToAnimatedValue for GreaterThanOrEqualToOneNumber {
    type AnimatedValue = CSSFloat;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.max(1.).into()
    }
}

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
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    ToAnimatedZero,
    ToCss,
    ToResolvedValue,
)]
#[repr(C, u8)]
pub enum NumberOrPercentage {
    Percentage(Percentage),
    Number(Number),
}

impl NumberOrPercentage {
    fn clamp_to_non_negative(self) -> Self {
        match self {
            NumberOrPercentage::Percentage(p) => {
                NumberOrPercentage::Percentage(p.clamp_to_non_negative())
            },
            NumberOrPercentage::Number(n) => NumberOrPercentage::Number(n.max(0.)),
        }
    }
}

impl ToComputedValue for specified::NumberOrPercentage {
    type ComputedValue = NumberOrPercentage;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> NumberOrPercentage {
        match *self {
            specified::NumberOrPercentage::Percentage(percentage) => {
                NumberOrPercentage::Percentage(percentage.to_computed_value(context))
            },
            specified::NumberOrPercentage::Number(number) => {
                NumberOrPercentage::Number(number.to_computed_value(context))
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &NumberOrPercentage) -> Self {
        match *computed {
            NumberOrPercentage::Percentage(percentage) => {
                specified::NumberOrPercentage::Percentage(ToComputedValue::from_computed_value(
                    &percentage,
                ))
            },
            NumberOrPercentage::Number(number) => {
                specified::NumberOrPercentage::Number(ToComputedValue::from_computed_value(&number))
            },
        }
    }
}

/// A non-negative <number-percentage>.
pub type NonNegativeNumberOrPercentage = NonNegative<NumberOrPercentage>;

impl NonNegativeNumberOrPercentage {
    /// Returns the `100%` value.
    #[inline]
    pub fn hundred_percent() -> Self {
        NonNegative(NumberOrPercentage::Percentage(Percentage::hundred()))
    }
}

impl ToAnimatedValue for NonNegativeNumberOrPercentage {
    type AnimatedValue = NumberOrPercentage;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        NonNegative(animated.clamp_to_non_negative())
    }
}

/// A type used for opacity.
pub type Opacity = CSSFloat;

/// A `<integer>` value.
pub type Integer = CSSInteger;

/// A wrapper of Integer, but only accept a value >= 1.
pub type PositiveInteger = GreaterThanOrEqualToOne<CSSInteger>;

impl ToAnimatedValue for PositiveInteger {
    type AnimatedValue = CSSInteger;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        cmp::max(animated, 1).into()
    }
}

impl From<CSSInteger> for PositiveInteger {
    #[inline]
    fn from(int: CSSInteger) -> PositiveInteger {
        GreaterThanOrEqualToOne::<CSSInteger>(int)
    }
}

/// A computed positive `<integer>` value or `none`.
pub type PositiveIntegerOrNone = Either<PositiveInteger, None_>;

/// rect(...) | auto
pub type ClipRect = generics::GenericClipRect<LengthOrAuto>;

/// rect(...) | auto
pub type ClipRectOrAuto = generics::GenericClipRectOrAuto<ClipRect>;

/// The computed value of a grid `<track-breadth>`
pub type TrackBreadth = GenericTrackBreadth<LengthPercentage>;

/// The computed value of a grid `<track-size>`
pub type TrackSize = GenericTrackSize<LengthPercentage>;

/// The computed value of a grid `<track-size>+`
pub type ImplicitGridTracks = GenericImplicitGridTracks<TrackSize>;

/// The computed value of a grid `<track-list>`
/// (could also be `<auto-track-list>` or `<explicit-track-list>`)
pub type TrackList = GenericTrackList<LengthPercentage, Integer>;

/// The computed value of a `<grid-line>`.
pub type GridLine = GenericGridLine<Integer>;

/// `<grid-template-rows> | <grid-template-columns>`
pub type GridTemplateComponent = GenericGridTemplateComponent<LengthPercentage, Integer>;
