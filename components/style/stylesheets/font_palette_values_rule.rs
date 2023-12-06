/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The [`@font-palette-values`][font-palette-values] at-rule.
//!
//! [font-palette-values]: https://drafts.csswg.org/css-fonts/#font-palette-values

use crate::error_reporting::ContextualParseError;
#[cfg(feature = "gecko")]
use crate::gecko_bindings::{
    bindings::Gecko_AppendPaletteValueHashEntry,
    bindings::{Gecko_SetFontPaletteBase, Gecko_SetFontPaletteOverride},
    structs::gfx::FontPaletteValueSet,
    structs::gfx::FontPaletteValueSet_PaletteValues_kDark,
    structs::gfx::FontPaletteValueSet_PaletteValues_kLight,
};
use crate::parser::{Parse, ParserContext};
use crate::shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::stylesheets::font_feature_values_rule::parse_family_name_list;
use crate::values::computed::font::FamilyName;
use crate::values::specified::Color as SpecifiedColor;
use crate::values::specified::NonNegativeInteger;
use crate::values::DashedIdent;
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, Parser, QualifiedRuleParser, RuleBodyItemParser,
    RuleBodyParser, SourceLocation,
};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{Comma, OneOrMoreSeparated};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

#[allow(missing_docs)]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub struct FontPaletteOverrideColor {
    index: NonNegativeInteger,
    color: SpecifiedColor,
}

impl Parse for FontPaletteOverrideColor {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontPaletteOverrideColor, ParseError<'i>> {
        let index = NonNegativeInteger::parse(context, input)?;
        let location = input.current_source_location();
        let color = SpecifiedColor::parse(context, input)?;
        // Only absolute colors are accepted here.
        if let SpecifiedColor::Absolute { .. } = color {
            Ok(FontPaletteOverrideColor { index, color })
        } else {
            Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

impl ToCss for FontPaletteOverrideColor {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.index.to_css(dest)?;
        dest.write_char(' ')?;
        self.color.to_css(dest)
    }
}

impl OneOrMoreSeparated for FontPaletteOverrideColor {
    type S = Comma;
}

impl OneOrMoreSeparated for FamilyName {
    type S = Comma;
}

#[allow(missing_docs)]
#[derive(Clone, Debug, MallocSizeOf, Parse, PartialEq, ToCss, ToShmem)]
pub enum FontPaletteBase {
    Light,
    Dark,
    Index(NonNegativeInteger),
}

/// The [`@font-palette-values`][font-palette-values] at-rule.
///
/// [font-palette-values]: https://drafts.csswg.org/css-fonts/#font-palette-values
#[derive(Clone, Debug, PartialEq, ToShmem)]
pub struct FontPaletteValuesRule {
    /// Palette name.
    pub name: DashedIdent,
    /// Font family list for @font-palette-values rule.
    /// Family names cannot contain generic families. FamilyName
    /// also accepts only non-generic names.
    pub family_names: Vec<FamilyName>,
    /// The base palette.
    pub base_palette: Option<FontPaletteBase>,
    /// The list of override colors.
    pub override_colors: Vec<FontPaletteOverrideColor>,
    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl FontPaletteValuesRule {
    /// Creates an empty FontPaletteValuesRule with given location and name.
    fn new(name: DashedIdent, location: SourceLocation) -> Self {
        FontPaletteValuesRule {
            name,
            family_names: vec![],
            base_palette: None,
            override_colors: vec![],
            source_location: location,
        }
    }

    /// Parses a `FontPaletteValuesRule`.
    pub fn parse(
        context: &ParserContext,
        input: &mut Parser,
        name: DashedIdent,
        location: SourceLocation,
    ) -> Self {
        let mut rule = FontPaletteValuesRule::new(name, location);
        let mut parser = FontPaletteValuesDeclarationParser {
            context,
            rule: &mut rule,
        };
        let mut iter = RuleBodyParser::new(input, &mut parser);
        while let Some(declaration) = iter.next() {
            if let Err((error, slice)) = declaration {
                let location = error.location;
                let error =
                    ContextualParseError::UnsupportedFontPaletteValuesDescriptor(slice, error);
                context.log_css_error(location, error);
            }
        }
        rule
    }

    /// Prints inside of `@font-palette-values` block.
    fn value_to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if !self.family_names.is_empty() {
            dest.write_str("font-family: ")?;
            self.family_names.to_css(dest)?;
            dest.write_str("; ")?;
        }
        if let Some(base) = &self.base_palette {
            dest.write_str("base-palette: ")?;
            base.to_css(dest)?;
            dest.write_str("; ")?;
        }
        if !self.override_colors.is_empty() {
            dest.write_str("override-colors: ")?;
            self.override_colors.to_css(dest)?;
            dest.write_str("; ")?;
        }
        Ok(())
    }

