/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified values.
//!
//! TODO(emilio): Enhance docs.

use super::computed::transform::DirectionVector;
use super::computed::{Context, ToComputedValue};
use super::generics::grid::ImplicitGridTracks as GenericImplicitGridTracks;
use super::generics::grid::{GridLine as GenericGridLine, TrackBreadth as GenericTrackBreadth};
use super::generics::grid::{TrackList as GenericTrackList, TrackSize as GenericTrackSize};
use super::generics::transform::IsParallelTo;
use super::generics::{self, GreaterThanOrEqualToOne, NonNegative};
use super::{CSSFloat, CSSInteger, Either, None_};
use crate::context::QuirksMode;
use crate::parser::{Parse, ParserContext};
use crate::values::serialize_atom_identifier;
use crate::values::specified::calc::CalcNode;
use crate::{Atom, Namespace, Prefix, Zero};
use cssparser::{Parser, Token};
use num_traits::One;
use std::f32;
use std::fmt::{self, Write};
use std::ops::Add;
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ParseError, SpecifiedValueInfo, StyleParseErrorKind, ToCss};

#[cfg(feature = "gecko")]
pub use self::align::{AlignContent, AlignItems, AlignSelf, ContentDistribution};
#[cfg(feature = "gecko")]
pub use self::align::{JustifyContent, JustifyItems, JustifySelf, SelfAlignment};
pub use self::angle::Angle;
pub use self::background::{BackgroundRepeat, BackgroundSize};
pub use self::basic_shape::FillRule;
pub use self::border::{BorderCornerRadius, BorderImageSlice, BorderImageWidth};
pub use self::border::{BorderImageRepeat, BorderImageSideWidth};
pub use self::border::{BorderRadius, BorderSideWidth, BorderSpacing, BorderStyle};
pub use self::box_::{AnimationIterationCount, AnimationName, Contain, Display};
pub use self::box_::{Appearance, BreakBetween, BreakWithin};
pub use self::box_::{Clear, Float, Overflow, OverflowAnchor};
pub use self::box_::{OverflowClipBox, OverscrollBehavior, Perspective, Resize};
pub use self::box_::{ScrollSnapAlign, ScrollSnapAxis, ScrollSnapStrictness, ScrollSnapType};
pub use self::box_::{TouchAction, TransitionProperty, VerticalAlign, WillChange};
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
pub use self::image::{ColorStop, EndingShape as GradientEndingShape, Gradient};
pub use self::image::{GradientItem, GradientKind, Image, ImageLayer, MozImageRect};
pub use self::length::{AbsoluteLength, CalcLengthPercentage, CharacterWidth};
pub use self::length::{FontRelativeLength, Length, LengthOrNumber, NonNegativeLengthOrNumber};
pub use self::length::{LengthOrAuto, LengthPercentage, LengthPercentageOrAuto};
pub use self::length::{MaxSize, Size};
pub use self::length::{NoCalcLength, ViewportPercentageLength};
pub use self::length::{NonNegativeLengthPercentage, NonNegativeLengthPercentageOrAuto};
#[cfg(feature = "gecko")]
pub use self::list::ListStyleType;
pub use self::list::MozListReversed;
pub use self::list::Quotes;
pub use self::motion::{OffsetPath, OffsetRotate};
pub use self::outline::OutlineStyle;
pub use self::percentage::Percentage;
pub use self::position::{GridAutoFlow, GridTemplateAreas, Position, PositionOrAuto};
pub use self::position::{PositionComponent, ZIndex};
pub use self::rect::NonNegativeLengthOrNumberRect;
pub use self::resolution::Resolution;
pub use self::svg::MozContextProperties;
pub use self::svg::{SVGLength, SVGOpacity, SVGPaint};
pub use self::svg::{SVGPaintOrder, SVGStrokeDashArray, SVGWidth};
pub use self::svg_path::SVGPathData;
pub use self::text::{InitialLetter, LetterSpacing, LineBreak, LineHeight, TextAlign};
pub use self::text::{OverflowWrap, TextEmphasisPosition, TextEmphasisStyle, WordBreak};
pub use self::text::{TextAlignKeyword, TextDecorationLine, TextOverflow, WordSpacing};
pub use self::text::{TextDecorationLength, TextDecorationSkipInk, TextTransform};
pub use self::time::Time;
pub use self::transform::{Rotate, Scale, Transform};
pub use self::transform::{TransformOrigin, TransformStyle, Translate};
#[cfg(feature = "gecko")]
pub use self::ui::CursorImage;
pub use self::ui::{Cursor, MozForceBrokenImageIcon, UserSelect};
pub use super::generics::grid::GridTemplateComponent as GenericGridTemplateComponent;

