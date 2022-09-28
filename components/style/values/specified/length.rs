/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [Length values][length].
//!
//! [length]: https://drafts.csswg.org/css-values/#lengths

use super::{AllowQuirks, Number, Percentage, ToComputedValue};
use crate::font_metrics::{FontMetrics, FontMetricsOrientation};
use crate::parser::{Parse, ParserContext};
use crate::values::computed::{self, CSSPixelLength, Context};
use crate::values::generics::length as generics;
use crate::values::generics::length::{
    GenericLengthOrNumber, GenericLengthPercentageOrNormal, GenericMaxSize, GenericSize,
};
use crate::values::generics::NonNegative;
use crate::values::specified::calc::{self, CalcNode};
use crate::values::specified::NonNegativeNumber;
use crate::values::CSSFloat;
use crate::{Zero, ZeroNoPercent};
use app_units::Au;
use cssparser::{Parser, Token};
use std::cmp;
use std::ops::{Add, Mul};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{ParseError, SpecifiedValueInfo, StyleParseErrorKind};

pub use super::image::Image;
pub use super::image::{EndingShape as GradientEndingShape, Gradient};
pub use crate::values::specified::calc::CalcLengthPercentage;

/// Number of pixels per inch
pub const PX_PER_IN: CSSFloat = 96.;
/// Number of pixels per centimeter
pub const PX_PER_CM: CSSFloat = PX_PER_IN / 2.54;
/// Number of pixels per millimeter
pub const PX_PER_MM: CSSFloat = PX_PER_IN / 25.4;
/// Number of pixels per quarter
pub const PX_PER_Q: CSSFloat = PX_PER_MM / 4.;
/// Number of pixels per point
pub const PX_PER_PT: CSSFloat = PX_PER_IN / 72.;
/// Number of pixels per pica
pub const PX_PER_PC: CSSFloat = PX_PER_PT * 12.;

/// A font relative length.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToShmem)]
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
    /// A "cap" value: https://drafts.csswg.org/css-values/#cap
    #[css(dimension)]
    Cap(CSSFloat),
    /// An "ic" value: https://drafts.csswg.org/css-values/#ic
    #[css(dimension)]
    Ic(CSSFloat),
    /// A "rem" value: https://drafts.csswg.org/css-values/#rem
    #[css(dimension)]
    Rem(CSSFloat),
}

/// A source to resolve font-relative units against
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontBaseSize {
    /// Use the font-size of the current element.
    CurrentStyle,
    /// Use the inherited font-size.
    InheritedStyle,
}

impl FontBaseSize {
    /// Calculate the actual size for a given context
    pub fn resolve(&self, context: &Context) -> computed::Length {
        match *self {
            FontBaseSize::CurrentStyle => context.style().get_font().clone_font_size().size(),
            FontBaseSize::InheritedStyle => {
                context.style().get_parent_font().clone_font_size().size()
            },
        }
    }
}

impl FontRelativeLength {
    /// Return true if this is a zero value.
    fn is_zero(&self) -> bool {
        self.unitless_value() == 0.
    }

    /// Return the unitless, raw value.
    fn unitless_value(&self) -> CSSFloat {
        match *self {
            FontRelativeLength::Em(v) |
            FontRelativeLength::Ex(v) |
            FontRelativeLength::Ch(v) |
            FontRelativeLength::Cap(v) |
            FontRelativeLength::Ic(v) |
            FontRelativeLength::Rem(v) => v,
        }
    }

    fn is_negative(&self) -> bool {
        self.unitless_value() < 0.
    }

    fn try_sum(&self, other: &Self) -> Result<Self, ()> {
        use self::FontRelativeLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        Ok(match (self, other) {
            (&Em(one), &Em(other)) => Em(one + other),
            (&Ex(one), &Ex(other)) => Ex(one + other),
            (&Ch(one), &Ch(other)) => Ch(one + other),
            (&Cap(one), &Cap(other)) => Cap(one + other),
            (&Ic(one), &Ic(other)) => Ic(one + other),
            (&Rem(one), &Rem(other)) => Rem(one + other),
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Em(..) | Ex(..) | Ch(..) | Cap(..) | Ic(..) | Rem(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in try_sum()")
            },
        })
    }

