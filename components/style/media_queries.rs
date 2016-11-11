/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [Media queries][mq].
//!
//! [mq]: https://drafts.csswg.org/mediaqueries/

use Atom;
use app_units::Au;
use cssparser::{Delimiter, Parser, Token};
use euclid::size::{Size2D, TypedSize2D};
use serialize_comma_separated_list;
use std::fmt::{self, Write};
use style_traits::{ToCss, ViewportPx};
use values::computed::{self, ToComputedValue};
use values::specified;


#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MediaList {
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

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Range<T> {
    Min(T),
    Max(T),
    //Eq(T),    // FIXME: Implement parsing support for equality then re-enable this.
}

impl Range<specified::Length> {
    fn to_computed_range(&self, viewport_size: Size2D<Au>) -> Range<Au> {
        // http://dev.w3.org/csswg/mediaqueries3/#units
        // em units are relative to the initial font-size.
        let context = computed::Context::initial(viewport_size, false);

        match *self {
            Range::Min(ref width) => Range::Min(width.to_computed_value(&context)),
            Range::Max(ref width) => Range::Max(width.to_computed_value(&context)),
            //Range::Eq(ref width) => Range::Eq(compute_width(width))
        }
    }
}

impl<T: Ord> Range<T> {
    fn evaluate(&self, value: T) -> bool {
        match *self {
            Range::Min(ref width) => { value >= *width },
            Range::Max(ref width) => { value <= *width },
            //Range::Eq(ref width) => { value == *width },
        }
    }
}

/// http://dev.w3.org/csswg/mediaqueries-3/#media1
#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Expression {
    /// http://dev.w3.org/csswg/mediaqueries-3/#width
    Width(Range<specified::Length>),
}

