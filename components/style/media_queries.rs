/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [Media queries][mq].
//!
//! [mq]: https://drafts.csswg.org/mediaqueries/

use Atom;
use cssparser::{Delimiter, Parser, Token};
use serialize_comma_separated_list;
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::ToCss;

#[cfg(feature = "servo")]
pub use servo::media_queries::{Device, Expression};
#[cfg(feature = "gecko")]
pub use gecko::media_queries::{Device, Expression};

/// A type that encapsulates a media query list.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MediaList {
    /// The list of media queries.
    pub media_queries: Vec<MediaQuery>
}

impl ToCss for MediaList {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        serialize_comma_separated_list(dest, &self.media_queries)
    }
}

impl Default for MediaList {
    fn default() -> MediaList {
        MediaList { media_queries: vec![] }
    }
}

/// https://drafts.csswg.org/mediaqueries/#mq-prefix
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Qualifier {
    /// Hide a media query from legacy UAs:
    /// https://drafts.csswg.org/mediaqueries/#mq-only
    Only,
    /// Negate a media query:
    /// https://drafts.csswg.org/mediaqueries/#mq-not
    Not,
}

impl ToCss for Qualifier {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            Qualifier::Not => write!(dest, "not"),
            Qualifier::Only => write!(dest, "only"),
        }
    }
}

/// A [media query][mq].
///
/// [mq]: https://drafts.csswg.org/mediaqueries/
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MediaQuery {
    /// The qualifier for this query.
    pub qualifier: Option<Qualifier>,
    /// The media type for this query, that can be known, unknown, or "all".
    pub media_type: MediaQueryType,
    /// The set of expressions that this media query contains.
    pub expressions: Vec<Expression>,
}

impl MediaQuery {
    /// Return a media query that never matches, used for when we fail to parse
    /// a given media query.
    fn never_matching() -> Self {
        Self::new(Some(Qualifier::Not), MediaQueryType::All, vec![])
    }

    /// Trivially constructs a new media query.
    pub fn new(qualifier: Option<Qualifier>,
               media_type: MediaQueryType,
               expressions: Vec<Expression>) -> MediaQuery {
        MediaQuery {
            qualifier: qualifier,
            media_type: media_type,
            expressions: expressions,
        }
    }
}

impl ToCss for MediaQuery {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        if let Some(qual) = self.qualifier {
            try!(qual.to_css(dest));
            try!(write!(dest, " "));
        }

        match self.media_type {
            MediaQueryType::All => {
                // We need to print "all" if there's a qualifier, or there's
                // just an empty list of expressions.
                //
                // Otherwise, we'd serialize media queries like "(min-width:
                // 40px)" in "all (min-width: 40px)", which is unexpected.
                if self.qualifier.is_some() || self.expressions.is_empty() {
                    try!(write!(dest, "all"));
                }
            },
            MediaQueryType::Known(MediaType::Screen) => try!(write!(dest, "screen")),
            MediaQueryType::Known(MediaType::Print) => try!(write!(dest, "print")),
            MediaQueryType::Unknown(ref desc) => try!(write!(dest, "{}", desc)),
        }

        if self.expressions.is_empty() {
            return Ok(());
        }

        if self.media_type != MediaQueryType::All || self.qualifier.is_some() {
            try!(write!(dest, " and "));
        }

        try!(self.expressions[0].to_css(dest));

        for expr in self.expressions.iter().skip(1) {
            try!(write!(dest, " and "));
            try!(expr.to_css(dest));
        }
        Ok(())
    }
}

/// http://dev.w3.org/csswg/mediaqueries-3/#media0
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum MediaQueryType {
    /// A media type that matches every device.
    All,
    /// A known media type, that we parse and understand.
    Known(MediaType),
    /// An unknown media type.
    Unknown(Atom),
}

impl MediaQueryType {
    fn parse(ident: &str) -> Self {
        if ident.eq_ignore_ascii_case("all") {
            return MediaQueryType::All;
        }

        match MediaType::parse(ident) {
            Some(media_type) => MediaQueryType::Known(media_type),
            None => MediaQueryType::Unknown(Atom::from(ident)),
        }
    }