    /// Computes the font-relative length.
    pub fn to_computed_value(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> computed::Length {
        let (reference_size, length) = self.reference_font_size_and_length(context, base_size);
        (reference_size * length).normalized()
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
    ) -> (computed::Length, CSSFloat) {
        fn query_font_metrics(
            context: &Context,
            base_size: FontBaseSize,
            orientation: FontMetricsOrientation,
        ) -> FontMetrics {
            let retrieve_math_scales = false;
            context.query_font_metrics(base_size, orientation, retrieve_math_scales)
        }

        let reference_font_size = base_size.resolve(context);
        match *self {
            FontRelativeLength::Em(length) => {
                if context.for_non_inherited_property.is_some() {
                    if base_size == FontBaseSize::CurrentStyle {
                        context
                            .rule_cache_conditions
                            .borrow_mut()
                            .set_font_size_dependency(reference_font_size.into());
                    }
                }

                (reference_font_size, length)
            },
            FontRelativeLength::Ex(length) => {
                // The x-height is an intrinsically horizontal metric.
                let metrics =
                    query_font_metrics(context, base_size, FontMetricsOrientation::Horizontal);
                let reference_size = metrics.x_height.unwrap_or_else(|| {
                    // https://drafts.csswg.org/css-values/#ex
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the x-height, a value of 0.5em must be
                    //     assumed.
                    //
                    reference_font_size * 0.5
                });
                (reference_size, length)
            },
            FontRelativeLength::Ch(length) => {
                // https://drafts.csswg.org/css-values/#ch:
                //
                //     Equal to the used advance measure of the “0” (ZERO,
                //     U+0030) glyph in the font used to render it. (The advance
                //     measure of a glyph is its advance width or height,
                //     whichever is in the inline axis of the element.)
                //
                let metrics = query_font_metrics(
                    context,
                    base_size,
                    FontMetricsOrientation::MatchContextPreferHorizontal,
                );
                let reference_size = metrics.zero_advance_measure.unwrap_or_else(|| {
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
                    let wm = context.style().writing_mode;
                    if wm.is_vertical() && wm.is_upright() {
                        reference_font_size
                    } else {
                        reference_font_size * 0.5
                    }
                });
                (reference_size, length)
            },
            FontRelativeLength::Cap(length) => {
                let metrics =
                    query_font_metrics(context, base_size, FontMetricsOrientation::Horizontal);
                let reference_size = metrics.cap_height.unwrap_or_else(|| {
                    // https://drafts.csswg.org/css-values/#cap
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the cap-height, the font’s ascent must be used.
                    //
                    metrics.ascent
                });
                (reference_size, length)
            },
            FontRelativeLength::Ic(length) => {
                let metrics = query_font_metrics(
                    context,
                    base_size,
                    FontMetricsOrientation::MatchContextPreferVertical,
                );
                let reference_size = metrics.ic_width.unwrap_or_else(|| {
                    // https://drafts.csswg.org/css-values/#ic
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the ideographic advance measure, it must be
                    //     assumed to be 1em.
                    //
                    reference_font_size
                });
                (reference_size, length)
            },
            FontRelativeLength::Rem(length) => {
                // https://drafts.csswg.org/css-values/#rem:
                //
                //     When specified on the font-size property of the root
                //     element, the rem units refer to the property's initial
                //     value.
                //
                let reference_size = if context.builder.is_root_element || context.in_media_query {
                    reference_font_size
                } else {
                    context.device().root_font_size()
                };
                (reference_size, length)
            },
        }
    }
}

/// https://drafts.csswg.org/css-values/#viewport-variants
pub enum ViewportVariant {
    /// https://drafts.csswg.org/css-values/#ua-default-viewport-size
    UADefault,
    /// https://drafts.csswg.org/css-values/#small-viewport-percentage-units
    Small,
    /// https://drafts.csswg.org/css-values/#large-viewport-percentage-units
    Large,
    /// https://drafts.csswg.org/css-values/#dynamic-viewport-percentage-units
    Dynamic,
}

/// https://drafts.csswg.org/css-values/#viewport-relative-units
#[derive(PartialEq)]
enum ViewportUnit {
    /// *vw units.
    Vw,
    /// *vh units.
    Vh,
    /// *vmin units.
    Vmin,
    /// *vmax units.
    Vmax,
    /// *vb units.
    Vb,
    /// *vi units.
    Vi,
}

/// A viewport-relative length.
///
/// <https://drafts.csswg.org/css-values/#viewport-relative-lengths>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToShmem)]
pub enum ViewportPercentageLength {
    /// <https://drafts.csswg.org/css-values/#valdef-length-vw>
    #[css(dimension)]
    Vw(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-svw>
    #[css(dimension)]
    Svw(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-lvw>
    #[css(dimension)]
    Lvw(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-dvw>
    #[css(dimension)]
    Dvw(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-vh>
    #[css(dimension)]
    Vh(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-svh>
    #[css(dimension)]
    Svh(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-lvh>
    #[css(dimension)]
    Lvh(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-dvh>
    #[css(dimension)]
    Dvh(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-vmin>
    #[css(dimension)]
    Vmin(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-svmin>
    #[css(dimension)]
    Svmin(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-lvmin>
    #[css(dimension)]
    Lvmin(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-dvmin>
    #[css(dimension)]
    Dvmin(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-vmax>
    #[css(dimension)]
    Vmax(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-svmax>
    #[css(dimension)]
    Svmax(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-lvmax>
    #[css(dimension)]
    Lvmax(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-dvmax>
    #[css(dimension)]
    Dvmax(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-vb>
    #[css(dimension)]
    Vb(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-svb>
    #[css(dimension)]
    Svb(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-lvb>
    #[css(dimension)]
    Lvb(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-dvb>
    #[css(dimension)]
    Dvb(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-vi>
    #[css(dimension)]
    Vi(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-svi>
    #[css(dimension)]
    Svi(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-lvi>
    #[css(dimension)]
    Lvi(CSSFloat),
    /// <https://drafts.csswg.org/css-values/#valdef-length-dvi>
    #[css(dimension)]
    Dvi(CSSFloat),
}

impl ViewportPercentageLength {
    /// Return true if this is a zero value.
    fn is_zero(&self) -> bool {
        self.unitless_value() == 0.
    }

    fn is_negative(&self) -> bool {
        self.unitless_value() < 0.
    }

    /// Return the unitless, raw value.
    fn unitless_value(&self) -> CSSFloat {
        self.unpack().2
    }

    fn unpack(&self) -> (ViewportVariant, ViewportUnit, CSSFloat) {
        match *self {
            ViewportPercentageLength::Vw(v) => (ViewportVariant::UADefault, ViewportUnit::Vw, v),
            ViewportPercentageLength::Svw(v) => (ViewportVariant::Small, ViewportUnit::Vw, v),
            ViewportPercentageLength::Lvw(v) => (ViewportVariant::Large, ViewportUnit::Vw, v),
            ViewportPercentageLength::Dvw(v) => (ViewportVariant::Dynamic, ViewportUnit::Vw, v),
            ViewportPercentageLength::Vh(v) => (ViewportVariant::UADefault, ViewportUnit::Vh, v),
            ViewportPercentageLength::Svh(v) => (ViewportVariant::Small, ViewportUnit::Vh, v),
            ViewportPercentageLength::Lvh(v) => (ViewportVariant::Large, ViewportUnit::Vh, v),
            ViewportPercentageLength::Dvh(v) => (ViewportVariant::Dynamic, ViewportUnit::Vh, v),
            ViewportPercentageLength::Vmin(v) => {
                (ViewportVariant::UADefault, ViewportUnit::Vmin, v)
            },
            ViewportPercentageLength::Svmin(v) => (ViewportVariant::Small, ViewportUnit::Vmin, v),
            ViewportPercentageLength::Lvmin(v) => (ViewportVariant::Large, ViewportUnit::Vmin, v),
            ViewportPercentageLength::Dvmin(v) => (ViewportVariant::Dynamic, ViewportUnit::Vmin, v),
            ViewportPercentageLength::Vmax(v) => {
                (ViewportVariant::UADefault, ViewportUnit::Vmax, v)
            },
            ViewportPercentageLength::Svmax(v) => (ViewportVariant::Small, ViewportUnit::Vmax, v),
            ViewportPercentageLength::Lvmax(v) => (ViewportVariant::Large, ViewportUnit::Vmax, v),
            ViewportPercentageLength::Dvmax(v) => (ViewportVariant::Dynamic, ViewportUnit::Vmax, v),
            ViewportPercentageLength::Vb(v) => (ViewportVariant::UADefault, ViewportUnit::Vb, v),
            ViewportPercentageLength::Svb(v) => (ViewportVariant::Small, ViewportUnit::Vb, v),
            ViewportPercentageLength::Lvb(v) => (ViewportVariant::Large, ViewportUnit::Vb, v),
            ViewportPercentageLength::Dvb(v) => (ViewportVariant::Dynamic, ViewportUnit::Vb, v),
            ViewportPercentageLength::Vi(v) => (ViewportVariant::UADefault, ViewportUnit::Vi, v),
            ViewportPercentageLength::Svi(v) => (ViewportVariant::Small, ViewportUnit::Vi, v),
            ViewportPercentageLength::Lvi(v) => (ViewportVariant::Large, ViewportUnit::Vi, v),
            ViewportPercentageLength::Dvi(v) => (ViewportVariant::Dynamic, ViewportUnit::Vi, v),
        }
    }
    fn try_sum(&self, other: &Self) -> Result<Self, ()> {
        use self::ViewportPercentageLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        Ok(match (self, other) {
            (&Vw(one), &Vw(other)) => Vw(one + other),
            (&Svw(one), &Svw(other)) => Svw(one + other),
            (&Lvw(one), &Lvw(other)) => Lvw(one + other),
            (&Dvw(one), &Dvw(other)) => Dvw(one + other),
            (&Vh(one), &Vh(other)) => Vh(one + other),
            (&Svh(one), &Svh(other)) => Svh(one + other),
            (&Lvh(one), &Lvh(other)) => Lvh(one + other),
            (&Dvh(one), &Dvh(other)) => Dvh(one + other),
            (&Vmin(one), &Vmin(other)) => Vmin(one + other),
            (&Svmin(one), &Svmin(other)) => Svmin(one + other),
            (&Lvmin(one), &Lvmin(other)) => Lvmin(one + other),
            (&Dvmin(one), &Dvmin(other)) => Dvmin(one + other),
            (&Vmax(one), &Vmax(other)) => Vmax(one + other),
            (&Svmax(one), &Svmax(other)) => Svmax(one + other),
            (&Lvmax(one), &Lvmax(other)) => Lvmax(one + other),
            (&Dvmax(one), &Dvmax(other)) => Dvmax(one + other),
            (&Vb(one), &Vb(other)) => Vb(one + other),
            (&Svb(one), &Svb(other)) => Svb(one + other),
            (&Lvb(one), &Lvb(other)) => Lvb(one + other),
            (&Dvb(one), &Dvb(other)) => Dvb(one + other),
            (&Vi(one), &Vi(other)) => Vi(one + other),
            (&Svi(one), &Svi(other)) => Svi(one + other),
            (&Lvi(one), &Lvi(other)) => Lvi(one + other),
            (&Dvi(one), &Dvi(other)) => Dvi(one + other),
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Vw(..) | Svw(..) | Lvw(..) | Dvw(..) | Vh(..) | Svh(..) | Lvh(..) |
                    Dvh(..) | Vmin(..) | Svmin(..) | Lvmin(..) | Dvmin(..) | Vmax(..) |
                    Svmax(..) | Lvmax(..) | Dvmax(..) | Vb(..) | Svb(..) | Lvb(..) | Dvb(..) |
                    Vi(..) | Svi(..) | Lvi(..) | Dvi(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in try_sum()")
            },
        })
    }

    /// Computes the given viewport-relative length for the given viewport size.
    pub fn to_computed_value(&self, context: &Context) -> CSSPixelLength {
        let (variant, unit, factor) = self.unpack();
        let size = context.viewport_size_for_viewport_unit_resolution(variant);
        let length = match unit {
            ViewportUnit::Vw => size.width,
            ViewportUnit::Vh => size.height,
            ViewportUnit::Vmin => cmp::min(size.width, size.height),
            ViewportUnit::Vmax => cmp::max(size.width, size.height),
            ViewportUnit::Vi | ViewportUnit::Vb => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if (unit == ViewportUnit::Vb) == context.style().writing_mode.is_vertical() {
                    size.width
                } else {
                    size.height
                }
            },
        };

        // FIXME: Bug 1396535, we need to fix the extremely small viewport length for transform.
        // See bug 989802. We truncate so that adding multiple viewport units
        // that add up to 100 does not overflow due to rounding differences
        let trunc_scaled = ((length.0 as f64) * factor as f64 / 100.).trunc();
        Au::from_f64_au(trunc_scaled).into()
    }
}

/// HTML5 "character width", as defined in HTML5 § 14.5.4.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToShmem)]
pub struct CharacterWidth(pub i32);

impl CharacterWidth {
    /// Computes the given character width.
    pub fn to_computed_value(&self, reference_font_size: computed::Length) -> computed::Length {
        // This applies the *converting a character width to pixels* algorithm
        // as specified in HTML5 § 14.5.4.
        //
        // TODO(pcwalton): Find these from the font.
        let average_advance = reference_font_size * 0.5;
        let max_advance = reference_font_size;
        average_advance * (self.0 as CSSFloat - 1.0) + max_advance
    }
}

/// Represents an absolute length with its unit
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToShmem)]
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
    /// Return the unitless, raw value.
    fn unitless_value(&self) -> CSSFloat {
        match *self {
            AbsoluteLength::Px(v) |
            AbsoluteLength::In(v) |
            AbsoluteLength::Cm(v) |
            AbsoluteLength::Mm(v) |
            AbsoluteLength::Q(v) |
            AbsoluteLength::Pt(v) |
            AbsoluteLength::Pc(v) => v,
        }
    }

    fn is_zero(&self) -> bool {
        self.unitless_value() == 0.
    }

    fn is_negative(&self) -> bool {
        self.unitless_value() < 0.
    }

    /// Convert this into a pixel value.
    #[inline]
    pub fn to_px(&self) -> CSSFloat {
        use std::f32;

        let pixel = match *self {
            AbsoluteLength::Px(value) => value,
            AbsoluteLength::In(value) => value * PX_PER_IN,
            AbsoluteLength::Cm(value) => value * PX_PER_CM,
            AbsoluteLength::Mm(value) => value * PX_PER_MM,
            AbsoluteLength::Q(value) => value * PX_PER_Q,
            AbsoluteLength::Pt(value) => value * PX_PER_PT,
            AbsoluteLength::Pc(value) => value * PX_PER_PC,
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

impl PartialOrd for AbsoluteLength {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.to_px().partial_cmp(&other.to_px())
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
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToShmem)]
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
    /// Return the unitless, raw value.
    pub fn unitless_value(&self) -> CSSFloat {
        match *self {
            NoCalcLength::Absolute(v) => v.unitless_value(),
            NoCalcLength::FontRelative(v) => v.unitless_value(),
            NoCalcLength::ViewportPercentage(v) => v.unitless_value(),
            NoCalcLength::ServoCharacterWidth(c) => c.0 as f32,
        }
    }

    /// Returns whether the value of this length without unit is less than zero.
    pub fn is_negative(&self) -> bool {
        match *self {
            NoCalcLength::Absolute(v) => v.is_negative(),
            NoCalcLength::FontRelative(v) => v.is_negative(),
            NoCalcLength::ViewportPercentage(v) => v.is_negative(),
            NoCalcLength::ServoCharacterWidth(c) => c.0 < 0,
        }
    }

    /// Whether text-only zoom should be applied to this length.
    ///
    /// Generally, font-dependent/relative units don't get text-only-zoomed,
    /// because the font they're relative to should be zoomed already.
    pub fn should_zoom_text(&self) -> bool {
        match *self {
            Self::Absolute(..) | Self::ViewportPercentage(..) => true,
            Self::ServoCharacterWidth(..) | Self::FontRelative(..) => false,
        }
    }

    /// Parse a given absolute or relative dimension.
    pub fn parse_dimension(
        context: &ParserContext,
        value: CSSFloat,
        unit: &str,
    ) -> Result<Self, ()> {
        Ok(match_ignore_ascii_case! { unit,
            "px" => NoCalcLength::Absolute(AbsoluteLength::Px(value)),
            "in" => NoCalcLength::Absolute(AbsoluteLength::In(value)),
            "cm" => NoCalcLength::Absolute(AbsoluteLength::Cm(value)),
            "mm" => NoCalcLength::Absolute(AbsoluteLength::Mm(value)),
            "q" => NoCalcLength::Absolute(AbsoluteLength::Q(value)),
            "pt" => NoCalcLength::Absolute(AbsoluteLength::Pt(value)),
            "pc" => NoCalcLength::Absolute(AbsoluteLength::Pc(value)),
            // font-relative
            "em" => NoCalcLength::FontRelative(FontRelativeLength::Em(value)),
            "ex" => NoCalcLength::FontRelative(FontRelativeLength::Ex(value)),
            "ch" => NoCalcLength::FontRelative(FontRelativeLength::Ch(value)),
            "cap" => NoCalcLength::FontRelative(FontRelativeLength::Cap(value)),
            "ic" => NoCalcLength::FontRelative(FontRelativeLength::Ic(value)),
            "rem" => NoCalcLength::FontRelative(FontRelativeLength::Rem(value)),
            // viewport percentages
            "vw" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(value))
            },
            "svw" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Svw(value))
            },
            "lvw" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Lvw(value))
            },
            "dvw" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Dvw(value))
            },
            "vh" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vh(value))
            },
            "svh" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Svh(value))
            },
            "lvh" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Lvh(value))
            },
            "dvh" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Dvh(value))
            },
            "vmin" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vmin(value))
            },
            "svmin" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Svmin(value))
            },
            "lvmin" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Lvmin(value))
            },
            "dvmin" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Dvmin(value))
            },
            "vmax" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vmax(value))
            },
            "svmax" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Svmax(value))
            },
            "lvmax" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Lvmax(value))
            },
            "dvmax" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Dvmax(value))
            },
            "vb" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vb(value))
            },
            "svb" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Svb(value))
            },
            "lvb" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Lvb(value))
            },
            "dvb" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Dvb(value))
            },
            "vi" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vi(value))
            },
            "svi" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Svi(value))
            },
            "lvi" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Lvi(value))
            },
            "dvi" if !context.in_page_rule() => {
                NoCalcLength::ViewportPercentage(ViewportPercentageLength::Dvi(value))
            },
            _ => return Err(()),
        })
    }

    /// Try to sume two lengths if compatible into the left hand side.
    pub(crate) fn try_sum(&self, other: &Self) -> Result<Self, ()> {
        use self::NoCalcLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        Ok(match (self, other) {
            (&Absolute(ref one), &Absolute(ref other)) => Absolute(*one + *other),
            (&FontRelative(ref one), &FontRelative(ref other)) => FontRelative(one.try_sum(other)?),
            (&ViewportPercentage(ref one), &ViewportPercentage(ref other)) => {
                ViewportPercentage(one.try_sum(other)?)
            },
            (&ServoCharacterWidth(ref one), &ServoCharacterWidth(ref other)) => {
                ServoCharacterWidth(CharacterWidth(one.0 + other.0))
            },
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Absolute(..) |
                    FontRelative(..) |
                    ViewportPercentage(..) |
                    ServoCharacterWidth(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in try_sum()")
            },
        })
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

impl SpecifiedValueInfo for NoCalcLength {}

impl PartialOrd for NoCalcLength {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use self::NoCalcLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
        }

        match (self, other) {
            (&Absolute(ref one), &Absolute(ref other)) => one.to_px().partial_cmp(&other.to_px()),
            (&FontRelative(ref one), &FontRelative(ref other)) => one.partial_cmp(other),
            (&ViewportPercentage(ref one), &ViewportPercentage(ref other)) => {
                one.partial_cmp(other)
            },
            (&ServoCharacterWidth(ref one), &ServoCharacterWidth(ref other)) => {
                one.0.partial_cmp(&other.0)
            },
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Absolute(..) |
                    FontRelative(..) |
                    ViewportPercentage(..) |
                    ServoCharacterWidth(..) => {},
                }
                debug_unreachable!("Forgot an arm in partial_cmp?")
            },
        }
    }
}

