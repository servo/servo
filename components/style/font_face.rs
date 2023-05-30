/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The [`@font-face`][ff] at-rule.
//!
//! [ff]: https://drafts.csswg.org/css-fonts/#at-font-face-rule

use crate::error_reporting::ContextualParseError;
use crate::parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use crate::properties::longhands::font_language_override;
use crate::shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::values::computed::font::{FamilyName, FontStretch};
use crate::values::generics::font::FontStyle as GenericFontStyle;
#[cfg(feature = "gecko")]
use crate::values::specified::font::MetricsOverride;
use crate::values::specified::font::SpecifiedFontStyle;
use crate::values::specified::font::{AbsoluteFontWeight, FontStretch as SpecifiedFontStretch};
#[cfg(feature = "gecko")]
use crate::values::specified::font::{FontFeatureSettings, FontVariationSettings};
use crate::values::specified::url::SpecifiedUrl;
use crate::values::specified::Angle;
#[cfg(feature = "gecko")]
use crate::values::specified::NonNegativePercentage;
#[cfg(feature = "gecko")]
use cssparser::UnicodeRange;
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, Parser, QualifiedRuleParser, RuleBodyItemParser,
    RuleBodyParser, SourceLocation,
};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError};
use style_traits::{StyleParseErrorKind, ToCss};

/// A source for a font-face rule.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, ToCss, ToShmem)]
pub enum Source {
    /// A `url()` source.
    Url(UrlSource),
    /// A `local()` source.
    #[css(function)]
    Local(FamilyName),
}

/// A list of sources for the font-face src descriptor.
#[derive(Clone, Debug, Eq, PartialEq, ToCss, ToShmem)]
#[css(comma)]
pub struct SourceList(#[css(iterable)] pub Vec<Source>);

// We can't just use OneOrMoreSeparated to derive Parse for the Source list,
// because we want to filter out components that parsed as None, then fail if no
// valid components remain. So we provide our own implementation here.
impl Parse for SourceList {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Parse the comma-separated list, then let filter_map discard any None items.
        let list = input
            .parse_comma_separated(|input| {
                let s = input.parse_entirely(|input| Source::parse(context, input));
                while input.next().is_ok() {}
                Ok(s.ok())
            })?
            .into_iter()
            .filter_map(|s| s)
            .collect::<Vec<Source>>();
        if list.is_empty() {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        } else {
            Ok(SourceList(list))
        }
    }
}

/// Keywords for the font-face src descriptor's format() function.
/// ('None' and 'Unknown' are for internal use in gfx, not exposed to CSS.)
#[derive(Clone, Copy, Debug, Eq, Parse, PartialEq, ToCss, ToShmem)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum FontFaceSourceFormatKeyword {
    #[css(skip)]
    None,
    Collection,
    EmbeddedOpentype,
    Opentype,
    Svg,
    Truetype,
    Woff,
    Woff2,
    #[css(skip)]
    Unknown,
}

bitflags! {
    /// Flags for the @font-face tech() function, indicating font technologies
    /// required by the resource.
    #[derive(ToShmem)]
    #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
    #[repr(C)]
    pub struct FontFaceSourceTechFlags: u16 {
        /// Font requires OpenType feature support.
        const FEATURES_OPENTYPE = 1 << 0;
        /// Font requires Apple Advanced Typography support.
        const FEATURES_AAT = 1 << 1;
        /// Font requires Graphite shaping support.
        const FEATURES_GRAPHITE = 1 << 2;
        /// Font requires COLRv0 rendering support (simple list of colored layers).
        const COLOR_COLRV0 = 1 << 3;
        /// Font requires COLRv1 rendering support (graph of paint operations).
        const COLOR_COLRV1 = 1 << 4;
        /// Font requires SVG glyph rendering support.
        const COLOR_SVG = 1 << 5;
        /// Font has bitmap glyphs in 'sbix' format.
        const COLOR_SBIX = 1 << 6;
        /// Font has bitmap glyphs in 'CBDT' format.
        const COLOR_CBDT = 1 << 7;
        /// Font requires OpenType Variations support.
        const VARIATIONS = 1 << 8;
        /// Font requires CPAL palette selection support.
        const PALETTES = 1 << 9;
        /// Font requires support for incremental downloading.
        const INCREMENTAL = 1 << 10;
    }
}

