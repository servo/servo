/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [Length values][length].
//!
//! [length]: https://drafts.csswg.org/css-values/#lengths

use super::{AllowQuirks, Number, Percentage, ToComputedValue};
use crate::computed_value_flags::ComputedValueFlags;
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
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ParseError, SpecifiedValueInfo, StyleParseErrorKind, ToCss};

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
    pub fn resolve(&self, context: &Context) -> computed::FontSize {
        match *self {
            Self::CurrentStyle => context.style().get_font().clone_font_size(),
            Self::InheritedStyle => context.style().get_parent_font().clone_font_size(),
        }
    }
}

impl FontRelativeLength {
    /// Return the unitless, raw value.
    fn unitless_value(&self) -> CSSFloat {
        match *self {
            Self::Em(v) | Self::Ex(v) | Self::Ch(v) | Self::Cap(v) | Self::Ic(v) | Self::Rem(v) => {
                v
            },
        }
    }

    // Return the unit, as a string.
    fn unit(&self) -> &'static str {
        match *self {
            Self::Em(_) => "em",
            Self::Ex(_) => "ex",
            Self::Ch(_) => "ch",
            Self::Cap(_) => "cap",
            Self::Ic(_) => "ic",
            Self::Rem(_) => "rem",
        }
    }

    fn try_op<O>(&self, other: &Self, op: O) -> Result<Self, ()>
    where
        O: Fn(f32, f32) -> f32,
    {
        use self::FontRelativeLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        Ok(match (self, other) {
            (&Em(one), &Em(other)) => Em(op(one, other)),
            (&Ex(one), &Ex(other)) => Ex(op(one, other)),
            (&Ch(one), &Ch(other)) => Ch(op(one, other)),
            (&Cap(one), &Cap(other)) => Cap(op(one, other)),
            (&Ic(one), &Ic(other)) => Ic(op(one, other)),
            (&Rem(one), &Rem(other)) => Rem(op(one, other)),
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Em(..) | Ex(..) | Ch(..) | Cap(..) | Ic(..) | Rem(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in try_op()")
            },
        })
    }

    fn map(&self, mut op: impl FnMut(f32) -> f32) -> Self {
        match self {
            Self::Em(x) => Self::Em(op(*x)),
            Self::Ex(x) => Self::Ex(op(*x)),
            Self::Ch(x) => Self::Ch(op(*x)),
            Self::Cap(x) => Self::Cap(op(*x)),
            Self::Ic(x) => Self::Ic(op(*x)),
            Self::Rem(x) => Self::Rem(op(*x)),
        }
    }

    /// Computes the font-relative length.
    pub fn to_computed_value(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> computed::Length {
        let (reference_size, length) = self.reference_font_size_and_length(context, base_size);
        (reference_size * length).finite()
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
            Self::Em(length) => {
                if context.for_non_inherited_property && base_size == FontBaseSize::CurrentStyle {
                    context
                        .rule_cache_conditions
                        .borrow_mut()
                        .set_font_size_dependency(reference_font_size.computed_size);
                }

                (reference_font_size.computed_size(), length)
            },
            Self::Ex(length) => {
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
                    // (But note we use 0.5em of the used, not computed
                    // font-size)
                    reference_font_size.used_size() * 0.5
                });
                (reference_size, length)
            },
            Self::Ch(length) => {
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
                    // Same caveat about computed vs. used font-size applies
                    // above.
                    let wm = context.style().writing_mode;
                    if wm.is_vertical() && wm.is_upright() {
                        reference_font_size.used_size()
                    } else {
                        reference_font_size.used_size() * 0.5
                    }
                });
                (reference_size, length)
            },
            Self::Cap(length) => {
                let metrics =
                    query_font_metrics(context, base_size, FontMetricsOrientation::Horizontal);
                let reference_size = metrics.cap_height.unwrap_or_else(|| {
                    // https://drafts.csswg.org/css-values/#cap
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the cap-height, the font’s ascent must be
                    //     used.
                    //
                    metrics.ascent
                });
                (reference_size, length)
            },
            Self::Ic(length) => {
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
                    // Same caveat about computed vs. used as for other
                    // metric-dependent units.
                    reference_font_size.used_size()
                });
                (reference_size, length)
            },
            Self::Rem(length) => {
                // https://drafts.csswg.org/css-values/#rem:
                //
                //     When specified on the font-size property of the root
                //     element, the rem units refer to the property's initial
                //     value.
                //
                let reference_size = if context.builder.is_root_element || context.in_media_query {
                    reference_font_size.computed_size()
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
    /// Return the unitless, raw value.
    fn unitless_value(&self) -> CSSFloat {
        self.unpack().2
    }

    // Return the unit, as a string.
    fn unit(&self) -> &'static str {
        match *self {
            Self::Vw(_) => "vw",
            Self::Lvw(_) => "lvw",
            Self::Svw(_) => "svw",
            Self::Dvw(_) => "dvw",
            Self::Vh(_) => "vh",
            Self::Svh(_) => "svh",
            Self::Lvh(_) => "lvh",
            Self::Dvh(_) => "dvh",
            Self::Vmin(_) => "vmin",
            Self::Svmin(_) => "svmin",
            Self::Lvmin(_) => "lvmin",
            Self::Dvmin(_) => "dvmin",
            Self::Vmax(_) => "vmax",
            Self::Svmax(_) => "svmax",
            Self::Lvmax(_) => "lvmax",
            Self::Dvmax(_) => "dvmax",
            Self::Vb(_) => "vb",
            Self::Svb(_) => "svb",
            Self::Lvb(_) => "lvb",
            Self::Dvb(_) => "dvb",
            Self::Vi(_) => "vi",
            Self::Svi(_) => "svi",
            Self::Lvi(_) => "lvi",
            Self::Dvi(_) => "dvi",
        }
    }

    fn unpack(&self) -> (ViewportVariant, ViewportUnit, CSSFloat) {
        match *self {
            Self::Vw(v) => (ViewportVariant::UADefault, ViewportUnit::Vw, v),
            Self::Svw(v) => (ViewportVariant::Small, ViewportUnit::Vw, v),
            Self::Lvw(v) => (ViewportVariant::Large, ViewportUnit::Vw, v),
            Self::Dvw(v) => (ViewportVariant::Dynamic, ViewportUnit::Vw, v),
            Self::Vh(v) => (ViewportVariant::UADefault, ViewportUnit::Vh, v),
            Self::Svh(v) => (ViewportVariant::Small, ViewportUnit::Vh, v),
            Self::Lvh(v) => (ViewportVariant::Large, ViewportUnit::Vh, v),
            Self::Dvh(v) => (ViewportVariant::Dynamic, ViewportUnit::Vh, v),
            Self::Vmin(v) => (ViewportVariant::UADefault, ViewportUnit::Vmin, v),
            Self::Svmin(v) => (ViewportVariant::Small, ViewportUnit::Vmin, v),
            Self::Lvmin(v) => (ViewportVariant::Large, ViewportUnit::Vmin, v),
            Self::Dvmin(v) => (ViewportVariant::Dynamic, ViewportUnit::Vmin, v),
            Self::Vmax(v) => (ViewportVariant::UADefault, ViewportUnit::Vmax, v),
            Self::Svmax(v) => (ViewportVariant::Small, ViewportUnit::Vmax, v),
            Self::Lvmax(v) => (ViewportVariant::Large, ViewportUnit::Vmax, v),
            Self::Dvmax(v) => (ViewportVariant::Dynamic, ViewportUnit::Vmax, v),
            Self::Vb(v) => (ViewportVariant::UADefault, ViewportUnit::Vb, v),
            Self::Svb(v) => (ViewportVariant::Small, ViewportUnit::Vb, v),
            Self::Lvb(v) => (ViewportVariant::Large, ViewportUnit::Vb, v),
            Self::Dvb(v) => (ViewportVariant::Dynamic, ViewportUnit::Vb, v),
            Self::Vi(v) => (ViewportVariant::UADefault, ViewportUnit::Vi, v),
            Self::Svi(v) => (ViewportVariant::Small, ViewportUnit::Vi, v),
            Self::Lvi(v) => (ViewportVariant::Large, ViewportUnit::Vi, v),
            Self::Dvi(v) => (ViewportVariant::Dynamic, ViewportUnit::Vi, v),
        }
    }

    fn try_op<O>(&self, other: &Self, op: O) -> Result<Self, ()>
    where
        O: Fn(f32, f32) -> f32,
    {
        use self::ViewportPercentageLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        Ok(match (self, other) {
            (&Vw(one), &Vw(other)) => Vw(op(one, other)),
            (&Svw(one), &Svw(other)) => Svw(op(one, other)),
            (&Lvw(one), &Lvw(other)) => Lvw(op(one, other)),
            (&Dvw(one), &Dvw(other)) => Dvw(op(one, other)),
            (&Vh(one), &Vh(other)) => Vh(op(one, other)),
            (&Svh(one), &Svh(other)) => Svh(op(one, other)),
            (&Lvh(one), &Lvh(other)) => Lvh(op(one, other)),
            (&Dvh(one), &Dvh(other)) => Dvh(op(one, other)),
            (&Vmin(one), &Vmin(other)) => Vmin(op(one, other)),
            (&Svmin(one), &Svmin(other)) => Svmin(op(one, other)),
            (&Lvmin(one), &Lvmin(other)) => Lvmin(op(one, other)),
            (&Dvmin(one), &Dvmin(other)) => Dvmin(op(one, other)),
            (&Vmax(one), &Vmax(other)) => Vmax(op(one, other)),
            (&Svmax(one), &Svmax(other)) => Svmax(op(one, other)),
            (&Lvmax(one), &Lvmax(other)) => Lvmax(op(one, other)),
            (&Dvmax(one), &Dvmax(other)) => Dvmax(op(one, other)),
            (&Vb(one), &Vb(other)) => Vb(op(one, other)),
            (&Svb(one), &Svb(other)) => Svb(op(one, other)),
            (&Lvb(one), &Lvb(other)) => Lvb(op(one, other)),
            (&Dvb(one), &Dvb(other)) => Dvb(op(one, other)),
            (&Vi(one), &Vi(other)) => Vi(op(one, other)),
            (&Svi(one), &Svi(other)) => Svi(op(one, other)),
            (&Lvi(one), &Lvi(other)) => Lvi(op(one, other)),
            (&Dvi(one), &Dvi(other)) => Dvi(op(one, other)),
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Vw(..) | Svw(..) | Lvw(..) | Dvw(..) | Vh(..) | Svh(..) | Lvh(..) |
                    Dvh(..) | Vmin(..) | Svmin(..) | Lvmin(..) | Dvmin(..) | Vmax(..) |
                    Svmax(..) | Lvmax(..) | Dvmax(..) | Vb(..) | Svb(..) | Lvb(..) | Dvb(..) |
                    Vi(..) | Svi(..) | Lvi(..) | Dvi(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in try_op()")
            },
        })
    }

    fn map(&self, mut op: impl FnMut(f32) -> f32) -> Self {
        match self {
            Self::Vw(x) => Self::Vw(op(*x)),
            Self::Svw(x) => Self::Svw(op(*x)),
            Self::Lvw(x) => Self::Lvw(op(*x)),
            Self::Dvw(x) => Self::Dvw(op(*x)),
            Self::Vh(x) => Self::Vh(op(*x)),
            Self::Svh(x) => Self::Svh(op(*x)),
            Self::Lvh(x) => Self::Lvh(op(*x)),
            Self::Dvh(x) => Self::Dvh(op(*x)),
            Self::Vmin(x) => Self::Vmin(op(*x)),
            Self::Svmin(x) => Self::Svmin(op(*x)),
            Self::Lvmin(x) => Self::Lvmin(op(*x)),
            Self::Dvmin(x) => Self::Dvmin(op(*x)),
            Self::Vmax(x) => Self::Vmax(op(*x)),
            Self::Svmax(x) => Self::Svmax(op(*x)),
            Self::Lvmax(x) => Self::Lvmax(op(*x)),
            Self::Dvmax(x) => Self::Dvmax(op(*x)),
            Self::Vb(x) => Self::Vb(op(*x)),
            Self::Svb(x) => Self::Svb(op(*x)),
            Self::Lvb(x) => Self::Lvb(op(*x)),
            Self::Dvb(x) => Self::Dvb(op(*x)),
            Self::Vi(x) => Self::Vi(op(*x)),
            Self::Svi(x) => Self::Svi(op(*x)),
            Self::Lvi(x) => Self::Lvi(op(*x)),
            Self::Dvi(x) => Self::Dvi(op(*x)),
        }
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
        Au::from_f64_au(if trunc_scaled.is_nan() {
            0.0f64
        } else {
            trunc_scaled
        })
        .into()
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
        (average_advance * (self.0 as CSSFloat - 1.0) + max_advance).finite()
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
            Self::Px(v) |
            Self::In(v) |
            Self::Cm(v) |
            Self::Mm(v) |
            Self::Q(v) |
            Self::Pt(v) |
            Self::Pc(v) => v,
        }
    }

    // Return the unit, as a string.
    fn unit(&self) -> &'static str {
        match *self {
            Self::Px(_) => "px",
            Self::In(_) => "in",
            Self::Cm(_) => "cm",
            Self::Mm(_) => "mm",
            Self::Q(_) => "q",
            Self::Pt(_) => "pt",
            Self::Pc(_) => "pc",
        }
    }

    /// Convert this into a pixel value.
    #[inline]
    pub fn to_px(&self) -> CSSFloat {
        match *self {
            Self::Px(value) => value,
            Self::In(value) => value * PX_PER_IN,
            Self::Cm(value) => value * PX_PER_CM,
            Self::Mm(value) => value * PX_PER_MM,
            Self::Q(value) => value * PX_PER_Q,
            Self::Pt(value) => value * PX_PER_PT,
            Self::Pc(value) => value * PX_PER_PC,
        }
    }

    fn try_op<O>(&self, other: &Self, op: O) -> Result<Self, ()>
    where
        O: Fn(f32, f32) -> f32,
    {
        Ok(Self::Px(op(self.to_px(), other.to_px())))
    }

    fn map(&self, mut op: impl FnMut(f32) -> f32) -> Self {
        Self::Px(op(self.to_px()))
    }
}