impl Zero for NoCalcLength {
    fn zero() -> Self {
        NoCalcLength::Absolute(AbsoluteLength::Px(0.))
    }

    fn is_zero(&self) -> bool {
        match *self {
            NoCalcLength::Absolute(v) => v.is_zero(),
            NoCalcLength::FontRelative(v) => v.is_zero(),
            NoCalcLength::ViewportPercentage(v) => v.is_zero(),
            NoCalcLength::ServoCharacterWidth(v) => v.0 == 0,
        }
    }
}

/// An extension to `NoCalcLength` to parse `calc` expressions.
/// This is commonly used for the `<length>` values.
///
/// <https://drafts.csswg.org/css-values/#lengths>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum Length {
    /// The internal length type that cannot parse `calc`
    NoCalc(NoCalcLength),
    /// A calc expression.
    ///
    /// <https://drafts.csswg.org/css-values/#calc-notation>
    Calc(Box<CalcLengthPercentage>),
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

impl PartialOrd for FontRelativeLength {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use self::FontRelativeLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
        }

        match (self, other) {
            (&Em(ref one), &Em(ref other)) => one.partial_cmp(other),
            (&Ex(ref one), &Ex(ref other)) => one.partial_cmp(other),
            (&Ch(ref one), &Ch(ref other)) => one.partial_cmp(other),
            (&Cap(ref one), &Cap(ref other)) => one.partial_cmp(other),
            (&Ic(ref one), &Ic(ref other)) => one.partial_cmp(other),
            (&Rem(ref one), &Rem(ref other)) => one.partial_cmp(other),
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Em(..) | Ex(..) | Ch(..) | Cap(..) | Ic(..) | Rem(..) => {},
                }
                debug_unreachable!("Forgot an arm in partial_cmp?")
            },
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
            FontRelativeLength::Cap(v) => FontRelativeLength::Cap(v * scalar),
            FontRelativeLength::Ic(v) => FontRelativeLength::Ic(v * scalar),
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
            ViewportPercentageLength::Svw(v) => ViewportPercentageLength::Svw(v * scalar),
            ViewportPercentageLength::Lvw(v) => ViewportPercentageLength::Lvw(v * scalar),
            ViewportPercentageLength::Dvw(v) => ViewportPercentageLength::Dvw(v * scalar),
            ViewportPercentageLength::Vh(v) => ViewportPercentageLength::Vh(v * scalar),
            ViewportPercentageLength::Svh(v) => ViewportPercentageLength::Svh(v * scalar),
            ViewportPercentageLength::Lvh(v) => ViewportPercentageLength::Lvh(v * scalar),
            ViewportPercentageLength::Dvh(v) => ViewportPercentageLength::Dvh(v * scalar),
            ViewportPercentageLength::Vmin(v) => ViewportPercentageLength::Vmin(v * scalar),
            ViewportPercentageLength::Svmin(v) => ViewportPercentageLength::Svmin(v * scalar),
            ViewportPercentageLength::Lvmin(v) => ViewportPercentageLength::Lvmin(v * scalar),
            ViewportPercentageLength::Dvmin(v) => ViewportPercentageLength::Dvmin(v * scalar),
            ViewportPercentageLength::Vmax(v) => ViewportPercentageLength::Vmax(v * scalar),
            ViewportPercentageLength::Svmax(v) => ViewportPercentageLength::Svmax(v * scalar),
            ViewportPercentageLength::Lvmax(v) => ViewportPercentageLength::Lvmax(v * scalar),
            ViewportPercentageLength::Dvmax(v) => ViewportPercentageLength::Dvmax(v * scalar),
            ViewportPercentageLength::Vb(v) => ViewportPercentageLength::Vb(v * scalar),
            ViewportPercentageLength::Svb(v) => ViewportPercentageLength::Svb(v * scalar),
            ViewportPercentageLength::Lvb(v) => ViewportPercentageLength::Lvb(v * scalar),
            ViewportPercentageLength::Dvb(v) => ViewportPercentageLength::Dvb(v * scalar),
            ViewportPercentageLength::Vi(v) => ViewportPercentageLength::Vi(v * scalar),
            ViewportPercentageLength::Svi(v) => ViewportPercentageLength::Svi(v * scalar),
            ViewportPercentageLength::Lvi(v) => ViewportPercentageLength::Lvi(v * scalar),
            ViewportPercentageLength::Dvi(v) => ViewportPercentageLength::Dvi(v * scalar),
        }
    }
}

