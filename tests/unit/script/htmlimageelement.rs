/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script::test::srcset::{Descriptor, ImageSource, parse_a_srcset_attribute};

#[test]
fn no_value() {
    let new_vec = Vec::new();
    assert_eq!(parse_a_srcset_attribute(" "), new_vec);
}

#[test]
fn width_one_value() {
    let first_descriptor = Descriptor {
        width: Some(320),
        density: None,
    };
    let first_imagesource = ImageSource {
        url: "small-image.jpg".to_string(),
        descriptor: first_descriptor,
    };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 320w"), sources);
}

#[test]
fn width_two_value() {
    let first_descriptor = Descriptor {
        width: Some(320),
        density: None,
    };
    let first_imagesource = ImageSource {
        url: "small-image.jpg".to_string(),
        descriptor: first_descriptor,
    };
    let second_descriptor = Descriptor {
        width: Some(480),
        density: None,
    };
    let second_imagesource = ImageSource {
        url: "medium-image.jpg".to_string(),
        descriptor: second_descriptor,
    };
    let sources = &[first_imagesource, second_imagesource];
    assert_eq!(
        parse_a_srcset_attribute("small-image.jpg 320w, medium-image.jpg 480w"),
        sources
    );
}

#[test]
fn width_three_value() {
    let first_descriptor = Descriptor {
        width: Some(320),
        density: None,
    };
    let first_imagesource = ImageSource {
        url: "smallImage.jpg".to_string(),
        descriptor: first_descriptor,
    };
    let second_descriptor = Descriptor {
        width: Some(480),
        density: None,
    };
    let second_imagesource = ImageSource {
        url: "mediumImage.jpg".to_string(),
        descriptor: second_descriptor,
    };
    let third_descriptor = Descriptor {
        width: Some(800),
        density: None,
    };
    let third_imagesource = ImageSource {
        url: "largeImage.jpg".to_string(),
        descriptor: third_descriptor,
    };
    let sources = &[first_imagesource, second_imagesource, third_imagesource];
    assert_eq!(
        parse_a_srcset_attribute(
            "smallImage.jpg 320w,
                                        mediumImage.jpg 480w,
                                        largeImage.jpg 800w"
        ),
        sources
    );
}

#[test]
fn density_value() {
    let first_descriptor = Descriptor {
        width: None,
        density: Some(1.0),
    };
    let first_imagesource = ImageSource {
        url: "small-image.jpg".to_string(),
        descriptor: first_descriptor,
    };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 1x"), sources);
}

#[test]
fn without_descriptor() {
    let first_descriptor = Descriptor {
        width: None,
        density: None,
    };
    let first_imagesource = ImageSource {
        url: "small-image.jpg".to_string(),
        descriptor: first_descriptor,
    };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg"), sources);
}

//Does not parse an ImageSource when both width and density descriptor present
#[test]
fn two_descriptor() {
    let empty_vec = Vec::new();
    assert_eq!(
        parse_a_srcset_attribute("small-image.jpg 320w 1.1x"),
        empty_vec
    );
}

#[test]
fn decimal_descriptor() {
    let first_descriptor = Descriptor {
        width: None,
        density: Some(2.2),
    };
    let first_imagesource = ImageSource {
        url: "small-image.jpg".to_string(),
        descriptor: first_descriptor,
    };
    let sources = &[first_imagesource];
    assert_eq!(parse_a_srcset_attribute("small-image.jpg 2.2x"), sources);
}

#[test]
fn different_descriptor() {
    let first_descriptor = Descriptor {
        width: Some(320),
        density: None,
    };
    let first_imagesource = ImageSource {
        url: "small-image.jpg".to_string(),
        descriptor: first_descriptor,
    };
    let second_descriptor = Descriptor {
        width: None,
        density: Some(2.2),
    };
    let second_imagesource = ImageSource {
        url: "medium-image.jpg".to_string(),
        descriptor: second_descriptor,
    };
    let sources = &[first_imagesource, second_imagesource];
    assert_eq!(
        parse_a_srcset_attribute("small-image.jpg 320w, medium-image.jpg 2.2x"),
        sources
    );
}
