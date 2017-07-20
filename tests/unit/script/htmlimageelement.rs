
use script::test::sizes::parse_a_sizes_attribute;
use script::test::DOMString;

#[test]
fn empty_vector() {
    assert!(parse_a_sizes_attribute(DOMString::new(), None).is_ok());
} 

#[test]
fn test_whitespace() {
    assert!(parse_a_sizes_attribute(DOMString::from("     (min-width: 500px)"),
            None).is_ok());
}