/// http://dev.w3.org/csswg/mediaqueries-3/#media0
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Qualifier {
    Only,
    Not,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MediaQuery {
    pub qualifier: Option<Qualifier>,
    pub media_type: MediaQueryType,
    pub expressions: Vec<Expression>,
}

impl MediaQuery {
    pub fn new(qualifier: Option<Qualifier>, media_type: MediaQueryType,
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
        where W: fmt::Write
    {
        if self.qualifier == Some(Qualifier::Not) {
            try!(write!(dest, "not "));
        }

        let mut type_ = String::new();
        match self.media_type {
            MediaQueryType::All => try!(write!(type_, "all")),
            MediaQueryType::MediaType(MediaType::Screen) => try!(write!(type_, "screen")),
            MediaQueryType::MediaType(MediaType::Print) => try!(write!(type_, "print")),
            MediaQueryType::MediaType(MediaType::Unknown(ref desc)) => try!(write!(type_, "{}", desc)),
        };
        if self.expressions.is_empty() {
            return write!(dest, "{}", type_)
        } else if type_ != "all" || self.qualifier == Some(Qualifier::Not) {
            try!(write!(dest, "{} and ", type_));
        }
        for (i, &e) in self.expressions.iter().enumerate() {
            try!(write!(dest, "("));
            let (mm, l) = match e {
                Expression::Width(Range::Min(ref l)) => ("min", l),
                Expression::Width(Range::Max(ref l)) => ("max", l),
            };
            try!(write!(dest, "{}-width: ", mm));
            try!(l.to_css(dest));
            if i == self.expressions.len() - 1 {
                try!(write!(dest, ")"));
            } else {
                try!(write!(dest, ") and "));
            }
        }
        Ok(())
    }
}

/// http://dev.w3.org/csswg/mediaqueries-3/#media0
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum MediaQueryType {
    All,  // Always true
    MediaType(MediaType),
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum MediaType {
    Screen,
    Print,
    Unknown(Atom),
}

#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Device {
    pub media_type: MediaType,
    pub viewport_size: TypedSize2D<f32, ViewportPx>,
}

impl Device {
    pub fn new(media_type: MediaType, viewport_size: TypedSize2D<f32, ViewportPx>) -> Device {
        Device {
            media_type: media_type,
            viewport_size: viewport_size,
        }
    }

    #[inline]
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        Size2D::new(Au::from_f32_px(self.viewport_size.width),
                    Au::from_f32_px(self.viewport_size.height))
    }

}

impl Expression {
    fn parse(input: &mut Parser) -> Result<Expression, ()> {
        try!(input.expect_parenthesis_block());
        input.parse_nested_block(|input| {
            let name = try!(input.expect_ident());
            try!(input.expect_colon());
            // TODO: Handle other media features
            match_ignore_ascii_case! { name,
                "min-width" => {
                    Ok(Expression::Width(Range::Min(try!(specified::Length::parse_non_negative(input)))))
                },
                "max-width" => {
                    Ok(Expression::Width(Range::Max(try!(specified::Length::parse_non_negative(input)))))
                },
                _ => Err(())
            }
        })
    }
}

impl MediaQuery {
    fn parse(input: &mut Parser) -> Result<MediaQuery, ()> {
        let mut expressions = vec![];

        let qualifier = if input.try(|input| input.expect_ident_matching("only")).is_ok() {
            Some(Qualifier::Only)
        } else if input.try(|input| input.expect_ident_matching("not")).is_ok() {
            Some(Qualifier::Not)
        } else {
            None
        };

        let media_type;
        if let Ok(ident) = input.try(|input| input.expect_ident()) {
            media_type = match_ignore_ascii_case! { ident,
                "screen" => MediaQueryType::MediaType(MediaType::Screen),
                "print" => MediaQueryType::MediaType(MediaType::Print),
                "all" => MediaQueryType::All,
                _ => MediaQueryType::MediaType(MediaType::Unknown(Atom::from(&*ident)))
            }
        } else {
            // Media type is only optional if qualifier is not specified.
            if qualifier.is_some() {
                return Err(())
            }
            media_type = MediaQueryType::All;
            // Without a media type, require at least one expression
            expressions.push(try!(Expression::parse(input)));
        }

        // Parse any subsequent expressions
        loop {
            if input.try(|input| input.expect_ident_matching("and")).is_err() {
                return Ok(MediaQuery::new(qualifier, media_type, expressions))
            }
            expressions.push(try!(Expression::parse(input)))
        }
    }
}

pub fn parse_media_query_list(input: &mut Parser) -> MediaList {
    if input.is_exhausted() {
        Default::default()
    } else {
        let mut media_queries = vec![];
        loop {
            media_queries.push(
                input.parse_until_before(Delimiter::Comma, MediaQuery::parse)
                     .unwrap_or(MediaQuery::new(Some(Qualifier::Not),
                                                MediaQueryType::All,
                                                vec!())));
            match input.next() {
                Ok(Token::Comma) => continue,
                Ok(_) => unreachable!(),
                Err(()) => break,
            }
        }
        MediaList { media_queries: media_queries }
    }
}

impl MediaList {
    pub fn evaluate(&self, device: &Device) -> bool {
        let viewport_size = device.au_viewport_size();

        // Check if it is an empty media query list or any queries match (OR condition)
        // https://drafts.csswg.org/mediaqueries-4/#mq-list
        self.media_queries.is_empty() || self.media_queries.iter().any(|mq| {
            // Check if media matches. Unknown media never matches.
            let media_match = match mq.media_type {
                MediaQueryType::MediaType(MediaType::Unknown(_)) => false,
                MediaQueryType::MediaType(ref media_type) => *media_type == device.media_type,
                MediaQueryType::All => true,
            };

            // Check if all conditions match (AND condition)
            let query_match = media_match && mq.expressions.iter().all(|expression| {
                match *expression {
                    Expression::Width(ref value) =>
                        value.to_computed_range(viewport_size).evaluate(viewport_size.width),
                }
            });

            // Apply the logical NOT qualifier to the result
            match mq.qualifier {
                Some(Qualifier::Not) => !query_match,
                _ => query_match,
            }
        })
    }
}