    fn matches(&self, other: MediaType) -> bool {
        match *self {
            MediaQueryType::All => true,
            MediaQueryType::Known(ref known_type) => *known_type == other,
            MediaQueryType::Unknown(..) => false,
        }
    }
}

/// https://drafts.csswg.org/mediaqueries/#media-types
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum MediaType {
    /// The "screen" media type.
    Screen,
    /// The "print" media type.
    Print,
}

impl MediaType {
    fn parse(name: &str) -> Option<Self> {
        Some(match_ignore_ascii_case! { name,
            "screen" => MediaType::Screen,
            "print" => MediaType::Print,
            _ => return None
        })
    }
}
impl MediaQuery {
    /// Parse a media query given css input.
    ///
    /// Returns an error if any of the expressions is unknown.
    pub fn parse(input: &mut Parser) -> Result<MediaQuery, ()> {
        let mut expressions = vec![];

        let qualifier = if input.try(|input| input.expect_ident_matching("only")).is_ok() {
            Some(Qualifier::Only)
        } else if input.try(|input| input.expect_ident_matching("not")).is_ok() {
            Some(Qualifier::Not)
        } else {
            None
        };

        let media_type = match input.try(|input| input.expect_ident()) {
            Ok(ident) => MediaQueryType::parse(&*ident),
            Err(()) => {
                // Media type is only optional if qualifier is not specified.
                if qualifier.is_some() {
                    return Err(())
                }

                // Without a media type, require at least one expression.
                expressions.push(try!(Expression::parse(input)));

                MediaQueryType::All
            }
        };

        // Parse any subsequent expressions
        loop {
            if input.try(|input| input.expect_ident_matching("and")).is_err() {
                return Ok(MediaQuery::new(qualifier, media_type, expressions))
            }
            expressions.push(try!(Expression::parse(input)))
        }
    }
}

/// Parse a media query list from CSS.
///
/// Always returns a media query list. If any invalid media query is found, the
/// media query list is only filled with the equivalent of "not all", see:
///
/// https://drafts.csswg.org/mediaqueries/#error-handling
pub fn parse_media_query_list(input: &mut Parser) -> MediaList {
    if input.is_exhausted() {
        return Default::default()
    }

    let mut media_queries = vec![];
    let mut found_invalid = false;
    loop {
        match input.parse_until_before(Delimiter::Comma, MediaQuery::parse) {
            Ok(mq) => if !found_invalid {
                media_queries.push(mq);
            },
            Err(..) => if !found_invalid {
                media_queries.clear();
                media_queries.push(MediaQuery::never_matching());
                // Consume the rest of the input as if they were valid
                // expressions (they might be, they might not), but ignore the
                // result, this allows correctly parsing invalid media queries.
                found_invalid = true;
            },
        }

        match input.next() {
            Ok(Token::Comma) => {},
            Ok(_) => unreachable!(),
            Err(()) => break,
        }
    }

    debug_assert!(!found_invalid || media_queries.len() == 1);

    MediaList {
        media_queries: media_queries,
    }
}

impl MediaList {
    /// Evaluate a whole `MediaList` against `Device`.
    pub fn evaluate(&self, device: &Device) -> bool {
        // Check if it is an empty media query list or any queries match (OR condition)
        // https://drafts.csswg.org/mediaqueries-4/#mq-list
        self.media_queries.is_empty() || self.media_queries.iter().any(|mq| {
            let media_match = mq.media_type.matches(device.media_type());

            // Check if all conditions match (AND condition)
            let query_match =
                media_match &&
                mq.expressions.iter()
                    .all(|expression| expression.matches(&device));

            // Apply the logical NOT qualifier to the result
            match mq.qualifier {
                Some(Qualifier::Not) => !query_match,
                _ => query_match,
            }
        })
    }

    /// Whether this `MediaList` contains no media queries.
    pub fn is_empty(&self) -> bool {
        self.media_queries.is_empty()
    }
}
