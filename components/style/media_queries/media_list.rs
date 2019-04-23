/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A media query list:
//!
//! https://drafts.csswg.org/mediaqueries/#typedef-media-query-list

use super::{Device, MediaQuery, Qualifier};
use crate::context::QuirksMode;
use crate::error_reporting::ContextualParseError;
use crate::parser::ParserContext;
use cssparser::{Delimiter, Parser};
use cssparser::{ParserInput, Token};

/// A type that encapsulates a media query list.
#[css(comma, derive_debug)]
#[derive(Clone, MallocSizeOf, ToCss, ToShmem)]
pub struct MediaList {
    /// The list of media queries.
    #[css(iterable)]
    pub media_queries: Vec<MediaQuery>,
}

impl MediaList {
    /// Parse a media query list from CSS.
    ///
    /// Always returns a media query list. If any invalid media query is
    /// found, the media query list is only filled with the equivalent of
    /// "not all", see:
    ///
    /// <https://drafts.csswg.org/mediaqueries/#error-handling>
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Self {
        if input.is_exhausted() {
            return Self::empty();
        }

        let mut media_queries = vec![];
        loop {
            let start_position = input.position();
            match input.parse_until_before(Delimiter::Comma, |i| MediaQuery::parse(context, i)) {
                Ok(mq) => {
                    media_queries.push(mq);
                },
                Err(err) => {
                    media_queries.push(MediaQuery::never_matching());
                    let location = err.location;
                    let error = ContextualParseError::InvalidMediaRule(
                        input.slice_from(start_position),
                        err,
                    );
                    context.log_css_error(location, error);
                },
            }

            match input.next() {
                Ok(&Token::Comma) => {},
                Ok(_) => unreachable!(),
                Err(_) => break,
            }
        }

        MediaList { media_queries }
    }

    /// Create an empty MediaList.
    pub fn empty() -> Self {
        MediaList {
            media_queries: vec![],
        }
    }

    /// Evaluate a whole `MediaList` against `Device`.
    pub fn evaluate(&self, device: &Device, quirks_mode: QuirksMode) -> bool {
        // Check if it is an empty media query list or any queries match.
        // https://drafts.csswg.org/mediaqueries-4/#mq-list
        self.media_queries.is_empty() ||
            self.media_queries.iter().any(|mq| {
                let media_match = mq.media_type.matches(device.media_type());

                // Check if the media condition match.
                let query_match = media_match &&
                    mq.condition
                        .as_ref()
                        .map_or(true, |c| c.matches(device, quirks_mode));

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

    /// Append a new media query item to the media list.
    /// <https://drafts.csswg.org/cssom/#dom-medialist-appendmedium>
    ///
    /// Returns true if added, false if fail to parse the medium string.
    pub fn append_medium(&mut self, context: &ParserContext, new_medium: &str) -> bool {
        let mut input = ParserInput::new(new_medium);
        let mut parser = Parser::new(&mut input);
        let new_query = match MediaQuery::parse(&context, &mut parser) {
            Ok(query) => query,
            Err(_) => {
                return false;
            },
        };
        // This algorithm doesn't actually matches the current spec,
        // but it matches the behavior of Gecko and Edge.
        // See https://github.com/w3c/csswg-drafts/issues/697
        self.media_queries.retain(|query| query != &new_query);
        self.media_queries.push(new_query);
        true
    }

    /// Delete a media query from the media list.
    /// <https://drafts.csswg.org/cssom/#dom-medialist-deletemedium>
    ///
    /// Returns true if found and deleted, false otherwise.
    pub fn delete_medium(&mut self, context: &ParserContext, old_medium: &str) -> bool {
        let mut input = ParserInput::new(old_medium);
        let mut parser = Parser::new(&mut input);
        let old_query = match MediaQuery::parse(context, &mut parser) {
            Ok(query) => query,
            Err(_) => {
                return false;
            },
        };
        let old_len = self.media_queries.len();
        self.media_queries.retain(|query| query != &old_query);
        old_len != self.media_queries.len()
    }
}
