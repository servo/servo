/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [Length values][length].
//!
//! [length]: https://drafts.csswg.org/css-values/#lengths

use app_units::Au;
use cssparser::{Parser, Token};
use euclid::Size2D;
use font_metrics::FontMetricsQueryResult;
use parser::{Parse, ParserContext};
use std::{cmp, mem};
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::ops::{Add, Mul};
use style_traits::{ParseError, StyleParseErrorKind};
use style_traits::values::specified::AllowedNumericType;
use stylesheets::CssRuleType;
use super::{AllowQuirks, Number, ToComputedValue, Percentage};
use values::{Auto, CSSFloat, Either, ExtremumLength, None_, Normal};
use values::computed::{self, CSSPixelLength, Context};
use values::generics::NonNegative;
use values::specified::NonNegativeNumber;
use values::specified::calc::CalcNode;

pub use values::specified::calc::CalcLengthOrPercentage;
pub use super::image::{ColorStop, EndingShape as GradientEndingShape, Gradient};
pub use super::image::{GradientKind, Image};

/// Number of app units per pixel
pub const AU_PER_PX: CSSFloat = 60.;
/// Number of app units per inch
pub const AU_PER_IN: CSSFloat = AU_PER_PX * 96.;
/// Number of app units per centimeter
pub const AU_PER_CM: CSSFloat = AU_PER_IN / 2.54;
/// Number of app units per millimeter
pub const AU_PER_MM: CSSFloat = AU_PER_IN / 25.4;
/// Number of app units per quarter
pub const AU_PER_Q: CSSFloat = AU_PER_MM / 4.;
/// Number of app units per point
pub const AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
/// Number of app units per pica
pub const AU_PER_PC: CSSFloat = AU_PER_PT * 12.;

/// Same as Gecko's AppUnitsToIntCSSPixels
///
/// Converts app units to integer pixel values,
/// rounding during the conversion
pub fn au_to_int_px(au: f32) -> i32 {
    (au / AU_PER_PX).round() as i32
}

/// A font relative length.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum FontRelativeLength {
    /// A "em" value: https://drafts.csswg.org/css-values/#em
    #[css(dimension)]
    Em(CSSFloat),
    /// A "ex" value: https://drafts.csswg.org/css-values/#ex
    #[css(dimension)]
    Ex(CSSFloat),
    /// A "ch" value: https://drafts.csswg.org/css-values/#ch
    #[css(dimension)]
    Ch(CSSFloat),
    /// A "rem" value: https://drafts.csswg.org/css-values/#rem
    #[css(dimension)]
    Rem(CSSFloat)
}

/// A source to resolve font-relative units against
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontBaseSize {
    /// Use the font-size of the current element.
    CurrentStyle,
    /// Use the inherited font-size.
    InheritedStyle,
    /// Use the inherited font-size, but strip em units.
    ///
    /// FIXME(emilio): This is very complex, and should go away.
    InheritedStyleButStripEmUnits,
    /// Use a custom base size.
    ///
    /// FIXME(emilio): This is very dubious, and only used for MathML.
    Custom(Au),
}

impl FontBaseSize {
    /// Calculate the actual size for a given context
    pub fn resolve(&self, context: &Context) -> Au {
        match *self {
            FontBaseSize::Custom(size) => size,
            FontBaseSize::CurrentStyle => context.style().get_font().clone_font_size().size(),
            FontBaseSize::InheritedStyleButStripEmUnits |
            FontBaseSize::InheritedStyle => context.style().get_parent_font().clone_font_size().size(),
        }
    }
}

impl FontRelativeLength {
    /// Computes the font-relative length.
    pub fn to_computed_value(&self, context: &Context, base_size: FontBaseSize) -> CSSPixelLength {
        use std::f32;
        let (reference_size, length) = self.reference_font_size_and_length(context, base_size);
        let pixel = (length * reference_size.to_f32_px()).min(f32::MAX).max(f32::MIN);
        CSSPixelLength::new(pixel)
    }