impl PartialOrd for ViewportPercentageLength {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use self::ViewportPercentageLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
        }

        match (self, other) {
            (&Vw(ref one), &Vw(ref other)) => one.partial_cmp(other),
            (&Svw(ref one), &Svw(ref other)) => one.partial_cmp(other),
            (&Lvw(ref one), &Lvw(ref other)) => one.partial_cmp(other),
            (&Dvw(ref one), &Dvw(ref other)) => one.partial_cmp(other),
            (&Vh(ref one), &Vh(ref other)) => one.partial_cmp(other),
            (&Svh(ref one), &Svh(ref other)) => one.partial_cmp(other),
            (&Lvh(ref one), &Lvh(ref other)) => one.partial_cmp(other),
            (&Dvh(ref one), &Dvh(ref other)) => one.partial_cmp(other),
            (&Vmin(ref one), &Vmin(ref other)) => one.partial_cmp(other),
            (&Svmin(ref one), &Svmin(ref other)) => one.partial_cmp(other),
            (&Lvmin(ref one), &Lvmin(ref other)) => one.partial_cmp(other),
            (&Dvmin(ref one), &Dvmin(ref other)) => one.partial_cmp(other),
            (&Vmax(ref one), &Vmax(ref other)) => one.partial_cmp(other),
            (&Svmax(ref one), &Svmax(ref other)) => one.partial_cmp(other),
            (&Lvmax(ref one), &Lvmax(ref other)) => one.partial_cmp(other),
            (&Dvmax(ref one), &Dvmax(ref other)) => one.partial_cmp(other),
            (&Vb(ref one), &Vb(ref other)) => one.partial_cmp(other),
            (&Svb(ref one), &Svb(ref other)) => one.partial_cmp(other),
            (&Lvb(ref one), &Lvb(ref other)) => one.partial_cmp(other),
            (&Dvb(ref one), &Dvb(ref other)) => one.partial_cmp(other),
            (&Vi(ref one), &Vi(ref other)) => one.partial_cmp(other),
            (&Svi(ref one), &Svi(ref other)) => one.partial_cmp(other),
            (&Lvi(ref one), &Lvi(ref other)) => one.partial_cmp(other),
            (&Dvi(ref one), &Dvi(ref other)) => one.partial_cmp(other),
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Vw(..) | Svw(..) | Lvw(..) | Dvw(..) | Vh(..) | Svh(..) | Lvh(..) |
                    Dvh(..) | Vmin(..) | Svmin(..) | Lvmin(..) | Dvmin(..) | Vmax(..) |
                    Svmax(..) | Lvmax(..) | Dvmax(..) | Vb(..) | Svb(..) | Lvb(..) | Dvb(..) |
                    Vi(..) | Svi(..) | Lvi(..) | Dvi(..) => {},
                }
                debug_unreachable!("Forgot an arm in partial_cmp?")
            },
        }
    }
}

