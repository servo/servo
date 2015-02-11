/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::AsciiExt;
use cssparser::{Token, Parser, Delimiter};

use geom::size::TypedSize2D;
use properties::longhands;
use util::geometry::{Au, ViewportPx};
use values::{computed, specified};


#[derive(Debug, PartialEq)]
pub struct MediaQueryList {
    media_queries: Vec<MediaQuery>
}

#[derive(PartialEq, Eq, Copy, Debug)]
pub enum Range<T> {
    Min(T),
    Max(T),
    //Eq(T),    // FIXME: Implement parsing support for equality then re-enable this.
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

#[derive(PartialEq, Eq, Copy, Debug)]
pub enum Expression {
    Width(Range<Au>),
}

#[derive(PartialEq, Eq, Copy, Debug)]
pub enum Qualifier {
    Only,
    Not,
}

#[derive(Debug, PartialEq)]
pub struct MediaQuery {
    qualifier: Option<Qualifier>,
    media_type: MediaQueryType,
    expressions: Vec<Expression>,
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

#[derive(PartialEq, Eq, Copy, Debug)]
pub enum MediaQueryType {
    All,  // Always true
    MediaType(MediaType),
}

#[derive(PartialEq, Eq, Copy, Debug)]
pub enum MediaType {
    Screen,
    Print,
    Unknown,
}

#[allow(missing_copy_implementations)]
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


fn parse_non_negative_length(input: &mut Parser) -> Result<Au, ()> {
    let length = try!(specified::Length::parse_non_negative(input));

    // http://dev.w3.org/csswg/mediaqueries3/ - Section 6
    // em units are relative to the initial font-size.
    let initial_font_size = longhands::font_size::get_initial_value();
    Ok(computed::compute_Au_with_font_size(length, initial_font_size, initial_font_size))
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
                    Ok(Expression::Width(Range::Min(try!(parse_non_negative_length(input)))))
                },
                "max-width" => {
                    Ok(Expression::Width(Range::Max(try!(parse_non_negative_length(input)))))
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
                match expression {
                    &Expression::Width(value) => value.evaluate(
                        Au::from_frac_px(device.viewport_size.to_untyped().width as f64)),
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

#[cfg(test)]
mod tests {
    use geom::size::TypedSize2D;
    use util::geometry::Au;
    use stylesheets::{iter_stylesheet_media_rules, iter_stylesheet_style_rules, Stylesheet};
    use stylesheets::Origin;
    use super::*;
    use url::Url;
    use std::borrow::ToOwned;

    fn test_media_rule<F>(css: &str, callback: F) where F: Fn(&MediaQueryList, &str) {
        let url = Url::parse("http://localhost").unwrap();
        let stylesheet = Stylesheet::from_str(css, url, Origin::Author);
        let mut rule_count: int = 0;
        iter_stylesheet_media_rules(&stylesheet, |rule| {
            rule_count += 1;
            callback(&rule.media_queries, css);
        });
        assert!(rule_count > 0);
    }

    fn media_query_test(device: &Device, css: &str, expected_rule_count: int) {
        let url = Url::parse("http://localhost").unwrap();
        let ss = Stylesheet::from_str(css, url, Origin::Author);
        let mut rule_count: int = 0;
        iter_stylesheet_style_rules(&ss, device, |_| rule_count += 1);
        assert!(rule_count == expected_rule_count, css.to_owned());
    }

    #[test]
    fn test_mq_empty() {
        test_media_rule("@media { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });
    }

    #[test]
    fn test_mq_screen() {
        test_media_rule("@media screen { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media only screen { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media not screen { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });
    }

    #[test]
    fn test_mq_print() {
        test_media_rule("@media print { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media only print { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media not print { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });
    }

    #[test]
    fn test_mq_unknown() {
        test_media_rule("@media fridge { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media only glass { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media not wood { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown), css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });
    }

    #[test]
    fn test_mq_all() {
        test_media_rule("@media all { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media only all { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media not all { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });
    }

    #[test]
    fn test_mq_or() {
        test_media_rule("@media screen, print { }", |list, css| {
            assert!(list.media_queries.len() == 2, css.to_owned());
            let q0 = &list.media_queries[0];
            assert!(q0.qualifier == None, css.to_owned());
            assert!(q0.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
            assert!(q0.expressions.len() == 0, css.to_owned());

            let q1 = &list.media_queries[1];
            assert!(q1.qualifier == None, css.to_owned());
            assert!(q1.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
            assert!(q1.expressions.len() == 0, css.to_owned());
        });
    }

    #[test]
    fn test_mq_default_expressions() {
        test_media_rule("@media (min-width: 100px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 1, css.to_owned());
            match q.expressions[0] {
                Expression::Width(Range::Min(w)) => assert!(w == Au::from_px(100)),
                _ => panic!("wrong expression type"),
            }
        });

        test_media_rule("@media (max-width: 43px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 1, css.to_owned());
            match q.expressions[0] {
                Expression::Width(Range::Max(w)) => assert!(w == Au::from_px(43)),
                _ => panic!("wrong expression type"),
            }
        });
    }

    #[test]
    fn test_mq_expressions() {
        test_media_rule("@media screen and (min-width: 100px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
            assert!(q.expressions.len() == 1, css.to_owned());
            match q.expressions[0] {
                Expression::Width(Range::Min(w)) => assert!(w == Au::from_px(100)),
                _ => panic!("wrong expression type"),
            }
        });

        test_media_rule("@media print and (max-width: 43px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
            assert!(q.expressions.len() == 1, css.to_owned());
            match q.expressions[0] {
                Expression::Width(Range::Max(w)) => assert!(w == Au::from_px(43)),
                _ => panic!("wrong expression type"),
            }
        });

        test_media_rule("@media fridge and (max-width: 52px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown), css.to_owned());
            assert!(q.expressions.len() == 1, css.to_owned());
            match q.expressions[0] {
                Expression::Width(Range::Max(w)) => assert!(w == Au::from_px(52)),
                _ => panic!("wrong expression type"),
            }
        });
    }

    #[test]
    fn test_mq_multiple_expressions() {
        test_media_rule("@media (min-width: 100px) and (max-width: 200px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == None, css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 2, css.to_owned());
            match q.expressions[0] {
                Expression::Width(Range::Min(w)) => assert!(w == Au::from_px(100)),
                _ => panic!("wrong expression type"),
            }
            match q.expressions[1] {
                Expression::Width(Range::Max(w)) => assert!(w == Au::from_px(200)),
                _ => panic!("wrong expression type"),
            }
        });

        test_media_rule("@media not screen and (min-width: 100px) and (max-width: 200px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
            assert!(q.expressions.len() == 2, css.to_owned());
            match q.expressions[0] {
                Expression::Width(Range::Min(w)) => assert!(w == Au::from_px(100)),
                _ => panic!("wrong expression type"),
            }
            match q.expressions[1] {
                Expression::Width(Range::Max(w)) => assert!(w == Au::from_px(200)),
                _ => panic!("wrong expression type"),
            }
        });
    }

    #[test]
    fn test_mq_malformed_expressions() {
        test_media_rule("@media (min-width: 100blah) and (max-width: 200px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media screen and (height: 200px) { }", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media (min-width: 30em foo bar) {}", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media not {}", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media not (min-width: 300px) {}", |list, css| {
            assert!(list.media_queries.len() == 1, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media , {}", |list, css| {
            assert!(list.media_queries.len() == 2, css.to_owned());
            let q = &list.media_queries[0];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
            let q = &list.media_queries[1];
            assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q.media_type == MediaQueryType::All, css.to_owned());
            assert!(q.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media screen 4px, print {}", |list, css| {
            assert!(list.media_queries.len() == 2, css.to_owned());
            let q0 = &list.media_queries[0];
            assert!(q0.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q0.media_type == MediaQueryType::All, css.to_owned());
            assert!(q0.expressions.len() == 0, css.to_owned());
            let q1 = &list.media_queries[1];
            assert!(q1.qualifier == None, css.to_owned());
            assert!(q1.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
            assert!(q1.expressions.len() == 0, css.to_owned());
        });

        test_media_rule("@media screen, {}", |list, css| {
            assert!(list.media_queries.len() == 2, css.to_owned());
            let q0 = &list.media_queries[0];
            assert!(q0.qualifier == None, css.to_owned());
            assert!(q0.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
            assert!(q0.expressions.len() == 0, css.to_owned());
            let q1 = &list.media_queries[1];
            assert!(q1.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(q1.media_type == MediaQueryType::All, css.to_owned());
            assert!(q1.expressions.len() == 0, css.to_owned());
        });
    }

    #[test]
    fn test_matching_simple() {
        let device = Device {
            media_type: MediaType::Screen,
            viewport_size: TypedSize2D(200.0, 100.0),
        };

        media_query_test(&device, "@media not all { a { color: red; } }", 0);
        media_query_test(&device, "@media not screen { a { color: red; } }", 0);
        media_query_test(&device, "@media not print { a { color: red; } }", 1);

        media_query_test(&device, "@media unknown { a { color: red; } }", 0);
        media_query_test(&device, "@media not unknown { a { color: red; } }", 1);

        media_query_test(&device, "@media { a { color: red; } }", 1);
        media_query_test(&device, "@media screen { a { color: red; } }", 1);
        media_query_test(&device, "@media print { a { color: red; } }", 0);
    }

    #[test]
    fn test_matching_width() {
        let device = Device {
            media_type: MediaType::Screen,
            viewport_size: TypedSize2D(200.0, 100.0),
        };

        media_query_test(&device, "@media { a { color: red; } }", 1);

        media_query_test(&device, "@media (min-width: 50px) { a { color: red; } }", 1);
        media_query_test(&device, "@media (min-width: 150px) { a { color: red; } }", 1);
        media_query_test(&device, "@media (min-width: 300px) { a { color: red; } }", 0);

        media_query_test(&device, "@media screen and (min-width: 50px) { a { color: red; } }", 1);
        media_query_test(&device, "@media screen and (min-width: 150px) { a { color: red; } }", 1);
        media_query_test(&device, "@media screen and (min-width: 300px) { a { color: red; } }", 0);

        media_query_test(&device, "@media not screen and (min-width: 50px) { a { color: red; } }", 0);
        media_query_test(&device, "@media not screen and (min-width: 150px) { a { color: red; } }", 0);
        media_query_test(&device, "@media not screen and (min-width: 300px) { a { color: red; } }", 1);

        media_query_test(&device, "@media (max-width: 50px) { a { color: red; } }", 0);
        media_query_test(&device, "@media (max-width: 150px) { a { color: red; } }", 0);
        media_query_test(&device, "@media (max-width: 300px) { a { color: red; } }", 1);

        media_query_test(&device, "@media screen and (min-width: 50px) and (max-width: 100px) { a { color: red; } }", 0);
        media_query_test(&device, "@media screen and (min-width: 250px) and (max-width: 300px) { a { color: red; } }", 0);
        media_query_test(&device, "@media screen and (min-width: 50px) and (max-width: 250px) { a { color: red; } }", 1);

        media_query_test(&device, "@media not screen and (min-width: 50px) and (max-width: 100px) { a { color: red; } }", 1);
        media_query_test(&device, "@media not screen and (min-width: 250px) and (max-width: 300px) { a { color: red; } }", 1);
        media_query_test(&device, "@media not screen and (min-width: 50px) and (max-width: 250px) { a { color: red; } }", 0);

        media_query_test(&device, "@media not screen and (min-width: 3.1em) and (max-width: 6em) { a { color: red; } }", 1);
        media_query_test(&device, "@media not screen and (min-width: 16em) and (max-width: 19.75em) { a { color: red; } }", 1);
        media_query_test(&device, "@media not screen and (min-width: 3em) and (max-width: 250px) { a { color: red; } }", 0);
    }

    #[test]
    fn test_matching_invalid() {
        let device = Device {
            media_type: MediaType::Screen,
            viewport_size: TypedSize2D(200.0, 100.0),
        };

        media_query_test(&device, "@media fridge { a { color: red; } }", 0);
        media_query_test(&device, "@media screen and (height: 100px) { a { color: red; } }", 0);
        media_query_test(&device, "@media not print and (width: 100) { a { color: red; } }", 0);
    }
}