impl ToComputedValue for AbsoluteLength {
    type ComputedValue = CSSPixelLength;

    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        CSSPixelLength::new(self.to_px()).finite()
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self::Px(computed.px())
    }
}

impl PartialOrd for AbsoluteLength {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.to_px().partial_cmp(&other.to_px())
    }
}

/// A container query length.
///
/// <https://drafts.csswg.org/css-contain-3/#container-lengths>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToShmem)]
pub enum ContainerRelativeLength {
    /// 1% of query container's width
    #[css(dimension)]
    Cqw(CSSFloat),
    /// 1% of query container's height
    #[css(dimension)]
    Cqh(CSSFloat),
    /// 1% of query container's inline size
    #[css(dimension)]
    Cqi(CSSFloat),
    /// 1% of query container's block size
    #[css(dimension)]
    Cqb(CSSFloat),
    /// The smaller value of `cqi` or `cqb`
    #[css(dimension)]
    Cqmin(CSSFloat),
    /// The larger value of `cqi` or `cqb`
    #[css(dimension)]
    Cqmax(CSSFloat),
}

impl ContainerRelativeLength {
    fn unitless_value(&self) -> CSSFloat {
        match *self {
            Self::Cqw(v) |
            Self::Cqh(v) |
            Self::Cqi(v) |
            Self::Cqb(v) |
            Self::Cqmin(v) |
            Self::Cqmax(v) => v,
        }
    }

