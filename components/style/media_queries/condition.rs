/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::feature::MediaFeature;

use ::FromCss;
use ::cssparser::{Parser, ToCss};

#[derive(Debug, PartialEq)]
pub struct MediaCondition(pub MediaConditionTerm);

derive_display_using_to_css!(MediaCondition);

impl FromCss for MediaCondition {
    type Err = ();

    #[inline]
    fn from_css(input: &mut Parser) -> Result<MediaCondition, ()> {
        FromCss::from_css(input).map(|r| MediaCondition(r))
    }
}

impl ToCss for MediaCondition {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        self.0.to_css(dest)
    }
}

macro_rules! ok_if_exhausted {
    ($input:ident, $term:path) => {
        FromCss::from_css($input)
            .and_then(|t| $input.expect_exhausted().and(Ok(t)))
            .map(|t| $term(t))
    }
}

#[derive(Debug, PartialEq)]
pub enum MediaConditionTerm {
    Connective(MediaConnectiveTerm),
    InParens(MediaInParensTerm)
}

derive_display_using_to_css!(MediaConditionTerm);

impl FromCss for MediaConditionTerm {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<MediaConditionTerm, ()> {
        // <media-condition> = <media-connective> | <media-in-parens>
        if let Ok(connective) = input.try(FromCss::from_css) {
            Ok(MediaConditionTerm::Connective(connective))
        } else {
            ok_if_exhausted!(input, MediaConditionTerm::InParens)
        }
    }
}

impl ToCss for MediaConditionTerm {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        match *self {
            MediaConditionTerm::Connective(ref term) => term.to_css(dest),
            MediaConditionTerm::InParens(ref term) => term.to_css(dest)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MediaConnectiveTerm {
    Not(MediaInParensTerm),
    And(Vec<MediaInParensTerm>),
    Or(Vec<MediaInParensTerm>)
}

derive_display_using_to_css!(MediaConnectiveTerm);

impl FromCss for MediaConnectiveTerm {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<MediaConnectiveTerm, ()> {
        use ::cssparser::Token;

        use std::ascii::AsciiExt;

        macro_rules! expect_whitespace {
            ($input:ident) => {
                match try!($input.next_including_whitespace()) {
                    Token::WhiteSpace(_) => Ok(()),
                    _ => Err(())
                }
            }
        }

        // <media-not> = not <media-in-parens>
        // <media-and> = <media-in-parens> [ and <media-in-parens> ]+
        // <media-or> = <media-in-parens> [ or <media-in-parens> ]+
        if input.try(|input| input.expect_ident_matching("not")).is_ok() {
            try!(expect_whitespace!(input));
            ok_if_exhausted!(input, MediaConnectiveTerm::Not)
        } else {
            // <media-in-parens>
            let mut terms = vec![];
            terms.push(try!(FromCss::from_css(input)));

            // and | or
            try!(expect_whitespace!(input));
            let connective = match try!(input.expect_ident()) {
                ref c if c.eq_ignore_ascii_case("and") => "and",
                ref c if c.eq_ignore_ascii_case("or") => "or",
                _ => return Err(())
            };

            // <media-in-parens> [ (and | or) <media-in-parens> ]*
            loop {
                try!(expect_whitespace!(input));
                terms.push(try!(FromCss::from_css(input)));
                if input.is_exhausted() {
                    break;
                }

                try!(expect_whitespace!(input));
                try!(input.expect_ident_matching(connective));
            }

            match connective {
                "and" => Ok(MediaConnectiveTerm::And(terms)),
                "or" => Ok(MediaConnectiveTerm::Or(terms)),
                _ => unreachable!()
            }
        }
    }
}

impl ToCss for MediaConnectiveTerm {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        macro_rules! write_terms {
            ($terms:ident, $connective:expr) => {{
                try!($terms[0].to_css(dest));
                for term in &$terms[1..] {
                    try!(write!(dest, concat!(" ", $connective, " ")));
                    try!(term.to_css(dest));
                }
                Ok(())
            }}
        }

