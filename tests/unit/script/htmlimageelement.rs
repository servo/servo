/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::test::DOMString;
use script::test::sizes::{parse_a_sizes_attribute, Size};
use style::media_queries::{MediaQuery, MediaQueryType};
use style::media_queries::Expression;
use style::servo::media_queries::{ExpressionKind, Range};
use style::values::specified::{Length, NoCalcLength, AbsoluteLength, ViewportPercentageLength};

pub fn test_length_for_no_default_provided(len: f32) -> Length {
    let length = Length::NoCalc(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(len)));
    return length;
}

#[test]
fn no_default_provided() {
    let mut a = vec![];
    let length = test_length_for_no_default_provided(100f32);
    let size = Size { query: None, length: length };
    a.push(size);
    assert_eq!(parse_a_sizes_attribute(DOMString::new(), None), a);
}

pub fn test_length_for_default_provided(len: f32) -> Length {
    let length = Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(len)));
    return length;
}

#[test]
fn default_provided() {
    let mut a = vec![];
    let length = test_length_for_default_provided(2f32);
    let size = Size { query: None, length: length };
    a.push(size);
    assert_eq!(parse_a_sizes_attribute(DOMString::new(), Some(2)), a);
}

pub fn test_media_query(len: f32) -> MediaQuery {
    let length = Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(len)));
    let expr = Expression(ExpressionKind::Width(Range::Max(length)));
    let media_query = MediaQuery {
        qualifier: None,
        media_type: MediaQueryType::All,
        expressions: vec![expr]
    };
    media_query
}

pub fn test_length(len: f32) -> Length {
    let length = Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(len)));
    return length;
}

#[test]
fn one_value() {
    let mut a = vec![];
    let media_query = test_media_query(200f32);
    let length = test_length(545f32);
    let size = Size { query: Some(media_query), length: length };
    a.push(size);
    assert_eq!(parse_a_sizes_attribute(DOMString::from("(max-width: 200px) 545px"), None), a);
}

#[test]
fn more_then_one_value() {
    let media_query = test_media_query(900f32);
    let length = test_length(1000f32);
    let size = Size { query: Some(media_query), length: length };
    let media_query1 = test_media_query(900f32);
    let length1 = test_length(50f32);
    let size1 = Size { query: Some(media_query1), length: length1 };
    let a = &[size, size1];
    assert_eq!(parse_a_sizes_attribute(DOMString::from("(max-width: 900px) 1000px, (max-width: 900px) 50px"),
                None), a);
}

#[test]
fn no_extra_whitespace() {
    let mut a = vec![];
    let media_query = test_media_query(200f32);
    let length = test_length(545f32);
    let size = Size { query: Some(media_query), length: length };
    a.push(size);
    assert_eq!(parse_a_sizes_attribute(DOMString::from("(max-width: 200px) 545px"), None), a);
}

#[test]
fn extra_whitespace() {
    let media_query = test_media_query(900f32);
    let length = test_length(1000f32);
    let size = Size { query: Some(media_query), length: length };
    let media_query1 = test_media_query(900f32);
    let length1 = test_length(50f32);
    let size1 = Size { query: Some(media_query1), length: length1 };
    let a = &[size, size1];
    assert_eq!(parse_a_sizes_attribute(
        DOMString::from("(max-width: 900px) 1000px,   (max-width: 900px) 50px"),
        None), a);
}