    // Return the unit, as a string.
    fn unit(&self) -> &'static str {
        match *self {
            Self::Cqw(_) => "cqw",
            Self::Cqh(_) => "cqh",
            Self::Cqi(_) => "cqi",
            Self::Cqb(_) => "cqb",
            Self::Cqmin(_) => "cqmin",
            Self::Cqmax(_) => "cqmax",
        }
    }

    pub(crate) fn try_op<O>(&self, other: &Self, op: O) -> Result<Self, ()>
    where
        O: Fn(f32, f32) -> f32,
    {
        use self::ContainerRelativeLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        Ok(match (self, other) {
            (&Cqw(one), &Cqw(other)) => Cqw(op(one, other)),
            (&Cqh(one), &Cqh(other)) => Cqh(op(one, other)),
            (&Cqi(one), &Cqi(other)) => Cqi(op(one, other)),
            (&Cqb(one), &Cqb(other)) => Cqb(op(one, other)),
            (&Cqmin(one), &Cqmin(other)) => Cqmin(op(one, other)),
            (&Cqmax(one), &Cqmax(other)) => Cqmax(op(one, other)),

            // See https://github.com/rust-lang/rust/issues/68867, then
            // https://github.com/rust-lang/rust/pull/95161. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Cqw(..) | Cqh(..) | Cqi(..) | Cqb(..) | Cqmin(..) | Cqmax(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in try_op()")
            },
        })
    }

    pub(crate) fn map(&self, mut op: impl FnMut(f32) -> f32) -> Self {
        match self {
            Self::Cqw(x) => Self::Cqw(op(*x)),
            Self::Cqh(x) => Self::Cqh(op(*x)),
            Self::Cqi(x) => Self::Cqi(op(*x)),
            Self::Cqb(x) => Self::Cqb(op(*x)),
            Self::Cqmin(x) => Self::Cqmin(op(*x)),
            Self::Cqmax(x) => Self::Cqmax(op(*x)),
        }
    }

    /// Computes the given container-relative length.
    pub fn to_computed_value(&self, context: &Context) -> CSSPixelLength {
        if context.for_non_inherited_property {
            context.rule_cache_conditions.borrow_mut().set_uncacheable();
        }
        context.builder.add_flags(ComputedValueFlags::USES_CONTAINER_UNITS);

        let size = context.get_container_size_query();
        let (factor, container_length) = match *self {
            Self::Cqw(v) => (v, size.get_container_width(context)),
            Self::Cqh(v) => (v, size.get_container_height(context)),
            Self::Cqi(v) => (v, size.get_container_inline_size(context)),
            Self::Cqb(v) => (v, size.get_container_block_size(context)),
            Self::Cqmin(v) => (
                v,
                cmp::min(
                    size.get_container_inline_size(context),
                    size.get_container_block_size(context),
                ),
            ),
            Self::Cqmax(v) => (
                v,
                cmp::max(
                    size.get_container_inline_size(context),
                    size.get_container_block_size(context),
                ),
            ),
        };
        CSSPixelLength::new((container_length.to_f64_px() * factor as f64 / 100.0) as f32).finite()
    }
}

