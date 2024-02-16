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
extern crate style;
extern crate style_traits;
extern crate test;
extern crate url;

mod animated_properties;
mod attr;
mod custom_properties;
mod logical_geometry;
mod parsing;
mod properties;
mod rule_tree;
mod str;
mod stylesheets;
mod stylist;