#[cfg(feature = "gecko")]
pub mod align;
pub mod angle;
pub mod background;
pub mod basic_shape;
pub mod border;
#[path = "box.rs"]
pub mod box_;
pub mod calc;
pub mod color;
pub mod column;
pub mod counters;
pub mod easing;
pub mod effects;
pub mod flex;
pub mod font;
#[cfg(feature = "gecko")]
pub mod gecko;
pub mod grid;
pub mod image;
pub mod length;
pub mod list;
pub mod motion;
pub mod outline;
pub mod percentage;
pub mod position;
pub mod rect;
pub mod resolution;
pub mod source_size_list;
pub mod svg;
pub mod svg_path;
pub mod text;
pub mod time;
pub mod transform;
pub mod ui;
pub mod url;

/// Parse a `<number>` value, with a given clamping mode.
fn parse_number_with_clamping_mode<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
    clamping_mode: AllowedNumericType,
) -> Result<Number, ParseError<'i>> {
    let location = input.current_source_location();
    // FIXME: remove early returns when lifetimes are non-lexical
    match *input.next()? {
        Token::Number { value, .. } if clamping_mode.is_ok(context.parsing_mode, value) => {
            return Ok(Number {
                value: value.min(f32::MAX).max(f32::MIN),
                calc_clamping_mode: None,
            });
        },
        Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {},
        ref t => return Err(location.new_unexpected_token_error(t.clone())),
    }

    let result = input.parse_nested_block(|i| CalcNode::parse_number(context, i))?;

    Ok(Number {
        value: result.min(f32::MAX).max(f32::MIN),
        calc_clamping_mode: Some(clamping_mode),
    })
}

/// A CSS `<number>` specified value.
///
/// https://drafts.csswg.org/css-values-3/#number-value
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, PartialOrd, ToShmem)]
pub struct Number {
    /// The numeric value itself.
    value: CSSFloat,
    /// If this number came from a calc() expression, this tells how clamping
    /// should be done on the value.
    calc_clamping_mode: Option<AllowedNumericType>,
}

impl Parse for Number {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        parse_number_with_clamping_mode(context, input, AllowedNumericType::All)
    }
}

impl Number {
    /// Returns a new number with the value `val`.
    pub fn new(val: CSSFloat) -> Self {
        Number {
            value: val,
            calc_clamping_mode: None,
        }
    }

    /// Returns whether this number came from a `calc()` expression.
    #[inline]
    pub fn was_calc(&self) -> bool {
        self.calc_clamping_mode.is_some()
    }

    /// Returns the numeric value, clamped if needed.
    #[inline]
    pub fn get(&self) -> f32 {
        self.calc_clamping_mode
            .map_or(self.value, |mode| mode.clamp(self.value))
    }

    #[allow(missing_docs)]
    pub fn parse_non_negative<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Number, ParseError<'i>> {
        parse_number_with_clamping_mode(context, input, AllowedNumericType::NonNegative)
    }

    #[allow(missing_docs)]
    pub fn parse_at_least_one<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Number, ParseError<'i>> {
        parse_number_with_clamping_mode(context, input, AllowedNumericType::AtLeastOne)
    }

    /// Clamp to 1.0 if the value is over 1.0.
    #[inline]
    pub fn clamp_to_one(self) -> Self {
        Number {
            value: self.value.min(1.),
            calc_clamping_mode: self.calc_clamping_mode,
        }
    }
}

impl ToComputedValue for Number {
    type ComputedValue = CSSFloat;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> CSSFloat {
        self.get()
    }

    #[inline]
    fn from_computed_value(computed: &CSSFloat) -> Self {
        Number {
            value: *computed,
            calc_clamping_mode: None,
        }
    }
}

impl ToCss for Number {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.calc_clamping_mode.is_some() {
            dest.write_str("calc(")?;
        }
        self.value.to_css(dest)?;
        if self.calc_clamping_mode.is_some() {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

impl IsParallelTo for (Number, Number, Number) {
    fn is_parallel_to(&self, vector: &DirectionVector) -> bool {
        use euclid::approxeq::ApproxEq;
        // If a and b is parallel, the angle between them is 0deg, so
        // a x b = |a|*|b|*sin(0)*n = 0 * n, |a x b| == 0.
        let self_vector = DirectionVector::new(self.0.get(), self.1.get(), self.2.get());
        self_vector
            .cross(*vector)
            .square_length()
            .approx_eq(&0.0f32)
    }
}

impl SpecifiedValueInfo for Number {}

impl Add for Number {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.get() + other.get())
    }
}