#[cfg(feature = "gecko")]
fn are_container_queries_enabled() -> bool {
    static_prefs::pref!("layout.css.container-queries.enabled")
}
#[cfg(feature = "servo")]
fn are_container_queries_enabled() -> bool {
    false
}

/// A `<length>` without taking `calc` expressions into account
///
/// <https://drafts.csswg.org/css-values/#lengths>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToShmem)]
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

    /// A container query length.
    ///
    /// <https://drafts.csswg.org/css-contain-3/#container-lengths>
    ContainerRelative(ContainerRelativeLength),
    /// HTML5 "character width", as defined in HTML5 § 14.5.4.
    ///
    /// This cannot be specified by the user directly and is only generated by
    /// `Stylist::synthesize_rules_for_legacy_attributes()`.
    ServoCharacterWidth(CharacterWidth),
}

impl NoCalcLength {
    /// Return the unitless, raw value.
    pub fn unitless_value(&self) -> CSSFloat {
        match *self {
            Self::Absolute(v) => v.unitless_value(),
            Self::FontRelative(v) => v.unitless_value(),
            Self::ViewportPercentage(v) => v.unitless_value(),
            Self::ContainerRelative(v) => v.unitless_value(),
            Self::ServoCharacterWidth(c) => c.0 as f32,
        }
    }

