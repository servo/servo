use util::logical_geometry::WritingMode;
use style::properties::{INITIAL_VALUES, get_writing_mode};


mod stylesheets;
mod media_queries;


#[test]
fn initial_writing_mode_is_empty() {
    assert_eq!(get_writing_mode(INITIAL_VALUES.get_inheritedbox()), WritingMode::empty())
}
