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
use error_reporting::{ContextualParseError, ParseErrorReporter};
#[cfg(feature = "gecko")]
use gecko_bindings::bindings::Gecko_AppendFeatureValueHashEntry;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::{self, gfxFontFeatureValueSet, nsTArray};
use parser::{ParserContext, ParserErrorContext, Parse};
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

/// A trait for @font-feature-values rule to gecko values conversion.
#[cfg(feature = "gecko")]
pub trait ToGeckoFontFeatureValues {
    /// Sets the equivalent of declaration to gecko `nsTArray<u32>` array.
    fn to_gecko_font_feature_values(&self, array: &mut nsTArray<u32>);
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

#[cfg(feature = "gecko")]
impl ToGeckoFontFeatureValues for SingleValue {
    fn to_gecko_font_feature_values(&self, array: &mut nsTArray<u32>) {
        unsafe { array.set_len_pod(1); }
        array[0] = self.0 as u32;
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

#[cfg(feature = "gecko")]
impl ToGeckoFontFeatureValues for PairValues {
    fn to_gecko_font_feature_values(&self, array: &mut nsTArray<u32>) {
        let len = if self.1.is_some() { 2 } else { 1 };

        unsafe { array.set_len_pod(len); }
        array[0] = self.0 as u32;
        if let Some(second) = self.1 {
            array[1] = second as u32;
        };
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

#[cfg(feature = "gecko")]
impl ToGeckoFontFeatureValues for VectorValues {
    fn to_gecko_font_feature_values(&self, array: &mut nsTArray<u32>) {
        unsafe { array.set_len_pod(self.0.len() as u32); }
        for (dest, value) in array.iter_mut().zip(self.0.iter()) {
            *dest = *value;
        }
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
    type PreludeNoBlock = ();
    type PreludeBlock = ();
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
            name: Atom::from(&*name).to_ascii_lowercase(),
            value: value,
        };
        update_or_push(&mut self.declarations, new);
        Ok(())
    }
}

macro_rules! font_feature_values_blocks {
    (
        blocks = [
            $( #[$doc: meta] $name: tt $ident: ident / $ident_camel: ident / $gecko_enum: ident: $ty: ty, )*
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
            pub fn parse<R>(context: &ParserContext,
                            error_context: &ParserErrorContext<R>,
                            input: &mut Parser,
                            family_names: Vec<FamilyName>,
                            location: SourceLocation)
                            -> FontFeatureValuesRule
                where R: ParseErrorReporter
            {
                let mut rule = FontFeatureValuesRule::new(family_names, location);

                {
                    let mut iter = RuleListParser::new_for_nested_rule(input, FontFeatureValuesRuleParser {
                        context: context,
                        error_context: error_context,
                        rule: &mut rule,
                    });
                    while let Some(result) = iter.next() {
                        if let Err(err) = result {
                            let error = ContextualParseError::UnsupportedRule(err.slice, err.error);
                            context.log_css_error(error_context, err.location, error);
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

            /// Returns length of all at-rules.
            pub fn len(&self) -> usize {
                let mut len = 0;
                $(
                    len += self.$ident.len();
                )*
                len
            }

            /// Convert to Gecko gfxFontFeatureValueSet.
            #[cfg(feature = "gecko")]
            pub fn set_at_rules(&self, dest: *mut gfxFontFeatureValueSet) {
                for ref family in self.family_names.iter() {
                    let family = family.name.to_ascii_lowercase();
                    $(
                        if self.$ident.len() > 0 {
                            for val in self.$ident.iter() {
                                let array = unsafe {
                                    Gecko_AppendFeatureValueHashEntry(
                                        dest,
                                        family.as_ptr(),
                                        structs::$gecko_enum,
                                        val.name.as_ptr()
                                    )
                                };
                                unsafe {
                                    val.value.to_gecko_font_feature_values(&mut *array);
                                }
                            }
                        }
                    )*
                }
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
        struct FontFeatureValuesRuleParser<'a, R: 'a> {
            context: &'a ParserContext<'a>,
            error_context: &'a ParserErrorContext<'a, R>,
            rule: &'a mut FontFeatureValuesRule,
        }

        /// Default methods reject all qualified rules.
        impl<'a, 'i, R: ParseErrorReporter> QualifiedRuleParser<'i> for FontFeatureValuesRuleParser<'a, R> {
            type Prelude = ();
            type QualifiedRule = ();
            type Error = SelectorParseError<'i, StyleParseError<'i>>;
        }

        impl<'a, 'i, R: ParseErrorReporter> AtRuleParser<'i> for FontFeatureValuesRuleParser<'a, R> {
            type PreludeNoBlock = ();
            type PreludeBlock = BlockType;
            type AtRule = ();
            type Error = SelectorParseError<'i, StyleParseError<'i>>;

            fn parse_prelude<'t>(&mut self,
                                 name: CowRcStr<'i>,
                                 _input: &mut Parser<'i, 't>)
                                 -> Result<AtRuleType<(), BlockType>, ParseError<'i>> {
                match_ignore_ascii_case! { &*name,
                    $(
                        $name => Ok(AtRuleType::WithBlock(BlockType::$ident_camel)),
                    )*
                    _ => Err(BasicParseError::AtRuleBodyInvalid.into()),
                }
            }

            fn parse_block<'t>(
                &mut self,
                prelude: BlockType,
                input: &mut Parser<'i, 't>
            ) -> Result<Self::AtRule, ParseError<'i>> {
                debug_assert_eq!(self.context.rule_type(), CssRuleType::FontFeatureValues);
                match prelude {
                    $(
                        BlockType::$ident_camel => {
                            let parser = FFVDeclarationsParser {
                                context: &self.context,
                                declarations: &mut self.rule.$ident,
                            };

                            let mut iter = DeclarationListParser::new(input, parser);
                            while let Some(declaration) = iter.next() {
                                if let Err(err) = declaration {
                                    let error = ContextualParseError::UnsupportedKeyframePropertyDeclaration(
                                        err.slice, err.error);
                                    self.context.log_css_error(self.error_context, err.location, error);
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
        "swash" swash / Swash / NS_FONT_VARIANT_ALTERNATES_SWASH: SingleValue,

        #[doc = "A @stylistic block. \
                 Specifies a feature name that will work with the annotation() \
                 functional notation of font-variant-alternates."]
        "stylistic" stylistic / Stylistic / NS_FONT_VARIANT_ALTERNATES_STYLISTIC: SingleValue,

        #[doc = "A @ornaments block. \
                 Specifies a feature name that will work with the ornaments() ] \
                 functional notation of font-variant-alternates."]
        "ornaments" ornaments / Ornaments / NS_FONT_VARIANT_ALTERNATES_ORNAMENTS: SingleValue,

        #[doc = "A @annotation block. \
                 Specifies a feature name that will work with the stylistic() \
                 functional notation of font-variant-alternates."]
        "annotation" annotation / Annotation / NS_FONT_VARIANT_ALTERNATES_ANNOTATION: SingleValue,

        #[doc = "A @character-variant block. \
                 Specifies a feature name that will work with the styleset() \
                 functional notation of font-variant-alternates. The value can be a pair."]
        "character-variant" character_variant / CharacterVariant / NS_FONT_VARIANT_ALTERNATES_CHARACTER_VARIANT:
            PairValues,

        #[doc = "A @styleset block. \
                 Specifies a feature name that will work with the character-variant() \
                 functional notation of font-variant-alternates. The value can be a list."]
        "styleset" styleset / Styleset / NS_FONT_VARIANT_ALTERNATES_STYLESET: VectorValues,
    ]
}
