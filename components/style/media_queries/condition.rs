/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{DeviceFeatureContext, EvaluateUsingContext, SpecificationLevel};
use super::feature::MediaFeature;

use ::FromCss;
use ::cssparser::{Parser, ToCss};

#[derive(Debug, PartialEq)]
pub struct MediaCondition(pub MediaConditionTerm);

derive_display_using_to_css!(MediaCondition);

impl<C> EvaluateUsingContext<C> for MediaCondition
    where C: DeviceFeatureContext
{
    #[inline]
    fn evaluate(&self, context: &C) -> bool {
        self.0.evaluate(context)
    }
}

impl FromCss for MediaCondition {
    type Err = ();
    type Context = SpecificationLevel;

    #[inline]
    fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaCondition, ()> {
        FromCss::from_css(input, level).map(|r| MediaCondition(r))
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
    ($input:ident, $context:ident, $term:path) => {
        FromCss::from_css($input, $context)
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

impl<C> EvaluateUsingContext<C> for MediaConditionTerm
    where C: DeviceFeatureContext
{
    fn evaluate(&self, context: &C) -> bool {
        match *self {
            MediaConditionTerm::Connective(ref term) => term.evaluate(context),
            MediaConditionTerm::InParens(ref term) => term.evaluate(context),
        }
    }
}

impl FromCss for MediaConditionTerm {
    type Err = ();
    type Context = SpecificationLevel;

    #[inline]
    fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaConditionTerm, ()> {
        // <media-condition> = <media-connective> | <media-in-parens>
        if let Ok(connective) = input.try(|input| FromCss::from_css(input, level)) {
            Ok(MediaConditionTerm::Connective(connective))
        } else {
            ok_if_exhausted!(input, level, MediaConditionTerm::InParens)
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

impl<C> EvaluateUsingContext<C> for MediaConnectiveTerm
    where C: DeviceFeatureContext
{
    fn evaluate(&self, context: &C) -> bool {
        match *self {
            MediaConnectiveTerm::Not(ref term) =>
                !term.evaluate(context),
            MediaConnectiveTerm::And(ref terms) =>
                terms.iter().all(|term| term.evaluate(context)),
            MediaConnectiveTerm::Or(ref terms) =>
                terms.iter().any(|term| term.evaluate(context))
        }
    }
}

impl FromCss for MediaConnectiveTerm {
    type Err = ();
    type Context = SpecificationLevel;

    fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaConnectiveTerm, ()> {
        use ::cssparser::Token;

        use std::ascii::AsciiExt;

        match level {
            &SpecificationLevel::MEDIAQ3 => {
                // <expression> and
                let mut terms = vec![];
                terms.push(try!(FromCss::from_css(input, level)));
                try!(expect_and!(input));

                // <expression> [ and <expression> ]*
                loop {
                    terms.push(try!(FromCss::from_css(input, level)));
                    if input.is_exhausted() {
                        break;
                    }
                    try!(expect_and!(input));
                }

                Ok(MediaConnectiveTerm::And(terms))
            }
            &SpecificationLevel::MEDIAQ4 => {
                // <media-not> = not <media-in-parens>
                // <media-and> = <media-in-parens> [ and <media-in-parens> ]+
                // <media-or> = <media-in-parens> [ or <media-in-parens> ]+
                if input.try(|input| input.expect_ident_matching("not")).is_ok() {
                    try!(expect_whitespace!(input));
                    ok_if_exhausted!(input, level, MediaConnectiveTerm::Not)
                } else {
                    // <media-in-parens>
                    let mut terms = vec![];
                    terms.push(try!(FromCss::from_css(input, level)));

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
                        terms.push(try!(FromCss::from_css(input, level)));
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
                try!(write!(dest, "not "));
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

impl<C> EvaluateUsingContext<C> for MediaInParensTerm
    where C: DeviceFeatureContext
{
    fn evaluate(&self, context: &C) -> bool {
        match *self {
            MediaInParensTerm::Condition(ref term) =>
                term.evaluate(context),
            MediaInParensTerm::Feature(ref feature) =>
                feature.evaluate(context),
            MediaInParensTerm::GeneralEnclosed(_) =>
                false
        }
    }
}

impl FromCss for MediaInParensTerm {
    type Err = ();
    type Context = SpecificationLevel;

    fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaInParensTerm, ()> {
        match level {
            &SpecificationLevel::MEDIAQ3 => {
                // <expression> = <media_feature>
                Ok(MediaInParensTerm::Feature(try!(FromCss::from_css(input, level))))
            }
            &SpecificationLevel::MEDIAQ4 => {
                // <media-in-parens> = <media-feature> | ( <media-condition> ) | <general-enclosed>
                if let Ok(feature) = input.try(|input| FromCss::from_css(input, level)) {
                    return Ok(MediaInParensTerm::Feature(feature))
                }

                if input.try(|input| input.expect_parenthesis_block()).is_ok() {
                    let condition = try!(input.parse_nested_block(|input| FromCss::from_css(input, level)));
                    Ok(MediaInParensTerm::Condition(Box::new(condition)))
                } else {
                    let name = try!(input.expect_function());
                    Ok(MediaInParensTerm::GeneralEnclosed(name.into_owned()))
                }
            }
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
    use super::super::SpecificationLevel;
    use super::super::feature::MediaFeature;
    use super::{MediaConditionTerm, MediaConnectiveTerm, MediaInParensTerm};

    macro_rules! from_css {
        ($css:expr, $level:ident) => {
            ::FromCss::from_css(&mut ::cssparser::Parser::new($css), &SpecificationLevel::$level)
        };
        ($css:expr => Err($term:ty), $level:ident) => {
            <$term as ::FromCss>::from_css(&mut ::cssparser::Parser::new($css), &SpecificationLevel::$level)
        }
    }

    #[test]
    fn media_condition_from_css() {
        match from_css!("(scan)", MEDIAQ3).unwrap() {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Scan(None))) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan)", MEDIAQ4).unwrap() {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Scan(None))) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and (not (hover)) or (grid)" => Err(MediaConditionTerm), MEDIAQ3) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and (not (hover)) or (grid)" => Err(MediaConditionTerm), MEDIAQ4) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and ((not (hover)) or (grid))" => Err(MediaConditionTerm), MEDIAQ3) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and ((not (hover)) or (grid))", MEDIAQ4).unwrap() {
            MediaConditionTerm::Connective(
                MediaConnectiveTerm::And(ref terms)) => match &terms[..] {
                [MediaInParensTerm::Feature(MediaFeature::Scan(None)),
                 MediaInParensTerm::Condition(
                     box MediaConditionTerm::Connective(
                         MediaConnectiveTerm::Or(ref terms)))] => match &terms[..] {
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
        // not
        match from_css!("not (scan)" => Err(MediaConnectiveTerm), MEDIAQ3) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("not (scan)", MEDIAQ4) {
            Ok(MediaConnectiveTerm::Not(
                MediaInParensTerm::Feature(MediaFeature::Scan(None)))) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t),
            Err(_) => panic!("condition did not match: error parsing"),
        }

        // and
        match from_css!("(scan)and(grid)", MEDIAQ3) {
            Ok(MediaConnectiveTerm::And(ref terms)) => match &terms[..] {
                [MediaInParensTerm::Feature(MediaFeature::Scan(None)),
                 MediaInParensTerm::Feature(MediaFeature::Grid(None))] => {}
                t => panic!("condition did not match: actual {:?}", t)
            },
            Ok(t) => panic!("condition did not match: actual {:?}", t),
            Err(_) => panic!("condition did not match: error parsing"),
        }
        match from_css!("(scan)and(grid)" => Err(MediaConnectiveTerm), MEDIAQ4) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and (grid)", MEDIAQ4) {
            Ok(MediaConnectiveTerm::And(ref terms)) => match &terms[..] {
                [MediaInParensTerm::Feature(MediaFeature::Scan(None)),
                 MediaInParensTerm::Feature(MediaFeature::Grid(None))] => {}
                t => panic!("condition did not match: actual {:?}", t)
            },
            Ok(t) => panic!("condition did not match: actual {:?}", t),
            Err(_) => panic!("condition did not match: error parsing"),
        }

        // or
        match from_css!("(scan) or (grid)" => Err(MediaConnectiveTerm), MEDIAQ3) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) or (grid)", MEDIAQ4) {
            Ok(MediaConnectiveTerm::Or(ref terms)) => match &terms[..] {
                [MediaInParensTerm::Feature(MediaFeature::Scan(None)),
                 MediaInParensTerm::Feature(MediaFeature::Grid(None))] => {}
                t => panic!("condition did not match: actual {:?}", t)
            },
            Ok(t) => panic!("condition did not match: actual {:?}", t),
            Err(_) => panic!("condition did not match: error parsing"),
        }

        // invalid
        match from_css!("not (scan) and (hover)" => Err(MediaConnectiveTerm), MEDIAQ4) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan) and (hover) or (grid)" => Err(MediaConnectiveTerm), MEDIAQ4) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }
    }

    #[test]
    fn media_in_parens_from_css() {
        match from_css!("(scan)", MEDIAQ3).unwrap() {
            MediaInParensTerm::Feature(MediaFeature::Scan(None)) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("(scan)", MEDIAQ4).unwrap() {
            MediaInParensTerm::Feature(MediaFeature::Scan(None)) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("((scan))" => Err(MediaInParensTerm), MEDIAQ3) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("((scan))", MEDIAQ4).unwrap() {
            MediaInParensTerm::Condition(
                box MediaConditionTerm::InParens(
                    MediaInParensTerm::Feature(MediaFeature::Scan(None)))) => {}
            t => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("calc(...)" => Err(MediaInParensTerm), MEDIAQ3) {
            Err(_) => {}
            Ok(t) => panic!("condition did not match: actual {:?}", t)
        }

        match from_css!("calc(...)", MEDIAQ4).unwrap() {
            MediaInParensTerm::GeneralEnclosed(ref name) => match &name[..] {
                "calc" => {}
                _ => panic!("condition did not match: actual {:?}", name)
            },
            t => panic!("condition did not match: actual {:?}", t)
        }
    }
}
