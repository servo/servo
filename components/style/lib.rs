/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Calculate [specified][specified] and [computed values][computed] from a
//! tree of DOM nodes and a set of stylesheets.
//!
//! [computed]: https://drafts.csswg.org/css-cascade/#computed
//! [specified]: https://drafts.csswg.org/css-cascade/#specified
//!
//! In particular, this crate contains the definitions of supported properties,
//! the code to parse them into specified values and calculate the computed
//! values based on the specified values, as well as the code to serialize both
//! specified and computed values.
//!
//! The main entry point is [`recalc_style_at`][recalc_style_at].
//!
//! [recalc_style_at]: traversal/fn.recalc_style_at.html
//!
//! Major dependencies are the [cssparser][cssparser] and [selectors][selectors]
//! crates.
//!
//! [cssparser]: ../cssparser/index.html
//! [selectors]: ../selectors/index.html

#![cfg_attr(feature = "servo", feature(custom_attribute))]
#![cfg_attr(feature = "servo", feature(custom_derive))]
#![cfg_attr(feature = "servo", feature(plugin))]
#![cfg_attr(feature = "servo", plugin(heapsize_plugin))]
#![cfg_attr(feature = "servo", plugin(serde_macros))]

#![deny(unsafe_code)]

#![recursion_limit = "500"]  // For match_ignore_ascii_case in PropertyDeclaration::parse

extern crate app_units;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;
extern crate core;
#[macro_use]
extern crate cssparser;
extern crate deque;
extern crate encoding;
extern crate euclid;
extern crate fnv;
#[cfg(feature = "gecko")]
extern crate gecko_bindings;
#[cfg(feature = "servo")] extern crate heapsize;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate matches;
extern crate num_traits;
extern crate rand;
extern crate rustc_serialize;
extern crate selectors;
#[cfg(feature = "servo")] extern crate serde;
extern crate smallvec;
#[macro_use(atom, ns)] extern crate string_cache;
#[macro_use]
extern crate style_traits;
extern crate time;
extern crate url;
extern crate util;

pub mod animation;
pub mod attr;
pub mod bezier;
pub mod context;
pub mod custom_properties;
pub mod data;
pub mod dom;
pub mod element_state;
pub mod error_reporting;
pub mod font_face;
pub mod keyframes;
pub mod logical_geometry;
pub mod matching;
pub mod media_queries;
pub mod parallel;
pub mod parser;
pub mod refcell;
pub mod restyle_hints;
pub mod selector_impl;
pub mod selector_matching;
pub mod sequential;
pub mod servo;
pub mod sink;
pub mod stylesheets;
pub mod traversal;
#[macro_use]
#[allow(non_camel_case_types)]
pub mod values;
pub mod viewport;
pub mod workqueue;

/// The CSS properties supported by the style system.
// Generated from the properties.mako.rs template by build.rs
#[macro_use]
#[allow(unsafe_code)]
pub mod properties {
    include!(concat!(env!("OUT_DIR"), "/properties.rs"));
}

macro_rules! reexport_computed_values {
    ( $( $name: ident )+ ) => {
        /// Types for [computed values][computed].
        ///
        /// [computed]: https://drafts.csswg.org/css-cascade/#computed
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
