use super::stylesheets::parse_stylesheet;

#[test]
fn test_bootstrap() {
    // Test that parsing bootstrap does not trigger an assertion or otherwise fail.
    let stylesheet = parse_stylesheet(include_str!("bootstrap-v3.0.0.css"));
    assert!(stylesheet.rules.len() > 100);  // This depends on whet selectors are supported.
}