impl Length {
    #[inline]
    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        num_context: AllowedNumericType,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let token = input.next()?;
        match *token {
            Token::Dimension {
                value, ref unit, ..
            } if num_context.is_ok(context.parsing_mode, value) => {
                NoCalcLength::parse_dimension(context, value, unit)
                    .map(Length::NoCalc)
                    .map_err(|()| location.new_unexpected_token_error(token.clone()))
            },
            Token::Number { value, .. } if num_context.is_ok(context.parsing_mode, value) => {
                if value != 0. &&
                    !context.parsing_mode.allows_unitless_lengths() &&
                    !allow_quirks.allowed(context.quirks_mode)
                {
                    return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                Ok(Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(
                    value,
                ))))
            },
            Token::Function(ref name) => {
                let function = CalcNode::math_function(name, location)?;
                let calc = CalcNode::parse_length(context, input, num_context, function)?;
                Ok(Length::Calc(Box::new(calc)))
            },
            ref token => return Err(location.new_unexpected_token_error(token.clone())),
        }
    }

    /// Parse a non-negative length
    #[inline]
    pub fn parse_non_negative<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_non_negative_quirky(context, input, AllowQuirks::No)
    }

    /// Parse a non-negative length, allowing quirks.
    #[inline]
    pub fn parse_non_negative_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(
            context,
            input,
            AllowedNumericType::NonNegative,
            allow_quirks,
        )
    }

    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> Length {
        Length::NoCalc(NoCalcLength::from_px(px_value))
    }
}