    /// Return reference font size.
    ///
    /// We use the base_size flag to pass a different size for computing
    /// font-size and unconstrained font-size.
    ///
    /// This returns a pair, the first one is the reference font size, and the
    /// second one is the unpacked relative length.
    fn reference_font_size_and_length(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> (Au, CSSFloat) {
        fn query_font_metrics(
            context: &Context,
            reference_font_size: Au,
        ) -> FontMetricsQueryResult {
            context.font_metrics_provider.query(
                context.style().get_font(),
                reference_font_size,
                context.style().writing_mode,
                context.in_media_query,
                context.device(),
            )
        }

        let reference_font_size = base_size.resolve(context);
        match *self {
            FontRelativeLength::Em(length) => {
                if context.for_non_inherited_property.is_some() {
                    if base_size == FontBaseSize::CurrentStyle {
                        context.rule_cache_conditions.borrow_mut()
                            .set_font_size_dependency(
                                reference_font_size.into()
                            );
                    }
                }

                if base_size == FontBaseSize::InheritedStyleButStripEmUnits {
                    (Au(0), length)
                } else {
                    (reference_font_size, length)
                }
            },
            FontRelativeLength::Ex(length) => {
                if context.for_non_inherited_property.is_some() {
                    context.rule_cache_conditions.borrow_mut().set_uncacheable();
                }
                let reference_size = match query_font_metrics(context, reference_font_size) {
                    FontMetricsQueryResult::Available(metrics) => {
                        metrics.x_height
                    },
                    // https://drafts.csswg.org/css-values/#ex
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the x-height, a value of 0.5em must be
                    //     assumed.
                    //
                    FontMetricsQueryResult::NotAvailable => {
                        reference_font_size.scale_by(0.5)
                    },
                };
                (reference_size, length)
            },
            FontRelativeLength::Ch(length) => {
                if context.for_non_inherited_property.is_some() {
                    context.rule_cache_conditions.borrow_mut().set_uncacheable();
                }
                let reference_size = match query_font_metrics(context, reference_font_size) {
                    FontMetricsQueryResult::Available(metrics) => {
                        metrics.zero_advance_measure
                    },
                    // https://drafts.csswg.org/css-values/#ch
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the measure of the “0” glyph, it must be
                    //     assumed to be 0.5em wide by 1em tall. Thus, the ch
                    //     unit falls back to 0.5em in the general case, and to
                    //     1em when it would be typeset upright (i.e.
                    //     writing-mode is vertical-rl or vertical-lr and
                    //     text-orientation is upright).
                    //
                    FontMetricsQueryResult::NotAvailable => {
                        if context.style().writing_mode.is_vertical() {
                            reference_font_size
                        } else {
                            reference_font_size.scale_by(0.5)
                        }
                    }
                };
                (reference_size, length)
            }
            FontRelativeLength::Rem(length) => {
                // https://drafts.csswg.org/css-values/#rem:
                //
                //     When specified on the font-size property of the root
                //     element, the rem units refer to the property's initial
                //     value.
                //
                let reference_size = if context.is_root_element || context.in_media_query {
                    reference_font_size
                } else {
                    context.device().root_font_size()
                };
                (reference_size, length)
            }
        }
    }
}

/// A viewport-relative length.
///
/// <https://drafts.csswg.org/css-values/#viewport-relative-lengths>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum ViewportPercentageLength {
    /// A vw unit: https://drafts.csswg.org/css-values/#vw
    #[css(dimension)]
    Vw(CSSFloat),
    /// A vh unit: https://drafts.csswg.org/css-values/#vh
    #[css(dimension)]
    Vh(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#vmin>
    #[css(dimension)]
    Vmin(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#vmax>
    #[css(dimension)]
    Vmax(CSSFloat)
}

impl ViewportPercentageLength {
    /// Computes the given viewport-relative length for the given viewport size.
    pub fn to_computed_value(&self, viewport_size: Size2D<Au>) -> CSSPixelLength {
        let (factor, length) = match *self {
            ViewportPercentageLength::Vw(length) =>
                (length, viewport_size.width),
            ViewportPercentageLength::Vh(length) =>
                (length, viewport_size.height),
            ViewportPercentageLength::Vmin(length) =>
                (length, cmp::min(viewport_size.width, viewport_size.height)),
            ViewportPercentageLength::Vmax(length) =>
                (length, cmp::max(viewport_size.width, viewport_size.height)),
        };

        // FIXME: Bug 1396535, we need to fix the extremely small viewport length for transform.
        // See bug 989802. We truncate so that adding multiple viewport units
        // that add up to 100 does not overflow due to rounding differences
        let trunc_scaled = ((length.0 as f64) * factor as f64 / 100.).trunc();
        Au::from_f64_au(trunc_scaled).into()
    }
}

/// HTML5 "character width", as defined in HTML5 § 14.5.4.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
pub struct CharacterWidth(pub i32);

impl CharacterWidth {
    /// Computes the given character width.
    pub fn to_computed_value(&self, reference_font_size: Au) -> CSSPixelLength {
        // This applies the *converting a character width to pixels* algorithm as specified
        // in HTML5 § 14.5.4.
        //
        // TODO(pcwalton): Find these from the font.
        let average_advance = reference_font_size.scale_by(0.5);
        let max_advance = reference_font_size;
        let au = average_advance.scale_by(self.0 as CSSFloat - 1.0) + max_advance;
        au.into()
    }
}

/// Represents an absolute length with its unit
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum AbsoluteLength {
    /// An absolute length in pixels (px)
    #[css(dimension)]
    Px(CSSFloat),
    /// An absolute length in inches (in)
    #[css(dimension)]
    In(CSSFloat),
    /// An absolute length in centimeters (cm)
    #[css(dimension)]
    Cm(CSSFloat),
    /// An absolute length in millimeters (mm)
    #[css(dimension)]
    Mm(CSSFloat),
    /// An absolute length in quarter-millimeters (q)
    #[css(dimension)]
    Q(CSSFloat),
    /// An absolute length in points (pt)
    #[css(dimension)]
    Pt(CSSFloat),
    /// An absolute length in pica (pc)
    #[css(dimension)]
    Pc(CSSFloat),
}

impl AbsoluteLength {
    fn is_zero(&self) -> bool {
        match *self {
            AbsoluteLength::Px(v)
            | AbsoluteLength::In(v)
            | AbsoluteLength::Cm(v)
            | AbsoluteLength::Mm(v)
            | AbsoluteLength::Q(v)
            | AbsoluteLength::Pt(v)
            | AbsoluteLength::Pc(v) => v == 0.,
        }
    }

    /// Convert this into a pixel value.
    #[inline]
    pub fn to_px(&self) -> CSSFloat {
        use std::f32;

        let pixel = match *self {
            AbsoluteLength::Px(value) => value,
            AbsoluteLength::In(value) => value * (AU_PER_IN / AU_PER_PX),
            AbsoluteLength::Cm(value) => value * (AU_PER_CM / AU_PER_PX),
            AbsoluteLength::Mm(value) => value * (AU_PER_MM / AU_PER_PX),
            AbsoluteLength::Q(value) => value * (AU_PER_Q / AU_PER_PX),
            AbsoluteLength::Pt(value) => value * (AU_PER_PT / AU_PER_PX),
            AbsoluteLength::Pc(value) => value * (AU_PER_PC / AU_PER_PX),
        };
        pixel.min(f32::MAX).max(f32::MIN)
    }
}

impl ToComputedValue for AbsoluteLength {
    type ComputedValue = CSSPixelLength;

    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        CSSPixelLength::new(self.to_px())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        AbsoluteLength::Px(computed.px())
    }
}

impl Mul<CSSFloat> for AbsoluteLength {
    type Output = AbsoluteLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> AbsoluteLength {
        match self {
            AbsoluteLength::Px(v) => AbsoluteLength::Px(v * scalar),
            AbsoluteLength::In(v) => AbsoluteLength::In(v * scalar),
            AbsoluteLength::Cm(v) => AbsoluteLength::Cm(v * scalar),
            AbsoluteLength::Mm(v) => AbsoluteLength::Mm(v * scalar),
            AbsoluteLength::Q(v) => AbsoluteLength::Q(v * scalar),
            AbsoluteLength::Pt(v) => AbsoluteLength::Pt(v * scalar),
            AbsoluteLength::Pc(v) => AbsoluteLength::Pc(v * scalar),
        }
    }
}

impl Add<AbsoluteLength> for AbsoluteLength {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (AbsoluteLength::Px(x), AbsoluteLength::Px(y)) => AbsoluteLength::Px(x + y),
            (AbsoluteLength::In(x), AbsoluteLength::In(y)) => AbsoluteLength::In(x + y),
            (AbsoluteLength::Cm(x), AbsoluteLength::Cm(y)) => AbsoluteLength::Cm(x + y),
            (AbsoluteLength::Mm(x), AbsoluteLength::Mm(y)) => AbsoluteLength::Mm(x + y),
            (AbsoluteLength::Q(x), AbsoluteLength::Q(y)) => AbsoluteLength::Q(x + y),
            (AbsoluteLength::Pt(x), AbsoluteLength::Pt(y)) => AbsoluteLength::Pt(x + y),
            (AbsoluteLength::Pc(x), AbsoluteLength::Pc(y)) => AbsoluteLength::Pc(x + y),
            _ => AbsoluteLength::Px(self.to_px() + rhs.to_px()),
        }
    }
}

