use script::test::String;
use script::test::srcset::{Descriptor, ImageSource, parse_a_srcset_attribute};

#[test]
fn no_value() {
    let new_vec = Vec::new();
    assert_eq!(parse_a_srcset_attribute(String::new()), new_vec);
}

#[test]
fn width_one_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("small-image.jpg, 320w")), sources);
}

#[test]
fn width_two_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let second_descriptor = Descriptor { wid: Some(480), den: None };
    let second_imagesource = ImageSource { url: "medium-image.jpg".to_string(), descriptor: second_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    sources.push(second_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("small-image.jpg 320w, medium-image.jpg 480w")), sources);
}

#[test]
fn width_three_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let second_descriptor = Descriptor { wid: Some(480), den: None };
    let second_imagesource = ImageSource { url: "medium-image.jpg".to_string(), descriptor: second_descriptor };
    let third_descriptor = Descriptor { wid: Some(800), den: None };
    let third_imagesource = ImageSource { url: "large-image.jpg".to_string(), descriptor: third_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    sources.push(second_imagesource);
    sources.push(third_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("small-image.jpg 320w,
                                                     medium-image.jpg 480w,
                                                     large-image.jpg 800w")), sources);
}

#[test]
fn density_value() {
    let first_descriptor = Descriptor { wid: None, den: Some(1.0) };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("small-image.jpg 1x")), sources);
}

#[test]
fn without_descriptor() {
    let first_descriptor = Descriptor { wid: None, den: None };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("small-image.jpg")), sources);
}
//Does not parse an ImageSource when both width and density descriptor present
#[test]
fn two_descriptor() {
    let first_descriptor = Descriptor { wid: Some(380), den: Some(22.0) };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    assert_ne!(parse_a_srcset_attribute(String::from("small-image.jpg 380w 22x")), sources);
}
#[test]
fn decimal_descriptor() {
    let first_descriptor = Descriptor { wid: None, den: Some(2.2) };
    let first_imagesource = ImageSource { url: "small-image.jpg".to_string(), descriptor: first_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("small-image.jpg 2.2x")), sources);
}
