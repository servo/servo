/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The [`@counter-style`][counter-style] at-rule.
//!
//! [counter-style]: https://drafts.csswg.org/css-counter-styles/

use crate::error_reporting::ContextualParseError;
use crate::parser::{Parse, ParserContext};
use crate::shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::values::specified::Integer;
use crate::values::CustomIdent;
use crate::Atom;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser};
use cssparser::{CowRcStr, Parser, SourceLocation, Token};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use std::mem;
use std::num::Wrapping;
use style_traits::{Comma, CssWriter, OneOrMoreSeparated, ParseError};
use style_traits::{StyleParseErrorKind, ToCss};

/// Parse a counter style name reference.
///
/// This allows the reserved counter style names "decimal" and "disc".
pub fn parse_counter_style_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CustomIdent, ParseError<'i>> {
    macro_rules! predefined {
        ($($name: expr,)+) => {
            {
                ascii_case_insensitive_phf_map! {
                    // FIXME: use static atoms https://github.com/rust-lang/rust/issues/33156
                    predefined -> &'static str = {
                        $(
                            $name => $name,
                        )+
                    }
                }

                let location = input.current_source_location();
                let ident = input.expect_ident()?;
                if let Some(&lower_cased) = predefined(&ident) {
                    Ok(CustomIdent(Atom::from(lower_cased)))
                } else {
                    // none is always an invalid <counter-style> value.
                    CustomIdent::from_ident(location, ident, &["none"])
                }
            }
        }
    }
    include!("predefined.rs")
}

fn is_valid_name_definition(ident: &CustomIdent) -> bool {
    ident.0 != atom!("decimal") && ident.0 != atom!("disc")
}

/// Parse the prelude of an @counter-style rule
pub fn parse_counter_style_name_definition<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CustomIdent, ParseError<'i>> {
    parse_counter_style_name(input).and_then(|ident| {
        if !is_valid_name_definition(&ident) {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        } else {
            Ok(ident)
        }
    })
}

/// Parse the body (inside `{}`) of an @counter-style rule
pub fn parse_counter_style_body<'i, 't>(
    name: CustomIdent,
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
    location: SourceLocation,
) -> Result<CounterStyleRuleData, ParseError<'i>> {
    let start = input.current_source_location();
    let mut rule = CounterStyleRuleData::empty(name, location);
    {
        let parser = CounterStyleRuleParser {
            context: context,
            rule: &mut rule,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err((error, slice)) = declaration {
                let location = error.location;
                let error = ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(
                    slice, error,
                );
                context.log_css_error(location, error)
            }
        }
    }
    let error = match *rule.resolved_system() {
        ref system @ System::Cyclic |
        ref system @ System::Fixed { .. } |
        ref system @ System::Symbolic |
        ref system @ System::Alphabetic |
        ref system @ System::Numeric
            if rule.symbols.is_none() =>
        {
            let system = system.to_css_string();
            Some(ContextualParseError::InvalidCounterStyleWithoutSymbols(
                system,
            ))
        }
        ref system @ System::Alphabetic | ref system @ System::Numeric
            if rule.symbols().unwrap().0.len() < 2 =>
        {
            let system = system.to_css_string();
            Some(ContextualParseError::InvalidCounterStyleNotEnoughSymbols(
                system,
            ))
        }
        System::Additive if rule.additive_symbols.is_none() => {
            Some(ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols)
        },
        System::Extends(_) if rule.symbols.is_some() => {
            Some(ContextualParseError::InvalidCounterStyleExtendsWithSymbols)
        },
        System::Extends(_) if rule.additive_symbols.is_some() => {
            Some(ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols)
        },
        _ => None,
    };
    if let Some(error) = error {
        context.log_css_error(start, error);
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    } else {
        Ok(rule)
    }
}

struct CounterStyleRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    rule: &'a mut CounterStyleRuleData,
}

/// Default methods reject all at rules.
impl<'a, 'b, 'i> AtRuleParser<'i> for CounterStyleRuleParser<'a, 'b> {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = ();
    type Error = StyleParseErrorKind<'i>;
}

macro_rules! checker {
    ($self:ident._($value:ident)) => {};
    ($self:ident. $checker:ident($value:ident)) => {
        if !$self.$checker(&$value) {
            return false;
        }
    };
}

