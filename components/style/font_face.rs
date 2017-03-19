/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@font-face`][ff] at-rule.
//!
//! [ff]: https://drafts.csswg.org/css-fonts/#at-font-face-rule

#![deny(missing_docs)]

#[cfg(feature = "gecko")]
use computed_values::{font_style, font_weight, font_stretch};
use computed_values::font_family::FamilyName;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser};
#[cfg(feature = "gecko")] use cssparser::UnicodeRange;
use parser::{ParserContext, log_css_error, Parse};
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use std::iter;
use style_traits::{ToCss, OneOrMoreCommaSeparated};
use values::specified::url::SpecifiedUrl;

/// A source for a font-face rule.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub enum Source {
    /// A `url()` source.
    Url(UrlSource),
    /// A `local()` source.
    Local(FamilyName),
}

impl ToCss for Source {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            Source::Url(ref url) => {
                try!(dest.write_str("url(\""));
                try!(url.to_css(dest));
            },
            Source::Local(ref family) => {
                try!(dest.write_str("local(\""));
                try!(family.to_css(dest));
            },
        }
        dest.write_str("\")")
    }
}

impl OneOrMoreCommaSeparated for Source {}

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
        dest.write_str(self.url.as_str())
    }
}

/// Parse the block inside a `@font-face` rule.
///
/// Note that the prelude parsing code lives in the `stylesheets` module.
pub fn parse_font_face_block(context: &ParserContext, input: &mut Parser)
                             -> Result<FontFaceRule, ()> {
    let mut rule = FontFaceRule::initial();
    {
        let parser = FontFaceRuleParser {
            context: context,
            rule: &mut rule,
            missing: MissingDescriptors::new(),
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err(range) = declaration {
                let pos = range.start;
                let message = format!("Unsupported @font-face descriptor declaration: '{}'",
                                      iter.input.slice(range));
                log_css_error(iter.input, pos, &*message, context);
            }
        }
        if iter.parser.missing.any() {
            return Err(())
        }
    }
    Ok(rule)
}

/// A list of effective sources that we send over through IPC to the font cache.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct EffectiveSources(Vec<Source>);

impl FontFaceRule {
    /// Returns the list of effective sources for that font-face, that is the
    /// sources which don't list any format hint, or the ones which list at
    /// least "truetype" or "opentype".
    pub fn effective_sources(&self) -> EffectiveSources {
        EffectiveSources(self.sources.iter().rev().filter(|source| {
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

impl iter::Iterator for EffectiveSources {
    type Item = Source;
    fn next(&mut self) -> Option<Source> {
        self.0.pop()
    }
}

struct FontFaceRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    rule: &'a mut FontFaceRule,
    missing: MissingDescriptors,
}

/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for FontFaceRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = ();
}

impl Parse for Source {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Source, ()> {
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

macro_rules! font_face_descriptors {
    (
        mandatory descriptors = [
            $( #[$m_doc: meta] $m_name: tt $m_ident: ident: $m_ty: ty = $m_initial: expr, )*
        ]
        optional descriptors = [
            $( #[$o_doc: meta] $o_name: tt $o_ident: ident: $o_ty: ty = $o_initial: expr, )*
        ]
    ) => {
        /// A `@font-face` rule.
        ///
        /// https://drafts.csswg.org/css-fonts/#font-face-rule
        #[derive(Debug, PartialEq, Eq)]
        pub struct FontFaceRule {
            $(
                #[$m_doc]
                pub $m_ident: $m_ty,
            )*
            $(
                #[$o_doc]
                pub $o_ident: $o_ty,
            )*
        }

        struct MissingDescriptors {
            $(
                $m_ident: bool,
            )*
        }

        impl MissingDescriptors {
            fn new() -> Self {
                MissingDescriptors {
                    $(
                        $m_ident: true,
                    )*
                }
            }

            fn any(&self) -> bool {
                $(
                    self.$m_ident
                )||*
            }
        }

        impl FontFaceRule {
            fn initial() -> Self {
                FontFaceRule {
                    $(
                        $m_ident: $m_initial,
                    )*
                    $(
                        $o_ident: $o_initial,
                    )*
                }
            }
        }

        impl ToCssWithGuard for FontFaceRule {
            // Serialization of FontFaceRule is not specced.
            fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
            where W: fmt::Write {
                dest.write_str("@font-face {\n")?;
                $(
                    dest.write_str(concat!("  ", $m_name, ": "))?;
                    ToCss::to_css(&self.$m_ident, dest)?;
                    dest.write_str(";\n")?;
                )*
                $(
                    // Because of parse_font_face_block,
                    // this condition is always true for "src" and "font-family".
                    // But it can be false for other descriptors.
                    if self.$o_ident != $o_initial {
                        dest.write_str(concat!("  ", $o_name, ": "))?;
                        ToCss::to_css(&self.$o_ident, dest)?;
                        dest.write_str(";\n")?;
                    }
                )*
                dest.write_str("}")
            }
        }

       impl<'a, 'b> DeclarationParser for FontFaceRuleParser<'a, 'b> {
            type Declaration = ();

            fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<(), ()> {
                match_ignore_ascii_case! { name,
                    $(
                        $m_name => {
                            self.rule.$m_ident = Parse::parse(self.context, input)?;
                            self.missing.$m_ident = false
                        },
                    )*
                    $(
                        $o_name => self.rule.$o_ident = Parse::parse(self.context, input)?,
                    )*
                    _ => return Err(())
                }
                Ok(())
            }
        }
    }
}

/// css-name rust_identifier: Type = initial_value,
#[cfg(feature = "gecko")]
font_face_descriptors! {
    mandatory descriptors = [
        /// The name of this font face
        "font-family" family: FamilyName = FamilyName(atom!("")),

        /// The alternative sources for this font face.
        "src" sources: Vec<Source> = Vec::new(),
    ]
    optional descriptors = [
        /// The style of this font face
        "font-style" style: font_style::T = font_style::T::normal,

        /// The weight of this font face
        "font-weight" weight: font_weight::T = font_weight::T::Weight400 /* normal */,

        /// The stretch of this font face
        "font-stretch" stretch: font_stretch::T = font_stretch::T::normal,

        /// The ranges of code points outside of which this font face should not be used.
        "unicode-range" unicode_range: Vec<UnicodeRange> = vec![
            UnicodeRange { start: 0, end: 0x10FFFF }
        ],
    ]
}

#[cfg(feature = "servo")]
font_face_descriptors! {
    mandatory descriptors = [
        /// The name of this font face
        "font-family" family: FamilyName = FamilyName(atom!("")),

        /// The alternative sources for this font face.
        "src" sources: Vec<Source> = Vec::new(),
    ]
    optional descriptors = [
    ]
}