/// A `<length>` without taking `calc` expressions into account
///
/// <https://drafts.csswg.org/css-values/#lengths>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum NoCalcLength {
    /// An absolute length
    ///
    /// <https://drafts.csswg.org/css-values/#absolute-length>
    Absolute(AbsoluteLength),

    /// A font-relative length:
    ///
    /// <https://drafts.csswg.org/css-values/#font-relative-lengths>
    FontRelative(FontRelativeLength),

    /// A viewport-relative length.
    ///
    /// <https://drafts.csswg.org/css-values/#viewport-relative-lengths>
    ViewportPercentage(ViewportPercentageLength),

    /// HTML5 "character width", as defined in HTML5 § 14.5.4.
    ///
    /// This cannot be specified by the user directly and is only generated by
    /// `Stylist::synthesize_rules_for_legacy_attributes()`.
    #[css(function)]
    ServoCharacterWidth(CharacterWidth),
}

impl Mul<CSSFloat> for NoCalcLength {
    type Output = NoCalcLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> NoCalcLength {
        match self {
            NoCalcLength::Absolute(v) => NoCalcLength::Absolute(v * scalar),
            NoCalcLength::FontRelative(v) => NoCalcLength::FontRelative(v * scalar),
            NoCalcLength::ViewportPercentage(v) => NoCalcLength::ViewportPercentage(v * scalar),
            NoCalcLength::ServoCharacterWidth(_) => panic!("Can't multiply ServoCharacterWidth!"),
        }
    }
}

