use script::test::srcset::{parse_a_srcset_attribute, ImageSource, Descriptor};
use script::test::String;

#[test]
fn no_value() {
    //println!("{:?}", parse_a_srcset_attribute(String::new()));
    let v = Vec::new();
    assert_eq!(parse_a_srcset_attribute(String::new()), v);
}

#[test]
fn one_value() {
    println!("test: {:?}", parse_a_srcset_attribute(String::from("elva-fairy-320w.jpg 320w, elva-fairy-480w.jpg 480w")));
    //let d = Descriptor { wid: Some(320), den: None };
    //let v = ImageSource {url: "elva-fairy-320w.jpg 320w".to_string(), descriptor: d};
    //let mut sources = Vec::new();
    //sources.push(v);
    //assert_eq!(parse_a_srcset_attribute(String::from("elva-fairy-320w.jpg 320w")), sources);
}