impl FontFaceSourceTechFlags {
    /// Parse a single font-technology keyword and return its flag.
    pub fn parse_one<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Ok(try_match_ident_ignore_ascii_case! { input,
            "features-opentype" => Self::FEATURES_OPENTYPE,
            "features-aat" => Self::FEATURES_AAT,
            "features-graphite" => Self::FEATURES_GRAPHITE,
            "color-colrv0" => Self::COLOR_COLRV0,
            "color-colrv1" => Self::COLOR_COLRV1,
            "color-svg" => Self::COLOR_SVG,
            "color-sbix" => Self::COLOR_SBIX,
            "color-cbdt" => Self::COLOR_CBDT,
            "variations" => Self::VARIATIONS,
            "palettes" => Self::PALETTES,
            "incremental" => Self::INCREMENTAL,
        })
    }
}

impl Parse for FontFaceSourceTechFlags {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        // We don't actually care about the return value of parse_comma_separated,
        // because we insert the flags into result as we go.
        let mut result = Self::empty();
        input.parse_comma_separated(|input| {
            let flag = Self::parse_one(input)?;
            result.insert(flag);
            Ok(())
        })?;
        if !result.is_empty() {
            Ok(result)
        } else {
            Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

#[allow(unused_assignments)]
impl ToCss for FontFaceSourceTechFlags {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        let mut first = true;

        macro_rules! write_if_flag {
            ($s:expr => $f:ident) => {
                if self.contains(Self::$f) {
                    if first {
                        first = false;
                    } else {
                        dest.write_str(", ")?;
                    }
                    dest.write_str($s)?;
                }
            };
        }

        write_if_flag!("features-opentype" => FEATURES_OPENTYPE);
        write_if_flag!("features-aat" => FEATURES_AAT);
        write_if_flag!("features-graphite" => FEATURES_GRAPHITE);
        write_if_flag!("color-colrv0" => COLOR_COLRV0);
        write_if_flag!("color-colrv1" => COLOR_COLRV1);
        write_if_flag!("color-svg" => COLOR_SVG);
        write_if_flag!("color-sbix" => COLOR_SBIX);
        write_if_flag!("color-cbdt" => COLOR_CBDT);
        write_if_flag!("variations" => VARIATIONS);
        write_if_flag!("palettes" => PALETTES);
        write_if_flag!("incremental" => INCREMENTAL);

        Ok(())
    }
}

/// A POD representation for Gecko. All pointers here are non-owned and as such
/// can't outlive the rule they came from, but we can't enforce that via C++.
///
/// All the strings are of course utf8.
#[cfg(feature = "gecko")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum FontFaceSourceListComponent {
    Url(*const crate::gecko::url::CssUrl),
    Local(*mut crate::gecko_bindings::structs::nsAtom),
    FormatHintKeyword(FontFaceSourceFormatKeyword),
    FormatHintString {
        length: usize,
        utf8_bytes: *const u8,
    },
    TechFlags(FontFaceSourceTechFlags),
}

#[derive(Clone, Debug, Eq, PartialEq, ToCss, ToShmem)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum FontFaceSourceFormat {
    Keyword(FontFaceSourceFormatKeyword),
    String(String),
}

/// A `UrlSource` represents a font-face source that has been specified with a
/// `url()` function.
///
/// <https://drafts.csswg.org/css-fonts/#src-desc>
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq, ToShmem)]
pub struct UrlSource {
    /// The specified url.
    pub url: SpecifiedUrl,
    /// The format hint specified with the `format()` function, if present.
    pub format_hint: Option<FontFaceSourceFormat>,
    /// The font technology flags specified with the `tech()` function, if any.
    pub tech_flags: FontFaceSourceTechFlags,
}