macro_rules! counter_style_descriptors {
    (
        $( #[$doc: meta] $name: tt $ident: ident / $setter: ident [$checker: tt]: $ty: ty, )+
    ) => {
        /// An @counter-style rule
        #[derive(Clone, Debug, ToShmem)]
        pub struct CounterStyleRuleData {
            name: CustomIdent,
            generation: Wrapping<u32>,
            $(
                #[$doc]
                $ident: Option<$ty>,
            )+
            /// Line and column of the @counter-style rule source code.
            pub source_location: SourceLocation,
        }

        impl CounterStyleRuleData {
            fn empty(name: CustomIdent, source_location: SourceLocation) -> Self {
                CounterStyleRuleData {
                    name: name,
                    generation: Wrapping(0),
                    $(
                        $ident: None,
                    )+
                    source_location,
                }
            }

            $(
                #[$doc]
                pub fn $ident(&self) -> Option<&$ty> {
                    self.$ident.as_ref()
                }
            )+

            $(
                #[$doc]
                pub fn $setter(&mut self, value: $ty) -> bool {
                    checker!(self.$checker(value));
                    self.$ident = Some(value);
                    self.generation += Wrapping(1);
                    true
                }
            )+
        }

        impl<'a, 'b, 'i> DeclarationParser<'i> for CounterStyleRuleParser<'a, 'b> {
            type Declaration = ();
            type Error = StyleParseErrorKind<'i>;

            fn parse_value<'t>(&mut self, name: CowRcStr<'i>, input: &mut Parser<'i, 't>)
                               -> Result<(), ParseError<'i>> {
                match_ignore_ascii_case! { &*name,
                    $(
                        $name => {
                            // DeclarationParser also calls parse_entirely
                            // so we’d normally not need to,
                            // but in this case we do because we set the value as a side effect
                            // rather than returning it.
                            let value = input.parse_entirely(|i| Parse::parse(self.context, i))?;
                            self.rule.$ident = Some(value)
                        },
                    )*
                    _ => return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone()))),
                }
                Ok(())
            }
        }

        impl ToCssWithGuard for CounterStyleRuleData {
            fn to_css(&self, _guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
                dest.write_str("@counter-style ")?;
                self.name.to_css(&mut CssWriter::new(dest))?;
                dest.write_str(" {\n")?;
                $(
                    if let Some(ref value) = self.$ident {
                        dest.write_str(concat!("  ", $name, ": "))?;
                        ToCss::to_css(value, &mut CssWriter::new(dest))?;
                        dest.write_str(";\n")?;
                    }
                )+
                dest.write_str("}")
            }
        }
    }
}

counter_style_descriptors! {
    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-system>
    "system" system / set_system [check_system]: System,

    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-negative>
    "negative" negative / set_negative [_]: Negative,

    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-prefix>
    "prefix" prefix / set_prefix [_]: Symbol,

    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-suffix>
    "suffix" suffix / set_suffix [_]: Symbol,

    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-range>
    "range" range / set_range [_]: CounterRanges,

    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-pad>
    "pad" pad / set_pad [_]: Pad,

    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-fallback>
    "fallback" fallback / set_fallback [_]: Fallback,

    /// <https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-symbols>
    "symbols" symbols / set_symbols [check_symbols]: Symbols,

    /// <https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-additive-symbols>
    "additive-symbols" additive_symbols /
        set_additive_symbols [check_additive_symbols]: AdditiveSymbols,

    /// <https://drafts.csswg.org/css-counter-styles/#counter-style-speak-as>
    "speak-as" speak_as / set_speak_as [_]: SpeakAs,
}

// Implements the special checkers for some setters.
// See <https://drafts.csswg.org/css-counter-styles/#the-csscounterstylerule-interface>
impl CounterStyleRuleData {
    /// Check that the system is effectively not changed. Only params
    /// of system descriptor is changeable.
    fn check_system(&self, value: &System) -> bool {
        mem::discriminant(self.resolved_system()) == mem::discriminant(value)
    }

    fn check_symbols(&self, value: &Symbols) -> bool {
        match *self.resolved_system() {
            // These two systems require at least two symbols.
            System::Numeric | System::Alphabetic => value.0.len() >= 2,
            // No symbols should be set for extends system.
            System::Extends(_) => false,
            _ => true,
        }
    }

    fn check_additive_symbols(&self, _value: &AdditiveSymbols) -> bool {
        match *self.resolved_system() {
            // No additive symbols should be set for extends system.
            System::Extends(_) => false,
            _ => true,
        }
    }
}