    /// Convert to Gecko FontPaletteValueSet.
    #[cfg(feature = "gecko")]
    pub fn to_gecko_palette_value_set(&self, dest: *mut FontPaletteValueSet) {
        for ref family in self.family_names.iter() {
            let family = family.name.to_ascii_lowercase();
            let palette_values = unsafe {
                Gecko_AppendPaletteValueHashEntry(dest, family.as_ptr(), self.name.0.as_ptr())
            };
            if let Some(base_palette) = &self.base_palette {
                unsafe {
                    Gecko_SetFontPaletteBase(
                        palette_values,
                        match &base_palette {
                            FontPaletteBase::Light => FontPaletteValueSet_PaletteValues_kLight,
                            FontPaletteBase::Dark => FontPaletteValueSet_PaletteValues_kDark,
                            FontPaletteBase::Index(i) => i.0.value() as i32,
                        },
                    );
                }
            }
            for c in &self.override_colors {
                if let SpecifiedColor::Absolute(ref absolute) = c.color {
                    unsafe {
                        Gecko_SetFontPaletteOverride(
                            palette_values,
                            c.index.0.value(),
                            (&absolute.color) as *const _ as *mut _,
                        );
                    }
                }
            }
        }
    }
}

impl ToCssWithGuard for FontPaletteValuesRule {
    fn to_css(&self, _guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@font-palette-values ")?;
        self.name.to_css(&mut CssWriter::new(dest))?;
        dest.write_str(" { ")?;
        self.value_to_css(&mut CssWriter::new(dest))?;
        dest.write_char('}')
    }
}

/// Parser for declarations in `FontPaletteValuesRule`.
struct FontPaletteValuesDeclarationParser<'a> {
    context: &'a ParserContext<'a>,
    rule: &'a mut FontPaletteValuesRule,
}

impl<'a, 'i> AtRuleParser<'i> for FontPaletteValuesDeclarationParser<'a> {
    type Prelude = ();
    type AtRule = ();
    type Error = StyleParseErrorKind<'i>;
}

impl<'a, 'i> QualifiedRuleParser<'i> for FontPaletteValuesDeclarationParser<'a> {
    type Prelude = ();
    type QualifiedRule = ();
    type Error = StyleParseErrorKind<'i>;
}

fn parse_override_colors<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
) -> Result<Vec<FontPaletteOverrideColor>, ParseError<'i>> {
    input.parse_comma_separated(|i| FontPaletteOverrideColor::parse(context, i))
}

impl<'a, 'b, 'i> DeclarationParser<'i> for FontPaletteValuesDeclarationParser<'a> {
    type Declaration = ();
    type Error = StyleParseErrorKind<'i>;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i>> {
        match_ignore_ascii_case! { &*name,
            "font-family" => {
                self.rule.family_names = parse_family_name_list(self.context, input)?
            },
            "base-palette" => {
                self.rule.base_palette = Some(input.parse_entirely(|i| FontPaletteBase::parse(self.context, i))?)
            },
            "override-colors" => {
                self.rule.override_colors = parse_override_colors(self.context, input)?
            },
            _ => return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone()))),
        }
        Ok(())
    }
}

impl<'a, 'i> RuleBodyItemParser<'i, (), StyleParseErrorKind<'i>>
    for FontPaletteValuesDeclarationParser<'a>
{
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}