impl ToCss for UrlSource {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.url.to_css(dest)?;
        if let Some(hint) = &self.format_hint {
            dest.write_str(" format(")?;
            hint.to_css(dest)?;
            dest.write_char(')')?;
        }
        if !self.tech_flags.is_empty() {
            dest.write_str(" tech(")?;
            self.tech_flags.to_css(dest)?;
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
#[derive(
    Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToComputedValue, ToCss, ToShmem,
)]
#[repr(u8)]
pub enum FontDisplay {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

macro_rules! impl_range {
    ($range:ident, $component:ident) => {
        impl Parse for $range {
            fn parse<'i, 't>(
                context: &ParserContext,
                input: &mut Parser<'i, 't>,
            ) -> Result<Self, ParseError<'i>> {
                let first = $component::parse(context, input)?;
                let second = input
                    .try_parse(|input| $component::parse(context, input))
                    .unwrap_or_else(|_| first.clone());
                Ok($range(first, second))
            }
        }
        impl ToCss for $range {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: fmt::Write,
            {
                self.0.to_css(dest)?;
                if self.0 != self.1 {
                    dest.write_char(' ')?;
                    self.1.to_css(dest)?;
                }
                Ok(())
            }
        }
    };
}

/// The font-weight descriptor:
///
/// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-weight
#[derive(Clone, Debug, PartialEq, ToShmem)]
pub struct FontWeightRange(pub AbsoluteFontWeight, pub AbsoluteFontWeight);
impl_range!(FontWeightRange, AbsoluteFontWeight);

/// The computed representation of the above so Gecko can read them easily.
///
/// This one is needed because cbindgen doesn't know how to generate
/// specified::Number.
#[repr(C)]
#[allow(missing_docs)]
pub struct ComputedFontWeightRange(f32, f32);

#[inline]
fn sort_range<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

impl FontWeightRange {
    /// Returns a computed font-stretch range.
    pub fn compute(&self) -> ComputedFontWeightRange {
        let (min, max) = sort_range(self.0.compute().value(), self.1.compute().value());
        ComputedFontWeightRange(min, max)
    }
}

/// The font-stretch descriptor:
///
/// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-stretch
#[derive(Clone, Debug, PartialEq, ToShmem)]
pub struct FontStretchRange(pub SpecifiedFontStretch, pub SpecifiedFontStretch);
impl_range!(FontStretchRange, SpecifiedFontStretch);

/// The computed representation of the above, so that Gecko can read them
/// easily.
#[repr(C)]
#[allow(missing_docs)]
pub struct ComputedFontStretchRange(FontStretch, FontStretch);

impl FontStretchRange {
    /// Returns a computed font-stretch range.
    pub fn compute(&self) -> ComputedFontStretchRange {
        fn compute_stretch(s: &SpecifiedFontStretch) -> FontStretch {
            match *s {
                SpecifiedFontStretch::Keyword(ref kw) => kw.compute(),
                SpecifiedFontStretch::Stretch(ref p) => FontStretch::from_percentage(p.0.get()),
                SpecifiedFontStretch::System(..) => unreachable!(),
            }
        }

        let (min, max) = sort_range(compute_stretch(&self.0), compute_stretch(&self.1));
        ComputedFontStretchRange(min, max)
    }
}

/// The font-style descriptor:
///
/// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-style
#[derive(Clone, Debug, PartialEq, ToShmem)]
#[allow(missing_docs)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(Angle, Angle),
}