impl NoCalcLength {
    /// Parse a given absolute or relative dimension.
    pub fn parse_dimension(context: &ParserContext, value: CSSFloat, unit: &str)
                           -> Result<NoCalcLength, ()> {
        let in_page_rule = context.rule_type.map_or(false, |rule_type| rule_type == CssRuleType::Page);
        match_ignore_ascii_case! { unit,
            "px" => Ok(NoCalcLength::Absolute(AbsoluteLength::Px(value))),
            "in" => Ok(NoCalcLength::Absolute(AbsoluteLength::In(value))),
            "cm" => Ok(NoCalcLength::Absolute(AbsoluteLength::Cm(value))),
            "mm" => Ok(NoCalcLength::Absolute(AbsoluteLength::Mm(value))),
            "q" => Ok(NoCalcLength::Absolute(AbsoluteLength::Q(value))),
            "pt" => Ok(NoCalcLength::Absolute(AbsoluteLength::Pt(value))),
            "pc" => Ok(NoCalcLength::Absolute(AbsoluteLength::Pc(value))),
            // font-relative
            "em" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Em(value))),
            "ex" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Ex(value))),
            "ch" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Ch(value))),
            "rem" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Rem(value))),
            // viewport percentages
            "vw" => {
                if in_page_rule {
                    return Err(())
                }
                Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(value)))
            },
            "vh" => {
                if in_page_rule {
                    return Err(())
                }
                Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vh(value)))
            },
            "vmin" => {
                if in_page_rule {
                    return Err(())
                }
                Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vmin(value)))
            },
            "vmax" => {
                if in_page_rule {
                    return Err(())
                }
                Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vmax(value)))
            },
            _ => Err(())
        }
    }

    #[inline]
    /// Returns a `zero` length.
    pub fn zero() -> NoCalcLength {
        NoCalcLength::Absolute(AbsoluteLength::Px(0.))
    }

    #[inline]
    /// Checks whether the length value is zero.
    pub fn is_zero(&self) -> bool {
        match *self {
            NoCalcLength::Absolute(length) => length.is_zero(),
            _ => false
        }
    }

    /// Get a px value without context.
    #[inline]
    pub fn to_computed_pixel_length_without_context(&self) -> Result<CSSFloat, ()> {
        match *self {
            NoCalcLength::Absolute(len) => Ok(len.to_px()),
            _ => Err(()),
        }
    }

    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> NoCalcLength {
        NoCalcLength::Absolute(AbsoluteLength::Px(px_value))
    }
}

/// An extension to `NoCalcLength` to parse `calc` expressions.
/// This is commonly used for the `<length>` values.
///
/// <https://drafts.csswg.org/css-values/#lengths>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum Length {
    /// The internal length type that cannot parse `calc`
    NoCalc(NoCalcLength),
    /// A calc expression.
    ///
    /// <https://drafts.csswg.org/css-values/#calc-notation>
    Calc(Box<CalcLengthOrPercentage>),
}

impl From<NoCalcLength> for Length {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        Length::NoCalc(len)
    }
}

impl Mul<CSSFloat> for Length {
    type Output = Length;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> Length {
        match self {
            Length::NoCalc(inner) => Length::NoCalc(inner * scalar),
            Length::Calc(..) => panic!("Can't multiply Calc!"),
        }
    }
}

impl Mul<CSSFloat> for FontRelativeLength {
    type Output = FontRelativeLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> FontRelativeLength {
        match self {
            FontRelativeLength::Em(v) => FontRelativeLength::Em(v * scalar),
            FontRelativeLength::Ex(v) => FontRelativeLength::Ex(v * scalar),
            FontRelativeLength::Ch(v) => FontRelativeLength::Ch(v * scalar),
            FontRelativeLength::Rem(v) => FontRelativeLength::Rem(v * scalar),
        }
    }
}

impl Mul<CSSFloat> for ViewportPercentageLength {
    type Output = ViewportPercentageLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> ViewportPercentageLength {
        match self {
            ViewportPercentageLength::Vw(v) => ViewportPercentageLength::Vw(v * scalar),
            ViewportPercentageLength::Vh(v) => ViewportPercentageLength::Vh(v * scalar),
            ViewportPercentageLength::Vmin(v) => ViewportPercentageLength::Vmin(v * scalar),
            ViewportPercentageLength::Vmax(v) => ViewportPercentageLength::Vmax(v * scalar),
        }
    }
}

impl Length {
    #[inline]
    /// Returns a `zero` length.
    pub fn zero() -> Length {
        Length::NoCalc(NoCalcLength::zero())
    }

