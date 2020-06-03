/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types that share their serialization implementations
//! for both specified and computed values.

use super::CustomIdent;
use crate::counter_style::{parse_counter_style_name, Symbols};
use crate::parser::{Parse, ParserContext};
use crate::Zero;
use cssparser::Parser;
use std::ops::Add;
use style_traits::{KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind};

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
pub mod grid;
pub mod image;
pub mod length;
pub mod motion;
pub mod position;
pub mod rect;
pub mod size;
pub mod svg;
pub mod text;
pub mod transform;
pub mod ui;
pub mod url;

/// https://drafts.csswg.org/css-counter-styles/#typedef-symbols-type
#[allow(missing_docs)]
#[derive(
    Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToComputedValue, ToCss, ToResolvedValue,
)]
#[cfg_attr(feature = "gecko", derive(ToShmem))]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(u8)]
pub enum SymbolsType {
    Cyclic,
    Numeric,
    Alphabetic,
    Symbolic,
    Fixed,
}

/// <https://drafts.csswg.org/css-counter-styles/#typedef-counter-style>
///
/// Note that 'none' is not a valid name.
#[derive(Clone, Debug, Eq, PartialEq, ToComputedValue, ToCss, ToResolvedValue)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf, ToShmem))]
#[repr(u8)]
pub enum CounterStyle {
    /// `<counter-style-name>`
    Name(CustomIdent),
    /// `symbols()`
    #[css(function)]
    Symbols(#[css(skip_if = "is_symbolic")] SymbolsType, Symbols),
}

#[inline]
fn is_symbolic(symbols_type: &SymbolsType) -> bool {
    *symbols_type == SymbolsType::Symbolic
}

impl CounterStyle {
    /// disc value
    pub fn disc() -> Self {
        CounterStyle::Name(CustomIdent(atom!("disc")))
    }

    /// decimal value
    pub fn decimal() -> Self {
        CounterStyle::Name(CustomIdent(atom!("decimal")))
    }
}

impl Parse for CounterStyle {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try(|i| parse_counter_style_name(i)) {
            return Ok(CounterStyle::Name(name));
        }
        input.expect_function_matching("symbols")?;
        input.parse_nested_block(|input| {
            let symbols_type = input
                .try(SymbolsType::parse)
                .unwrap_or(SymbolsType::Symbolic);
            let symbols = Symbols::parse(context, input)?;
            // There must be at least two symbols for alphabetic or
            // numeric system.
            if (symbols_type == SymbolsType::Alphabetic || symbols_type == SymbolsType::Numeric) &&
                symbols.0.len() < 2
            {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
            // Identifier is not allowed in symbols() function.
            if symbols.0.iter().any(|sym| !sym.is_allowed_in_symbols()) {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
            Ok(CounterStyle::Symbols(symbols_type, symbols))
        })
    }
}

impl SpecifiedValueInfo for CounterStyle {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        // XXX The best approach for implementing this is probably
        // having a CounterStyleName type wrapping CustomIdent, and
        // put the predefined list for that type in counter_style mod.
        // But that's a non-trivial change itself, so we use a simpler
        // approach here.
        macro_rules! predefined {
            ($($name:expr,)+) => {
                f(&["symbols", $($name,)+]);
            }
        }
        include!("../../counter_style/predefined.rs");
    }
}

/// A wrapper of Non-negative values.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Hash,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "gecko", derive(ToShmem))]
#[repr(transparent)]
pub struct NonNegative<T>(pub T);

impl<T: Add<Output = T>> Add<NonNegative<T>> for NonNegative<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        NonNegative(self.0 + other.0)
    }
}

impl<T: Zero> Zero for NonNegative<T> {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn zero() -> Self {
        NonNegative(T::zero())
    }
}

/// A wrapper of greater-than-or-equal-to-one values.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "gecko", derive(ToShmem))]
pub struct GreaterThanOrEqualToOne<T>(pub T);

/// A wrapper of values between zero and one.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Hash,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
)]
#[cfg_attr(feature = "gecko", derive(ToShmem))]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(transparent)]
pub struct ZeroToOne<T>(pub T);

/// A clip rect for clip and image-region
#[allow(missing_docs)]
#[derive(
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
)]
#[cfg_attr(feature = "gecko", derive(ToShmem))]
#[css(function = "rect", comma)]
#[repr(C)]
pub struct GenericClipRect<LengthOrAuto> {
    pub top: LengthOrAuto,
    pub right: LengthOrAuto,
    pub bottom: LengthOrAuto,
    pub left: LengthOrAuto,
}

pub use self::GenericClipRect as ClipRect;

/// Either a clip-rect or `auto`.
#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
)]
#[cfg_attr(feature = "gecko", derive(ToShmem))]
#[repr(C, u8)]
pub enum GenericClipRectOrAuto<R> {
    Auto,
    Rect(R),
}

pub use self::GenericClipRectOrAuto as ClipRectOrAuto;

impl<L> ClipRectOrAuto<L> {
    /// Returns the `auto` value.
    #[inline]
    pub fn auto() -> Self {
        ClipRectOrAuto::Auto
    }

    /// Returns whether this value is the `auto` value.
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(*self, ClipRectOrAuto::Auto)
    }
}
