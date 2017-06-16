/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@counter-style`][counter-style] at-rule.
//!
//! [counter-style]: https://drafts.csswg.org/css-counter-styles/

use Atom;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser};
use cssparser::{Parser, Token, serialize_identifier, BasicParseError, CompactCowStr};
use error_reporting::ContextualParseError;
#[cfg(feature = "gecko")] use gecko::rules::CounterStyleDescriptors;
#[cfg(feature = "gecko")] use gecko_bindings::structs::nsCSSCounterDesc;
use parser::{ParserContext, log_css_error, Parse};
use selectors::parser::SelectorParseError;
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::fmt;
use std::ops::Range;
use style_traits::{ToCss, OneOrMoreCommaSeparated, ParseError, StyleParseError};
use values::CustomIdent;

/// Parse the prelude of an @counter-style rule
pub fn parse_counter_style_name<'i, 't>(input: &mut Parser<'i, 't>) -> Result<CustomIdent, ParseError<'i>> {
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

                let ident = input.expect_ident()?;
                if let Some(&lower_cased) = predefined(&ident) {
                    Ok(CustomIdent(Atom::from(lower_cased)))
                } else {
                    // https://github.com/w3c/csswg-drafts/issues/1295 excludes "none"
                    CustomIdent::from_ident(ident, &["none"])
                }
            }
        }
    }
    include!("predefined.rs")
}

/// Parse the body (inside `{}`) of an @counter-style rule
pub fn parse_counter_style_body<'i, 't>(name: CustomIdent, context: &ParserContext, input: &mut Parser<'i, 't>)
                                        -> Result<CounterStyleRuleData, ParseError<'i>> {
    let start = input.position();
    let mut rule = CounterStyleRuleData::empty(name);
    {
        let parser = CounterStyleRuleParser {
            context: context,
            rule: &mut rule,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err(err) = declaration {
                let pos = err.span.start;
                let error = ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(
                    iter.input.slice(err.span), err.error);
                log_css_error(iter.input, pos, error, context);
            }
        }
    }
    let error = match *rule.system() {
        ref system @ System::Cyclic |
        ref system @ System::Fixed { .. } |
        ref system @ System::Symbolic |
        ref system @ System::Alphabetic |
        ref system @ System::Numeric
        if rule.symbols.is_none() => {
            let system = system.to_css_string();
            Some(ContextualParseError::InvalidCounterStyleWithoutSymbols(system))
        }
        ref system @ System::Alphabetic |
        ref system @ System::Numeric
        if rule.symbols().unwrap().0.len() < 2 => {
            let system = system.to_css_string();
            Some(ContextualParseError::InvalidCounterStyleNotEnoughSymbols(system))
        }
        System::Additive if rule.additive_symbols.is_none() => {
            Some(ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols)
        }
        System::Extends(_) if rule.symbols.is_some() => {
            Some(ContextualParseError::InvalidCounterStyleExtendsWithSymbols)
        }
        System::Extends(_) if rule.additive_symbols.is_some() => {
            Some(ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols)
        }
        _ => None
    };
    if let Some(error) = error {
        log_css_error(input, start, error, context);
        Err(StyleParseError::UnspecifiedError.into())
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
    type Prelude = ();
    type AtRule = ();
    type Error = SelectorParseError<'i, StyleParseError<'i>>;
}

macro_rules! accessor {
    (#[$doc: meta] $name: tt $ident: ident: $ty: ty = !) => {
        #[$doc]
        pub fn $ident(&self) -> Option<&$ty> {
            self.$ident.as_ref()
        }
    };

    (#[$doc: meta] $name: tt $ident: ident: $ty: ty = $initial: expr) => {
        #[$doc]
        pub fn $ident(&self) -> Cow<$ty> {
            if let Some(ref value) = self.$ident {
                Cow::Borrowed(value)
            } else {
                Cow::Owned($initial)
            }
        }
    }
}

macro_rules! counter_style_descriptors {
    (
        $( #[$doc: meta] $name: tt $ident: ident / $gecko_ident: ident: $ty: ty = $initial: tt )+
    ) => {
        /// An @counter-style rule
        #[derive(Clone, Debug)]
        pub struct CounterStyleRuleData {
            name: CustomIdent,
            $(
                #[$doc]
                $ident: Option<$ty>,
            )+
        }

        impl CounterStyleRuleData {
            fn empty(name: CustomIdent) -> Self {
                CounterStyleRuleData {
                    name: name,
                    $(
                        $ident: None,
                    )+
                }
            }

            /// Get the name of the counter style rule.
            pub fn name(&self) -> &CustomIdent {
                &self.name
            }

            $(
                accessor!(#[$doc] $name $ident: $ty = $initial);
            )+

            /// Convert to Gecko types
            #[cfg(feature = "gecko")]
            pub fn set_descriptors(self, descriptors: &mut CounterStyleDescriptors) {
                $(
                    if let Some(value) = self.$ident {
                        descriptors[nsCSSCounterDesc::$gecko_ident as usize].set_from(value)
                    }
                )*
            }
        }

        impl<'a, 'b, 'i> DeclarationParser<'i> for CounterStyleRuleParser<'a, 'b> {
            type Declaration = ();
            type Error = SelectorParseError<'i, StyleParseError<'i>>;

            fn parse_value<'t>(&mut self, name: CompactCowStr<'i>, input: &mut Parser<'i, 't>)
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
                        }
                    )*
                    _ => return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
                }
                Ok(())
            }
        }

        impl ToCssWithGuard for CounterStyleRuleData {
            fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
            where W: fmt::Write {
                dest.write_str("@counter-style ")?;
                self.name.to_css(dest)?;
                dest.write_str(" {\n")?;
                $(
                    if let Some(ref value) = self.$ident {
                        dest.write_str(concat!("  ", $name, ": "))?;
                        ToCss::to_css(value, dest)?;
                        dest.write_str(";\n")?;
                    }
                )+
                dest.write_str("}")
            }
        }
    }
}

counter_style_descriptors! {
    /// https://drafts.csswg.org/css-counter-styles/#counter-style-system
    "system" system / eCSSCounterDesc_System: System = {
        System::Symbolic
    }

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-negative
    "negative" negative / eCSSCounterDesc_Negative: Negative = {
        Negative(Symbol::String("-".to_owned()), None)
    }

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-prefix
    "prefix" prefix / eCSSCounterDesc_Prefix: Symbol = {
        Symbol::String("".to_owned())
    }

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-suffix
    "suffix" suffix / eCSSCounterDesc_Suffix: Symbol = {
        Symbol::String(". ".to_owned())
    }

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-range
    "range" range / eCSSCounterDesc_Range: Ranges = {
        Ranges(Vec::new())  // Empty Vec represents 'auto'
    }

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-pad
    "pad" pad / eCSSCounterDesc_Pad: Pad = {
        Pad(0, Symbol::String("".to_owned()))
    }

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-fallback
    "fallback" fallback / eCSSCounterDesc_Fallback: Fallback = {
        // FIXME https://bugzilla.mozilla.org/show_bug.cgi?id=1359323 use atom!()
        Fallback(CustomIdent(Atom::from("decimal")))
    }

    /// https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-symbols
    "symbols" symbols / eCSSCounterDesc_Symbols: Symbols = !

    /// https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-additive-symbols
    "additive-symbols" additive_symbols / eCSSCounterDesc_AdditiveSymbols: AdditiveSymbols = !

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-speak-as
    "speak-as" speak_as / eCSSCounterDesc_SpeakAs: SpeakAs = {
        SpeakAs::Auto
    }
}

/// https://drafts.csswg.org/css-counter-styles/#counter-style-system
#[derive(Debug, Clone)]
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
        first_symbol_value: Option<i32>
    },
    /// 'extends <counter-style-name>'
    Extends(CustomIdent),
}

impl Parse for System {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input.expect_ident()?,
            "cyclic" => Ok(System::Cyclic),
            "numeric" => Ok(System::Numeric),
            "alphabetic" => Ok(System::Alphabetic),
            "symbolic" => Ok(System::Symbolic),
            "additive" => Ok(System::Additive),
            "fixed" => {
                let first_symbol_value = input.try(|i| i.expect_integer()).ok();
                Ok(System::Fixed { first_symbol_value: first_symbol_value })
            }
            "extends" => {
                let other = parse_counter_style_name(input)?;
                Ok(System::Extends(other))
            }
        }
    }
}

impl ToCss for System {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            System::Cyclic => dest.write_str("cyclic"),
            System::Numeric => dest.write_str("numeric"),
            System::Alphabetic => dest.write_str("alphabetic"),
            System::Symbolic => dest.write_str("symbolic"),
            System::Additive => dest.write_str("additive"),
            System::Fixed { first_symbol_value } => {
                if let Some(value) = first_symbol_value {
                    write!(dest, "fixed {}", value)
                } else {
                    dest.write_str("fixed")
                }
            }
            System::Extends(ref other) => {
                dest.write_str("extends ")?;
                other.to_css(dest)
            }
        }
    }
}

/// https://drafts.csswg.org/css-counter-styles/#typedef-symbol
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// <string>
    String(String),
    /// <ident>
    Ident(String),
    // Not implemented:
    // /// <image>
    // Image(Image),
}

impl Parse for Symbol {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        match input.next() {
            Ok(Token::QuotedString(s)) => Ok(Symbol::String(s.into_owned())),
            Ok(Token::Ident(s)) => Ok(Symbol::Ident(s.into_owned())),
            Ok(t) => Err(BasicParseError::UnexpectedToken(t).into()),
            Err(e) => Err(e.into()),
        }
    }
}

impl ToCss for Symbol {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Symbol::String(ref s) => s.to_css(dest),
            Symbol::Ident(ref s) => serialize_identifier(s, dest),
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

/// https://drafts.csswg.org/css-counter-styles/#counter-style-negative
#[derive(Debug, Clone)]
pub struct Negative(pub Symbol, pub Option<Symbol>);

impl Parse for Negative {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Ok(Negative(
            Symbol::parse(context, input)?,
            input.try(|input| Symbol::parse(context, input)).ok(),
        ))
    }
}

impl ToCss for Negative {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)?;
        if let Some(ref symbol) = self.1 {
            dest.write_char(' ')?;
            symbol.to_css(dest)?
        }
        Ok(())
    }
}

/// https://drafts.csswg.org/css-counter-styles/#counter-style-range
///
/// Empty Vec represents 'auto'
#[derive(Debug, Clone)]
pub struct Ranges(pub Vec<Range<Option<i32>>>);

impl Parse for Ranges {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            Ok(Ranges(Vec::new()))
        } else {
            input.parse_comma_separated(|input| {
                let opt_start = parse_bound(input)?;
                let opt_end = parse_bound(input)?;
                if let (Some(start), Some(end)) = (opt_start, opt_end) {
                    if start > end {
                        return Err(StyleParseError::UnspecifiedError.into())
                    }
                }
                Ok(opt_start..opt_end)
            }).map(Ranges)
        }
    }
}

fn parse_bound<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Option<i32>, ParseError<'i>> {
    match input.next() {
        Ok(Token::Number { int_value: Some(v), .. }) => Ok(Some(v)),
        Ok(Token::Ident(ref ident)) if ident.eq_ignore_ascii_case("infinite") => Ok(None),
        Ok(t) => Err(BasicParseError::UnexpectedToken(t).into()),
        Err(e) => Err(e.into()),
    }
}

impl ToCss for Ranges {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            range_to_css(first, dest)?;
            for item in iter {
                dest.write_str(", ")?;
                range_to_css(item, dest)?;
            }
            Ok(())
        } else {
            dest.write_str("auto")
        }
    }
}

fn range_to_css<W>(range: &Range<Option<i32>>, dest: &mut W) -> fmt::Result
where W: fmt::Write {
    bound_to_css(range.start, dest)?;
    dest.write_char(' ')?;
    bound_to_css(range.end, dest)
}

fn bound_to_css<W>(range: Option<i32>, dest: &mut W) -> fmt::Result where W: fmt::Write {
    if let Some(finite) = range {
        write!(dest, "{}", finite)
    } else {
        dest.write_str("infinite")
    }
}

/// https://drafts.csswg.org/css-counter-styles/#counter-style-pad
#[derive(Debug, Clone)]
pub struct Pad(pub u32, pub Symbol);

impl Parse for Pad {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let pad_with = input.try(|input| Symbol::parse(context, input));
        let min_length = input.expect_integer()?;
        if min_length < 0 {
            return Err(StyleParseError::UnspecifiedError.into())
        }
        let pad_with = pad_with.or_else(|_| Symbol::parse(context, input))?;
        Ok(Pad(min_length as u32, pad_with))
    }
}

impl ToCss for Pad {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{} ", self.0)?;
        self.1.to_css(dest)
    }
}

/// https://drafts.csswg.org/css-counter-styles/#counter-style-fallback
#[derive(Debug, Clone)]
pub struct Fallback(pub CustomIdent);

impl Parse for Fallback {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        parse_counter_style_name(input).map(Fallback)
    }
}

impl ToCss for Fallback {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

/// https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-symbols
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbols(pub Vec<Symbol>);

impl Parse for Symbols {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let mut symbols = Vec::new();
        loop {
            if let Ok(s) = input.try(|input| Symbol::parse(context, input)) {
                symbols.push(s)
            } else {
                if symbols.is_empty() {
                    return Err(StyleParseError::UnspecifiedError.into())
                } else {
                    return Ok(Symbols(symbols))
                }
            }
        }
    }
}

impl ToCss for Symbols {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.0.iter();
        let first = iter.next().expect("expected at least one symbol");
        first.to_css(dest)?;
        for item in iter {
            dest.write_char(' ')?;
            item.to_css(dest)?;
        }
        Ok(())
    }
}

/// https://drafts.csswg.org/css-counter-styles/#descdef-counter-style-additive-symbols
#[derive(Debug, Clone)]
pub struct AdditiveSymbols(pub Vec<AdditiveTuple>);

impl Parse for AdditiveSymbols {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let tuples = Vec::<AdditiveTuple>::parse(context, input)?;
        // FIXME maybe? https://github.com/w3c/csswg-drafts/issues/1220
        if tuples.windows(2).any(|window| window[0].weight <= window[1].weight) {
            return Err(StyleParseError::UnspecifiedError.into())
        }
        Ok(AdditiveSymbols(tuples))
    }
}

impl ToCss for AdditiveSymbols {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

/// <integer> && <symbol>
#[derive(Debug, Clone)]
pub struct AdditiveTuple {
    /// <integer>
    pub weight: u32,
    /// <symbol>
    pub symbol: Symbol,
}

impl OneOrMoreCommaSeparated for AdditiveTuple {}

impl Parse for AdditiveTuple {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let symbol = input.try(|input| Symbol::parse(context, input));
        let weight = input.expect_integer()?;
        if weight < 0 {
            return Err(StyleParseError::UnspecifiedError.into())
        }
        let symbol = symbol.or_else(|_| Symbol::parse(context, input))?;
        Ok(AdditiveTuple {
            weight: weight as u32,
            symbol: symbol,
        })
    }
}

impl ToCss for AdditiveTuple {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{} ", self.weight)?;
        self.symbol.to_css(dest)
    }
}

/// https://drafts.csswg.org/css-counter-styles/#counter-style-speak-as
#[derive(Debug, Clone)]
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
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
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
                }
                _ => Err(()),
            }
        });
        if is_spell_out {
            // spell-out is not supported, but don’t parse it as a <counter-style-name>.
            // See bug 1024178.
            return Err(StyleParseError::UnspecifiedError.into())
        }
        result.or_else(|_| {
            Ok(SpeakAs::Other(parse_counter_style_name(input)?))
        })
    }
}

impl ToCss for SpeakAs {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            SpeakAs::Auto => dest.write_str("auto"),
            SpeakAs::Bullets => dest.write_str("bullets"),
            SpeakAs::Numbers => dest.write_str("numbers"),
            SpeakAs::Words => dest.write_str("words"),
            SpeakAs::Other(ref other) => other.to_css(dest),
        }
    }
}