    /// Parse a given absolute or relative dimension.
    pub fn parse_dimension(context: &ParserContext, value: CSSFloat, unit: &str)
                           -> Result<Length, ()> {
        NoCalcLength::parse_dimension(context, value, unit).map(Length::NoCalc)
    }

    #[inline]
    fn parse_internal<'i, 't>(context: &ParserContext,
                              input: &mut Parser<'i, 't>,
                              num_context: AllowedNumericType,
                              allow_quirks: AllowQuirks)
                              -> Result<Length, ParseError<'i>> {
        // FIXME: remove early returns when lifetimes are non-lexical
        {
            let location = input.current_source_location();
            let token = input.next()?;
            match *token {
                Token::Dimension { value, ref unit, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    return Length::parse_dimension(context, value, unit)
                        .map_err(|()| location.new_unexpected_token_error(token.clone()))
                }
                Token::Number { value, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    if value != 0. &&
                       !context.parsing_mode.allows_unitless_lengths() &&
                       !allow_quirks.allowed(context.quirks_mode) {
                        return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    }
                    return Ok(Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(value))))
                },
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {}
                ref token => return Err(location.new_unexpected_token_error(token.clone()))
            }
        }
        input.parse_nested_block(|input| {
            CalcNode::parse_length(context, input, num_context).map(|calc| Length::Calc(Box::new(calc)))
        })
    }

    /// Parse a non-negative length
    #[inline]
    pub fn parse_non_negative<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                      -> Result<Length, ParseError<'i>> {
        Self::parse_non_negative_quirky(context, input, AllowQuirks::No)
    }

    /// Parse a non-negative length, allowing quirks.
    #[inline]
    pub fn parse_non_negative_quirky<'i, 't>(context: &ParserContext,
                                             input: &mut Parser<'i, 't>,
                                             allow_quirks: AllowQuirks)
                                             -> Result<Length, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::NonNegative, allow_quirks)
    }

    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> Length {
        Length::NoCalc(NoCalcLength::from_px(px_value))
    }

    /// Extract inner length without a clone, replacing it with a 0 Au
    ///
    /// Use when you need to move out of a length array without cloning
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::replace(self, Length::zero())
    }
}

impl Parse for Length {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl Length {
    /// Parses a length, with quirks.
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks)
                                -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::All, allow_quirks)
    }
}

impl<T: Parse> Either<Length, T> {
    /// Parse a non-negative length
    #[inline]
    pub fn parse_non_negative_length<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                             -> Result<Self, ParseError<'i>> {
        if let Ok(v) = input.try(|input| T::parse(context, input)) {
            return Ok(Either::Second(v));
        }
        Length::parse_internal(context, input, AllowedNumericType::NonNegative, AllowQuirks::No).map(Either::First)
    }
}

/// A wrapper of Length, whose value must be >= 0.
pub type NonNegativeLength = NonNegative<Length>;

impl From<NoCalcLength> for NonNegativeLength {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        NonNegative::<Length>(Length::NoCalc(len))
    }
}

impl From<Length> for NonNegativeLength {
    #[inline]
    fn from(len: Length) -> Self {
        NonNegative::<Length>(len)
    }
}

impl<T: Parse> Parse for Either<NonNegativeLength, T> {
    #[inline]
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(v) = input.try(|input| T::parse(context, input)) {
            return Ok(Either::Second(v));
        }
        Length::parse_internal(context, input, AllowedNumericType::NonNegative, AllowQuirks::No)
            .map(NonNegative::<Length>).map(Either::First)
    }
}

impl NonNegativeLength {
    /// Returns a `zero` length.
    #[inline]
    pub fn zero() -> Self {
        Length::zero().into()
    }

    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> Self {
        Length::from_px(px_value.max(0.)).into()
    }
}

/// Either a NonNegativeLength or the `normal` keyword.
pub type NonNegativeLengthOrNormal = Either<NonNegativeLength, Normal>;

/// Either a NonNegativeLength or the `auto` keyword.
pub type NonNegativeLengthOrAuto = Either<NonNegativeLength, Auto>;

/// Either a NonNegativeLength or a NonNegativeNumber value.
pub type NonNegativeLengthOrNumber = Either<NonNegativeLength, NonNegativeNumber>;

/// A length or a percentage value.
#[allow(missing_docs)]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum LengthOrPercentage {
    Length(NoCalcLength),
    Percentage(computed::Percentage),
    Calc(Box<CalcLengthOrPercentage>),
}

impl From<Length> for LengthOrPercentage {
    fn from(len: Length) -> LengthOrPercentage {
        match len {
            Length::NoCalc(l) => LengthOrPercentage::Length(l),
            Length::Calc(l) => LengthOrPercentage::Calc(l),
        }
    }
}

impl From<NoCalcLength> for LengthOrPercentage {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        LengthOrPercentage::Length(len)
    }
}

