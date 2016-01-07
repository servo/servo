/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(concat_idents)]
#![feature(core_intrinsics)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(plugin)]

#![plugin(serde_macros)]
#![plugin(serde_macros)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
extern crate core;
#[macro_use]
extern crate cssparser;
extern crate encoding;
extern crate euclid;
extern crate fnv;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate matches;
extern crate msg;
extern crate num;
extern crate rustc_serialize;
#[macro_use(state_pseudo_classes)] extern crate selectors;
extern crate serde;
extern crate smallvec;
#[macro_use(atom, ns)] extern crate string_cache;
#[macro_use]
extern crate style_traits;
extern crate time;
extern crate url;
extern crate util;

pub mod animation;
pub mod attr;
pub mod context;
mod custom_properties;
pub mod data;
pub mod dom;
pub mod font_face;
pub mod matching;
pub mod media_queries;
pub mod parallel;
pub mod parser;
pub mod restyle_hints;
pub mod selector_matching;
pub mod stylesheets;
pub mod traversal;
#[macro_use]
#[allow(non_camel_case_types)]
pub mod values;
pub mod viewport;

// Generated from the properties.mako.rs template by build.rs
#[macro_use]
#[allow(unsafe_code)]
pub mod properties {
    include!(concat!(env!("OUT_DIR"), "/properties.rs"));
}

macro_rules! reexport_computed_values {
    ( $( $name: ident )+ ) => {
        pub mod computed_values {
            $(
                pub use properties::longhands::$name::computed_value as $name;
            )+
            // Don't use a side-specific name needlessly:
            pub use properties::longhands::border_top_style::computed_value as border_style;
        }
    }
}
longhand_properties_idents!(reexport_computed_values);