impl Parse for Length {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl Zero for Length {
    fn zero() -> Self {
        Length::NoCalc(NoCalcLength::zero())
    }

    fn is_zero(&self) -> bool {
        // FIXME(emilio): Seems a bit weird to treat calc() unconditionally as
        // non-zero here?
        match *self {
            Length::NoCalc(ref l) => l.is_zero(),
            Length::Calc(..) => false,
        }
    }
}

impl Length {
    /// Parses a length, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::All, allow_quirks)
    }
}

/// A wrapper of Length, whose value must be >= 0.
pub type NonNegativeLength = NonNegative<Length>;

impl Parse for NonNegativeLength {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(NonNegative(Length::parse_non_negative(context, input)?))
    }
}

impl From<NoCalcLength> for NonNegativeLength {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        NonNegative(Length::NoCalc(len))
    }
}

impl From<Length> for NonNegativeLength {
    #[inline]
    fn from(len: Length) -> Self {
        NonNegative(len)
    }
}

impl NonNegativeLength {
    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> Self {
        Length::from_px(px_value.max(0.)).into()
    }

    /// Parses a non-negative length, optionally with quirks.
    #[inline]
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Ok(NonNegative(Length::parse_non_negative_quirky(
            context,
            input,
            allow_quirks,
        )?))
    }
}

