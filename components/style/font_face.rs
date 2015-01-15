/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Token, Parser, DeclarationListParser, AtRuleParser, DeclarationParser};
use std::ascii::AsciiExt;
use stylesheets::CSSRule;
use properties::longhands::font_family::parse_one_family;
use properties::computed_values::font_family::FontFamily;
use media_queries::Device;
use url::{Url, UrlParser};
use parser::ParserContext;


pub fn iter_font_face_rules_inner<F>(rules: &[CSSRule], device: &Device,
                                     callback: &F) where F: Fn(&str, &Source) {
    for rule in rules.iter() {
        match *rule {
            CSSRule::Style(..) |
            CSSRule::Charset(..) |
            CSSRule::Namespace(..) => {},
            CSSRule::Media(ref rule) => if rule.media_queries.evaluate(device) {
                iter_font_face_rules_inner(rule.rules.as_slice(), device, callback)
            },
            CSSRule::FontFace(ref rule) => {
                for source in rule.sources.iter() {
                    callback(rule.family.as_slice(), source)
                }
            },
        }
    }
}

#[derive(Clone, Show, PartialEq, Eq)]
pub enum Source {
    Url(UrlSource),
    Local(String),
}

#[derive(Clone, Show, PartialEq, Eq)]
pub struct UrlSource {
    pub url: Url,
    pub format_hints: Vec<String>,
}

#[derive(Show, PartialEq, Eq)]
pub struct FontFaceRule {
    pub family: String,
    pub sources: Vec<Source>,
}


pub fn parse_font_face_block(context: &ParserContext, input: &mut Parser)
                             -> Result<FontFaceRule, ()> {
    let parser = FontFaceRuleParser {
        context: context,
        family: None,
        src: None,
    };
    match DeclarationListParser::new(input, parser).run() {
        FontFaceRuleParser { family: Some(family), src: Some(src), .. } => {
            Ok(FontFaceRule {
                family: family,
                sources: src,
            })
        }
        _ => Err(())
    }
}


struct FontFaceRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    family: Option<String>,
    src: Option<Vec<Source>>,
}


/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser<(), ()> for FontFaceRuleParser<'a, 'b> {}


impl<'a, 'b> DeclarationParser<()> for FontFaceRuleParser<'a, 'b> {
    fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<(), ()> {
        match_ignore_ascii_case! { name,
            "font-family" => {
                self.family = Some(try!(parse_one_non_generic_family_name(input)));
                Ok(())
            },
            "src" => {
                self.src = Some(try!(input.parse_comma_separated(|input| {
                    parse_one_src(self.context, input)
                })));
                Ok(())
            }
            _ => Err(())
        }
    }
}

fn parse_one_non_generic_family_name(input: &mut Parser) -> Result<String, ()> {
    match parse_one_family(input) {
        Ok(FontFamily::FamilyName(name)) => Ok(name),
        _ => Err(())
    }
}


fn parse_one_src(context: &ParserContext, input: &mut Parser) -> Result<Source, ()> {
    let url = match input.next() {
        // Parsing url()
        Ok(Token::Url(url)) => {
            UrlParser::new().base_url(context.base_url).parse(url.as_slice()).unwrap_or_else(
                |_error| Url::parse("about:invalid").unwrap())
        },
        // Parsing local() with early return
        Ok(Token::Function(name)) => {
            if name.eq_ignore_ascii_case("local") {
                return Ok(Source::Local(try!(input.parse_nested_block(|input| {
                    parse_one_non_generic_family_name(input)
                }))))
            }
            return Err(())
        },
        _ => return Err(())
    };

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
