/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@font-face`][ff] at-rule.
//!
//! [ff]: https://drafts.csswg.org/css-fonts/#at-font-face-rule

use computed_values::font_family::FontFamily;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser};
use parser::{ParserContext, log_css_error};
use properties::longhands::font_family::parse_one_family;
use std::fmt;
use std::iter;
use style_traits::ToCss;
use values::specified::url::SpecifiedUrl;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub enum Source {
    Url(UrlSource),
    Local(FontFamily),
}

impl ToCss for Source {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Source::Url(ref url) => {
                try!(dest.write_str("local(\""));
                try!(url.to_css(dest));
            },
            Source::Local(ref family) => {
                try!(dest.write_str("url(\""));
                try!(family.to_css(dest));
            },
        }
        dest.write_str("\")")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub struct UrlSource {
    pub url: SpecifiedUrl,
    pub format_hints: Vec<String>,
}

impl ToCss for UrlSource {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(self.url.as_str())
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct FontFaceRule {
    pub family: FontFamily,
    pub sources: Vec<Source>,
}

impl ToCss for FontFaceRule {
    // Serialization of FontFaceRule is not specced.
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("@font-face { font-family: "));
        try!(self.family.to_css(dest));
        try!(dest.write_str(";"));

        if self.sources.len() > 0 {
            try!(dest.write_str(" src: "));
            let mut iter = self.sources.iter();
            try!(iter.next().unwrap().to_css(dest));
            for source in iter {
                try!(dest.write_str(", "));
                try!(source.to_css(dest));
            }
            try!(dest.write_str(";"));
        }

        dest.write_str(" }")
    }
}

pub fn parse_font_face_block(context: &ParserContext, input: &mut Parser)
                             -> Result<FontFaceRule, ()> {
    let mut family = None;
    let mut src = None;
    let mut iter = DeclarationListParser::new(input, FontFaceRuleParser { context: context });
    while let Some(declaration) = iter.next() {
        match declaration {
            Err(range) => {
                let pos = range.start;
                let message = format!("Unsupported @font-face descriptor declaration: '{}'",
                                      iter.input.slice(range));
                log_css_error(iter.input, pos, &*message, context);
            }
            Ok(FontFaceDescriptorDeclaration::Family(value)) => {
                family = Some(value);
            }
            Ok(FontFaceDescriptorDeclaration::Src(value)) => {
                src = Some(value);
            }
        }
    }
    match (family, src) {
        (Some(family), Some(src)) => {
            Ok(FontFaceRule {
                family: family,
                sources: src,
            })
        }
        _ => Err(())
    }
}

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

enum FontFaceDescriptorDeclaration {
    Family(FontFamily),
    Src(Vec<Source>),
}


struct FontFaceRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}


/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for FontFaceRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = FontFaceDescriptorDeclaration;
}


impl<'a, 'b> DeclarationParser for FontFaceRuleParser<'a, 'b> {
    type Declaration = FontFaceDescriptorDeclaration;

    fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<FontFaceDescriptorDeclaration, ()> {
        match_ignore_ascii_case! { name,
            "font-family" => {
                Ok(FontFaceDescriptorDeclaration::Family(try!(
                            parse_one_family(input))))
            },
            "src" => {
                Ok(FontFaceDescriptorDeclaration::Src(try!(input.parse_comma_separated(|input| {
                    parse_one_src(self.context, input)
                }))))
            },
            _ => Err(())
        }
    }
}

fn parse_one_src(context: &ParserContext, input: &mut Parser) -> Result<Source, ()> {
    if input.try(|input| input.expect_function_matching("local")).is_ok() {
        return Ok(Source::Local(try!(input.parse_nested_block(parse_one_family))))
    }

    let url = try!(SpecifiedUrl::parse(context, input));

    // Parsing optional format()
    let format_hints = if input.try(|input| input.expect_function_matching("format")).is_ok() {
        try!(input.parse_nested_block(|input| {
            input.parse_comma_separated(|input| {
                Ok((try!(input.expect_string())).into_owned())
            })
        }))
    } else {
        vec![]
    };

    Ok(Source::Url(UrlSource {
        url: url,
        format_hints: format_hints,
    }))
}
