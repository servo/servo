/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@font-face`][ff] at-rule.
//!
//! [ff]: https://drafts.csswg.org/css-fonts/#at-font-face-rule

use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser};
use cssparser::{CowRcStr, SourceLocation};
#[cfg(feature = "gecko")]
use cssparser::UnicodeRange;
use error_reporting::ContextualParseError;
use parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use properties::longhands::font_language_override;
use selectors::parser::SelectorParseErrorKind;
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt::{self, Write};
use str::CssStringWriter;
use style_traits::{Comma, CssWriter, OneOrMoreSeparated, ParseError};
use style_traits::{StyleParseErrorKind, ToCss};
use style_traits::values::SequenceWriter;
use values::computed::font::FamilyName;
use values::generics::font::FontStyle as GenericFontStyle;
use values::specified::Angle;
use values::specified::font::{AbsoluteFontWeight, FontStretch as SpecifiedFontStretch};
#[cfg(feature = "gecko")]
use values::specified::font::{SpecifiedFontFeatureSettings, SpecifiedFontVariationSettings};
use values::specified::font::SpecifiedFontStyle;
use values::specified::url::SpecifiedUrl;

/// A source for a font-face rule.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, ToCss)]
pub enum Source {
    /// A `url()` source.
    Url(UrlSource),
    /// A `local()` source.
    #[css(function)]
    Local(FamilyName),
}

impl OneOrMoreSeparated for Source {
    type S = Comma;
}

/// A `UrlSource` represents a font-face source that has been specified with a
/// `url()` function.
///
/// <https://drafts.csswg.org/css-fonts/#src-desc>
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UrlSource {
    /// The specified url.
    pub url: SpecifiedUrl,
    /// The format hints specified with the `format()` function.
    pub format_hints: Vec<String>,
}

impl ToCss for UrlSource {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.url.to_css(dest)?;
        if !self.format_hints.is_empty() {
            dest.write_str(" format(")?;
            {
                let mut writer = SequenceWriter::new(dest, ", ");
                for hint in self.format_hints.iter() {
                    writer.item(hint)?;
                }
            }
            dest.write_char(')')?;
        }
        Ok(())
    }
}

/// A font-display value for a @font-face rule.
/// The font-display descriptor determines how a font face is displayed based
/// on whether and when it is downloaded and ready to use.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToComputedValue, ToCss)]
pub enum FontDisplay {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

/// The font-weight descriptor:
///
/// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-weight
#[derive(Clone, Debug, PartialEq, ToCss)]
pub struct FontWeight(pub AbsoluteFontWeight, pub Option<AbsoluteFontWeight>);

impl Parse for FontWeight {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let first = AbsoluteFontWeight::parse(context, input)?;
        let second =
            input.try(|input| AbsoluteFontWeight::parse(context, input)).ok();
        Ok(FontWeight(first, second))
    }
}

/// The font-stretch descriptor:
///
/// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-stretch
#[derive(Clone, Debug, PartialEq, ToCss)]
pub struct FontStretch(pub SpecifiedFontStretch, pub Option<SpecifiedFontStretch>);

impl Parse for FontStretch {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let first = SpecifiedFontStretch::parse(context, input)?;
        let second =
            input.try(|input| SpecifiedFontStretch::parse(context, input)).ok();
        Ok(FontStretch(first, second))
    }
}

/// The font-style descriptor:
///
/// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-style
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(Angle, Angle),
}

impl Parse for FontStyle {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let style = SpecifiedFontStyle::parse(context, input)?;
        Ok(match style {
            GenericFontStyle::Normal => FontStyle::Normal,
            GenericFontStyle::Italic => FontStyle::Italic,
            GenericFontStyle::Oblique(angle) => {
                let second_angle = input.try(|input| {
                    SpecifiedFontStyle::parse_angle(context, input)
                }).unwrap_or_else(|_| angle.clone());

                FontStyle::Oblique(angle, second_angle)
            }
        })
    }
}

impl ToCss for FontStyle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            FontStyle::Normal => dest.write_str("normal"),
            FontStyle::Italic => dest.write_str("italic"),
            FontStyle::Oblique(ref first, ref second) => {
                dest.write_str("oblique")?;
                if *first != SpecifiedFontStyle::default_angle() || first != second {
                    dest.write_char(' ')?;
                    first.to_css(dest)?;
                }
                if first != second {
                    dest.write_char(' ')?;
                    second.to_css(dest)?;
                }
                Ok(())
            }
        }
    }
}

/// Parse the block inside a `@font-face` rule.
///
/// Note that the prelude parsing code lives in the `stylesheets` module.
pub fn parse_font_face_block(
    context: &ParserContext,
    input: &mut Parser,
    location: SourceLocation,
) -> FontFaceRuleData {
    let mut rule = FontFaceRuleData::empty(location);
    {
        let parser = FontFaceRuleParser {
            context: context,
            rule: &mut rule,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err((error, slice)) = declaration {
                let location = error.location;
                let error = ContextualParseError::UnsupportedFontFaceDescriptor(slice, error);
                context.log_css_error(location, error)
            }
        }
    }
    rule
}

/// A @font-face rule that is known to have font-family and src declarations.
#[cfg(feature = "servo")]
pub struct FontFace<'a>(&'a FontFaceRuleData);

/// A list of effective sources that we send over through IPC to the font cache.
#[cfg(feature = "servo")]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct EffectiveSources(Vec<Source>);

#[cfg(feature = "servo")]
impl<'a> FontFace<'a> {
    /// Returns the list of effective sources for that font-face, that is the
    /// sources which don't list any format hint, or the ones which list at
    /// least "truetype" or "opentype".
    pub fn effective_sources(&self) -> EffectiveSources {
        EffectiveSources(
            self.sources()
                .iter()
                .rev()
                .filter(|source| {
                    if let Source::Url(ref url_source) = **source {
                        let hints = &url_source.format_hints;
                        // We support only opentype fonts and truetype is an alias for
                        // that format. Sources without format hints need to be
                        // downloaded in case we support them.
                        hints.is_empty() ||
                            hints.iter().any(|hint| {
                                hint == "truetype" || hint == "opentype" || hint == "woff"
                            })
                    } else {
                        true
                    }
                })
                .cloned()
                .collect(),
        )
    }
}

#[cfg(feature = "servo")]
impl Iterator for EffectiveSources {
    type Item = Source;
    fn next(&mut self) -> Option<Source> {
        self.0.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

struct FontFaceRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    rule: &'a mut FontFaceRuleData,
}

/// Default methods reject all at rules.
impl<'a, 'b, 'i> AtRuleParser<'i> for FontFaceRuleParser<'a, 'b> {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = ();
    type Error = StyleParseErrorKind<'i>;
}

impl Parse for Source {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Source, ParseError<'i>> {
        if input
            .try(|input| input.expect_function_matching("local"))
            .is_ok()
        {
            return input
                .parse_nested_block(|input| FamilyName::parse(context, input))
                .map(Source::Local);
        }

        let url = SpecifiedUrl::parse(context, input)?;

        // Parsing optional format()
        let format_hints = if input
            .try(|input| input.expect_function_matching("format"))
            .is_ok()
        {
            input.parse_nested_block(|input| {
                input.parse_comma_separated(|input| Ok(input.expect_string()?.as_ref().to_owned()))
            })?
        } else {
            vec![]
        };

        Ok(Source::Url(UrlSource {
            url: url,
            format_hints: format_hints,
        }))
    }
}

macro_rules! is_descriptor_enabled {
    ("font-display") => {
        unsafe {
            use gecko_bindings::structs::mozilla;
            mozilla::StaticPrefs_sVarCache_layout_css_font_display_enabled
        }
    };
    ("font-variation-settings") => {
        unsafe {
            use gecko_bindings::structs::mozilla;
            mozilla::StaticPrefs_sVarCache_layout_css_font_variations_enabled != 0
        }
    };
    ($name:tt) => {
        true
    };
}

macro_rules! font_face_descriptors_common {
    (
        $( #[$doc: meta] $name: tt $ident: ident / $gecko_ident: ident: $ty: ty, )*
    ) => {
        /// Data inside a `@font-face` rule.
        ///
        /// <https://drafts.csswg.org/css-fonts/#font-face-rule>
        #[derive(Clone, Debug, PartialEq)]
        pub struct FontFaceRuleData {
            $(
                #[$doc]
                pub $ident: Option<$ty>,
            )*
            /// Line and column of the @font-face rule source code.
            pub source_location: SourceLocation,
        }

        impl FontFaceRuleData {
            /// Create an empty font-face rule
            pub fn empty(location: SourceLocation) -> Self {
                FontFaceRuleData {
                    $(
                        $ident: None,
                    )*
                    source_location: location,
                }
            }

            /// Serialization of declarations in the FontFaceRule
            pub fn decl_to_css(&self, dest: &mut CssStringWriter) -> fmt::Result {
                $(
                    if let Some(ref value) = self.$ident {
                        dest.write_str(concat!("  ", $name, ": "))?;
                        ToCss::to_css(value, &mut CssWriter::new(dest))?;
                        dest.write_str(";\n")?;
                    }
                )*
                Ok(())
            }
        }

       impl<'a, 'b, 'i> DeclarationParser<'i> for FontFaceRuleParser<'a, 'b> {
           type Declaration = ();
           type Error = StyleParseErrorKind<'i>;

           fn parse_value<'t>(&mut self, name: CowRcStr<'i>, input: &mut Parser<'i, 't>)
                              -> Result<(), ParseError<'i>> {
                match_ignore_ascii_case! { &*name,
                    $(
                        $name if is_descriptor_enabled!($name) => {
                            // DeclarationParser also calls parse_entirely
                            // so we’d normally not need to,
                            // but in this case we do because we set the value as a side effect
                            // rather than returning it.
                            let value = input.parse_entirely(|i| Parse::parse(self.context, i))?;
                            self.rule.$ident = Some(value)
                        }
                    )*
                    _ => return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
                }
                Ok(())
            }
        }
    }
}

impl ToCssWithGuard for FontFaceRuleData {
    // Serialization of FontFaceRule is not specced.
    fn to_css(&self, _guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@font-face {\n")?;
        self.decl_to_css(dest)?;
        dest.write_str("}")
    }
}

macro_rules! font_face_descriptors {
    (
        mandatory descriptors = [
            $( #[$m_doc: meta] $m_name: tt $m_ident: ident / $m_gecko_ident: ident: $m_ty: ty, )*
        ]
        optional descriptors = [
            $( #[$o_doc: meta] $o_name: tt $o_ident: ident / $o_gecko_ident: ident: $o_ty: ty, )*
        ]
    ) => {
        font_face_descriptors_common! {
            $( #[$m_doc] $m_name $m_ident / $m_gecko_ident: $m_ty, )*
            $( #[$o_doc] $o_name $o_ident / $o_gecko_ident: $o_ty, )*
        }

        impl FontFaceRuleData {
            /// Per https://github.com/w3c/csswg-drafts/issues/1133 an @font-face rule
            /// is valid as far as the CSS parser is concerned even if it doesn’t have
            /// a font-family or src declaration.
            ///
            /// However both are required for the rule to represent an actual font face.
            #[cfg(feature = "servo")]
            pub fn font_face(&self) -> Option<FontFace> {
                if $( self.$m_ident.is_some() )&&* {
                    Some(FontFace(self))
                } else {
                    None
                }
            }
        }

        #[cfg(feature = "servo")]
        impl<'a> FontFace<'a> {
            $(
                #[$m_doc]
                pub fn $m_ident(&self) -> &$m_ty {
                    self.0 .$m_ident.as_ref().unwrap()
                }
            )*
        }
    }
}

/// css-name rust_identifier: Type,
#[cfg(feature = "gecko")]
font_face_descriptors! {
    mandatory descriptors = [
        /// The name of this font face
        "font-family" family / mFamily: FamilyName,

        /// The alternative sources for this font face.
        "src" sources / mSrc: Vec<Source>,
    ]
    optional descriptors = [
        /// The style of this font face.
        "font-style" style / mStyle: FontStyle,

        /// The weight of this font face.
        "font-weight" weight / mWeight: FontWeight,

        /// The stretch of this font face.
        "font-stretch" stretch / mStretch: FontStretch,

        /// The display of this font face.
        "font-display" display / mDisplay: FontDisplay,

        /// The ranges of code points outside of which this font face should not be used.
        "unicode-range" unicode_range / mUnicodeRange: Vec<UnicodeRange>,

        /// The feature settings of this font face.
        "font-feature-settings" feature_settings / mFontFeatureSettings: SpecifiedFontFeatureSettings,

        /// The variation settings of this font face.
        "font-variation-settings" variation_settings / mFontVariationSettings: SpecifiedFontVariationSettings,

        /// The language override of this font face.
        "font-language-override" language_override / mFontLanguageOverride: font_language_override::SpecifiedValue,
    ]
}

#[cfg(feature = "servo")]
font_face_descriptors! {
    mandatory descriptors = [
        /// The name of this font face
        "font-family" family / mFamily: FamilyName,

        /// The alternative sources for this font face.
        "src" sources / mSrc: Vec<Source>,
    ]
    optional descriptors = [
    ]
}
