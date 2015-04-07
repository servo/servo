// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use util::logical_geometry::WritingMode;
use style::properties::{INITIAL_VALUES, get_writing_mode};


mod stylesheets;
mod media_queries;


#[test]
fn initial_writing_mode_is_empty() {
    assert_eq!(get_writing_mode(INITIAL_VALUES.get_inheritedbox()), WritingMode::empty())
}