impl From<Percentage> for LengthOrPercentage {
    #[inline]
    fn from(pc: Percentage) -> Self {
        if pc.is_calc() {
            LengthOrPercentage::Calc(Box::new(CalcLengthOrPercentage {
                percentage: Some(computed::Percentage(pc.get())),
                .. Default::default()
            }))
        } else {
            LengthOrPercentage::Percentage(computed::Percentage(pc.get()))
        }
    }
}

impl From<computed::Percentage> for LengthOrPercentage {
    #[inline]
    fn from(pc: computed::Percentage) -> Self {
        LengthOrPercentage::Percentage(pc)
    }
}

impl LengthOrPercentage {
    #[inline]
    /// Returns a `zero` length.
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(NoCalcLength::zero())
    }

    fn parse_internal<'i, 't>(context: &ParserContext,
                              input: &mut Parser<'i, 't>,
                              num_context: AllowedNumericType,
                              allow_quirks: AllowQuirks)
                              -> Result<LengthOrPercentage, ParseError<'i>>
    {
        // FIXME: remove early returns when lifetimes are non-lexical
        {
            let location = input.current_source_location();
            let token = input.next()?;
            match *token {
                Token::Dimension { value, ref unit, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    return NoCalcLength::parse_dimension(context, value, unit)
                        .map(LengthOrPercentage::Length)
                        .map_err(|()| location.new_unexpected_token_error(token.clone()))
                }
                Token::Percentage { unit_value, .. } if num_context.is_ok(context.parsing_mode, unit_value) => {
                    return Ok(LengthOrPercentage::Percentage(computed::Percentage(unit_value)))
                }
                Token::Number { value, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    if value != 0. &&
                       !context.parsing_mode.allows_unitless_lengths() &&
                       !allow_quirks.allowed(context.quirks_mode) {
                        return Err(location.new_unexpected_token_error(token.clone()))
                    } else {
                        return Ok(LengthOrPercentage::Length(NoCalcLength::from_px(value)))
                    }
                }
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {}
                _ => return Err(location.new_unexpected_token_error(token.clone()))
            }
        }

        let calc = input.parse_nested_block(|i| {
            CalcNode::parse_length_or_percentage(context, i, num_context)
        })?;
        Ok(LengthOrPercentage::Calc(Box::new(calc)))
    }

    /// Parse a non-negative length.
    #[inline]
    pub fn parse_non_negative<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                      -> Result<LengthOrPercentage, ParseError<'i>> {
        Self::parse_non_negative_quirky(context, input, AllowQuirks::No)
    }

    /// Parse a non-negative length, with quirks.
    #[inline]
    pub fn parse_non_negative_quirky<'i, 't>(context: &ParserContext,
                                             input: &mut Parser<'i, 't>,
                                             allow_quirks: AllowQuirks)
                                             -> Result<LengthOrPercentage, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::NonNegative, allow_quirks)
    }

    /// Parse a length, treating dimensionless numbers as pixels
    ///
    /// <https://www.w3.org/TR/SVG2/types.html#presentation-attribute-css-value>
    pub fn parse_numbers_are_pixels<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                            -> Result<LengthOrPercentage, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| Self::parse(context, i)) {
            return Ok(lop)
        }

        // TODO(emilio): Probably should use Number::parse_non_negative to
        // handle calc()?
        let num = input.expect_number()?;
        Ok(LengthOrPercentage::Length(NoCalcLength::Absolute(AbsoluteLength::Px(num))))
    }

    /// Parse a non-negative length, treating dimensionless numbers as pixels
    ///
    /// This is nonstandard behavior used by Firefox for SVG
    pub fn parse_numbers_are_pixels_non_negative<'i, 't>(context: &ParserContext,
                                                         input: &mut Parser<'i, 't>)
                                                         -> Result<LengthOrPercentage, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| Self::parse_non_negative(context, i)) {
            return Ok(lop)
        }

        // TODO(emilio): Probably should use Number::parse_non_negative to
        // handle calc()?
        let num = input.expect_number()?;
        if num >= 0. {
            Ok(LengthOrPercentage::Length(NoCalcLength::Absolute(AbsoluteLength::Px(num))))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }

    /// Extract value from ref without a clone, replacing it with a 0 Au
    ///
    /// Use when you need to move out of a length array without cloning
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::replace(self, LengthOrPercentage::zero())
    }
}

impl Parse for LengthOrPercentage {
    #[inline]
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl LengthOrPercentage {
    /// Parses a length or a percentage, allowing the unitless length quirk.
    /// <https://quirks.spec.whatwg.org/#the-unitless-length-quirk>
    #[inline]
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::All, allow_quirks)
    }
}

/// Either a `<length>`, a `<percentage>`, or the `auto` keyword.
#[allow(missing_docs)]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum LengthOrPercentageOrAuto {
    Length(NoCalcLength),
    Percentage(computed::Percentage),
    Auto,
    Calc(Box<CalcLengthOrPercentage>),
}

impl From<NoCalcLength> for LengthOrPercentageOrAuto {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        LengthOrPercentageOrAuto::Length(len)
    }
}