/// A `<length-percentage>` value. This can be either a `<length>`, a
/// `<percentage>`, or a combination of both via `calc()`.
///
/// https://drafts.csswg.org/css-values-4/#typedef-length-percentage
#[allow(missing_docs)]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum LengthPercentage {
    Length(NoCalcLength),
    Percentage(computed::Percentage),
    Calc(Box<CalcLengthPercentage>),
}

impl From<Length> for LengthPercentage {
    fn from(len: Length) -> LengthPercentage {
        match len {
            Length::NoCalc(l) => LengthPercentage::Length(l),
            Length::Calc(l) => LengthPercentage::Calc(l),
        }
    }
}

impl From<NoCalcLength> for LengthPercentage {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        LengthPercentage::Length(len)
    }
}

impl From<Percentage> for LengthPercentage {
    #[inline]
    fn from(pc: Percentage) -> Self {
        if let Some(clamping_mode) = pc.calc_clamping_mode() {
            LengthPercentage::Calc(Box::new(CalcLengthPercentage {
                clamping_mode,
                node: CalcNode::Leaf(calc::Leaf::Percentage(pc.get())),
            }))
        } else {
            LengthPercentage::Percentage(computed::Percentage(pc.get()))
        }
    }
}

impl From<computed::Percentage> for LengthPercentage {
    #[inline]
    fn from(pc: computed::Percentage) -> Self {
        LengthPercentage::Percentage(pc)
    }
}

impl Parse for LengthPercentage {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl LengthPercentage {
    #[inline]
    /// Returns a `0%` value.
    pub fn zero_percent() -> LengthPercentage {
        LengthPercentage::Percentage(computed::Percentage::zero())
    }

    #[inline]
    /// Returns a `100%` value.
    pub fn hundred_percent() -> LengthPercentage {
        LengthPercentage::Percentage(computed::Percentage::hundred())
    }

    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        num_context: AllowedNumericType,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let token = input.next()?;
        match *token {
            Token::Dimension {
                value, ref unit, ..
            } if num_context.is_ok(context.parsing_mode, value) => {
                return NoCalcLength::parse_dimension(context, value, unit)
                    .map(LengthPercentage::Length)
                    .map_err(|()| location.new_unexpected_token_error(token.clone()));
            },
            Token::Percentage { unit_value, .. }
                if num_context.is_ok(context.parsing_mode, unit_value) =>
            {
                return Ok(LengthPercentage::Percentage(computed::Percentage(
                    unit_value,
                )));
            },
            Token::Number { value, .. } if num_context.is_ok(context.parsing_mode, value) => {
                if value != 0. &&
                    !context.parsing_mode.allows_unitless_lengths() &&
                    !allow_quirks.allowed(context.quirks_mode)
                {
                    return Err(location.new_unexpected_token_error(token.clone()));
                } else {
                    return Ok(LengthPercentage::Length(NoCalcLength::from_px(value)));
                }
            },
            Token::Function(ref name) => {
                let function = CalcNode::math_function(name, location)?;
                let calc =
                    CalcNode::parse_length_or_percentage(context, input, num_context, function)?;
                Ok(LengthPercentage::Calc(Box::new(calc)))
            },
            _ => return Err(location.new_unexpected_token_error(token.clone())),
        }
    }

    /// Parses allowing the unitless length quirk.
    /// <https://quirks.spec.whatwg.org/#the-unitless-length-quirk>
    #[inline]
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowedNumericType::All, allow_quirks)
    }

    /// Parse a non-negative length.
    ///
    /// FIXME(emilio): This should be not public and we should use
    /// NonNegativeLengthPercentage instead.
    #[inline]
    pub fn parse_non_negative<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_non_negative_quirky(context, input, AllowQuirks::No)
    }

    /// Parse a non-negative length, with quirks.
    #[inline]
    pub fn parse_non_negative_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(
            context,
            input,
            AllowedNumericType::NonNegative,
            allow_quirks,
        )
    }
}

impl Zero for LengthPercentage {
    fn zero() -> Self {
        LengthPercentage::Length(NoCalcLength::zero())
    }

    fn is_zero(&self) -> bool {
        match *self {
            LengthPercentage::Length(l) => l.is_zero(),
            LengthPercentage::Percentage(p) => p.0 == 0.0,
            LengthPercentage::Calc(_) => false,
        }
    }
}

impl ZeroNoPercent for LengthPercentage {
    fn is_zero_no_percent(&self) -> bool {
        match *self {
            LengthPercentage::Percentage(_) => false,
            _ => self.is_zero(),
        }
    }
}

/// A specified type for `<length-percentage> | auto`.
pub type LengthPercentageOrAuto = generics::LengthPercentageOrAuto<LengthPercentage>;

impl LengthPercentageOrAuto {
    /// Returns a value representing `0%`.
    #[inline]
    pub fn zero_percent() -> Self {
        generics::LengthPercentageOrAuto::LengthPercentage(LengthPercentage::zero_percent())
    }

    /// Parses a length or a percentage, allowing the unitless length quirk.
    /// <https://quirks.spec.whatwg.org/#the-unitless-length-quirk>
    #[inline]
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with(context, input, |context, input| {
            LengthPercentage::parse_quirky(context, input, allow_quirks)
        })
    }
}