impl Zero for Number {
    #[inline]
    fn zero() -> Self {
        Self::new(0.)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.get() == 0.
    }
}

impl From<Number> for f32 {
    #[inline]
    fn from(n: Number) -> Self {
        n.get()
    }
}

impl From<Number> for f64 {
    #[inline]
    fn from(n: Number) -> Self {
        n.get() as f64
    }
}

/// A Number which is >= 0.0.
pub type NonNegativeNumber = NonNegative<Number>;

impl Parse for NonNegativeNumber {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        parse_number_with_clamping_mode(context, input, AllowedNumericType::NonNegative)
            .map(NonNegative::<Number>)
    }
}

impl NonNegativeNumber {
    /// Returns a new non-negative number with the value `val`.
    pub fn new(val: CSSFloat) -> Self {
        NonNegative::<Number>(Number::new(val.max(0.)))
    }
}

/// A Number which is >= 1.0.
pub type GreaterThanOrEqualToOneNumber = GreaterThanOrEqualToOne<Number>;

impl Parse for GreaterThanOrEqualToOneNumber {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        parse_number_with_clamping_mode(context, input, AllowedNumericType::AtLeastOne)
            .map(GreaterThanOrEqualToOne::<Number>)
    }
}

/// <number> | <percentage>
///
/// Accepts only non-negative numbers.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum NumberOrPercentage {
    Percentage(Percentage),
    Number(Number),
}

impl NumberOrPercentage {
    fn parse_with_clamping_mode<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        type_: AllowedNumericType,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(per) = input.try(|i| Percentage::parse_with_clamping_mode(context, i, type_)) {
            return Ok(NumberOrPercentage::Percentage(per));
        }

        parse_number_with_clamping_mode(context, input, type_).map(NumberOrPercentage::Number)
    }

    /// Parse a non-negative number or percentage.
    pub fn parse_non_negative<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with_clamping_mode(context, input, AllowedNumericType::NonNegative)
    }
}

impl Parse for NumberOrPercentage {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with_clamping_mode(context, input, AllowedNumericType::All)
    }
}

/// A non-negative <number> | <percentage>.
pub type NonNegativeNumberOrPercentage = NonNegative<NumberOrPercentage>;

impl NonNegativeNumberOrPercentage {
    /// Returns the `100%` value.
    #[inline]
    pub fn hundred_percent() -> Self {
        NonNegative(NumberOrPercentage::Percentage(Percentage::hundred()))
    }
}

impl Parse for NonNegativeNumberOrPercentage {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(NonNegative(NumberOrPercentage::parse_non_negative(
            context, input,
        )?))
    }
}

/// The value of Opacity is <alpha-value>, which is "<number> | <percentage>".
/// However, we serialize the specified value as number, so it's ok to store
/// the Opacity as Number.
#[derive(
    Clone, Copy, Debug, MallocSizeOf, PartialEq, PartialOrd, SpecifiedValueInfo, ToCss, ToShmem,
)]
pub struct Opacity(Number);

impl Parse for Opacity {
    /// Opacity accepts <number> | <percentage>, so we parse it as NumberOrPercentage,
    /// and then convert into an Number if it's a Percentage.
    /// https://drafts.csswg.org/cssom/#serializing-css-values
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let number = match NumberOrPercentage::parse(context, input)? {
            NumberOrPercentage::Percentage(p) => Number {
                value: p.get(),
                calc_clamping_mode: if p.is_calc() {
                    Some(AllowedNumericType::All)
                } else {
                    None
                },
            },
            NumberOrPercentage::Number(n) => n,
        };
        Ok(Opacity(number))
    }
}

impl ToComputedValue for Opacity {
    type ComputedValue = CSSFloat;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> CSSFloat {
        let value = self.0.to_computed_value(context);
        if context.for_smil_animation {
            // SMIL expects to be able to interpolate between out-of-range
            // opacity values.
            value
        } else {
            value.min(1.0).max(0.0)
        }
    }

    #[inline]
    fn from_computed_value(computed: &CSSFloat) -> Self {
        Opacity(Number::from_computed_value(computed))
    }
}

/// A specified `<integer>`, optionally coming from a `calc()` expression.
///
/// <https://drafts.csswg.org/css-values/#integers>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, PartialOrd, ToShmem)]
pub struct Integer {
    value: CSSInteger,
    was_calc: bool,
}

