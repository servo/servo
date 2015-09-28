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
#![feature(vec_push_all)]

#![plugin(serde_macros)]
#![plugin(string_cache_plugin)]
#![plugin(serde_macros)]
#![plugin(plugins)]

extern crate app_units;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate cssparser;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate style_traits;
extern crate encoding;
extern crate euclid;
extern crate fnv;
extern crate num;
extern crate rustc_serialize;
extern crate selectors;
extern crate serde;
extern crate smallvec;
extern crate string_cache;
extern crate url;
extern crate util;

pub mod animation;
mod custom_properties;
pub mod font_face;
pub mod legacy;
pub mod media_queries;
pub mod node;
pub mod parser;
pub mod selector_matching;
pub mod stylesheets;
#[macro_use]
#[allow(non_camel_case_types)]
pub mod values;
pub mod viewport;

// Generated from the properties.mako.rs template by build.rs
#[macro_use]
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
