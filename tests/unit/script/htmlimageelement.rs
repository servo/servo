/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::test::DOMString;
use script::test::sizes::{parse_a_sizes_attribute, Size};
use script::test::srcset::{Descriptor, ImageSource, parse_a_srcset_attribute};
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

#[test]
fn no_value() {
    let new_vec = Vec::new();
    assert_eq!(parse_a_srcset_attribute(" "), new_vec);
}

#[test]
fn width_one_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg, 320w"), sources);
}

#[test]
fn width_two_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let second_descriptor = Descriptor { wid: Some(480), den: None };
    let second_imagesource = ImageSource { url: "medium-image.jpg".to_string(), descriptor: second_descriptor };
    let sources = &[first_imagesource, second_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 320w, medium-image.jpg 480w"), sources);
}

#[test]
fn width_three_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "smallImage.jpg".to_string(), descriptor: first_descriptor };
    let second_descriptor = Descriptor { wid: Some(480), den: None };
    let second_imagesource = ImageSource { url: "mediumImage.jpg".to_string(), descriptor: second_descriptor };
    let third_descriptor = Descriptor { wid: Some(800), den: None };
    let third_imagesource = ImageSource { url: "largeImage.jpg".to_string(), descriptor: third_descriptor };
    let sources = &[first_imagesource, second_imagesource, third_imagesource];
    assert_eq!(parse_a_srcset_attribute("smallImage.jpg 320w,
                                        mediumImage.jpg 480w,
                                        largeImage.jpg 800w"), sources);
}

#[test]
fn density_value() {
    let first_descriptor = Descriptor { wid: None, den: Some(1.0) };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 1x"), sources);
}

#[test]
fn without_descriptor() {
    let first_descriptor = Descriptor { wid: None, den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg"), sources);
}

//Does not parse an ImageSource when both width and density descriptor present
#[test]
fn two_descriptor() {
    let empty_vec = Vec::new();
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 320w 1.1x"), empty_vec);
}

#[test]
fn decimal_descriptor() {
    let first_descriptor = Descriptor { wid: None, den: Some(2.2) };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 2.2x"), sources);
}

#[test]
fn different_descriptor() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let second_descriptor = Descriptor { wid: None, den: Some(2.2) };
    let second_imagesource = ImageSource { url: "medium-image.jpg".to_string(), descriptor: second_descriptor };
    let sources = &[first_imagesource, second_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 320w, medium-image.jpg 2.2x"), sources);
}
