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
use properties::ComputedValues;
use serialize_comma_separated_list;
use std::ascii::AsciiExt;
use std::fmt;
#[cfg(feature = "gecko")]
use std::sync::Arc;
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
    Eq(T),
}

impl Range<specified::Length> {
    fn to_computed_range(&self, viewport_size: Size2D<Au>, default_values: &ComputedValues) -> Range<Au> {
        // http://dev.w3.org/csswg/mediaqueries3/#units
        // em units are relative to the initial font-size.
        let context = computed::Context {
            is_root_element: false,
            viewport_size: viewport_size,
            inherited_style: default_values,
            // This cloning business is kind of dumb.... It's because Context
            // insists on having an actual ComputedValues inside itself.
            style: default_values.clone(),
            font_metrics_provider: None
        };

        match *self {
            Range::Min(ref width) => Range::Min(width.to_computed_value(&context)),
            Range::Max(ref width) => Range::Max(width.to_computed_value(&context)),
            Range::Eq(ref width) => Range::Eq(width.to_computed_value(&context))
        }
    }
}

impl<T: Ord> Range<T> {
    fn evaluate(&self, value: T) -> bool {
        match *self {
            Range::Min(ref width) => { value >= *width },
            Range::Max(ref width) => { value <= *width },
            Range::Eq(ref width) => { value == *width },
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

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MediaQuery {
    pub qualifier: Option<Qualifier>,
    pub media_type: MediaQueryType,
    pub expressions: Vec<Expression>,
}

impl MediaQuery {
    /// Return a media query that never matches, used for when we fail to parse
    /// a given media query.
    fn never_matching() -> Self {
        Self::new(Some(Qualifier::Not), MediaQueryType::All, vec![])
    }

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

        for (i, &e) in self.expressions.iter().enumerate() {
            try!(write!(dest, "("));
            let (mm, l) = match e {
                Expression::Width(Range::Min(ref l)) => ("min-", l),
                Expression::Width(Range::Max(ref l)) => ("max-", l),
                Expression::Width(Range::Eq(ref l)) => ("", l),
            };
            try!(write!(dest, "{}width: ", mm));
            try!(l.to_css(dest));
            try!(write!(dest, ")"));
            if i != self.expressions.len() - 1 {
                try!(write!(dest, " and "));
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
    Known(MediaType),
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

    fn matches(&self, other: &MediaType) -> bool {
        match *self {
            MediaQueryType::All => true,
            MediaQueryType::Known(ref known_type) => known_type == other,
            MediaQueryType::Unknown(..) => false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum MediaType {
    Screen,
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

#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Device {
    pub media_type: MediaType,
    pub viewport_size: TypedSize2D<f32, ViewportPx>,
    #[cfg(feature = "gecko")]
    pub default_values: Arc<ComputedValues>,
}

impl Device {
    #[cfg(feature = "servo")]
    pub fn new(media_type: MediaType, viewport_size: TypedSize2D<f32, ViewportPx>) -> Device {
        Device {
            media_type: media_type,
            viewport_size: viewport_size,
        }
    }

    #[cfg(feature = "servo")]
    pub fn default_values(&self) -> &ComputedValues {
        ComputedValues::initial_values()
    }

    #[cfg(feature = "gecko")]
    pub fn new(media_type: MediaType, viewport_size: TypedSize2D<f32, ViewportPx>,
               default_values: &Arc<ComputedValues>) -> Device {
        Device {
            media_type: media_type,
            viewport_size: viewport_size,
            default_values: default_values.clone(),
        }
    }

    #[cfg(feature = "gecko")]
    pub fn default_values(&self) -> &ComputedValues {
        &*self.default_values
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
                "width" => {
                    Ok(Expression::Width(Range::Eq(try!(specified::Length::parse_non_negative(input)))))
                },
                _ => Err(())
            }
        })
    }
}

impl MediaQuery {
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

pub fn parse_media_query_list(input: &mut Parser) -> MediaList {
    if input.is_exhausted() {
        return Default::default()
    }

    let mut media_queries = vec![];
    loop {
        media_queries.push(
            input.parse_until_before(Delimiter::Comma, MediaQuery::parse).ok()
                 .unwrap_or_else(MediaQuery::never_matching));
        match input.next() {
            Ok(Token::Comma) => {},
            Ok(_) => unreachable!(),
            Err(()) => break,
        }
    }
    MediaList {
        media_queries: media_queries,
    }
}

impl MediaList {
    pub fn evaluate(&self, device: &Device) -> bool {
        let viewport_size = device.au_viewport_size();

        // Check if it is an empty media query list or any queries match (OR condition)
        // https://drafts.csswg.org/mediaqueries-4/#mq-list
        self.media_queries.is_empty() || self.media_queries.iter().any(|mq| {
            let media_match = mq.media_type.matches(&device.media_type);

            // Check if all conditions match (AND condition)
            let query_match = media_match && mq.expressions.iter().all(|expression| {
                match *expression {
                    Expression::Width(ref value) =>
                        value.to_computed_range(viewport_size, device.default_values()).evaluate(viewport_size.width),
                }
            });

            // Apply the logical NOT qualifier to the result
            match mq.qualifier {
                Some(Qualifier::Not) => !query_match,
                _ => query_match,
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.media_queries.is_empty()
    }
}