impl Zero for Integer {
    #[inline]
    fn zero() -> Self {
        Self::new(0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.value() == 0
    }
}

impl One for Integer {
    #[inline]
    fn one() -> Self {
        Self::new(1)
    }
}

// This is not great, because it loses calc-ness, but it's necessary for One.
impl ::std::ops::Mul<Integer> for Integer {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.value * other.value)
    }
}

impl Integer {
    /// Trivially constructs a new `Integer` value.
    pub fn new(val: CSSInteger) -> Self {
        Integer {
            value: val,
            was_calc: false,
        }
    }

    /// Returns the integer value associated with this value.
    pub fn value(&self) -> CSSInteger {
        self.value
    }

    /// Trivially constructs a new integer value from a `calc()` expression.
    fn from_calc(val: CSSInteger) -> Self {
        Integer {
            value: val,
            was_calc: true,
        }
    }
}

impl Parse for Integer {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();

        // FIXME: remove early returns when lifetimes are non-lexical
        match *input.next()? {
            Token::Number {
                int_value: Some(v), ..
            } => return Ok(Integer::new(v)),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {},
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
        }

        let result = input.parse_nested_block(|i| CalcNode::parse_integer(context, i))?;

        Ok(Integer::from_calc(result))
    }
}

impl Integer {
    /// Parse an integer value which is at least `min`.
    pub fn parse_with_minimum<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        min: i32,
    ) -> Result<Integer, ParseError<'i>> {
        match Integer::parse(context, input) {
            // FIXME(emilio): The spec asks us to avoid rejecting it at parse
            // time except until computed value time.
            //
            // It's not totally clear it's worth it though, and no other browser
            // does this.
            Ok(value) if value.value() >= min => Ok(value),
            Ok(_value) => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
            Err(e) => Err(e),
        }
    }

    /// Parse a non-negative integer.
    pub fn parse_non_negative<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Integer, ParseError<'i>> {
        Integer::parse_with_minimum(context, input, 0)
    }

    /// Parse a positive integer (>= 1).
    pub fn parse_positive<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Integer, ParseError<'i>> {
        Integer::parse_with_minimum(context, input, 1)
    }
}

impl ToComputedValue for Integer {
    type ComputedValue = i32;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> i32 {
        self.value
    }

    #[inline]
    fn from_computed_value(computed: &i32) -> Self {
        Integer::new(*computed)
    }
}

impl ToCss for Integer {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.was_calc {
            dest.write_str("calc(")?;
        }
        self.value.to_css(dest)?;
        if self.was_calc {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

impl SpecifiedValueInfo for Integer {}

/// A wrapper of Integer, with value >= 1.
pub type PositiveInteger = GreaterThanOrEqualToOne<Integer>;

impl Parse for PositiveInteger {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Integer::parse_positive(context, input).map(GreaterThanOrEqualToOne)
    }
}

/// A specified positive `<integer>` value or `none`.
pub type PositiveIntegerOrNone = Either<PositiveInteger, None_>;

/// The specified value of a grid `<track-breadth>`
pub type TrackBreadth = GenericTrackBreadth<LengthPercentage>;

/// The specified value of a grid `<track-size>`
pub type TrackSize = GenericTrackSize<LengthPercentage>;

/// The specified value of a grid `<track-size>+`
pub type ImplicitGridTracks = GenericImplicitGridTracks<TrackSize>;

/// The specified value of a grid `<track-list>`
/// (could also be `<auto-track-list>` or `<explicit-track-list>`)
pub type TrackList = GenericTrackList<LengthPercentage, Integer>;

/// The specified value of a `<grid-line>`.
pub type GridLine = GenericGridLine<Integer>;

/// `<grid-template-rows> | <grid-template-columns>`
pub type GridTemplateComponent = GenericGridTemplateComponent<LengthPercentage, Integer>;

/// rect(...)
pub type ClipRect = generics::GenericClipRect<LengthOrAuto>;

impl Parse for ClipRect {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl ClipRect {
    /// Parses a rect(<top>, <left>, <bottom>, <right>), allowing quirks.
    fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("rect")?;

        fn parse_argument<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
            allow_quirks: AllowQuirks,
        ) -> Result<LengthOrAuto, ParseError<'i>> {
            LengthOrAuto::parse_quirky(context, input, allow_quirks)
        }

