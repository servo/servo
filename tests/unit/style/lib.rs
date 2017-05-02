/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(test)]
#![feature(plugin, test)]

extern crate app_units;
extern crate cssparser;
extern crate euclid;
#[macro_use] extern crate html5ever_atoms;
extern crate parking_lot;
extern crate rayon;
extern crate rustc_serialize;
extern crate selectors;
extern crate servo_atoms;
extern crate servo_config;
extern crate servo_url;
extern crate style;
extern crate style_traits;
extern crate test;

mod animated_properties;
mod attr;
mod keyframes;
mod logical_geometry;
mod media_queries;
mod parsing;
mod properties;
mod rule_tree;
mod size_of;
mod str;
mod stylesheets;
mod stylist;
mod viewport;

mod writing_modes {
    use style::logical_geometry::WritingMode;
    use style::properties::{INITIAL_SERVO_VALUES, get_writing_mode};

    #[test]
    fn initial_writing_mode_is_empty() {
        assert_eq!(get_writing_mode(INITIAL_SERVO_VALUES.get_inheritedbox()), WritingMode::empty())
    }
}
