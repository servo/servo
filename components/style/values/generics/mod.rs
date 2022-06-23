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
use style_traits::{KeywordsCollectFn, ParseError, SpecifiedValueInfo, StyleParseErrorKind};

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
pub mod page;
pub mod position;
pub mod ratio;
pub mod rect;
pub mod size;
pub mod svg;
pub mod text;
pub mod transform;
pub mod ui;
pub mod url;

/// https://drafts.csswg.org/css-counter-styles/#typedef-symbols-type
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
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
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, Eq, PartialEq, ToComputedValue, ToCss, ToResolvedValue, ToShmem)]
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

    /// Is this a bullet? (i.e. `list-style-type: disc|circle|square|disclosure-closed|disclosure-open`)
    #[inline]
    pub fn is_bullet(&self) -> bool {
        match self {
            CounterStyle::Name(CustomIdent(ref name)) => {
                name == &atom!("disc") ||
                    name == &atom!("circle") ||
                    name == &atom!("square") ||
                    name == &atom!("disclosure-closed") ||
                    name == &atom!("disclosure-open")
            },
            _ => false,
        }
    }
}

impl Parse for CounterStyle {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try_parse(|i| parse_counter_style_name(i)) {
            return Ok(CounterStyle::Name(name));
        }
        input.expect_function_matching("symbols")?;
        input.parse_nested_block(|input| {
            let symbols_type = input
                .try_parse(SymbolsType::parse)
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
                f(&["symbols", $($name,)+])
            }
        }
        include!("../../counter_style/predefined.rs");
    }
}

/// A wrapper of Non-negative values.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
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
    ToShmem,
)]
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
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
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
    ToShmem,
)]
pub struct GreaterThanOrEqualToOne<T>(pub T);

/// A wrapper of values between zero and one.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
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
    ToShmem,
)]
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
    ToShmem,
)]
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
    ToShmem,
)]
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

pub use page::PageSize;

/// An optional value, much like `Option<T>`, but with a defined struct layout
/// to be able to use it from C++ as well.
///
/// Note that this is relatively inefficient, struct-layout-wise, as you have
/// one byte for the tag, but padding to the alignment of T. If you have
/// multiple optional values and care about struct compactness, you might be
/// better off "coalescing" the combinations into a parent enum. But that
/// shouldn't matter for most use cases.
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
    ToShmem,
    Serialize,
    Deserialize,
)]
#[repr(C, u8)]
pub enum Optional<T> {
    #[css(skip)]
    None,
    Some(T),
}

impl<T> Optional<T> {
    /// Returns whether this value is present.
    pub fn is_some(&self) -> bool {
        matches!(*self, Self::Some(..))
    }

    /// Returns whether this value is not present.
    pub fn is_none(&self) -> bool {
        matches!(*self, Self::None)
    }

    /// Turns this Optional<> into a regular rust Option<>.
    pub fn into_rust(self) -> Option<T> {
        match self {
            Self::Some(v) => Some(v),
            Self::None => None,
        }
    }

    /// Return a reference to the containing value, if any, as a plain rust
    /// Option<>.
    pub fn as_ref(&self) -> Option<&T> {
        match *self {
            Self::Some(ref v) => Some(v),
            Self::None => None,
        }
    }
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(rust: Option<T>) -> Self {
        match rust {
            Some(t) => Self::Some(t),
            None => Self::None,
        }
    }
}
