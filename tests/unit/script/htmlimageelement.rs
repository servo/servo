/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::test::DOMString;
use script::test::sizes::parse_a_sizes_attribute;
use script::test::srcset::{Descriptor, ImageSource, parse_a_srcset_attribute};
use style::media_queries::{MediaCondition, MediaFeatureExpression};
use style::servo::media_queries::{ExpressionKind, Range};
use style::values::specified::{Length, NoCalcLength, AbsoluteLength, ViewportPercentageLength};
use style::values::specified::{source_size_list::SourceSizeList, source_size_list::SourceSize};


pub fn test_length_for_no_default_provided(len: f32) -> Length {
    let length = Length::NoCalc(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(len)));
    return length;
}

#[test]
fn no_default_provided() {
    let length = test_length_for_no_default_provided(100f32);
    let source_size_list = SourceSizeList {
        source_sizes: vec![],
        value: Some(length)
    };
    assert_eq!(parse_a_sizes_attribute(DOMString::new(), None), source_size_list);
}

pub fn test_length_for_default_provided(len: f32) -> Length {
    let length = Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(len)));
    return length;
}

#[test]
fn default_provided() {
    let length = test_length_for_default_provided(2f32);
    let source_size_list = SourceSizeList {
        source_sizes: vec![],
        value: Some(length)
    };
    assert_eq!(parse_a_sizes_attribute(DOMString::new(), Some(2)), source_size_list);
}

pub fn test_source_size(len: f32, input_length: f32) -> SourceSize {
    let length = Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(len)));
    let media_feature_exp = MediaFeatureExpression(ExpressionKind::Width(Range::Max(length)));
    let media_condition = MediaCondition::Feature(media_feature_exp);
    let length = test_length(input_length);
    let source_size = SourceSize {
        value: length,
        condition: media_condition
    };
    source_size
}

pub fn test_length(len: f32) -> Length {
    let length = Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(len)));
    return length;
}

#[test]
fn one_value() {
    let source_size = test_source_size(200f32, 545f32);
    let source_size_list = SourceSizeList {
        source_sizes: vec![source_size],
        value: None
    };
    assert_eq!(parse_a_sizes_attribute(DOMString::from("(max-width: 200px) 545px"), None), source_size_list);
}

#[test]
fn more_then_one_value() {
    let source_size1 = test_source_size(900f32, 1000f32);
    let source_size2 = test_source_size(900f32, 50f32);
    let mut a = vec![];
    a.push(source_size1);
    a.push(source_size2);
    let source_size_list = SourceSizeList {
        source_sizes: a,
        value: None
    };
    assert_eq!(parse_a_sizes_attribute(DOMString::from("(max-width: 900px) 1000px, (max-width: 900px) 50px"),
                                       None), source_size_list);
}

#[test]
fn no_extra_whitespace() {
    let source_size = test_source_size(200f32, 545f32);
    let source_size_list = SourceSizeList {
        source_sizes: vec![source_size],
        value: None
    };
    assert_eq!(parse_a_sizes_attribute(DOMString::from("(max-width: 200px) 545px"), None), source_size_list);
}

#[test]
fn extra_whitespace() {
    let source_size1 = test_source_size(900f32, 1000f32);
    let source_size2 = test_source_size(900f32, 50f32);
    let mut a = vec![];
    a.push(source_size1);
    a.push(source_size2);
    let source_size_list = SourceSizeList {
        source_sizes: a,
        value: None
    };
    assert_eq!(parse_a_sizes_attribute(
        DOMString::from("(max-width: 900px) 1000px,   (max-width: 900px) 50px"),
        None), source_size_list);
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
