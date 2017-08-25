use script::test::String;
use script::test::srcset::{Descriptor, ImageSource, parse_a_srcset_attribute};

#[test]
fn no_value() {
    //println!("{:?}", parse_a_srcset_attribute(String::new()));
    let new_vec = Vec::new();
    assert_eq!(parse_a_srcset_attribute(String::new()), new_vec);
}

#[test]
fn one_value() {
    //println!("test: {:?}", parse_a_srcset_attribute(String::from("elva-fairy-320w.jpg 320w")));
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "elva-fairy-320w.jpg".to_string(), descriptor: first_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("elva-fairy-320w.jpg 320w")), sources);
}

#[test]
fn two_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "elva-fairy-320w.jpg".to_string(), descriptor: first_descriptor };
    let second_descriptor = Descriptor { wid: Some(480), den: None };
    let second_imagesource = ImageSource { url: "elva-fairy-480w.jpg".to_string(), descriptor: second_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    sources.push(second_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("elva-fairy-320w.jpg 320w, elva-fairy-480w.jpg 480w")), sources);
}

#[test]
fn three_value() {
    let first_descriptor = Descriptor { wid: Some(320), den: None };
    let first_imagesource = ImageSource { url: "elva-fairy-320w.jpg".to_string(), descriptor: first_descriptor };
    let second_descriptor = Descriptor { wid: Some(480), den: None };
    let second_imagesource = ImageSource { url: "elva-fairy-480w.jpg".to_string(), descriptor: second_descriptor };
    let third_descriptor = Descriptor { wid: Some(800), den: None };
    let third_imagesource = ImageSource { url: "elva-fairy-800w.jpg".to_string(), descriptor: third_descriptor };
    let mut sources = Vec::new();
    sources.push(first_imagesource);
    sources.push(second_imagesource);
    sources.push(third_imagesource);
    assert_eq!(parse_a_srcset_attribute(String::from("elva-fairy-320w.jpg 320w,
                                                    elva-fairy-480w.jpg 480w, elva-fairy-800w.jpg 800w")), sources);
}