impl From<computed::Percentage> for LengthOrPercentageOrAuto {
    #[inline]
    fn from(pc: computed::Percentage) -> Self {
        LengthOrPercentageOrAuto::Percentage(pc)
    }
}

impl LengthOrPercentageOrAuto {
    fn parse_internal<'i, 't>(context: &ParserContext,
                              input: &mut Parser<'i, 't>,
                              num_context: AllowedNumericType,
                              allow_quirks: AllowQuirks)
                              -> Result<Self, ParseError<'i>> {
        // FIXME: remove early returns when lifetimes are non-lexical
        {
            let location = input.current_source_location();
            let token = input.next()?;
            match *token {
                Token::Dimension { value, ref unit, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    return NoCalcLength::parse_dimension(context, value, unit)
                        .map(LengthOrPercentageOrAuto::Length)
                        .map_err(|()| location.new_unexpected_token_error(token.clone()))
                }
                Token::Percentage { unit_value, .. } if num_context.is_ok(context.parsing_mode, unit_value) => {
                    return Ok(LengthOrPercentageOrAuto::Percentage(computed::Percentage(unit_value)))
                }
                Token::Number { value, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    if value != 0. &&
                       !context.parsing_mode.allows_unitless_lengths() &&
                       !allow_quirks.allowed(context.quirks_mode) {
                        return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    }
                    return Ok(LengthOrPercentageOrAuto::Length(
                        NoCalcLength::Absolute(AbsoluteLength::Px(value))
                    ))
                }
                Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") => {
                    return Ok(LengthOrPercentageOrAuto::Auto)
                }
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {}
                _ => return Err(location.new_unexpected_token_error(token.clone()))
            }
        }

        let calc = input.parse_nested_block(|i| {
            CalcNode::parse_length_or_percentage(context, i, num_context)
        })?;
        Ok(LengthOrPercentageOrAuto::Calc(Box::new(calc)))
    }

    /// Parse a non-negative length, percentage, or auto.
    #[inline]
    pub fn parse_non_negative<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                      -> Result<LengthOrPercentageOrAuto, ParseError<'i>> {
        Self::parse_non_negative_quirky(context, input, AllowQuirks::No)
    }

    /// Parse a non-negative length, percentage, or auto.
    #[inline]
    pub fn parse_non_negative_quirky<'i, 't>(context: &ParserContext,
                                             input: &mut Parser<'i, 't>,
                                             allow_quirks: AllowQuirks)
                                             -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::NonNegative, allow_quirks)
    }

    /// Returns the `auto` value.
    pub fn auto() -> Self {
        LengthOrPercentageOrAuto::Auto
    }

    /// Returns a value representing a `0` length.
    pub fn zero() -> Self {
        LengthOrPercentageOrAuto::Length(NoCalcLength::zero())
    }

    /// Returns a value representing `0%`.
    pub fn zero_percent() -> Self {
        LengthOrPercentageOrAuto::Percentage(computed::Percentage::zero())
    }
}

impl Parse for LengthOrPercentageOrAuto {
    #[inline]
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl LengthOrPercentageOrAuto {
    /// Parses, with quirks.
    #[inline]
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks)
                                -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::All, allow_quirks)
    }
}

/// Either a `<length>`, a `<percentage>`, or the `none` keyword.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
#[allow(missing_docs)]
pub enum LengthOrPercentageOrNone {
    Length(NoCalcLength),
    Percentage(computed::Percentage),
    Calc(Box<CalcLengthOrPercentage>),
    None,
}

impl LengthOrPercentageOrNone {
    fn parse_internal<'i, 't>(context: &ParserContext,
                              input: &mut Parser<'i, 't>,
                              num_context: AllowedNumericType,
                              allow_quirks: AllowQuirks)
                              -> Result<LengthOrPercentageOrNone, ParseError<'i>>
    {
        // FIXME: remove early returns when lifetimes are non-lexical
        {
            let location = input.current_source_location();
            let token = input.next()?;
            match *token {
                Token::Dimension { value, ref unit, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    return NoCalcLength::parse_dimension(context, value, unit)
                        .map(LengthOrPercentageOrNone::Length)
                        .map_err(|()| location.new_unexpected_token_error(token.clone()))
                }
                Token::Percentage { unit_value, .. } if num_context.is_ok(context.parsing_mode, unit_value) => {
                    return Ok(LengthOrPercentageOrNone::Percentage(computed::Percentage(unit_value)))
                }
                Token::Number { value, .. } if num_context.is_ok(context.parsing_mode, value) => {
                    if value != 0. && !context.parsing_mode.allows_unitless_lengths() &&
                       !allow_quirks.allowed(context.quirks_mode) {
                        return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    }
                    return Ok(LengthOrPercentageOrNone::Length(
                        NoCalcLength::Absolute(AbsoluteLength::Px(value))
                    ))
                }
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {}
                Token::Ident(ref value) if value.eq_ignore_ascii_case("none") => {
                    return Ok(LengthOrPercentageOrNone::None)
                }
                _ => return Err(location.new_unexpected_token_error(token.clone()))
            }
        }

        let calc = input.parse_nested_block(|i| {
            CalcNode::parse_length_or_percentage(context, i, num_context)
        })?;
        Ok(LengthOrPercentageOrNone::Calc(Box::new(calc)))
    }

