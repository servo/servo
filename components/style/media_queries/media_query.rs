/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A media query:
//!
//! https://drafts.csswg.org/mediaqueries/#typedef-media-query

use Atom;
use cssparser::Parser;
use parser::ParserContext;
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use str::string_as_ascii_lowercase;
use style_traits::{CssWriter, ParseError, ToCss};
use super::media_condition::MediaCondition;
use values::CustomIdent;


/// <https://drafts.csswg.org/mediaqueries/#mq-prefix>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToCss)]
pub enum Qualifier {
    /// Hide a media query from legacy UAs:
    /// <https://drafts.csswg.org/mediaqueries/#mq-only>
    Only,
    /// Negate a media query:
    /// <https://drafts.csswg.org/mediaqueries/#mq-not>
    Not,
}

/// <https://drafts.csswg.org/mediaqueries/#media-types>
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
pub struct MediaType(pub CustomIdent);

impl MediaType {
    /// The `screen` media type.
    pub fn screen() -> Self {
        MediaType(CustomIdent(atom!("screen")))
    }

    /// The `print` media type.
    pub fn print() -> Self {
        MediaType(CustomIdent(atom!("print")))
    }

    fn parse(name: &str) -> Result<Self, ()> {
        // From https://drafts.csswg.org/mediaqueries/#mq-syntax:
        //
        //   The <media-type> production does not include the keywords not, or, and, and only.
        //
        // Here we also perform the to-ascii-lowercase part of the serialization
        // algorithm: https://drafts.csswg.org/cssom/#serializing-media-queries
        match_ignore_ascii_case! { name,
            "not" | "or" | "and" | "only" => Err(()),
            _ => Ok(MediaType(CustomIdent(Atom::from(string_as_ascii_lowercase(name))))),
        }
    }
}

/// A [media query][mq].
///
/// [mq]: https://drafts.csswg.org/mediaqueries/
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct MediaQuery {
    /// The qualifier for this query.
    pub qualifier: Option<Qualifier>,
    /// The media type for this query, that can be known, unknown, or "all".
    pub media_type: MediaQueryType,
    /// The condition that this media query contains. This cannot have `or`
    /// in the first level.
    pub condition: Option<MediaCondition>,
}

impl ToCss for MediaQuery {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if let Some(qual) = self.qualifier {
            qual.to_css(dest)?;
            dest.write_char(' ')?;
        }

        match self.media_type {
            MediaQueryType::All => {
                // We need to print "all" if there's a qualifier, or there's
                // just an empty list of expressions.
                //
                // Otherwise, we'd serialize media queries like "(min-width:
                // 40px)" in "all (min-width: 40px)", which is unexpected.
                if self.qualifier.is_some() || self.condition.is_none() {
                    dest.write_str("all")?;
                }
            },
            MediaQueryType::Concrete(MediaType(ref desc)) => desc.to_css(dest)?,
        }

        let condition = match self.condition {
            Some(ref c) => c,
            None => return Ok(()),
        };

        if self.media_type != MediaQueryType::All || self.qualifier.is_some() {
            dest.write_str(" and ")?;
        }

        condition.to_css(dest)
    }
}

impl MediaQuery {
    /// Return a media query that never matches, used for when we fail to parse
    /// a given media query.
    pub fn never_matching() -> Self {
        Self {
            qualifier: Some(Qualifier::Not),
            media_type: MediaQueryType::All,
            condition: None,
        }
    }

    /// Parse a media query given css input.
    ///
    /// Returns an error if any of the expressions is unknown.
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MediaQuery, ParseError<'i>> {
        if let Ok(condition) = input.try(|i| MediaCondition::parse(context, i)) {
            return Ok(Self {
                qualifier: None,
                media_type: MediaQueryType::All,
                condition: Some(condition),
            })
        }

        let qualifier = input.try(Qualifier::parse).ok();
        let media_type = {
            let location = input.current_source_location();
            let ident = input.expect_ident()?;
            match MediaQueryType::parse(&ident) {
                Ok(t) => t,
                Err(..) => return Err(location.new_custom_error(
                    SelectorParseErrorKind::UnexpectedIdent(ident.clone())
                ))
            }
        };

        let condition =
            if input.try(|i| i.expect_ident_matching("and")).is_ok() {
                Some(MediaCondition::parse_disallow_or(context, input)?)
            } else {
                None
            };

        Ok(Self { qualifier, media_type, condition })
    }
}

/// <http://dev.w3.org/csswg/mediaqueries-3/#media0>
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
pub enum MediaQueryType {
    /// A media type that matches every device.
    All,
    /// A specific media type.
    Concrete(MediaType),
}

impl MediaQueryType {
    fn parse(ident: &str) -> Result<Self, ()> {
        match_ignore_ascii_case! { ident,
            "all" => return Ok(MediaQueryType::All),
            _ => (),
        };

        // If parseable, accept this type as a concrete type.
        MediaType::parse(ident).map(MediaQueryType::Concrete)
    }

    /// Returns whether this media query type matches a MediaType.
    pub fn matches(&self, other: MediaType) -> bool {
        match *self {
            MediaQueryType::All => true,
            MediaQueryType::Concrete(ref known_type) => *known_type == other,
        }
    }
}
