/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![cfg_attr(test, feature(core_intrinsics))]
#![plugin(plugins)]

extern crate app_units;
extern crate cssparser;
extern crate euclid;
extern crate msg;
extern crate selectors;
#[macro_use(atom, ns)] extern crate string_cache;
extern crate style;
extern crate style_traits;
extern crate url;
extern crate util;

#[cfg(test)] mod attr;
#[cfg(test)] mod logical_geometry;
#[cfg(test)] mod media_queries;
#[cfg(test)] mod properties;
#[cfg(test)] mod stylesheets;
#[cfg(test)] mod viewport;

#[cfg(test)] mod writing_modes {
    use style::logical_geometry::WritingMode;
    use style::properties::{INITIAL_SERVO_VALUES, ComputedValues, get_writing_mode};

    #[test]
    fn initial_writing_mode_is_empty() {
        assert_eq!(get_writing_mode(INITIAL_SERVO_VALUES.get_inheritedbox()), WritingMode::empty())
    }
}
