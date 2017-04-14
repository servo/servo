/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@counter-style`][counter-style] at-rule.
//!
//! [counter-style]: https://drafts.csswg.org/css-counter-styles/

use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser, Token};
use cssparser::{serialize_string, serialize_identifier};
#[cfg(feature = "gecko")] use gecko::rules::CounterStyleDescriptors;
#[cfg(feature = "gecko")] use gecko_bindings::structs::nsCSSCounterDesc;
use parser::{ParserContext, log_css_error, Parse};
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::ToCss;
use values::CustomIdent;

/// Parse the prelude of an @counter-style rule
pub fn parse_counter_style_name(input: &mut Parser) -> Result<CustomIdent, ()> {
    CustomIdent::from_ident(input.expect_ident()?, &["decimal", "none"])
}

/// Parse the body (inside `{}`) of an @counter-style rule
pub fn parse_counter_style_body(name: CustomIdent, context: &ParserContext, input: &mut Parser)
                            -> Result<CounterStyleRule, ()> {
    let mut rule = CounterStyleRule::initial(name);
    {
        let parser = CounterStyleRuleParser {
            context: context,
            rule: &mut rule,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err(range) = declaration {
                let pos = range.start;
                let message = format!("Unsupported @counter-style descriptor declaration: '{}'",
                                      iter.input.slice(range));
                log_css_error(iter.input, pos, &*message, context);
            }
        }
    }
    Ok(rule)
}

struct CounterStyleRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    rule: &'a mut CounterStyleRule,
}

/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for CounterStyleRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = ();
}


macro_rules! counter_style_descriptors {
    (
        $( #[$doc: meta] $name: tt $ident: ident / $gecko_ident: ident: $ty: ty = $initial: expr; )+
    ) => {
        /// An @counter-style rule
        #[derive(Debug)]
        pub struct CounterStyleRule {
            name: CustomIdent,
            $(
                #[$doc]
                $ident: $ty,
            )+
        }

        impl CounterStyleRule {
            fn initial(name: CustomIdent) -> Self {
                CounterStyleRule {
                    name: name,
                    $(
                        $ident: $initial,
                    )+
                }
            }

            /// Convert to Gecko types
            #[cfg(feature = "gecko")]
            pub fn set_descriptors(&self, descriptors: &mut CounterStyleDescriptors) {
                $(
                    descriptors[nsCSSCounterDesc::$gecko_ident as usize].set_from(&self.$ident);
                )*
            }
        }

       impl<'a, 'b> DeclarationParser for CounterStyleRuleParser<'a, 'b> {
            type Declaration = ();

            fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<(), ()> {
                match_ignore_ascii_case! { name,
                    $(
                        $name => {
                            // DeclarationParser also calls parse_entirely
                            // so we’d normally not need to,
                            // but in this case we do because we set the value as a side effect
                            // rather than returning it.
                            let value = input.parse_entirely(|i| Parse::parse(self.context, i))?;
                            self.rule.$ident = value
                        }
                    )*
                    _ => return Err(())
                }
                Ok(())
            }
        }

        impl ToCssWithGuard for CounterStyleRule {
            fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
            where W: fmt::Write {
                dest.write_str("@counter-style ")?;
                self.name.to_css(dest)?;
                dest.write_str(" {\n")?;
                $(
                    dest.write_str(concat!("  ", $name, ": "))?;
                    ToCss::to_css(&self.$ident, dest)?;
                    dest.write_str(";\n")?;
                )+
                dest.write_str("}")
            }
        }
    }
}

counter_style_descriptors! {
    /// https://drafts.csswg.org/css-counter-styles/#counter-style-system
    "system" system / eCSSCounterDesc_System: System = System::Symbolic;

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-negative
    "negative" negative / eCSSCounterDesc_Negative: Negative =
        Negative(Symbol::String("-".to_owned()), None);

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-prefix
    "prefix" prefix / eCSSCounterDesc_Prefix: Symbol = Symbol::String("".to_owned());

    /// https://drafts.csswg.org/css-counter-styles/#counter-style-suffix
    "suffix" suffix / eCSSCounterDesc_Suffix: Symbol = Symbol::String(". ".to_owned());
}

/// https://drafts.csswg.org/css-counter-styles/#counter-style-system
#[derive(Debug)]
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
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { &input.expect_ident()?,
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
            _ => Err(())
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
                dest.write_str("symbolic ")?;
                other.to_css(dest)
            }
        }
    }
}

/// https://drafts.csswg.org/css-counter-styles/#typedef-symbol
#[derive(Debug)]
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
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.next() {
            Ok(Token::QuotedString(s)) => Ok(Symbol::String(s.into_owned())),
            Ok(Token::Ident(s)) => Ok(Symbol::Ident(s.into_owned())),
            _ => Err(())
        }
    }
}

impl ToCss for Symbol {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Symbol::String(ref s) => serialize_string(s, dest),
            Symbol::Ident(ref s) => serialize_identifier(s, dest),
        }
    }
}

/// https://drafts.csswg.org/css-counter-styles/#counter-style-negative
#[derive(Debug)]
pub struct Negative(pub Symbol, pub Option<Symbol>);

impl Parse for Negative {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
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