/// The computed representation of the above, with angles in degrees, so that
/// Gecko can read them easily.
#[repr(u8)]
#[allow(missing_docs)]
pub enum ComputedFontStyleDescriptor {
    Normal,
    Italic,
    Oblique(f32, f32),
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
                let second_angle = input
                    .try_parse(|input| SpecifiedFontStyle::parse_angle(context, input))
                    .unwrap_or_else(|_| angle.clone());

                FontStyle::Oblique(angle, second_angle)
            },
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
            },
        }
    }
}

impl FontStyle {
    /// Returns a computed font-style descriptor.
    pub fn compute(&self) -> ComputedFontStyleDescriptor {
        match *self {
            FontStyle::Normal => ComputedFontStyleDescriptor::Normal,
            FontStyle::Italic => ComputedFontStyleDescriptor::Italic,
            FontStyle::Oblique(ref first, ref second) => {
                let (min, max) = sort_range(
                    SpecifiedFontStyle::compute_angle_degrees(first),
                    SpecifiedFontStyle::compute_angle_degrees(second),
                );
                ComputedFontStyleDescriptor::Oblique(min, max)
            },
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
        let mut parser = FontFaceRuleParser {
            context,
            rule: &mut rule,
        };
        let mut iter = RuleBodyParser::new(input, &mut parser);
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
                .0
                .iter()
                .rev()
                .filter(|source| {
                    if let Source::Url(ref url_source) = **source {
                        // We support only opentype fonts and truetype is an alias for
                        // that format. Sources without format hints need to be
                        // downloaded in case we support them.
                        url_source
                            .format_hint
                            .as_ref()
                            .map_or(true, |hint| match hint {
                                FontFaceSourceFormat::Keyword(
                                    FontFaceSourceFormatKeyword::Truetype
                                    | FontFaceSourceFormatKeyword::Opentype
                                    | FontFaceSourceFormatKeyword::Woff,
                                ) => true,
                                FontFaceSourceFormat::String(s) => {
                                    s == "truetype" || s == "opentype" || s == "woff"
                                }
                                _ => false,
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
    type Prelude = ();
    type AtRule = ();
    type Error = StyleParseErrorKind<'i>;
}

impl<'a, 'b, 'i> QualifiedRuleParser<'i> for FontFaceRuleParser<'a, 'b> {
    type Prelude = ();
    type QualifiedRule = ();
    type Error = StyleParseErrorKind<'i>;
}

impl<'a, 'b, 'i> RuleBodyItemParser<'i, (), StyleParseErrorKind<'i>>
    for FontFaceRuleParser<'a, 'b>
{
    fn parse_qualified(&self) -> bool {
        false
    }
    fn parse_declarations(&self) -> bool {
        true
    }
}

fn font_tech_enabled() -> bool {
    #[cfg(feature = "gecko")]
    return static_prefs::pref!("layout.css.font-tech.enabled");
    #[cfg(feature = "servo")]
    return false;
}

impl Parse for Source {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Source, ParseError<'i>> {
        if input
            .try_parse(|input| input.expect_function_matching("local"))
            .is_ok()
        {
            return input
                .parse_nested_block(|input| FamilyName::parse(context, input))
                .map(Source::Local);
        }

        let url = SpecifiedUrl::parse(context, input)?;

        // Parsing optional format()
        let format_hint = if input
            .try_parse(|input| input.expect_function_matching("format"))
            .is_ok()
        {
            input.parse_nested_block(|input| {
                if let Ok(kw) = input.try_parse(FontFaceSourceFormatKeyword::parse) {
                    Ok(Some(FontFaceSourceFormat::Keyword(kw)))
                } else {
                    let s = input.expect_string()?.as_ref().to_owned();
                    Ok(Some(FontFaceSourceFormat::String(s)))
                }
            })?
        } else {
            None
        };

        // Parse optional tech()
        let tech_flags = if font_tech_enabled() && input
            .try_parse(|input| input.expect_function_matching("tech"))
            .is_ok()
        {
            input.parse_nested_block(|input| FontFaceSourceTechFlags::parse(context, input))?
        } else {
            FontFaceSourceTechFlags::empty()
        };

        Ok(Source::Url(UrlSource {
            url,
            format_hint,
            tech_flags,
        }))
    }
}

macro_rules! is_descriptor_enabled {
    ("font-display") => {
        static_prefs::pref!("layout.css.font-display.enabled")
    };
    ("font-variation-settings") => {
        static_prefs::pref!("layout.css.font-variations.enabled")
    };
    ("ascent-override") => {
        static_prefs::pref!("layout.css.font-metrics-overrides.enabled")
    };
    ("descent-override") => {
        static_prefs::pref!("layout.css.font-metrics-overrides.enabled")
    };
    ("line-gap-override") => {
        static_prefs::pref!("layout.css.font-metrics-overrides.enabled")
    };
    ("size-adjust") => {
        static_prefs::pref!("layout.css.size-adjust.enabled")
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
        #[derive(Clone, Debug, PartialEq, ToShmem)]
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
                        dest.write_str(concat!($name, ": "))?;
                        value.to_css(&mut CssWriter::new(dest))?;
                        dest.write_str("; ")?;
                    }
                )*
                Ok(())
            }
        }

       impl<'a, 'b, 'i> DeclarationParser<'i> for FontFaceRuleParser<'a, 'b> {
           type Declaration = ();
           type Error = StyleParseErrorKind<'i>;

           fn parse_value<'t>(
               &mut self,
               name: CowRcStr<'i>,
               input: &mut Parser<'i, 't>,
            ) -> Result<(), ParseError<'i>> {
                match_ignore_ascii_case! { &*name,
                    $(
                        $name if is_descriptor_enabled!($name) => {
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
    }
}

impl ToCssWithGuard for FontFaceRuleData {
    // Serialization of FontFaceRule is not specced.
    fn to_css(&self, _guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@font-face { ")?;
        self.decl_to_css(dest)?;
        dest.write_char('}')
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

#[cfg(feature = "gecko")]
font_face_descriptors! {
    mandatory descriptors = [
        /// The name of this font face
        "font-family" family / mFamily: FamilyName,

        /// The alternative sources for this font face.
        "src" sources / mSrc: SourceList,
    ]
    optional descriptors = [
        /// The style of this font face.
        "font-style" style / mStyle: FontStyle,

        /// The weight of this font face.
        "font-weight" weight / mWeight: FontWeightRange,

        /// The stretch of this font face.
        "font-stretch" stretch / mStretch: FontStretchRange,

        /// The display of this font face.
        "font-display" display / mDisplay: FontDisplay,

        /// The ranges of code points outside of which this font face should not be used.
        "unicode-range" unicode_range / mUnicodeRange: Vec<UnicodeRange>,

        /// The feature settings of this font face.
        "font-feature-settings" feature_settings / mFontFeatureSettings: FontFeatureSettings,

        /// The variation settings of this font face.
        "font-variation-settings" variation_settings / mFontVariationSettings: FontVariationSettings,

        /// The language override of this font face.
        "font-language-override" language_override / mFontLanguageOverride: font_language_override::SpecifiedValue,

        /// The ascent override for this font face.
        "ascent-override" ascent_override / mAscentOverride: MetricsOverride,

        /// The descent override for this font face.
        "descent-override" descent_override / mDescentOverride: MetricsOverride,

        /// The line-gap override for this font face.
        "line-gap-override" line_gap_override / mLineGapOverride: MetricsOverride,

        /// The size adjustment for this font face.
        "size-adjust" size_adjust / mSizeAdjust: NonNegativePercentage,
    ]
}

#[cfg(feature = "servo")]
font_face_descriptors! {
    mandatory descriptors = [
        /// The name of this font face
        "font-family" family / mFamily: FamilyName,

        /// The alternative sources for this font face.
        "src" sources / mSrc: SourceList,
    ]
    optional descriptors = [
    ]
}