/// A wrapper of LengthPercentageOrAuto, whose value must be >= 0.
pub type NonNegativeLengthPercentageOrAuto =
    generics::LengthPercentageOrAuto<NonNegativeLengthPercentage>;

impl NonNegativeLengthPercentageOrAuto {
    /// Returns a value representing `0%`.
    #[inline]
    pub fn zero_percent() -> Self {
        generics::LengthPercentageOrAuto::LengthPercentage(
            NonNegativeLengthPercentage::zero_percent(),
        )
    }

    /// Parses a non-negative length-percentage, allowing the unitless length
    /// quirk.
    #[inline]
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with(context, input, |context, input| {
            NonNegativeLengthPercentage::parse_quirky(context, input, allow_quirks)
        })
    }
}

/// A wrapper of LengthPercentage, whose value must be >= 0.
pub type NonNegativeLengthPercentage = NonNegative<LengthPercentage>;

/// Either a NonNegativeLengthPercentage or the `normal` keyword.
pub type NonNegativeLengthPercentageOrNormal =
    GenericLengthPercentageOrNormal<NonNegativeLengthPercentage>;

impl From<NoCalcLength> for NonNegativeLengthPercentage {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        NonNegative(LengthPercentage::from(len))
    }
}

impl Parse for NonNegativeLengthPercentage {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl NonNegativeLengthPercentage {
    #[inline]
    /// Returns a `0%` value.
    pub fn zero_percent() -> Self {
        NonNegative(LengthPercentage::zero_percent())
    }

    /// Parses a length or a percentage, allowing the unitless length quirk.
    /// <https://quirks.spec.whatwg.org/#the-unitless-length-quirk>
    #[inline]
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        LengthPercentage::parse_non_negative_quirky(context, input, allow_quirks).map(NonNegative)
    }
}

/// Either a `<length>` or the `auto` keyword.
///
/// Note that we use LengthPercentage just for convenience, since it pretty much
/// is everything we care about, but we could just add a similar LengthOrAuto
/// instead if we think getting rid of this weirdness is worth it.
pub type LengthOrAuto = generics::LengthPercentageOrAuto<Length>;

impl LengthOrAuto {
    /// Parses a length, allowing the unitless length quirk.
    /// <https://quirks.spec.whatwg.org/#the-unitless-length-quirk>
    #[inline]
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with(context, input, |context, input| {
            Length::parse_quirky(context, input, allow_quirks)
        })
    }
}

/// Either a non-negative `<length>` or the `auto` keyword.
pub type NonNegativeLengthOrAuto = generics::LengthPercentageOrAuto<NonNegativeLength>;

/// Either a `<length>` or a `<number>`.
pub type LengthOrNumber = GenericLengthOrNumber<Length, Number>;

/// A specified value for `min-width`, `min-height`, `width` or `height` property.
pub type Size = GenericSize<NonNegativeLengthPercentage>;

impl Parse for Size {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Size::parse_quirky(context, input, AllowQuirks::No)
    }
}

macro_rules! parse_size_non_length {
    ($size:ident, $input:expr, $auto_or_none:expr => $auto_or_none_ident:ident) => {{
        let size = $input.try_parse(|input| {
            Ok(try_match_ident_ignore_ascii_case! { input,
                #[cfg(feature = "gecko")]
                "min-content" | "-moz-min-content" => $size::MinContent,
                #[cfg(feature = "gecko")]
                "max-content" | "-moz-max-content" => $size::MaxContent,
                #[cfg(feature = "gecko")]
                "fit-content" | "-moz-fit-content" => $size::FitContent,
                #[cfg(feature = "gecko")]
                "-moz-available" => $size::MozAvailable,
                $auto_or_none => $size::$auto_or_none_ident,
            })
        });
        if size.is_ok() {
            return size;
        }
    }};
}

#[cfg(feature = "gecko")]
fn is_fit_content_function_enabled() -> bool {
    static_prefs::pref!("layout.css.fit-content-function.enabled")
}

#[cfg(feature = "gecko")]
macro_rules! parse_fit_content_function {
    ($size:ident, $input:expr, $context:expr, $allow_quirks:expr) => {
        if is_fit_content_function_enabled() {
            if let Ok(length) = $input.try_parse(|input| {
                input.expect_function_matching("fit-content")?;
                input.parse_nested_block(|i| {
                    NonNegativeLengthPercentage::parse_quirky($context, i, $allow_quirks)
                })
            }) {
                return Ok($size::FitContentFunction(length));
            }
        }
    };
}

impl Size {
    /// Parses, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        parse_size_non_length!(Size, input, "auto" => Auto);
        #[cfg(feature = "gecko")]
        parse_fit_content_function!(Size, input, context, allow_quirks);

        let length = NonNegativeLengthPercentage::parse_quirky(context, input, allow_quirks)?;
        Ok(GenericSize::LengthPercentage(length))
    }

    /// Returns `0%`.
    #[inline]
    pub fn zero_percent() -> Self {
        GenericSize::LengthPercentage(NonNegativeLengthPercentage::zero_percent())
    }
}

/// A specified value for `max-width` or `max-height` property.
pub type MaxSize = GenericMaxSize<NonNegativeLengthPercentage>;

impl Parse for MaxSize {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        MaxSize::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl MaxSize {
    /// Parses, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        parse_size_non_length!(MaxSize, input, "none" => None);
        #[cfg(feature = "gecko")]
        parse_fit_content_function!(MaxSize, input, context, allow_quirks);

        let length = NonNegativeLengthPercentage::parse_quirky(context, input, allow_quirks)?;
        Ok(GenericMaxSize::LengthPercentage(length))
    }
}

/// A specified non-negative `<length>` | `<number>`.
pub type NonNegativeLengthOrNumber = GenericLengthOrNumber<NonNegativeLength, NonNegativeNumber>;
