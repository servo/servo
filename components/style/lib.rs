/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(alloc)]
#![feature(plugin)]
#![feature(box_syntax)]
#![feature(core)]
#![feature(collections)]
#![feature(hash)]
#![feature(rustc_private)]

#![plugin(string_cache_plugin)]
#![plugin(mod_path)]

#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;

extern crate collections;
extern crate geom;
extern crate url;

#[macro_use]
extern crate cssparser;

#[macro_use]
extern crate matches;

extern crate encoding;
extern crate rustc_serialize;
extern crate string_cache;
extern crate selectors;

#[macro_use]
extern crate lazy_static;

extern crate num;
extern crate util;


pub mod stylesheets;
pub mod parser;
pub mod selector_matching;
#[macro_use] pub mod values;

// Generated from the properties.mako.rs template by build.rs
mod_path! properties (concat!(env!("OUT_DIR"), "/properties.rs"));

pub mod node;
pub mod media_queries;
pub mod font_face;
pub mod legacy;
pub mod animation;
pub mod viewport;

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
