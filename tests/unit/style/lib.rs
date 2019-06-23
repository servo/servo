/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(test)]
#![feature(test)]

extern crate app_units;
extern crate cssparser;
extern crate euclid;
#[macro_use]
extern crate html5ever;
extern crate rayon;
extern crate selectors;
extern crate serde_json;
extern crate servo_arc;
extern crate servo_atoms;
extern crate servo_config;
extern crate servo_url;
#[macro_use]
extern crate size_of_test;
#[macro_use]
extern crate style;
extern crate style_traits;
extern crate test;

mod animated_properties;
mod attr;
mod custom_properties;
mod logical_geometry;
mod parsing;
mod properties;
mod rule_tree;
mod size_of;
mod specified_values;
mod str;
mod stylesheets;
mod stylist;
mod viewport;