impl CounterStyleRuleData {
    /// Get the name of the counter style rule.
    pub fn name(&self) -> &CustomIdent {
        &self.name
    }

    /// Set the name of the counter style rule. Caller must ensure that
    /// the name is valid.
    pub fn set_name(&mut self, name: CustomIdent) {
        debug_assert!(is_valid_name_definition(&name));
        self.name = name;
    }

    /// Get the current generation of the counter style rule.
    pub fn generation(&self) -> u32 {
        self.generation.0
    }

    /// Get the system of this counter style rule, default to
    /// `symbolic` if not specified.
    pub fn resolved_system(&self) -> &System {
        match self.system {
            Some(ref system) => system,
            None => &System::Symbolic,
        }
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#counter-style-system>
#[derive(Clone, Debug, ToShmem)]
pub enum System {
    /// 'cyclic'
    Cyclic,
    /// 'numeric'
    Numeric,
    /// 'alphabetic'
    Alphabetic,
    /// 'symbolic'
    Symbolic,
    /// 'additive'
    Additive,
    /// 'fixed <integer>?'
    Fixed {
        /// '<integer>?'
        first_symbol_value: Option<Integer>,
    },
    /// 'extends <counter-style-name>'
    Extends(CustomIdent),
}

impl Parse for System {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input,
            "cyclic" => Ok(System::Cyclic),
            "numeric" => Ok(System::Numeric),
            "alphabetic" => Ok(System::Alphabetic),
            "symbolic" => Ok(System::Symbolic),
            "additive" => Ok(System::Additive),
            "fixed" => {
                let first_symbol_value = input.try(|i| Integer::parse(context, i)).ok();
                Ok(System::Fixed { first_symbol_value })
            },
            "extends" => {
                let other = parse_counter_style_name(input)?;
                Ok(System::Extends(other))
            },
        }
    }
}

impl ToCss for System {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            System::Cyclic => dest.write_str("cyclic"),
            System::Numeric => dest.write_str("numeric"),
            System::Alphabetic => dest.write_str("alphabetic"),
            System::Symbolic => dest.write_str("symbolic"),
            System::Additive => dest.write_str("additive"),
            System::Fixed { first_symbol_value } => {
                if let Some(value) = first_symbol_value {
                    dest.write_str("fixed ")?;
                    value.to_css(dest)
                } else {
                    dest.write_str("fixed")
                }
            },
            System::Extends(ref other) => {
                dest.write_str("extends ")?;
                other.to_css(dest)
            },
        }
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#typedef-symbol>
#[derive(
    Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToCss, ToShmem,
)]
#[repr(u8)]
pub enum Symbol {
    /// <string>
    String(crate::OwnedStr),
    /// <custom-ident>
    Ident(CustomIdent),
    // Not implemented:
    // /// <image>
    // Image(Image),
}

