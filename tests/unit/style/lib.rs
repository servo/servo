/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(test)]
#![feature(plugin)]
#![feature(core_intrinsics)]

extern crate app_units;
extern crate cssparser;
extern crate euclid;
extern crate parking_lot;
extern crate rustc_serialize;
extern crate selectors;
#[macro_use(atom, ns)] extern crate string_cache;
extern crate style;
extern crate style_traits;
extern crate url;
extern crate util;

mod attr;
mod cache;
mod logical_geometry;
mod media_queries;
mod parsing;
mod properties;
mod selector_matching;
mod str;
mod stylesheets;
mod value;
mod viewport;

mod writing_modes {
    use style::logical_geometry::WritingMode;
    use style::properties::{INITIAL_SERVO_VALUES, get_writing_mode};

    #[test]
    fn initial_writing_mode_is_empty() {
        assert_eq!(get_writing_mode(INITIAL_SERVO_VALUES.get_inheritedbox()), WritingMode::empty())
    }
}