    // Return the unit, as a string.
    fn unit(&self) -> &'static str {
        match *self {
            Self::Absolute(v) => v.unit(),
            Self::FontRelative(v) => v.unit(),
            Self::ViewportPercentage(v) => v.unit(),
            Self::ContainerRelative(v) => v.unit(),
            Self::ServoCharacterWidth(_) => "",
        }
    }

    /// Returns whether the value of this length without unit is less than zero.
    pub fn is_negative(&self) -> bool {
        self.unitless_value().is_sign_negative()
    }

    /// Returns whether the value of this length without unit is equal to zero.
    pub fn is_zero(&self) -> bool {
        self.unitless_value() == 0.0
    }

    /// Returns whether the value of this length without unit is infinite.
    pub fn is_infinite(&self) -> bool {
        self.unitless_value().is_infinite()
    }

    /// Returns whether the value of this length without unit is NaN.
    pub fn is_nan(&self) -> bool {
        self.unitless_value().is_nan()
    }

    /// Whether text-only zoom should be applied to this length.
    ///
    /// Generally, font-dependent/relative units don't get text-only-zoomed,
    /// because the font they're relative to should be zoomed already.
    pub fn should_zoom_text(&self) -> bool {
        match *self {
            Self::Absolute(..) | Self::ViewportPercentage(..) | Self::ContainerRelative(..) => true,
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
            "px" => Self::Absolute(AbsoluteLength::Px(value)),
            "in" => Self::Absolute(AbsoluteLength::In(value)),
            "cm" => Self::Absolute(AbsoluteLength::Cm(value)),
            "mm" => Self::Absolute(AbsoluteLength::Mm(value)),
            "q" => Self::Absolute(AbsoluteLength::Q(value)),
            "pt" => Self::Absolute(AbsoluteLength::Pt(value)),
            "pc" => Self::Absolute(AbsoluteLength::Pc(value)),
            // font-relative
            "em" => Self::FontRelative(FontRelativeLength::Em(value)),
            "ex" => Self::FontRelative(FontRelativeLength::Ex(value)),
            "ch" => Self::FontRelative(FontRelativeLength::Ch(value)),
            "cap" => Self::FontRelative(FontRelativeLength::Cap(value)),
            "ic" => Self::FontRelative(FontRelativeLength::Ic(value)),
            "rem" => Self::FontRelative(FontRelativeLength::Rem(value)),
            // viewport percentages
            "vw" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Vw(value))
            },
            "svw" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Svw(value))
            },
            "lvw" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Lvw(value))
            },
            "dvw" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Dvw(value))
            },
            "vh" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Vh(value))
            },
            "svh" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Svh(value))
            },
            "lvh" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Lvh(value))
            },
            "dvh" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Dvh(value))
            },
            "vmin" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Vmin(value))
            },
            "svmin" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Svmin(value))
            },
            "lvmin" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Lvmin(value))
            },
            "dvmin" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Dvmin(value))
            },
            "vmax" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Vmax(value))
            },
            "svmax" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Svmax(value))
            },
            "lvmax" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Lvmax(value))
            },
            "dvmax" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Dvmax(value))
            },
            "vb" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Vb(value))
            },
            "svb" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Svb(value))
            },
            "lvb" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Lvb(value))
            },
            "dvb" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Dvb(value))
            },
            "vi" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Vi(value))
            },
            "svi" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Svi(value))
            },
            "lvi" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Lvi(value))
            },
            "dvi" if !context.in_page_rule() => {
                Self::ViewportPercentage(ViewportPercentageLength::Dvi(value))
            },
            // Container query lengths. Inherit the limitation from viewport units since
            // we may fall back to them.
            "cqw" if !context.in_page_rule() && are_container_queries_enabled() => {
                Self::ContainerRelative(ContainerRelativeLength::Cqw(value))
            },
            "cqh" if !context.in_page_rule() && are_container_queries_enabled() => {
                Self::ContainerRelative(ContainerRelativeLength::Cqh(value))
            },
            "cqi" if !context.in_page_rule() && are_container_queries_enabled() => {
                Self::ContainerRelative(ContainerRelativeLength::Cqi(value))
            },
            "cqb" if !context.in_page_rule() && are_container_queries_enabled() => {
                Self::ContainerRelative(ContainerRelativeLength::Cqb(value))
            },
            "cqmin" if !context.in_page_rule() && are_container_queries_enabled() => {
                Self::ContainerRelative(ContainerRelativeLength::Cqmin(value))
            },
            "cqmax" if !context.in_page_rule() && are_container_queries_enabled() => {
                Self::ContainerRelative(ContainerRelativeLength::Cqmax(value))
            },
            _ => return Err(()),
        })
    }

    pub(crate) fn try_op<O>(&self, other: &Self, op: O) -> Result<Self, ()>
    where
        O: Fn(f32, f32) -> f32,
    {
        use self::NoCalcLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        Ok(match (self, other) {
            (&Absolute(ref one), &Absolute(ref other)) => Absolute(one.try_op(other, op)?),
            (&FontRelative(ref one), &FontRelative(ref other)) => {
                FontRelative(one.try_op(other, op)?)
            },
            (&ViewportPercentage(ref one), &ViewportPercentage(ref other)) => {
                ViewportPercentage(one.try_op(other, op)?)
            },
            (&ContainerRelative(ref one), &ContainerRelative(ref other)) => {
                ContainerRelative(one.try_op(other, op)?)
            },
            (&ServoCharacterWidth(ref one), &ServoCharacterWidth(ref other)) => {
                ServoCharacterWidth(CharacterWidth(op(one.0 as f32, other.0 as f32) as i32))
            },
            // See https://github.com/rust-lang/rust/issues/68867. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Absolute(..) |
                    FontRelative(..) |
                    ViewportPercentage(..) |
                    ContainerRelative(..) |
                    ServoCharacterWidth(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in try_op()")
            },
        })
    }

    pub(crate) fn map(&self, mut op: impl FnMut(f32) -> f32) -> Self {
        use self::NoCalcLength::*;

        match self {
            Absolute(ref one) => Absolute(one.map(op)),
            FontRelative(ref one) => FontRelative(one.map(op)),
            ViewportPercentage(ref one) => ViewportPercentage(one.map(op)),
            ContainerRelative(ref one) => ContainerRelative(one.map(op)),
            ServoCharacterWidth(ref one) => {
                ServoCharacterWidth(CharacterWidth(op(one.0 as f32) as i32))
            },
        }
    }

    /// Get a px value without context.
    #[inline]
    pub fn to_computed_pixel_length_without_context(&self) -> Result<CSSFloat, ()> {
        match *self {
            Self::Absolute(len) => Ok(CSSPixelLength::new(len.to_px()).finite().px()),
            _ => Err(()),
        }
    }

    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> NoCalcLength {
        NoCalcLength::Absolute(AbsoluteLength::Px(px_value))
    }
}