impl Parse for Symbol {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        match *input.next()? {
            Token::QuotedString(ref s) => Ok(Symbol::String(s.as_ref().to_owned().into())),
            Token::Ident(ref s) => Ok(Symbol::Ident(CustomIdent::from_ident(location, s, &[])?)),
            ref t => Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}

impl Symbol {
    /// Returns whether this symbol is allowed in symbols() function.
    pub fn is_allowed_in_symbols(&self) -> bool {
        match self {
            // Identifier is not allowed.
            &Symbol::Ident(_) => false,
            _ => true,
        }
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#counter-style-negative>
#[derive(Clone, Debug, ToCss, ToShmem)]
pub struct Negative(pub Symbol, pub Option<Symbol>);

impl Parse for Negative {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(Negative(
            Symbol::parse(context, input)?,
            input.try(|input| Symbol::parse(context, input)).ok(),
        ))
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#counter-style-range>
#[derive(Clone, Debug, ToCss, ToShmem)]
pub struct CounterRange {
    /// The start of the range.
    pub start: CounterBound,
    /// The end of the range.
    pub end: CounterBound,
}

/// <https://drafts.csswg.org/css-counter-styles/#counter-style-range>
///
/// Empty represents 'auto'
#[derive(Clone, Debug, ToCss, ToShmem)]
#[css(comma)]
pub struct CounterRanges(#[css(iterable, if_empty = "auto")] pub crate::OwnedSlice<CounterRange>);

/// A bound found in `CounterRanges`.
#[derive(Clone, Copy, Debug, ToCss, ToShmem)]
pub enum CounterBound {
    /// An integer bound.
    Integer(Integer),
    /// The infinite bound.
    Infinite,
}

impl Parse for CounterRanges {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            return Ok(CounterRanges(Default::default()));
        }

        let ranges = input.parse_comma_separated(|input| {
            let start = parse_bound(context, input)?;
            let end = parse_bound(context, input)?;
            if let (CounterBound::Integer(start), CounterBound::Integer(end)) = (start, end) {
                if start > end {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
            }
            Ok(CounterRange { start, end })
        })?;

        Ok(CounterRanges(ranges.into()))
    }
}

fn parse_bound<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
) -> Result<CounterBound, ParseError<'i>> {
    if let Ok(integer) = input.try(|input| Integer::parse(context, input)) {
        return Ok(CounterBound::Integer(integer));
    }
    input.expect_ident_matching("infinite")?;
    Ok(CounterBound::Infinite)
}

/// <https://drafts.csswg.org/css-counter-styles/#counter-style-pad>
#[derive(Clone, Debug, ToCss, ToShmem)]
pub struct Pad(pub Integer, pub Symbol);

impl Parse for Pad {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let pad_with = input.try(|input| Symbol::parse(context, input));
        let min_length = Integer::parse_non_negative(context, input)?;
        let pad_with = pad_with.or_else(|_| Symbol::parse(context, input))?;
        Ok(Pad(min_length, pad_with))
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#counter-style-fallback>
#[derive(Clone, Debug, ToCss, ToShmem)]
pub struct Fallback(pub CustomIdent);

impl Parse for Fallback {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(Fallback(parse_counter_style_name(input)?))
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-symbols>
#[derive(
    Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToCss, ToShmem,
)]
#[repr(C)]
pub struct Symbols(#[css(iterable)] pub crate::OwnedSlice<Symbol>);

impl Parse for Symbols {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut symbols = Vec::new();
        while let Ok(s) = input.try(|input| Symbol::parse(context, input)) {
            symbols.push(s);
        }
        if symbols.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(Symbols(symbols.into()))
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-additive-symbols>
#[derive(Clone, Debug, ToCss, ToShmem)]
#[css(comma)]
pub struct AdditiveSymbols(#[css(iterable)] pub crate::OwnedSlice<AdditiveTuple>);

impl Parse for AdditiveSymbols {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let tuples = Vec::<AdditiveTuple>::parse(context, input)?;
        // FIXME maybe? https://github.com/w3c/csswg-drafts/issues/1220
        if tuples
            .windows(2)
            .any(|window| window[0].weight <= window[1].weight)
        {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(AdditiveSymbols(tuples.into()))
    }
}

/// <integer> && <symbol>
#[derive(Clone, Debug, ToCss, ToShmem)]
pub struct AdditiveTuple {
    /// <integer>
    pub weight: Integer,
    /// <symbol>
    pub symbol: Symbol,
}

impl OneOrMoreSeparated for AdditiveTuple {
    type S = Comma;
}

impl Parse for AdditiveTuple {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let symbol = input.try(|input| Symbol::parse(context, input));
        let weight = Integer::parse_non_negative(context, input)?;
        let symbol = symbol.or_else(|_| Symbol::parse(context, input))?;
        Ok(Self { weight, symbol })
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#counter-style-speak-as>
#[derive(Clone, Debug, ToCss, ToShmem)]
pub enum SpeakAs {
    /// auto
    Auto,
    /// bullets
    Bullets,
    /// numbers
    Numbers,
    /// words
    Words,
    // /// spell-out, not supported, see bug 1024178
    // SpellOut,
    /// <counter-style-name>
    Other(CustomIdent),
}

impl Parse for SpeakAs {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut is_spell_out = false;
        let result = input.try(|input| {
            let ident = input.expect_ident().map_err(|_| ())?;
            match_ignore_ascii_case! { &*ident,
                "auto" => Ok(SpeakAs::Auto),
                "bullets" => Ok(SpeakAs::Bullets),
                "numbers" => Ok(SpeakAs::Numbers),
                "words" => Ok(SpeakAs::Words),
                "spell-out" => {
                    is_spell_out = true;
                    Err(())
                },
                _ => Err(()),
            }
        });
        if is_spell_out {
            // spell-out is not supported, but don’t parse it as a <counter-style-name>.
            // See bug 1024178.
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        result.or_else(|_| Ok(SpeakAs::Other(parse_counter_style_name(input)?)))
    }
}
