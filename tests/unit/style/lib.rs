/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(string_cache_plugin)]

extern crate cssparser;
extern crate euclid;
extern crate selectors;
extern crate string_cache;
extern crate style;
extern crate style_traits;
extern crate url;
extern crate util;


#[cfg(test)] mod stylesheets;
#[cfg(test)] mod media_queries;
#[cfg(test)] mod viewport;

#[cfg(test)] mod writing_modes {
    use style::properties::{INITIAL_VALUES, get_writing_mode};
    use util::logical_geometry::WritingMode;

    #[test]
    fn initial_writing_mode_is_empty() {
        assert_eq!(get_writing_mode(INITIAL_VALUES.get_inheritedbox()), WritingMode::empty())
    }
}