        match *self {
            MediaConnectiveTerm::Not(ref term) => {
                try!(dest.write_str("not "));
                term.to_css(dest)
            }
            MediaConnectiveTerm::And(ref terms) => write_terms!(terms, "and"),
            MediaConnectiveTerm::Or(ref terms) => write_terms!(terms, "or"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MediaInParensTerm {
    Condition(Box<MediaConditionTerm>),
    Feature(MediaFeature),
    GeneralEnclosed(String)
}

derive_display_using_to_css!(MediaInParensTerm);

impl FromCss for MediaInParensTerm {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<MediaInParensTerm, ()> {
        // <media-in-parens> = <media-feature> | ( <media-condition> ) | <general-enclosed>
        if let Ok(feature) = input.try(FromCss::from_css) {
            return Ok(MediaInParensTerm::Feature(feature))
        }

        if input.try(|input| input.expect_parenthesis_block()).is_ok() {
            let condition = try!(input.parse_nested_block(FromCss::from_css));
            Ok(MediaInParensTerm::Condition(Box::new(condition)))
        } else {
            let name = try!(input.expect_function());
            Ok(MediaInParensTerm::GeneralEnclosed(name.into_owned()))
        }
    }
}

impl ToCss for MediaInParensTerm {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        match *self {
            MediaInParensTerm::Condition(ref term) => {
                try!(write!(dest, "("));
                try!(term.to_css(dest));
                write!(dest, ")")
            }
            MediaInParensTerm::Feature(ref term) =>
                term.to_css(dest),
            MediaInParensTerm::GeneralEnclosed(ref name) =>
                write!(dest, "{}()", name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::feature::MediaFeature;
    use super::{MediaConditionTerm, MediaConnectiveTerm, MediaInParensTerm};

    macro_rules! from_css {
        ($css:expr) => {
            ::FromCss::from_css(&mut ::cssparser::Parser::new($css))
        };
        ($css:expr => Err($term:ty)) => {
            <$term as ::FromCss>::from_css(&mut ::cssparser::Parser::new($css))
        }
    }

    #[test]
    fn media_condition_from_css() {
        match from_css!("(scan)").unwrap() {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Scan(None))) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and (not (hover)) or (grid)" => Err(MediaConditionTerm)) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and ((not (hover)) or (grid))").unwrap() {
            MediaConditionTerm::Connective(
                MediaConnectiveTerm::And(ref terms)) => match &terms {
                [MediaInParensTerm::Feature(MediaFeature::Scan(None)),
                 MediaInParensTerm::Condition(
                    box MediaConditionTerm::Connective(
                        MediaConnectiveTerm::Or(ref terms)))] => match &terms {
                        [MediaInParensTerm::Condition(
                             box MediaConditionTerm::Connective(
                                 MediaConnectiveTerm::Not(
                                     MediaInParensTerm::Feature(MediaFeature::Hover(None))))),
                         MediaInParensTerm::Feature(MediaFeature::Grid(None))] => {}
                        t => panic!("condition did not match: actual {:?}", t)
                    },
                    t => panic!("condition did not match: actual {:?}", t)
                },
            t => panic!("condition did not match: actual {:?}", t)
        }
    }

    #[test]
    fn media_connective_from_css() {
        match from_css!("not (scan)").unwrap() {
            MediaConnectiveTerm::Not(
                MediaInParensTerm::Feature(MediaFeature::Scan(None))) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and (grid)").unwrap() {
            MediaConnectiveTerm::And(ref terms) => match &terms {
                [MediaInParensTerm::Feature(MediaFeature::Scan(None)),
                 MediaInParensTerm::Feature(MediaFeature::Grid(None))] => {}
                t => panic!("condition did not match: actual {:?}", t)
            },
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) or (grid)").unwrap() {
            MediaConnectiveTerm::Or(ref terms) => match &terms {
                [MediaInParensTerm::Feature(MediaFeature::Scan(None)),
                 MediaInParensTerm::Feature(MediaFeature::Grid(None))] => {}
                t => panic!("condition did not match: actual {:?}", t)
            },
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("not (scan) and (hover)" => Err(MediaConnectiveTerm)) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and (hover) or (grid)" => Err(MediaConnectiveTerm)) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }
    }

    #[test]
    fn media_in_parens_from_css() {
        match from_css!("(scan)").unwrap() {
            MediaInParensTerm::Feature(MediaFeature::Scan(None)) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("((scan))").unwrap() {
            MediaInParensTerm::Condition(
                box MediaConditionTerm::InParens(
                    MediaInParensTerm::Feature(MediaFeature::Scan(None)))) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("calc(...)").unwrap() {
            MediaInParensTerm::GeneralEnclosed(ref name) => match &name {
                "calc" => {}
                _ => panic!("condition did not match: actual {:?}", name)
            },
            t => panic!("condition did not match: actual {:?}", t)
        }
    }
}
