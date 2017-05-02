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

#![deny(warnings)]
#![deny(missing_docs)]

// FIXME(bholley): We need to blanket-allow unsafe code in order to make the
// gecko atom!() macro work. When Rust 1.14 is released [1], we can uncomment
// the commented-out attributes in regen_atoms.py and go back to denying unsafe
// code by default.
//
// [1] https://github.com/rust-lang/rust/issues/15701#issuecomment-251900615
//#![deny(unsafe_code)]
#![allow(unused_unsafe)]

#![recursion_limit = "500"]  // For define_css_keyword_enum! in -moz-appearance

extern crate app_units;
extern crate atomic_refcell;
extern crate bit_vec;
#[macro_use]
extern crate bitflags;
#[allow(unused_extern_crates)] extern crate byteorder;
#[cfg(feature = "gecko")] #[macro_use] #[no_link] extern crate cfg_if;
#[macro_use] extern crate cssparser;
extern crate euclid;
extern crate fnv;
#[cfg(feature = "gecko")] #[macro_use] pub mod gecko_string_cache;
#[cfg(feature = "servo")] extern crate heapsize;
#[cfg(feature = "servo")] #[macro_use] extern crate heapsize_derive;
#[cfg(feature = "servo")] #[macro_use] extern crate html5ever_atoms;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate matches;
#[cfg(feature = "gecko")]
#[macro_use]
extern crate nsstring_vendor as nsstring;
#[cfg(feature = "gecko")] extern crate num_cpus;
extern crate num_integer;
extern crate num_traits;
extern crate ordered_float;
extern crate parking_lot;
extern crate pdqsort;
#[cfg(feature = "gecko")] extern crate precomputed_hash;
extern crate rayon;
extern crate selectors;
#[cfg(feature = "servo")] #[macro_use] extern crate serde_derive;
#[cfg(feature = "servo")] #[macro_use] extern crate servo_atoms;
#[cfg(feature = "servo")] extern crate servo_config;
#[cfg(feature = "servo")] extern crate servo_url;
extern crate smallvec;
#[macro_use]
extern crate style_traits;
extern crate time;
#[allow(unused_extern_crates)]
extern crate unicode_segmentation;

pub mod animation;
#[allow(missing_docs)] // TODO.
#[cfg(feature = "servo")] pub mod attr;
pub mod bezier;
pub mod bloom;
pub mod cache;
pub mod cascade_info;
pub mod context;
pub mod counter_style;
pub mod custom_properties;
pub mod data;
pub mod dom;
pub mod element_state;
#[cfg(feature = "servo")] mod encoding_support;
pub mod error_reporting;
pub mod font_face;
pub mod font_metrics;
#[cfg(feature = "gecko")] #[allow(unsafe_code)] pub mod gecko;
#[cfg(feature = "gecko")] #[allow(unsafe_code)] pub mod gecko_bindings;
pub mod keyframes;
#[allow(missing_docs)] // TODO.
pub mod logical_geometry;
pub mod matching;
pub mod media_queries;
pub mod parallel;
pub mod parser;
pub mod restyle_hints;
pub mod rule_tree;
pub mod scoped_tls;
pub mod selector_parser;
pub mod shared_lock;
pub mod stylist;
#[cfg(feature = "servo")] #[allow(unsafe_code)] pub mod servo;
pub mod sequential;
pub mod sink;
pub mod str;
pub mod style_adjuster;
pub mod stylesheet_set;
pub mod stylesheets;
pub mod supports;
pub mod thread_state;
pub mod timer;
pub mod traversal;
#[macro_use]
#[allow(non_camel_case_types)]
pub mod values;
pub mod viewport;

use std::fmt;
use std::sync::Arc;
use style_traits::ToCss;

#[cfg(feature = "gecko")] pub use gecko_string_cache as string_cache;
#[cfg(feature = "gecko")] pub use gecko_string_cache::Atom;
#[cfg(feature = "gecko")] pub use gecko_string_cache::Namespace;
#[cfg(feature = "gecko")] pub use gecko_string_cache::Atom as Prefix;
#[cfg(feature = "gecko")] pub use gecko_string_cache::Atom as LocalName;

#[cfg(feature = "servo")] pub use servo_atoms::Atom;
#[cfg(feature = "servo")] pub use html5ever_atoms::Prefix;
#[cfg(feature = "servo")] pub use html5ever_atoms::LocalName;
#[cfg(feature = "servo")] pub use html5ever_atoms::Namespace;

/// The CSS properties supported by the style system.
/// Generated from the properties.mako.rs template by build.rs
#[macro_use]
#[allow(unsafe_code)]
#[deny(missing_docs)]
pub mod properties {
    include!(concat!(env!("OUT_DIR"), "/properties.rs"));
}

#[cfg(feature = "gecko")]
#[allow(unsafe_code, missing_docs)]
pub mod gecko_properties {
    include!(concat!(env!("OUT_DIR"), "/gecko_properties.rs"));
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

/// Returns whether the two arguments point to the same value.
///
/// FIXME: Remove this and use Arc::ptr_eq once we require Rust 1.17
#[inline]
pub fn arc_ptr_eq<T: 'static>(a: &Arc<T>, b: &Arc<T>) -> bool {
    ptr_eq::<T>(&**a, &**b)
}

/// Pointer equality
///
/// FIXME: Remove this and use std::ptr::eq once we require Rust 1.17
#[inline]
pub fn ptr_eq<T: ?Sized>(a: *const T, b: *const T) -> bool {
    a == b
}

/// Serializes as CSS a comma-separated list of any `T` that supports being
/// serialized as CSS.
pub fn serialize_comma_separated_list<W, T>(dest: &mut W,
                                            list: &[T])
                                            -> fmt::Result
    where W: fmt::Write,
          T: ToCss,
{
    if list.is_empty() {
        return Ok(());
    }

    try!(list[0].to_css(dest));

    for item in list.iter().skip(1) {
        try!(write!(dest, ", "));
        try!(item.to_css(dest));
    }

    Ok(())
}
