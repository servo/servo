/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@font-feature-values`][font-feature-values] at-rule.
//!
//! [font-feature-values]: https://drafts.csswg.org/css-fonts-3/#at-font-feature-values-rule

use Atom;
use computed_values::font_family::FamilyName;
use cssparser::{AtRuleParser, AtRuleType, BasicParseError, DeclarationListParser, DeclarationParser, Parser};
use cssparser::{CowRcStr, RuleListParser, SourceLocation, QualifiedRuleParser, Token, serialize_identifier};
use error_reporting::ContextualParseError;
use parser::{ParserContext, log_css_error, Parse};
use selectors::parser::SelectorParseError;
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::{ParseError, StyleParseError, ToCss};
use stylesheets::CssRuleType;

/// A @font-feature-values block declaration.
/// It is `<ident>: <integer>+`.
/// This struct can take 3 value types.
/// - `SingleValue` is to keep just one unsigned integer value.
/// - `PairValues` is to keep one or two unsigned integer values.
/// - `VectorValues` is to keep a list of unsigned integer values.
#[derive(Clone, Debug, PartialEq)]
pub struct FFVDeclaration<T> {
    /// An `<ident>` for declaration name.
    pub name: Atom,
    /// An `<integer>+` for declaration value.
    pub value: T,
}

impl<T: ToCss> ToCss for FFVDeclaration<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_identifier(&self.name.to_string(), dest)?;
        dest.write_str(": ")?;
        self.value.to_css(dest)?;
        dest.write_str(";")
    }
}

/// A @font-feature-values block declaration value that keeps one value.
#[derive(Clone, Debug, PartialEq)]
pub struct SingleValue(pub u32);

impl Parse for SingleValue {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<SingleValue, ParseError<'i>> {
        match *input.next()? {
            Token::Number { int_value: Some(v), .. } if v >= 0 => Ok(SingleValue(v as u32)),
            ref t => Err(BasicParseError::UnexpectedToken(t.clone()).into()),
        }
    }
}

impl ToCss for SingleValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}", self.0)
    }
}

/// A @font-feature-values block declaration value that keeps one or two values.
#[derive(Clone, Debug, PartialEq)]
pub struct PairValues(pub u32, pub Option<u32>);

impl Parse for PairValues {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<PairValues, ParseError<'i>> {
        let first = match *input.next()? {
            Token::Number { int_value: Some(a), .. } if a >= 0 => a as u32,
            ref t => return Err(BasicParseError::UnexpectedToken(t.clone()).into()),
        };
        match input.next() {
            Ok(&Token::Number { int_value: Some(b), .. }) if b >= 0 => {
                Ok(PairValues(first, Some(b as u32)))
            }
            // It can't be anything other than number.
            Ok(t) => Err(BasicParseError::UnexpectedToken(t.clone()).into()),
            // It can be just one value.
            Err(_) => Ok(PairValues(first, None))
        }
    }
}

impl ToCss for PairValues {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}", self.0)?;
        if let Some(second) = self.1 {
            write!(dest, " {}", second)?;
        }
        Ok(())
    }
}

/// A @font-feature-values block declaration value that keeps a list of values.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorValues(pub Vec<u32>);

impl Parse for VectorValues {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<VectorValues, ParseError<'i>> {
        let mut vec = vec![];
        loop {
            match input.next() {
                Ok(&Token::Number { int_value: Some(a), .. }) if a >= 0 => {
                    vec.push(a as u32);
                },
                // It can't be anything other than number.
                Ok(t) => return Err(BasicParseError::UnexpectedToken(t.clone()).into()),
                Err(_) => break,
            }
        }

        if vec.len() == 0 {
            return Err(BasicParseError::EndOfInput.into());
        }

        Ok(VectorValues(vec))
    }
}

impl ToCss for VectorValues {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.0.iter();
        let first = iter.next();
        if let Some(first) = first {
            write!(dest, "{}", first)?;
            for value in iter {
                dest.write_str(" ")?;
                write!(dest, "{}", value)?;
            }
        }
        Ok(())
    }
}

/// Parses a list of `FamilyName`s.
pub fn parse_family_name_list<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                  -> Result<Vec<FamilyName>, ParseError<'i>> {
    input.parse_comma_separated(|i| FamilyName::parse(context, i)).map_err(|e| e.into())
}

/// @font-feature-values inside block parser. Parses a list of `FFVDeclaration`.
/// (`<ident>: <integer>+`)
struct FFVDeclarationsParser<'a, 'b: 'a, T: 'a> {
    context: &'a ParserContext<'b>,
    declarations: &'a mut Vec<FFVDeclaration<T>>,
}

/// Default methods reject all at rules.
impl<'a, 'b, 'i, T> AtRuleParser<'i> for FFVDeclarationsParser<'a, 'b, T> {
    type Prelude = ();
    type AtRule = ();
    type Error = SelectorParseError<'i, StyleParseError<'i>>;
}

impl<'a, 'b, 'i, T> DeclarationParser<'i> for FFVDeclarationsParser<'a, 'b, T>
    where T: Parse
{
    type Declaration = ();
    type Error = SelectorParseError<'i, StyleParseError<'i>>;

    fn parse_value<'t>(&mut self, name: CowRcStr<'i>, input: &mut Parser<'i, 't>)
                       -> Result<(), ParseError<'i>> {
        let value = input.parse_entirely(|i| T::parse(self.context, i))?;
        let new = FFVDeclaration {
            name: Atom::from(&*name),
            value: value,
        };
        update_or_push(&mut self.declarations, new);
        Ok(())
    }
}

macro_rules! font_feature_values_blocks {
    (
        blocks = [
            $( #[$doc: meta] $name: tt $ident: ident / $ident_camel: ident: $ty: ty, )*
        ]
    ) => {
        /// The [`@font-feature-values`][font-feature-values] at-rule.
        ///
        /// [font-feature-values]: https://drafts.csswg.org/css-fonts-3/#at-font-feature-values-rule
        #[derive(Clone, Debug, PartialEq)]
        pub struct FontFeatureValuesRule {
            /// Font family list for @font-feature-values rule.
            /// Family names cannot contain generic families. FamilyName
            /// also accepts only non-generic names.
            pub family_names: Vec<FamilyName>,
            $(
                #[$doc]
                pub $ident: Vec<FFVDeclaration<$ty>>,
            )*
            /// The line and column of the rule's source code.
            pub source_location: SourceLocation,
        }

        impl FontFeatureValuesRule {
            /// Creates an empty FontFeatureValuesRule with given location and family name list.
            fn new(family_names: Vec<FamilyName>, location: SourceLocation) -> Self {
                FontFeatureValuesRule {
                    family_names: family_names,
                    $(
                        $ident: vec![],
                    )*
                    source_location: location,
                }
            }

            /// Parses a `FontFeatureValuesRule`.
            pub fn parse(context: &ParserContext, input: &mut Parser,
                         family_names: Vec<FamilyName>, location: SourceLocation)
                         -> FontFeatureValuesRule {
                let mut rule = FontFeatureValuesRule::new(family_names, location);

                {
                    let mut iter = RuleListParser::new_for_nested_rule(input, FontFeatureValuesRuleParser {
                        context: context,
                        rule: &mut rule,
                    });
                    while let Some(result) = iter.next() {
                        if let Err(err) = result {
                            let pos = err.span.start;
                            let error = ContextualParseError::UnsupportedRule(
                                iter.input.slice(err.span), err.error);
                            log_css_error(iter.input, pos, error, context);
                        }
                    }
                }
                rule
            }

            /// Prints font family names.
            pub fn font_family_to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.family_names.iter();
                iter.next().unwrap().to_css(dest)?;
                for val in iter {
                    dest.write_str(", ")?;
                    val.to_css(dest)?;
                }
                Ok(())
            }

            /// Prints inside of `@font-feature-values` block.
            pub fn value_to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                $(
                    if self.$ident.len() > 0 {
                        dest.write_str(concat!("@", $name, " {\n"))?;
                        let iter = self.$ident.iter();
                        for val in iter {
                            val.to_css(dest)?;
                            dest.write_str("\n")?
                        }
                        dest.write_str("}\n")?
                    }
                )*
                Ok(())
            }
        }

        impl ToCssWithGuard for FontFeatureValuesRule {
            fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
                where W: fmt::Write
            {
                dest.write_str("@font-feature-values ")?;
                self.font_family_to_css(dest)?;
                dest.write_str(" {\n")?;
                self.value_to_css(dest)?;
                dest.write_str("}")
            }
        }

        /// Updates with new value if same `ident` exists, otherwise pushes to the vector.
        fn update_or_push<T>(vec: &mut Vec<FFVDeclaration<T>>, element: FFVDeclaration<T>) {
            let position = vec.iter().position(|ref val| val.name == element.name);
            if let Some(index) = position {
                vec[index].value = element.value;
            } else {
                vec.push(element);
            }
        }

        /// Keeps the information about block type like @swash, @styleset etc.
        enum BlockType {
            $(
                $ident_camel,
            )*
        }

        /// Parser for `FontFeatureValuesRule`. Parses all blocks
        /// <feature-type> {
        ///   <feature-value-declaration-list>
        /// }
        /// <feature-type> = @stylistic | @historical-forms | @styleset |
        /// @character-variant | @swash | @ornaments | @annotation
        struct FontFeatureValuesRuleParser<'a> {
            context: &'a ParserContext<'a>,
            rule: &'a mut FontFeatureValuesRule,
        }

        /// Default methods reject all qualified rules.
        impl<'a, 'i> QualifiedRuleParser<'i> for FontFeatureValuesRuleParser<'a> {
            type Prelude = ();
            type QualifiedRule = ();
            type Error = SelectorParseError<'i, StyleParseError<'i>>;
        }

        impl<'a, 'i> AtRuleParser<'i> for FontFeatureValuesRuleParser<'a> {
            type Prelude = BlockType;
            type AtRule = ();
            type Error = SelectorParseError<'i, StyleParseError<'i>>;

            fn parse_prelude<'t>(&mut self,
                                 name: CowRcStr<'i>,
                                 _input: &mut Parser<'i, 't>)
                                 -> Result<AtRuleType<Self::Prelude, Self::AtRule>, ParseError<'i>> {
                match_ignore_ascii_case! { &*name,
                    $(
                        $name => Ok(AtRuleType::WithBlock(BlockType::$ident_camel)),
                    )*
                    _ => Err(BasicParseError::AtRuleBodyInvalid.into()),
                }
            }

            fn parse_block<'t>(&mut self, prelude: Self::Prelude, input: &mut Parser<'i, 't>)
                               -> Result<Self::AtRule, ParseError<'i>> {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::FontFeatureValues));
                match prelude {
                    $(
                        BlockType::$ident_camel => {
                            let parser = FFVDeclarationsParser {
                                context: &context,
                                declarations: &mut self.rule.$ident,
                            };

                            let mut iter = DeclarationListParser::new(input, parser);
                            while let Some(declaration) = iter.next() {
                                if let Err(err) = declaration {
                                    let pos = err.span.start;
                                    let error = ContextualParseError::UnsupportedKeyframePropertyDeclaration(
                                        iter.input.slice(err.span), err.error);
                                    log_css_error(iter.input, pos, error, &context);
                                }
                            }
                        },
                    )*
                }

                Ok(())
            }
        }
    }
}

font_feature_values_blocks! {
    blocks = [
        #[doc = "A @swash blocksck. \
                 Specifies a feature name that will work with the swash() \
                 functional notation of font-variant-alternates."]
        "swash" swash / Swash: SingleValue,

        #[doc = "A @stylistic block. \
                 Specifies a feature name that will work with the annotation() \
                 functional notation of font-variant-alternates."]
        "stylistic" stylistic / Stylistic: SingleValue,

        #[doc = "A @ornaments block. \
                 Specifies a feature name that will work with the ornaments() ] \
                 functional notation of font-variant-alternates."]
        "ornaments" ornaments / Ornaments: SingleValue,

        #[doc = "A @annotation block. \
                 Specifies a feature name that will work with the stylistic() \
                 functional notation of font-variant-alternates."]
        "annotation" annotation / Annotation: SingleValue,

        #[doc = "A @character-variant block. \
                 Specifies a feature name that will work with the styleset() \
                 functional notation of font-variant-alternates. The value can be a pair."]
        "character-variant" character_variant / CharacterVariant: PairValues,

        #[doc = "A @styleset block. \
                 Specifies a feature name that will work with the character-variant() \
                 functional notation of font-variant-alternates. The value can be a list."]
        "styleset" styleset / Styleset: VectorValues,
    ]
}