impl ToCss for NoCalcLength {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        crate::values::serialize_specified_dimension(
            self.unitless_value(),
            self.unit(),
            false,
            dest,
        )
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
            (&ContainerRelative(ref one), &ContainerRelative(ref other)) => one.partial_cmp(other),
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
                    ContainerRelative(..) |
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
        NoCalcLength::is_zero(self)
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

impl PartialOrd for ContainerRelativeLength {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use self::ContainerRelativeLength::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
        }

        match (self, other) {
            (&Cqw(ref one), &Cqw(ref other)) => one.partial_cmp(other),
            (&Cqh(ref one), &Cqh(ref other)) => one.partial_cmp(other),
            (&Cqi(ref one), &Cqi(ref other)) => one.partial_cmp(other),
            (&Cqb(ref one), &Cqb(ref other)) => one.partial_cmp(other),
            (&Cqmin(ref one), &Cqmin(ref other)) => one.partial_cmp(other),
            (&Cqmax(ref one), &Cqmax(ref other)) => one.partial_cmp(other),

            // See https://github.com/rust-lang/rust/issues/68867, then
            // https://github.com/rust-lang/rust/pull/95161. rustc isn't
            // able to figure it own on its own so we help.
            _ => unsafe {
                match *self {
                    Cqw(..) | Cqh(..) | Cqi(..) | Cqb(..) | Cqmin(..) | Cqmax(..) => {},
                }
                debug_unreachable!("Forgot to handle unit in partial_cmp()")
            },
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
                let function = CalcNode::math_function(context, name, location)?;
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

    /// Get a px value without context.
    pub fn to_computed_pixel_length_without_context(&self) -> Result<CSSFloat, ()> {
        match *self {
            Self::NoCalc(ref l) => l.to_computed_pixel_length_without_context(),
            Self::Calc(ref l) => l.to_computed_pixel_length_without_context(),
        }
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
                let function = CalcNode::math_function(context, name, location)?;
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