        input.parse_nested_block(|input| {
            let top = parse_argument(context, input, allow_quirks)?;
            let right;
            let bottom;
            let left;

            if input.try(|input| input.expect_comma()).is_ok() {
                right = parse_argument(context, input, allow_quirks)?;
                input.expect_comma()?;
                bottom = parse_argument(context, input, allow_quirks)?;
                input.expect_comma()?;
                left = parse_argument(context, input, allow_quirks)?;
            } else {
                right = parse_argument(context, input, allow_quirks)?;
                bottom = parse_argument(context, input, allow_quirks)?;
                left = parse_argument(context, input, allow_quirks)?;
            }

            Ok(ClipRect {
                top,
                right,
                bottom,
                left,
            })
        })
    }
}

/// rect(...) | auto
pub type ClipRectOrAuto = generics::GenericClipRectOrAuto<ClipRect>;

impl ClipRectOrAuto {
    /// Parses a ClipRect or Auto, allowing quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(v) = input.try(|i| ClipRect::parse_quirky(context, i, allow_quirks)) {
            return Ok(generics::GenericClipRectOrAuto::Rect(v));
        }
        input.expect_ident_matching("auto")?;
        Ok(generics::GenericClipRectOrAuto::Auto)
    }
}

/// Whether quirks are allowed in this context.
#[derive(Clone, Copy, PartialEq)]
pub enum AllowQuirks {
    /// Quirks are not allowed.
    No,
    /// Quirks are allowed, in quirks mode.
    Yes,
    /// Quirks are always allowed, used for SVG lengths.
    Always,
}

impl AllowQuirks {
    /// Returns `true` if quirks are allowed in this context.
    pub fn allowed(self, quirks_mode: QuirksMode) -> bool {
        match self {
            AllowQuirks::Always => true,
            AllowQuirks::No => false,
            AllowQuirks::Yes => quirks_mode == QuirksMode::Quirks,
        }
    }
}

/// An attr(...) rule
///
/// `[namespace? `|`]? ident`
#[derive(
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[css(function)]
pub struct Attr {
    /// Optional namespace prefix and URL.
    pub namespace: Option<(Prefix, Namespace)>,
    /// Attribute name
    pub attribute: Atom,
}

impl Parse for Attr {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Attr, ParseError<'i>> {
        input.expect_function_matching("attr")?;
        input.parse_nested_block(|i| Attr::parse_function(context, i))
    }
}

/// Get the Namespace for a given prefix from the namespace map.
fn get_namespace_for_prefix(prefix: &Prefix, context: &ParserContext) -> Option<Namespace> {
    context
        .namespaces
        .as_ref()?
        .prefixes
        .get(prefix)
        .map(|x| x.clone())
}

impl Attr {
    /// Parse contents of attr() assuming we have already parsed `attr` and are
    /// within a parse_nested_block()
    pub fn parse_function<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Attr, ParseError<'i>> {
        // Syntax is `[namespace? `|`]? ident`
        // no spaces allowed
        let first = input.try(|i| i.expect_ident_cloned()).ok();
        if let Ok(token) = input.try(|i| i.next_including_whitespace().map(|t| t.clone())) {
            match token {
                Token::Delim('|') => {
                    let location = input.current_source_location();
                    // must be followed by an ident
                    let second_token = match *input.next_including_whitespace()? {
                        Token::Ident(ref second) => second,
                        ref t => return Err(location.new_unexpected_token_error(t.clone())),
                    };

                    let prefix_and_ns = if let Some(ns) = first {
                        let prefix = Prefix::from(ns.as_ref());
                        let ns = match get_namespace_for_prefix(&prefix, context) {
                            Some(ns) => ns,
                            None => {
                                return Err(location
                                    .new_custom_error(StyleParseErrorKind::UnspecifiedError));
                            },
                        };
                        Some((prefix, ns))
                    } else {
                        None
                    };
                    return Ok(Attr {
                        namespace: prefix_and_ns,
                        attribute: Atom::from(second_token.as_ref()),
                    });
                },
                // In the case of attr(foobar    ) we don't want to error out
                // because of the trailing whitespace
                Token::WhiteSpace(..) => {},
                ref t => return Err(input.new_unexpected_token_error(t.clone())),
            }
        }

        if let Some(first) = first {
            Ok(Attr {
                namespace: None,
                attribute: Atom::from(first.as_ref()),
            })
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

impl ToCss for Attr {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("attr(")?;
        if let Some((ref prefix, ref _url)) = self.namespace {
            serialize_atom_identifier(prefix, dest)?;
            dest.write_str("|")?;
        }
        serialize_atom_identifier(&self.attribute, dest)?;
        dest.write_str(")")
    }
}
