/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types that share their serialization implementations
//! for both specified and computed values.

use counter_style::{parse_counter_style_name, Symbols};
use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::{KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind};
use super::CustomIdent;

pub mod background;
pub mod basic_shape;
pub mod border;
#[path = "box.rs"]
pub mod box_;
pub mod column;
pub mod counters;
pub mod effects;
pub mod flex;
pub mod font;
#[cfg(feature = "gecko")]
pub mod gecko;
pub mod grid;
pub mod image;
pub mod pointing;
pub mod position;
pub mod rect;
pub mod size;
pub mod svg;
pub mod text;
pub mod transform;
pub mod url;

// https://drafts.csswg.org/css-counter-styles/#typedef-symbols-type
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToComputedValue, ToCss)]
pub enum SymbolsType {
    Cyclic,
    Numeric,
    Alphabetic,
    Symbolic,
    Fixed,
}

#[cfg(feature = "gecko")]
impl SymbolsType {
    /// Convert symbols type to their corresponding Gecko values.
    pub fn to_gecko_keyword(self) -> u8 {
        use gecko_bindings::structs;
        match self {
            SymbolsType::Cyclic => structs::NS_STYLE_COUNTER_SYSTEM_CYCLIC as u8,
            SymbolsType::Numeric => structs::NS_STYLE_COUNTER_SYSTEM_NUMERIC as u8,
            SymbolsType::Alphabetic => structs::NS_STYLE_COUNTER_SYSTEM_ALPHABETIC as u8,
            SymbolsType::Symbolic => structs::NS_STYLE_COUNTER_SYSTEM_SYMBOLIC as u8,
            SymbolsType::Fixed => structs::NS_STYLE_COUNTER_SYSTEM_FIXED as u8,
        }
    }

    /// Convert Gecko value to symbol type.
    pub fn from_gecko_keyword(gecko_value: u32) -> SymbolsType {
        use gecko_bindings::structs;
        match gecko_value {
            structs::NS_STYLE_COUNTER_SYSTEM_CYCLIC => SymbolsType::Cyclic,
            structs::NS_STYLE_COUNTER_SYSTEM_NUMERIC => SymbolsType::Numeric,
            structs::NS_STYLE_COUNTER_SYSTEM_ALPHABETIC => SymbolsType::Alphabetic,
            structs::NS_STYLE_COUNTER_SYSTEM_SYMBOLIC => SymbolsType::Symbolic,
            structs::NS_STYLE_COUNTER_SYSTEM_FIXED => SymbolsType::Fixed,
            x => panic!("Unexpected value for symbol type {}", x),
        }
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#typedef-counter-style>
///
/// Since wherever <counter-style> is used, 'none' is a valid value as
/// well, we combine them into one type to make code simpler.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, Eq, PartialEq, ToComputedValue, ToCss)]
pub enum CounterStyleOrNone {
    /// `none`
    None,
    /// `<counter-style-name>`
    Name(CustomIdent),
    /// `symbols()`
    #[css(function)]
    Symbols(SymbolsType, Symbols),
}

impl CounterStyleOrNone {
    /// disc value
    pub fn disc() -> Self {
        CounterStyleOrNone::Name(CustomIdent(atom!("disc")))
    }

    /// decimal value
    pub fn decimal() -> Self {
        CounterStyleOrNone::Name(CustomIdent(atom!("decimal")))
    }
}

impl Parse for CounterStyleOrNone {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try(|i| parse_counter_style_name(i)) {
            return Ok(CounterStyleOrNone::Name(name));
        }
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(CounterStyleOrNone::None);
        }
        if input.try(|i| i.expect_function_matching("symbols")).is_ok() {
            return input.parse_nested_block(|input| {
                let symbols_type = input
                    .try(|i| SymbolsType::parse(i))
                    .unwrap_or(SymbolsType::Symbolic);
                let symbols = Symbols::parse(context, input)?;
                // There must be at least two symbols for alphabetic or
                // numeric system.
                if (symbols_type == SymbolsType::Alphabetic ||
                    symbols_type == SymbolsType::Numeric) && symbols.0.len() < 2
                {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                // Identifier is not allowed in symbols() function.
                if symbols.0.iter().any(|sym| !sym.is_allowed_in_symbols()) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                Ok(CounterStyleOrNone::Symbols(symbols_type, symbols))
            });
        }
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl SpecifiedValueInfo for CounterStyleOrNone {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        // XXX The best approach for implementing this is probably
        // having a CounterStyleName type wrapping CustomIdent, and
        // put the predefined list for that type in counter_style mod.
        // But that's a non-trivial change itself, so we use a simpler
        // approach here.
        macro_rules! predefined {
            ($($name:expr,)+) => {
                f(&["none", "symbols", $($name,)+]);
            }
        }
        include!("../../counter_style/predefined.rs");
    }
}

/// A wrapper of Non-negative values.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf,
         PartialEq, PartialOrd, SpecifiedValueInfo, ToAnimatedZero,
         ToComputedValue, ToCss)]
pub struct NonNegative<T>(pub T);

/// A wrapper of greater-than-or-equal-to-one values.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf,
         PartialEq, PartialOrd, SpecifiedValueInfo, ToAnimatedZero,
         ToComputedValue, ToCss)]
pub struct GreaterThanOrEqualToOne<T>(pub T);
