/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Token, Parser, Delimiter};
use std::ascii::AsciiExt;

use euclid::size::{Size2D, TypedSize2D};
use properties::longhands;
use util::geometry::{Au, ViewportPx};
use values::specified;


#[derive(Debug, PartialEq)]
pub struct MediaQueryList {
    pub media_queries: Vec<MediaQuery>
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Range<T> {
    Min(T),
    Max(T),
    //Eq(T),    // FIXME: Implement parsing support for equality then re-enable this.
}

impl Range<specified::Length> {
    fn to_computed_range(&self, viewport_size: Size2D<Au>) -> Range<Au> {
        let compute_width = |&width| {
            match width {
                specified::Length::Absolute(value) => value,
                specified::Length::FontRelative(value) => {
                    // http://dev.w3.org/csswg/mediaqueries3/#units
                    // em units are relative to the initial font-size.
                    let initial_font_size = longhands::font_size::get_initial_value();
                    value.to_computed_value(initial_font_size, initial_font_size)
                }
                specified::Length::ViewportPercentage(value) =>
                    value.to_computed_value(viewport_size),
                _ => unreachable!()
            }
        };

        match *self {
            Range::Min(ref width) => Range::Min(compute_width(width)),
            Range::Max(ref width) => Range::Max(compute_width(width)),
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
pub enum Expression {
    /// http://dev.w3.org/csswg/mediaqueries-3/#width
    Width(Range<specified::Length>),
}

/// http://dev.w3.org/csswg/mediaqueries-3/#media0
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Qualifier {
    Only,
    Not,
}

#[derive(Debug, PartialEq)]
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

/// http://dev.w3.org/csswg/mediaqueries-3/#media0
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum MediaQueryType {
    All,  // Always true
    MediaType(MediaType),
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum MediaType {
    Screen,
    Print,
    Unknown,
}

#[derive(Debug)]
pub struct Device {
    pub media_type: MediaType,
    pub viewport_size: TypedSize2D<ViewportPx, f32>,
}

impl Device {
    pub fn new(media_type: MediaType, viewport_size: TypedSize2D<ViewportPx, f32>) -> Device {
        Device {
            media_type: media_type,
            viewport_size: viewport_size,
        }
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
                }
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
                "all" => MediaQueryType::All
                _ => MediaQueryType::MediaType(MediaType::Unknown)
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

pub fn parse_media_query_list(input: &mut Parser) -> MediaQueryList {
    let queries = if input.is_exhausted() {
        vec![MediaQuery::new(None, MediaQueryType::All, vec!())]
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
        media_queries
    };
    MediaQueryList { media_queries: queries }
}

impl MediaQueryList {
    pub fn evaluate(&self, device: &Device) -> bool {
        let viewport_size = Size2D::new(Au::from_f32_px(device.viewport_size.width.get()),
                                        Au::from_f32_px(device.viewport_size.height.get()));

        // Check if any queries match (OR condition)
        self.media_queries.iter().any(|mq| {
            // Check if media matches. Unknown media never matches.
            let media_match = match mq.media_type {
                MediaQueryType::MediaType(MediaType::Unknown) => false,
                MediaQueryType::MediaType(media_type) => media_type == device.media_type,
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
