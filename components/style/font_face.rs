/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@font-face`][ff] at-rule.
//!
//! [ff]: https://drafts.csswg.org/css-fonts/#at-font-face-rule

#![deny(missing_docs)]

#[cfg(feature = "gecko")]
use computed_values::{font_feature_settings, font_stretch, font_style, font_weight};
use computed_values::font_family::FamilyName;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser};
use cssparser::{SourceLocation, CompactCowStr};
use error_reporting::ContextualParseError;
#[cfg(feature = "gecko")] use gecko_bindings::structs::CSSFontFaceDescriptors;
#[cfg(feature = "gecko")] use cssparser::UnicodeRange;
use parser::{ParserContext, log_css_error, Parse};
use selectors::parser::SelectorParseError;
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::{ToCss, OneOrMoreSeparated, CommaSeparator, ParseError, StyleParseError};
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
    type S = CommaSeparator;
}

/// A `UrlSource` represents a font-face source that has been specified with a
/// `url()` function.
///
/// https://drafts.csswg.org/css-fonts/#src-desc
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct UrlSource {
    /// The specified url.
    pub url: SpecifiedUrl,
    /// The format hints specified with the `format()` function.
    pub format_hints: Vec<String>,
}

impl ToCss for UrlSource {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        self.url.to_css(dest)
    }
}

/// A font-display value for a @font-face rule.
/// The font-display descriptor determines how a font face is displayed based
/// on whether and when it is downloaded and ready to use.
define_css_keyword_enum!(FontDisplay:
                         "auto" => Auto,
                         "block" => Block,
                         "swap" => Swap,
                         "fallback" => Fallback,
                         "optional" => Optional);
add_impls_for_keyword_enum!(FontDisplay);

/// Parse the block inside a `@font-face` rule.
///
/// Note that the prelude parsing code lives in the `stylesheets` module.
pub fn parse_font_face_block(context: &ParserContext, input: &mut Parser, location: SourceLocation)
    -> FontFaceRuleData {
    let mut rule = FontFaceRuleData::empty();
    rule.source_location = location;
    {
        let parser = FontFaceRuleParser {
            context: context,
            rule: &mut rule,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err(err) = declaration {
                let pos = err.span.start;
                let error = ContextualParseError::UnsupportedFontFaceDescriptor(
                    iter.input.slice(err.span), err.error);
                log_css_error(iter.input, pos, error, context);
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
        EffectiveSources(self.sources().iter().rev().filter(|source| {
            if let Source::Url(ref url_source) = **source {
                let hints = &url_source.format_hints;
                // We support only opentype fonts and truetype is an alias for
                // that format. Sources without format hints need to be
                // downloaded in case we support them.
                hints.is_empty() || hints.iter().any(|hint| {
                    hint == "truetype" || hint == "opentype" || hint == "woff"
                })
            } else {
                true
            }
        }).cloned().collect())
    }
}

#[cfg(feature = "servo")]
impl Iterator for EffectiveSources {
    type Item = Source;
    fn next(&mut self) -> Option<Source> {
        self.0.pop()
    }
}

struct FontFaceRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    rule: &'a mut FontFaceRuleData,
}

/// Default methods reject all at rules.
impl<'a, 'b, 'i> AtRuleParser<'i> for FontFaceRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = ();
    type Error = SelectorParseError<'i, StyleParseError<'i>>;
}

impl Parse for Source {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Source, ParseError<'i>> {
        if input.try(|input| input.expect_function_matching("local")).is_ok() {
            return input.parse_nested_block(|input| {
                FamilyName::parse(context, input)
            }).map(Source::Local)
        }

        let url = SpecifiedUrl::parse(context, input)?;

        // Parsing optional format()
        let format_hints = if input.try(|input| input.expect_function_matching("format")).is_ok() {
            input.parse_nested_block(|input| {
                input.parse_comma_separated(|input| {
                    Ok(input.expect_string()?.into_owned())
                })
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

macro_rules! font_face_descriptors_common {
    (
        $( #[$doc: meta] $name: tt $ident: ident / $gecko_ident: ident: $ty: ty, )*
    ) => {
        /// Data inside a `@font-face` rule.
        ///
        /// https://drafts.csswg.org/css-fonts/#font-face-rule
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct FontFaceRuleData {
            $(
                #[$doc]
                pub $ident: Option<$ty>,
            )*
            /// Line and column of the @font-face rule source code.
            pub source_location: SourceLocation,
        }

        impl FontFaceRuleData {
            fn empty() -> Self {
                FontFaceRuleData {
                    $(
                        $ident: None,
                    )*
                    source_location: SourceLocation {
                        line: 0,
                        column: 0,
                    },
                }
            }

            /// Convert to Gecko types
            #[cfg(feature = "gecko")]
            pub fn set_descriptors(self, descriptors: &mut CSSFontFaceDescriptors) {
                $(
                    if let Some(value) = self.$ident {
                        descriptors.$gecko_ident.set_from(value)
                    }
                )*
                // Leave unset descriptors to eCSSUnit_Null,
                // FontFaceSet::FindOrCreateUserFontEntryFromFontFace does the defaulting
                // to initial values.
            }
        }

        impl ToCssWithGuard for FontFaceRuleData {
            // Serialization of FontFaceRule is not specced.
            fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
            where W: fmt::Write {
                dest.write_str("@font-face {\n")?;
                $(
                    if let Some(ref value) = self.$ident {
                        dest.write_str(concat!("  ", $name, ": "))?;
                        ToCss::to_css(value, dest)?;
                        dest.write_str(";\n")?;
                    }
                )*
                dest.write_str("}")
            }
        }

       impl<'a, 'b, 'i> DeclarationParser<'i> for FontFaceRuleParser<'a, 'b> {
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
    }
}

macro_rules! font_face_descriptors {
    (
        mandatory descriptors = [
            $( #[$m_doc: meta] $m_name: tt $m_ident: ident / $m_gecko_ident: ident: $m_ty: ty, )*
        ]
        optional descriptors = [
            $( #[$o_doc: meta] $o_name: tt $o_ident: ident / $o_gecko_ident: ident: $o_ty: ty =
                $o_initial: expr, )*
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
            $(
                #[$o_doc]
                pub fn $o_ident(&self) -> $o_ty {
                    if let Some(ref value) = self.0 .$o_ident {
                        value.clone()
                    } else {
                        $o_initial
                    }
                }
            )*
        }
    }
}

/// css-name rust_identifier: Type = initial_value,
#[cfg(feature = "gecko")]
font_face_descriptors! {
    mandatory descriptors = [
        /// The name of this font face
        "font-family" family / mFamily: FamilyName,

        /// The alternative sources for this font face.
        "src" sources / mSrc: Vec<Source>,
    ]
    optional descriptors = [
        /// The style of this font face
        "font-style" style / mStyle: font_style::T = font_style::T::normal,

        /// The weight of this font face
        "font-weight" weight / mWeight: font_weight::T = font_weight::T::Weight400 /* normal */,

        /// The stretch of this font face
        "font-stretch" stretch / mStretch: font_stretch::T = font_stretch::T::normal,

        /// The display of this font face
        "font-display" display / mDisplay: FontDisplay = FontDisplay::Auto,

        /// The ranges of code points outside of which this font face should not be used.
        "unicode-range" unicode_range / mUnicodeRange: Vec<UnicodeRange> = vec![
            UnicodeRange { start: 0, end: 0x10FFFF }
        ],

        /// The feature settings of this font face.
        "font-feature-settings" feature_settings / mFontFeatureSettings: font_feature_settings::T = {
            font_feature_settings::T::Normal
        },

        // FIXME: add font-language-override.
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
