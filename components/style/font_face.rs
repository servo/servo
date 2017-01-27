/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@font-face`][ff] at-rule.
//!
//! [ff]: https://drafts.csswg.org/css-fonts/#at-font-face-rule

#![deny(missing_docs)]

use computed_values::font_family::FontFamily;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser};
use parser::{ParserContext, log_css_error, Parse};
use std::fmt;
use std::iter;
use style_traits::ToCss;
use values::specified::url::SpecifiedUrl;

/// A source for a font-face rule.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub enum Source {
    /// A `url()` source.
    Url(UrlSource),
    /// A `local()` source.
    Local(FontFamily),
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

/// A `UrlSource` represents a font-face source that has been specified with a
/// `url()` function.
///
/// https://drafts.csswg.org/css-fonts/#src-desc
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
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

/// A `@font-face` rule.
///
/// https://drafts.csswg.org/css-fonts/#font-face-rule
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct FontFaceRule {
    /// The font family specified with the `font-family` property declaration.
    pub family: FontFamily,
    /// The list of sources specified with the different `src` property
    /// declarations.
    pub sources: Vec<Source>,
}

impl ToCss for FontFaceRule {
    // Serialization of FontFaceRule is not specced.
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
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

/// Parse the block inside a `@font-face` rule.
///
/// Note that the prelude parsing code lives in the `stylesheets` module.
pub fn parse_font_face_block(context: &ParserContext, input: &mut Parser)
                             -> Result<FontFaceRule, ()> {
    let mut rule = FontFaceRule {
        family: FontFamily::Generic(atom!("")),
        sources: Vec::new(),
    };
    {
        let parser = FontFaceRuleParser { context: context, rule: &mut rule };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            if let Err(range) = declaration {
                let pos = range.start;
                let message = format!("Unsupported @font-face descriptor declaration: '{}'",
                                      iter.input.slice(range));
                log_css_error(iter.input, pos, &*message, context);
            }
        }
    }
    if rule.family != FontFamily::Generic(atom!("")) && !rule.sources.is_empty() {
        Ok(rule)
    } else {
        Err(())
    }
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
}

/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for FontFaceRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = ();
}


impl<'a, 'b> DeclarationParser for FontFaceRuleParser<'a, 'b> {
    type Declaration = ();

    fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<(), ()> {
        match_ignore_ascii_case! { name,
            "font-family" => self.rule.family = Parse::parse(self.context, input)?,
            "src" => self.rule.sources = Parse::parse(self.context, input)?,
            _ => return Err(())
        }
        Ok(())
    }
}

impl Parse for Vec<Source> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.parse_comma_separated(|input| Source::parse(context, input))
    }
}

impl Parse for Source {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Source, ()> {
        if input.try(|input| input.expect_function_matching("local")).is_ok() {
            return input.parse_nested_block(|input| {
                FontFamily::parse(context, input)
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