    /// Parse a non-negative LengthOrPercentageOrNone.
    #[inline]
    pub fn parse_non_negative<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                      -> Result<Self, ParseError<'i>> {
        Self::parse_non_negative_quirky(context, input, AllowQuirks::No)
    }

    /// Parse a non-negative LengthOrPercentageOrNone, with quirks.
    #[inline]
    pub fn parse_non_negative_quirky<'i, 't>(context: &ParserContext,
                                             input: &mut Parser<'i, 't>,
                                             allow_quirks: AllowQuirks)
                                             -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::NonNegative, allow_quirks)
    }
}

impl Parse for LengthOrPercentageOrNone {
    #[inline]
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::All, AllowQuirks::No)
    }
}

/// A wrapper of LengthOrPercentage, whose value must be >= 0.
pub type NonNegativeLengthOrPercentage = NonNegative<LengthOrPercentage>;

impl From<NoCalcLength> for NonNegativeLengthOrPercentage {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        NonNegative::<LengthOrPercentage>(LengthOrPercentage::from(len))
    }
}

impl Parse for NonNegativeLengthOrPercentage {
    #[inline]
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        LengthOrPercentage::parse_non_negative(context, input).map(NonNegative::<LengthOrPercentage>)
    }
}

impl NonNegativeLengthOrPercentage {
    #[inline]
    /// Returns a `zero` length.
    pub fn zero() -> Self {
        NonNegative::<LengthOrPercentage>(LengthOrPercentage::zero())
    }

    /// Parses a length or a percentage, allowing the unitless length quirk.
    /// <https://quirks.spec.whatwg.org/#the-unitless-length-quirk>
    #[inline]
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks) -> Result<Self, ParseError<'i>> {
        LengthOrPercentage::parse_non_negative_quirky(context, input, allow_quirks)
            .map(NonNegative::<LengthOrPercentage>)
    }
}

/// Either a `<length>` or the `none` keyword.
pub type LengthOrNone = Either<Length, None_>;

/// Either a `<length>` or the `normal` keyword.
pub type LengthOrNormal = Either<Length, Normal>;

/// Either a `<length>` or the `auto` keyword.
pub type LengthOrAuto = Either<Length, Auto>;

/// Either a `<length>` or a `<number>`.
pub type LengthOrNumber = Either<Length, Number>;

impl LengthOrNumber {
    /// Parse a non-negative LengthOrNumber.
    pub fn parse_non_negative<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                      -> Result<Self, ParseError<'i>> {
        // We try to parse as a Number first because, for cases like
        // LengthOrNumber, we want "0" to be parsed as a plain Number rather
        // than a Length (0px); this matches the behaviour of all major browsers
        if let Ok(v) = input.try(|i| Number::parse_non_negative(context, i)) {
            return Ok(Either::Second(v))
        }

        Length::parse_non_negative(context, input).map(Either::First)
    }

    /// Returns `0`.
    #[inline]
    pub fn zero() -> Self {
        Either::Second(Number::new(0.))
    }
}

/// A value suitable for a `min-width` or `min-height` property.
/// Unlike `max-width` or `max-height` properties, a MozLength can be
/// `auto`, and cannot be `none`.
#[allow(missing_docs)]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum MozLength {
    LengthOrPercentageOrAuto(LengthOrPercentageOrAuto),
    ExtremumLength(ExtremumLength),
}

impl Parse for MozLength {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        MozLength::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl MozLength {
    /// Parses, with quirks.
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks) -> Result<Self, ParseError<'i>> {
        input.try(ExtremumLength::parse).map(MozLength::ExtremumLength)
            .or_else(|_| input.try(|i| LengthOrPercentageOrAuto::parse_non_negative_quirky(context, i, allow_quirks))
                               .map(MozLength::LengthOrPercentageOrAuto))
    }
}

/// A value suitable for a `max-width` or `max-height` property.
#[allow(missing_docs)]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum MaxLength {
    LengthOrPercentageOrNone(LengthOrPercentageOrNone),
    ExtremumLength(ExtremumLength),
}

impl Parse for MaxLength {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        MaxLength::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl MaxLength {
    /// Parses, with quirks.
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks) -> Result<Self, ParseError<'i>> {
        input.try(ExtremumLength::parse).map(MaxLength::ExtremumLength)
            .or_else(|_| input.try(|i| LengthOrPercentageOrNone::parse_non_negative_quirky(context, i, allow_quirks))
                               .map(MaxLength::LengthOrPercentageOrNone))
    }
}
